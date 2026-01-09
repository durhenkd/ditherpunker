use std::{fmt::Display, simd::cmp::SimdPartialOrd};

use itertools::Itertools;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    color_palette,
    texture::{TextureMutRef, TextureOps, TextureRef},
    transform::Transform,
    utils,
};

/// Arguments for one transform pass.
///
/// input can be changed, output is expected to be the same all the time.
#[derive(Debug)]
pub struct BayerArgs<'a> {
    input: TextureRef<'a, f32>,
    output: TextureMutRef<'a, utils::pixel::RGB>,
}

impl<'a> BayerArgs<'a> {
    pub fn new(input: TextureRef<'a, f32>, output: TextureMutRef<'a, utils::pixel::RGB>) -> Self {
        Self { input, output }
    }

    pub fn replace_input(&mut self, input: TextureRef<'a, f32>) {
        self.input = input;
    }

    pub fn replace_output(&mut self, output: TextureMutRef<'a, utils::pixel::RGB>) {
        self.output = output;
    }
}

/// Configuration for Bayer transforms, shared for all
/// transform passes.
#[derive(Debug, Clone)]
pub struct BayerConfig {
    /// bayer matrix data
    matrix: Vec<f32>,
    /// color map used for transform
    map: Vec<color_palette::ColorMapElement>,
    /// bayer matrix order.
    ///
    /// > bayer matrix M2 (2x2) is order 1.
    /// > (alias for "first matrix")
    /// >
    /// > coincidently M4 (4x4) is order 2.
    order: usize,
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
    side_mask: usize,
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
    pub fn bayer_complement(&self) -> Vec<f32> {
        self.matrix.iter().map(|v| 1.0 - *v).collect()
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
    /// Simple parallel scalar implementation.
    ScalarPar,
    /// SIMD with dynamic fitting (can handle any color_map size)
    Simd { lanes: usize },
    /// SIMD with fixed fitting requirements (requires color_map.len() == LANES)
    SimdFixed { lanes: usize },
    /// Parallel SIMD with dynamic fitting
    SimdPar { lanes: usize },
    /// Parallel SIMD with fixed fitting requirements
    SimdFixedPar { lanes: usize },
}

impl Display for BayerStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BayerStrategy::Scalar => write!(f, "scalar"),
            BayerStrategy::ScalarPar => write!(f, "scalar-par"),
            BayerStrategy::Simd { lanes } => write!(f, "simd{}", lanes),
            BayerStrategy::SimdFixed { lanes } => write!(f, "simd-fixed{}", lanes),
            BayerStrategy::SimdPar { lanes } => write!(f, "simd-par{}", lanes),
            BayerStrategy::SimdFixedPar { lanes } => write!(f, "simd-fixed-par{}", lanes),
        }
    }
}

impl BayerStrategy {
    /// Detect best-fit strategy
    pub fn auto(config: &BayerConfig) -> Self {
        // required width to process one thing
        let size_hint: usize = config.map.len();
        // supported simd lanes
        let lane_hint = multiversion::target_features::CURRENT_TARGET
            .suggested_simd_width::<f32>()
            .unwrap_or(0);
        // estimated usable threads
        let par_hint = rayon::current_num_threads();

        // sync
        if par_hint == 1 {
            if size_hint < 2 {
                return Self::Scalar;
            }
            if size_hint == lane_hint {
                return Self::SimdFixed { lanes: lane_hint };
            }
            let lanes = BayerStrategy::lanes_fit(size_hint);
            if lanes < 2 || lanes > 16 {
                return Self::Scalar;
            }
            return Self::Simd { lanes };
        };
        // par
        if size_hint < 2 {
            return Self::ScalarPar;
        }
        if lane_hint == size_hint {
            return Self::SimdFixedPar { lanes: size_hint };
        }
        let lanes = BayerStrategy::lanes_fit(size_hint);
        if lanes < 2 || lanes > 16 {
            return Self::ScalarPar;
        }
        Self::SimdPar { lanes }
    }

