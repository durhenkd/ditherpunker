use std::{fmt::Display, simd::cmp::SimdPartialOrd};

use itertools::Itertools;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    color_palette,
    dithering::threshold::multi_impl,
    texture::prelude::*,
    transform::prelude::*,
    utils::{self, transform::precompute_tiled_rows},
};

// pub type BayerArgs<'a> = TextureIO<'a, f32, utils::pixel::RGB>;

/// Configuration for Bayer transforms, shared for all
/// transform passes.
#[derive(Debug, Clone)]
pub struct BayerConfig {
    /// bayer matrix data
    pub(crate) matrix: Vec<f32>,
    /// color map used for transform
    pub(crate) map: Vec<color_palette::ColorMapElement>,
    /// bayer matrix order.
    ///
    /// > bayer matrix M2 (2x2) is order 1.
    /// > (alias for "first matrix")
    /// >
    /// > coincidently M4 (4x4) is order 2.
    pub(crate) order: usize,
    /// bits of side_size set to 1.
    ///
    /// > matrix side size == 2^order == sqrt(matrix.len()))
    ///
    /// Used for faster % computations on power of 2s.
    ///
    /// > x % 2^k === x & (2^k - 1)
    ///
    /// > equivalent to side_size - 1.
    /// >
    /// > this works because it's a power of 2.
    pub(crate) side_mask: usize,
}

impl BayerConfig {
    pub fn new(order: usize, matrix: Vec<f32>, map: Vec<color_palette::ColorMapElement>) -> Self {
        // matrix side size (i.e. sqrt(matrix.len()))
        let side_size = 1_usize << order;
        assert!(
            side_size * side_size == matrix.len(),
            "bayer order does not match matrix buffer length"
        );

        let side_mask = side_size - 1;
        Self {
            order,
            matrix,
            map,
            side_mask,
        }
    }

    /// Equivalent to 1.0 - bayer_value
    pub fn cache_bayer_complement(&self) -> Vec<f32> {
        self.matrix.iter().map(|v| 1.0 - *v).collect()
    }

    /// Pre-compute bayer complement as row patterns for the given image width.
    fn cache_tiled_pattern(&mut self, width: usize) -> Vec<f32> {
        precompute_tiled_rows(1 << self.order, width, |x, y, _| {
            1.0 - self.matrix[self.bayer_idx(x, y)]
        })
    }

    /// Get the idx in the bayer matrix corresponding to a pixel coordinate
    #[inline(always)]
    pub fn bayer_idx(&self, x: usize, y: usize) -> usize {
        ((y & self.side_mask) << self.order) + (x & self.side_mask)
    }

    pub fn colors_len(&self) -> usize {
        self.map.len()
    }
}

/// Strategy enum for selecting Bayer transform implementation
#[derive(Debug, Clone, Copy)]
pub enum BayerStrategy {
    /// Simple scalar implementation
    Scalar,
    /// SIMD with dynamic fitting (can handle any color_map size)
    Simd { lanes: usize },
    /// SIMD with fixed fitting requirements (requires color_map.len() == LANES)
    SimdFixed { lanes: usize },
}

impl Display for BayerStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BayerStrategy::Scalar => write!(f, "scalar"),
            BayerStrategy::Simd { lanes } => write!(f, "simd{}", lanes),
            BayerStrategy::SimdFixed { lanes } => write!(f, "simd-fixed{}", lanes),
        }
    }
}

impl BayerStrategy {
    /// Detect best-fit strategy
    pub fn auto(config: &BayerConfig) -> Self {
        // required width to process one thing
        let size_hint: usize = config.map.len();
        if size_hint < 2 {
            return Self::Scalar;
        }

        // supported simd lanes
        let mut lane_hint = multiversion::target_features::CURRENT_TARGET
            .suggested_simd_width::<f32>()
            .unwrap_or(0);

        // TODO add other targets in multi_impl and match against them here too
        if is_x86_feature_detected!("avx512f") {
            lane_hint = 16;
        } else if is_x86_feature_detected!("avx2") {
            lane_hint = 8;
        } else if is_x86_feature_detected!("sse2") {
            lane_hint = 4;
        }

        if size_hint == lane_hint {
            return Self::SimdFixed { lanes: lane_hint };
        }

        let lanes = BayerStrategy::lanes_fit(size_hint);
        if lanes < 2 || lanes > 16 {
            return Self::Scalar;
        }
        Self::Simd { lanes }
    }