    /// Create a transform instance for this strategy
    ///
    /// Returns `impl Transform<&mut Texture>` which is monomorphized at compile time
    /// for optimal performance while hiding implementation details
    pub fn build<'a>(self, config: BayerConfig) -> impl Transform<BayerArgs<'a>> {
        type I = BayerTransformImpl;
        match self {
            // scalar impls
            Self::Scalar => I::Scalar(Scalar::new(config)),
            Self::ScalarPar => I::ScalarPar(ScalarPar::new(config)),
            // dynamic fitting impls
            Self::Simd { lanes: 2 } => I::Simd2(SimdFit::<2>::new(config)),
            Self::Simd { lanes: 4 } => I::Simd4(SimdFit::<4>::new(config)),
            Self::Simd { lanes: 8 } => I::Simd8(SimdFit::<8>::new(config)),
            Self::Simd { lanes: 16 } => I::Simd16(SimdFit::<16>::new(config)),
            Self::SimdPar { lanes: 2 } => I::SimdPar2(SimdFitPar::<2>::new(config)),
            Self::SimdPar { lanes: 4 } => I::SimdPar4(SimdFitPar::<4>::new(config)),
            Self::SimdPar { lanes: 8 } => I::SimdPar8(SimdFitPar::<8>::new(config)),
            Self::SimdPar { lanes: 16 } => I::SimdPar16(SimdFitPar::<16>::new(config)),
            // fixed fitting impls
            Self::SimdFixed { lanes: 2 } => I::SimdFixed2(SimdFixed::<2>::new(config)),
            Self::SimdFixed { lanes: 4 } => I::SimdFixed4(SimdFixed::<4>::new(config)),
            Self::SimdFixed { lanes: 8 } => I::SimdFixed8(SimdFixed::<8>::new(config)),
            Self::SimdFixed { lanes: 16 } => I::SimdFixed16(SimdFixed::<16>::new(config)),
            Self::SimdFixedPar { lanes: 2 } => I::SimdFixedPar2(SimdFixedPar::<2>::new(config)),
            Self::SimdFixedPar { lanes: 4 } => I::SimdFixedPar4(SimdFixedPar::<4>::new(config)),
            Self::SimdFixedPar { lanes: 8 } => I::SimdFixedPar8(SimdFixedPar::<8>::new(config)),
            Self::SimdFixedPar { lanes: 16 } => I::SimdFixedPar16(SimdFixedPar::<16>::new(config)),
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
            BayerStrategy::ScalarPar => "scalar-par",
            BayerStrategy::Simd { .. } => "simd",
            BayerStrategy::SimdFixed { .. } => "simd-fixed",
            BayerStrategy::SimdPar { .. } => "simd-par",
            BayerStrategy::SimdFixedPar { .. } => "simd-fixed-par",
        }
    }
}

/// Internal enum that wraps all possible transform implementations
///
/// This is returned as `impl Transform`, so the concrete type is hidden
/// while still allowing full monomorphization
enum BayerTransformImpl {
    Scalar(Scalar),
    ScalarPar(ScalarPar),
    Simd2(SimdFit<2>),
    Simd4(SimdFit<4>),
    Simd8(SimdFit<8>),
    Simd16(SimdFit<16>),
    SimdPar2(SimdFitPar<2>),
    SimdPar4(SimdFitPar<4>),
    SimdPar8(SimdFitPar<8>),
    SimdPar16(SimdFitPar<16>),
    SimdFixed2(SimdFixed<2>),
    SimdFixed4(SimdFixed<4>),
    SimdFixed8(SimdFixed<8>),
    SimdFixed16(SimdFixed<16>),
    SimdFixedPar2(SimdFixedPar<2>),
    SimdFixedPar4(SimdFixedPar<4>),
    SimdFixedPar8(SimdFixedPar<8>),
    SimdFixedPar16(SimdFixedPar<16>),
}

impl<'a> Transform<BayerArgs<'a>> for BayerTransformImpl {
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        match self {
            Self::Scalar(t) => t.apply(rhs),
            Self::ScalarPar(t) => t.apply(rhs),
            Self::Simd2(t) => t.apply(rhs),
            Self::Simd4(t) => t.apply(rhs),
            Self::Simd8(t) => t.apply(rhs),
            Self::Simd16(t) => t.apply(rhs),
            Self::SimdPar2(t) => t.apply(rhs),
            Self::SimdPar4(t) => t.apply(rhs),
            Self::SimdPar8(t) => t.apply(rhs),
            Self::SimdPar16(t) => t.apply(rhs),
            Self::SimdFixed2(t) => t.apply(rhs),
            Self::SimdFixed4(t) => t.apply(rhs),
            Self::SimdFixed8(t) => t.apply(rhs),
            Self::SimdFixed16(t) => t.apply(rhs),
            Self::SimdFixedPar2(t) => t.apply(rhs),
            Self::SimdFixedPar4(t) => t.apply(rhs),
            Self::SimdFixedPar8(t) => t.apply(rhs),
            Self::SimdFixedPar16(t) => t.apply(rhs),
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
        let bayer = config.bayer_complement();
        Self { config, bayer }
    }
}