    /// Create a transform instance for this strategy
    ///
    /// Returns `impl Transform<&mut Texture>` which is monomorphized at compile time
    /// for optimal performance while hiding implementation details
    pub fn build(
        self,
        config: BayerConfig,
    ) -> impl TextureTransform<Input = f32, Output = utils::pixel::RGB> {
        type I = BayerTransformImpl;
        match self {
            // scalar impls
            Self::Scalar => I::Scalar(Scalar::new(config)),
            // dynamic fitting impls
            Self::Simd { lanes: 2 } => I::Simd2(SimdFit::<2>::new(config)),
            Self::Simd { lanes: 4 } => I::Simd4(SimdFit::<4>::new(config)),
            Self::Simd { lanes: 8 } => I::Simd8(SimdFit::<8>::new(config)),
            Self::Simd { lanes: 16 } => I::Simd16(SimdFit::<16>::new(config)),
            // fixed fitting impls
            Self::SimdFixed { lanes: 2 } => I::SimdFixed2(SimdFixed::<2>::new(config)),
            Self::SimdFixed { lanes: 4 } => I::SimdFixed4(SimdFixed::<4>::new(config)),
            Self::SimdFixed { lanes: 8 } => I::SimdFixed8(SimdFixed::<8>::new(config)),
            Self::SimdFixed { lanes: 16 } => I::SimdFixed16(SimdFixed::<16>::new(config)),
            strategy => panic!("Unsupported lane configuration {}", strategy),
        }
    }

    /// Compute the best-fit amount of lanes to use for
    /// SIMD fit strategy.
    pub fn lanes_fit(lane_hint: usize) -> usize {
        // minimize dummy data used for simd ops
        let mut lanes = utils::num::closest_pow_2(lane_hint);
        // ensure chosen_size fits in the suggested width
        while lanes > lane_hint {
            lanes >>= 1;
        }
        lanes
    }

    /// Name of the strategy. This hides away implementation details,
    /// such as simd lanes used.
    pub fn name(&self) -> &'static str {
        match self {
            BayerStrategy::Scalar => "scalar",
            BayerStrategy::Simd { .. } => "simd",
            BayerStrategy::SimdFixed { .. } => "simd-fixed",
        }
    }
}

/// Internal enum that wraps all possible transform implementations
///
/// This is returned as `impl Transform`, so the concrete type is hidden
/// while still allowing full monomorphization
enum BayerTransformImpl {
    Scalar(Scalar),
    Simd2(SimdFit<2>),
    Simd4(SimdFit<4>),
    Simd8(SimdFit<8>),
    Simd16(SimdFit<16>),
    SimdFixed2(SimdFixed<2>),
    SimdFixed4(SimdFixed<4>),
    SimdFixed8(SimdFixed<8>),
    SimdFixed16(SimdFixed<16>),
}

impl TextureTransform for BayerTransformImpl {
    type Input = f32;
    type Output = utils::pixel::RGB;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        match self {
            Self::Scalar(t) => t.apply(input, output),
            Self::Simd2(t) => t.apply(input, output),
            Self::Simd4(t) => t.apply(input, output),
            Self::Simd8(t) => t.apply(input, output),
            Self::Simd16(t) => t.apply(input, output),
            Self::SimdFixed2(t) => t.apply(input, output),
            Self::SimdFixed4(t) => t.apply(input, output),
            Self::SimdFixed8(t) => t.apply(input, output),
            Self::SimdFixed16(t) => t.apply(input, output),
        }
    }

    fn prepare(&mut self, in_shape: crate::texture::TextureShape, out_shape: crate::texture::TextureShape) {
        match self {
            Self::Scalar(t) => t.prepare(in_shape, out_shape),
            Self::Simd2(t) => t.prepare(in_shape, out_shape),
            Self::Simd4(t) => t.prepare(in_shape, out_shape),
            Self::Simd8(t) => t.prepare(in_shape, out_shape),
            Self::Simd16(t) => t.prepare(in_shape, out_shape),
            Self::SimdFixed2(t) => t.prepare(in_shape, out_shape),
            Self::SimdFixed4(t) => t.prepare(in_shape, out_shape),
            Self::SimdFixed8(t) => t.prepare(in_shape, out_shape),
            Self::SimdFixed16(t) => t.prepare(in_shape, out_shape),
        }
    }
}

// Concrete Transform Implementations

/// Simple non-vectorized bayer dithering transform
struct Scalar {
    config: BayerConfig,
    bayer: Vec<f32>,
}

impl Scalar {
    fn new(config: BayerConfig) -> Self {
        Self {
            config,
            bayer: Vec::new(),
        }
    }
}

impl TextureTransform for Scalar {
    type Input = f32;
    type Output = utils::pixel::RGB;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        mut output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        multi_impl::scalar_par(
            input.as_ref(),
            input.shape(),
            output.as_mut(),
            self.bayer.as_slice(),
            &self.config,
        );

        (input, output)
    }

    fn prepare(
        &mut self,
        in_shape: crate::texture::TextureShape,
        out_shape: crate::texture::TextureShape,
    ) {
        debug_assert_eq!(in_shape, out_shape);
        self.bayer = self.config.cache_tiled_pattern(in_shape.0);
    }
}

/// Constrained SIMD accelerated bayer dithering transform.
///
/// This method computes the threshold for one pixel against all colors in
/// the color map.
///
/// It requires SIMD_LANES to have have the same size as color_map, so it is
/// limited to power of 2s and practically ranges up to 32.
struct SimdFixed<const SIMD_LANES: usize>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    config: BayerConfig,
    // Precomputed SIMD data
    scale: std::simd::Simd<f32, SIMD_LANES>,
    offset: std::simd::Simd<f32, SIMD_LANES>,
    /// Pre-computed bayer rows for cache-friendly sequential access.
    /// Each row contains the full-width bayer-complement pattern
    bayer: Vec<f32>,
}

impl<const SIMD_LANES: usize> SimdFixed<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn new(config: BayerConfig) -> Self {
        assert_eq!(
            config.map.len(),
            SIMD_LANES,
            "color map size must equal SIMD_LANES for fixed strategy"
        );

        let mut scale_array = [0.0f32; SIMD_LANES];
        let mut offset_array = [0.0f32; SIMD_LANES];

        for (i, color) in config.map.iter().enumerate() {
            scale_array[i] = color.scale;
            offset_array[i] = color.offset;
        }

        let scale = std::simd::Simd::<f32, SIMD_LANES>::from_array(scale_array);
        let offset = std::simd::Simd::<f32, SIMD_LANES>::from_array(offset_array);

        Self {
            config,
            scale,
            offset,
            bayer: Vec::new(),
        }
    }
}

impl<const SIMD_LANES: usize> TextureTransform for SimdFixed<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    type Input = f32;
    type Output = utils::pixel::RGB;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        mut output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        multi_impl::fixed_par(
            input.as_ref(),
            input.shape(),
            output.as_mut(),
            &self.bayer,
            &self.config,
            self.scale,
            self.offset,
        );

        (input, output)
    }

    fn prepare(
        &mut self,
        in_shape: crate::texture::TextureShape,
        out_shape: crate::texture::TextureShape,
    ) {
        debug_assert_eq!(in_shape, out_shape);
        self.bayer = self.config.cache_tiled_pattern(in_shape.0);
    }
}