impl<'a> Transform<BayerArgs<'a>> for Scalar {
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let height = rhs.input.height() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.config.map.last().unwrap().color;
        let mut pixel_idx = 0_usize;
        for y in 0..height {
            for x in 0..width {
                let bayer = self.bayer[self.config.bayer_idx(x, y)];
                let mut color: utils::pixel::RGB = fallback_color;
                for map in &self.config.map {
                    if input[pixel_idx] < bayer * map.scale + map.offset {
                        color = map.color;
                        break;
                    }
                }
                output[pixel_idx] = color;
                pixel_idx += 1;
            }
        }
    }
}

/// Parallel non-vectorized bayer dithering transform
struct ScalarPar {
    scalar: Scalar,
}

impl ScalarPar {
    fn new(config: BayerConfig) -> Self {
        Self {
            scalar: Scalar::new(config),
        }
    }
}

impl<'a> Transform<BayerArgs<'a>> for ScalarPar {
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.scalar.config.map.last().unwrap().color;
        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let idx_offset = y * width;
                for (x, col) in row.iter_mut().enumerate() {
                    let pixel_idx = idx_offset + x;

                    let bayer = self.scalar.bayer[self.scalar.config.bayer_idx(x, y)];
                    let mut color: utils::pixel::RGB = fallback_color;
                    for map in &self.scalar.config.map {
                        if input[pixel_idx] < bayer * map.scale + map.offset {
                            color = map.color;
                            break;
                        }
                    }
                    *col = color;
                }
            });
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
    /// Precomputed bayer matrix where each value is the complement
    /// of the original.
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

        // Use a fixed-size array on the stack instead of heap allocation
        let mut scale_array = [0.0f32; SIMD_LANES];
        let mut offset_array = [0.0f32; SIMD_LANES];

        for (i, color) in config.map.iter().enumerate() {
            scale_array[i] = color.scale;
            offset_array[i] = color.offset;
        }

        let scale = std::simd::Simd::<f32, SIMD_LANES>::from_array(scale_array);
        let offset = std::simd::Simd::<f32, SIMD_LANES>::from_array(offset_array);

        let bayer: Vec<f32> = config.bayer_complement();

        Self {
            config,
            scale,
            offset,
            bayer,
        }
    }
}

impl<'a, const SIMD_LANES: usize> Transform<BayerArgs<'a>> for SimdFixed<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let height = rhs.input.height() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.config.map.last().unwrap().color;
        let mut pixel_idx = 0_usize;
        for y in 0..height {
            for x in 0..width {
                let bayer_value = self.bayer[self.config.bayer_idx(x, y)];

                let pixel = std::simd::Simd::<f32, SIMD_LANES>::splat(input[pixel_idx]);
                let bayer = std::simd::Simd::<f32, SIMD_LANES>::splat(bayer_value);

                let threshold = bayer * self.scale + self.offset;
                let result = pixel.simd_lt(threshold).to_bitmask();

                let mut color: utils::pixel::RGB = fallback_color;
                for (lane, map) in self.config.map.iter().enumerate() {
                    if result & (1 << lane) != 0 {
                        color = map.color;
                        break;
                    }
                }
                output[pixel_idx] = color;
                pixel_idx += 1;
            }
        }
    }
}

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
struct SimdFitPassData<const LANES: usize>
where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    /// number of addresable items within the result.
    size: usize,
    scale: std::simd::Simd<f32, LANES>,
    offset: std::simd::Simd<f32, LANES>,
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

        let bayer = config.bayer_complement();

        Self {
            config,
            compute_iters,
            simd: compute_data,
            bayer,
        }
    }
}