// /// SIMD accelerated bayer dithering that processes multiple pixels simultaneously.
// ///
// /// Unlike SimdFixed which uses SIMD to compare one pixel against multiple thresholds,
// /// this processes SIMD_LANES pixels at once, providing better memory bandwidth utilization.
// ///
// /// Requires color_map.len() to equal SIMD_LANES (same constraint as SimdFixed).
// struct SimdPixelwise<const SIMD_LANES: usize>
// where
//     std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
// {
//     config: BayerConfig,
//     /// Precomputed bayer matrix where each value is the complement
//     /// of the original.
//     bayer: Vec<f32>,
//     /// Pre-computed bayer rows for cache-friendly sequential access.
//     /// Each row contains the full-width bayer pattern for one y-coordinate.
//     /// The outer vec has `bayer_side` entries (one per unique row pattern).
//     bayer_rows: Vec<Vec<f32>>,
//     /// Maximum width that was pre-computed. Used to detect when recomputation is needed.
//     precomputed_width: usize,
// }

// impl<const SIMD_LANES: usize> SimdPixelwise<SIMD_LANES>
// where
//     std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
// {
//     fn new(config: BayerConfig) -> Self {
//         assert_eq!(
//             config.map.len(),
//             SIMD_LANES,
//             "color map size must equal SIMD_LANES for pixelwise strategy"
//         );

//         let bayer = config.cache_bayer_complement();

//         Self {
//             config,
//             bayer,
//             bayer_rows: Vec::new(),
//             precomputed_width: 0,
//         }
//     }

//     /// Pre-compute bayer row patterns for the given image width.
//     /// This enables cache-friendly sequential SIMD loads instead of scattered index lookups.
//     fn ensure_bayer_rows(&mut self, width: usize) {
//         if width <= self.precomputed_width {
//             return; // Already computed for this width or larger
//         }

//         let bayer_side = 1 << self.config.order;
//         self.bayer_rows.clear();
//         self.bayer_rows.reserve(bayer_side);

//         for y in 0..bayer_side {
//             let mut row = Vec::with_capacity(width);
//             for x in 0..width {
//                 row.push(self.bayer[self.config.bayer_idx(x, y)]);
//             }
//             self.bayer_rows.push(row);
//         }

//         self.precomputed_width = width;
//     }
// }

// impl<'a, const SIMD_LANES: usize> Transform<BayerArgs<'a>> for SimdPixelwise<SIMD_LANES>
// where
//     std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
// {
//     fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
//         let width = rhs.input.width() as usize;
//         let height = rhs.input.height() as usize;
//         let input = rhs.input.as_ref();
//         let output = rhs.output.as_mut();

//         // Ensure bayer rows are pre-computed for this width
//         self.ensure_bayer_rows(width);

//         let fallback_color = self.config.map.last().unwrap().color;
//         let bayer_side = 1 << self.config.order;

//         for y in 0..height {
//             let row_start = y * width;
//             let row_end = row_start + width;

//             // Select pre-computed bayer row (cycles every bayer_side rows)
//             let bayer_row_idx = y & self.config.side_mask;
//             let bayer_row = &self.bayer_rows[bayer_row_idx];

//             // Try to get aligned SIMD chunks
//             let (prefix, middle, suffix) = input[row_start..row_end].as_simd::<SIMD_LANES>();
//             let mut pixel_idx = row_start;

//             // Process prefix with scalar code
//             for &pixel_value in prefix {
//                 let x = pixel_idx - row_start;
//                 let bayer = bayer_row[x];
//                 let mut color = fallback_color;
//                 for map in &self.config.map {
//                     if pixel_value < bayer * map.scale + map.offset {
//                         color = map.color;
//                         break;
//                     }
//                 }
//                 output[pixel_idx] = color;
//                 pixel_idx += 1;
//             }

//             // Process middle SIMD chunks - this is where the performance gain happens
//             for simd_pixels in middle {
//                 let x_base = pixel_idx - row_start;

//                 // Load bayer values sequentially - CACHE FRIENDLY!
//                 let bayer_simd = std::simd::Simd::<f32, SIMD_LANES>::from_slice(
//                     &bayer_row[x_base..x_base + SIMD_LANES],
//                 );

//                 // Initialize all pixels to fallback color
//                 let mut colors = [fallback_color; SIMD_LANES];

//                 // For each color threshold, check all SIMD_LANES pixels at once
//                 for color_elem in &self.config.map {
//                     let scale_simd = std::simd::Simd::<f32, SIMD_LANES>::splat(color_elem.scale);
//                     let offset_simd = std::simd::Simd::<f32, SIMD_LANES>::splat(color_elem.offset);
//                     let threshold = bayer_simd * scale_simd + offset_simd;
//                     let mask = simd_pixels.simd_lt(threshold);

//                     // Update colors where mask is true
//                     let bitmask = mask.to_bitmask();
//                     for lane in 0..SIMD_LANES {
//                         if bitmask & (1 << lane) != 0 {
//                             colors[lane] = color_elem.color;
//                         }
//                     }
//                 }

//                 // Write results
//                 output[pixel_idx..pixel_idx + SIMD_LANES].copy_from_slice(&colors);
//                 pixel_idx += SIMD_LANES;
//             }

//             // Process suffix with scalar code
//             for &pixel_value in suffix {
//                 let x = pixel_idx - row_start;
//                 let bayer = bayer_row[x];
//                 let mut color = fallback_color;
//                 for map in &self.config.map {
//                     if pixel_value < bayer * map.scale + map.offset {
//                         color = map.color;
//                         break;
//                     }
//                 }
//                 output[pixel_idx] = color;
//                 pixel_idx += 1;
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod bayer_transform_internal_benches {
//     use std::{hint::black_box, str::FromStr};

//     use crate::{
//         color_palette::ColorMapElement,
//         dithering::threshold::{
//             bayer_transform::{BayerArgs, BayerConfig, SimdFixed, SimdPixelwise},
//             matrices,
//         },
//         texture::Texture,
//         transform::prelude::*,
//         utils::{image::read_image, pixel::RGB},
//     };

//     extern crate test;

//     /// Get owned data to perform bayer transformations
//     pub fn data(size: u32) -> (Texture<f32>, Texture<RGB>) {
//         (
//             std::hint::black_box(
//                 read_image(&"./assets/bench_asset.png".to_string())
//                     .unwrap()
//                     .resize(size, size, image::imageops::FilterType::Gaussian)
//                     .grayscale()
//                     .brighten(60)
//                     .adjust_contrast(10_f32)
//                     .to_luma32f()
//                     .into(),
//             ),
//             std::hint::black_box(Texture::new(size, size)),
//         )
//     }

//     #[bench]
//     fn bench_pixel_wise_strategy(b: &mut test::Bencher) {
//         let cmap = |hex: &str, offset: f32, scale: f32| -> ColorMapElement {
//             ColorMapElement {
//                 color: RGB::from_hex(String::from_str(hex).unwrap()).unwrap(),
//                 offset,
//                 scale,
//             }
//         };

//         let config = black_box(BayerConfig::new(
//             1,
//             black_box(matrices::BAYER0.to_vec()),
//             black_box(vec![
//                 cmap("#020217", 0.0, 0.8),
//                 cmap("#2e2627", 0.05, 0.9),
//                 cmap("#60594b", 0.1, 0.7),
//                 cmap("#e6e2c2", 0.15, 0.85),
//             ]),
//         ));
//         let (input, mut output) = data(300);
//         let mut args = black_box(BayerArgs::new(
//             input.as_ref_texture(),
//             output.as_ref_mut_texture(),
//         ));
//         let mut transform = black_box(SimdPixelwise::<4>::new(config));

//         b.iter(|| {
//             transform.apply(&mut args);
//         });
//     }

//     #[bench]
//     fn bench_fixed_strategy(b: &mut test::Bencher) {
//         let cmap = |hex: &str, offset: f32, scale: f32| -> ColorMapElement {
//             ColorMapElement {
//                 color: RGB::from_hex(String::from_str(hex).unwrap()).unwrap(),
//                 offset,
//                 scale,
//             }
//         };

//         let config = black_box(BayerConfig::new(
//             1,
//             black_box(matrices::BAYER0.to_vec()),
//             black_box(vec![
//                 cmap("#020217", 0.0, 0.8),
//                 cmap("#2e2627", 0.05, 0.9),
//                 cmap("#60594b", 0.1, 0.7),
//                 cmap("#e6e2c2", 0.15, 0.85),
//             ]),
//         ));
//         let (input, mut output) = data(300);
//         let mut args = black_box(BayerArgs::new(
//             input.as_ref_texture(),
//             output.as_ref_mut_texture(),
//         ));
//         let mut transform = black_box(SimdFixed::<4>::new(config));

//         b.iter(|| {
//             transform.apply(&mut args);
//         });
//     }
// }

/// Flexible SIMD accelerated bayer dithering transform.
///
/// This method computes the threshold for one pixel against all colors in
/// the color map iteratively, until finished.
///
/// If the color map is not a multiple of SIMD_LANES,
/// remaining data is filled with dummy data.
struct SimdFit<const SIMD_LANES: usize>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    config: BayerConfig,
    // Precomputed data for fitting arbitrary color map sizes
    simd: Vec<SimdFitPassData<SIMD_LANES>>,
    // iterations required per pixel to finish computing a pixel
    compute_iters: usize,
    /// Precomputed bayer matrix where each value is the complement
    /// of the original.
    bayer: Vec<f32>,
}