impl<'a, const SIMD_LANES: usize> Transform<BayerArgs<'a>> for SimdFit<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let height = rhs.input.height() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.config.map.last().unwrap().color;
        let mut pixel_idx = 0_usize;
        for y in 0..height {
            for x in 0..width {
                let bayer_value = self.bayer[self.config.bayer_idx(x, y)];

                let pixel = std::simd::Simd::<f32, SIMD_LANES>::splat(input[pixel_idx]);
                let bayer = std::simd::Simd::<f32, SIMD_LANES>::splat(bayer_value);

                let mut color: utils::pixel::RGB = fallback_color;
                'compute: for iter in 0..self.compute_iters {
                    let pool = &self.simd[iter];
                    let threshold = bayer * pool.scale + pool.offset;
                    let result = pixel.simd_lt(threshold).to_bitmask();

                    for lane in 0..pool.size {
                        if result & (1 << lane) != 0 {
                            color = self.config.map[iter * SIMD_LANES + lane].color;
                            break 'compute;
                        }
                    }
                }

                output[pixel_idx] = color;
                pixel_idx += 1;
            }
        }
    }
}

/// Constrained SIMD accelerated bayer dithering transform,
/// parallelized by image rows.
///
/// See [SimdBayerFixed].
struct SimdFixedPar<const SIMD_LANES: usize>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fixed: SimdFixed<SIMD_LANES>,
}

impl<const SIMD_LANES: usize> SimdFixedPar<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn new(config: BayerConfig) -> Self {
        Self {
            fixed: SimdFixed::new(config),
        }
    }
}

impl<'a, const SIMD_LANES: usize> Transform<BayerArgs<'a>> for SimdFixedPar<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.fixed.config.map.last().unwrap().color;
        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let idx_offset = y * width;
                for (x, col) in row.iter_mut().enumerate() {
                    let pixel_idx = idx_offset + x;
                    let bayer_value = self.fixed.bayer[self.fixed.config.bayer_idx(x, y)];

                    let pixel = std::simd::Simd::<f32, SIMD_LANES>::splat(input[pixel_idx]);
                    let bayer = std::simd::Simd::<f32, SIMD_LANES>::splat(bayer_value);

                    let threshold = bayer * self.fixed.scale + self.fixed.offset;
                    let result = pixel.simd_lt(threshold).to_bitmask();

                    let mut color: utils::pixel::RGB = fallback_color;
                    for (lane, map) in self.fixed.config.map.iter().enumerate() {
                        if result & (1 << lane) != 0 {
                            color = map.color;
                            break;
                        }
                    }
                    *col = color;
                }
            });
    }
}

/// Flexible SIMD accelerated bayer dithering transform,
/// parallelized by image rows.
///
/// See [SimdBayerFit].
struct SimdFitPar<const SIMD_LANES: usize>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fit: SimdFit<SIMD_LANES>,
}

impl<const SIMD_LANES: usize> SimdFitPar<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn new(config: BayerConfig) -> Self {
        Self {
            fit: SimdFit::new(config),
        }
    }
}

impl<'a, const SIMD_LANES: usize> Transform<BayerArgs<'a>> for SimdFitPar<SIMD_LANES>
where
    std::simd::LaneCount<SIMD_LANES>: std::simd::SupportedLaneCount,
{
    fn apply(&mut self, rhs: &mut BayerArgs<'a>) {
        let width = rhs.input.width() as usize;
        let input = rhs.input.as_ref();
        let output = rhs.output.as_mut();

        let fallback_color = self.fit.config.map.last().unwrap().color;
        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let idx_offset = y * width;
                for (x, col) in row.iter_mut().enumerate() {
                    let pixel_idx = idx_offset + x;
                    let bayer_value = self.fit.bayer[self.fit.config.bayer_idx(x, y)];

                    let pixel = std::simd::Simd::<f32, SIMD_LANES>::splat(input[pixel_idx]);
                    let bayer = std::simd::Simd::<f32, SIMD_LANES>::splat(bayer_value);

                    let mut color: utils::pixel::RGB = fallback_color;
                    'compute: for iter in 0..self.fit.compute_iters {
                        let pool = &self.fit.simd[iter];
                        let threshold = bayer * pool.scale + pool.offset;
                        let result = pixel.simd_lt(threshold).to_bitmask();

                        for lane in 0..pool.size {
                            if result & (1 << lane) != 0 {
                                color = self.fit.config.map[iter * SIMD_LANES + lane].color;
                                break 'compute;
                            }
                        }
                    }

                    *col = color;
                }
            });
    }
}