/// Cached data for one pixel compute pass
pub(crate) struct SimdFitPassData<const LANES: usize>
where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    /// number of addresable items within the result.
    pub(crate) size: usize,
    pub(crate) scale: std::simd::Simd<f32, LANES>,
    pub(crate) offset: std::simd::Simd<f32, LANES>,
}

impl<const SIMD_LANES: usize> SimdFit<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn new(config: BayerConfig) -> Self {
        // (color_map.len() + SIMD_LANES - 1) / SIMD_LANES
        let compute_iters = config.map.len().div_ceil(SIMD_LANES);
        // SAFETY: compute_data is immediately initialized
        let mut compute_data: Vec<SimdFitPassData<SIMD_LANES>> = unsafe {
            utils::buffer::uninitialized_buffer::<SimdFitPassData<SIMD_LANES>>(compute_iters)
        };
        for (iter, compute) in compute_data.iter_mut().enumerate() {
            let range_size = if iter == compute_iters - 1 {
                match config.map.len() % SIMD_LANES {
                    0 => SIMD_LANES,
                    remainder => remainder,
                }
            } else {
                SIMD_LANES
            };
            let remainder = SIMD_LANES - range_size;

            let range_start = iter * SIMD_LANES;
            let range_end = range_start + range_size;
            *compute = SimdFitPassData {
                size: range_size,
                scale: std::simd::Simd::<f32, SIMD_LANES>::from_slice(
                    config.map[range_start..range_end]
                        .iter()
                        .map(|color| color.scale)
                        .chain(std::iter::repeat_n(0.0, remainder))
                        .collect_vec()
                        .as_slice(),
                ),
                offset: std::simd::Simd::<f32, SIMD_LANES>::from_slice(
                    config.map[range_start..range_end]
                        .iter()
                        .map(|color| color.offset)
                        .chain(std::iter::repeat_n(0.0, remainder))
                        .collect_vec()
                        .as_slice(),
                ),
            }
        }

        Self {
            config,
            compute_iters,
            simd: compute_data,
            bayer: Vec::new(),
        }
    }
}

impl<const SIMD_LANES: usize> TextureTransform for SimdFit<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    type Input = f32;
    type Output = utils::pixel::RGB;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        mut output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        multi_impl::fit_par(
            input.as_ref(),
            input.shape(),
            output.as_mut(),
            &self.bayer,
            &self.config,
            &self.simd,
            self.compute_iters,
        );

        (input, output)
    }

    fn prepare(
        &mut self,
        in_shape: crate::texture::TextureShape,
        out_shape: crate::texture::TextureShape,
    ) {
        debug_assert_eq!(in_shape, out_shape);
        self.bayer = self.config.cache_tiled_pattern(in_shape.0);
    }
}
