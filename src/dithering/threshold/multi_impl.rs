use multiversion::multiversion;
use rayon::prelude::*;
use std::simd::cmp::SimdPartialOrd;

use crate::{
    dithering::threshold::threshold_transform::{SimdFitPassData, ThresholdConfig},
    utils,
};

// Simd alias for f32 transform implementations
type Simd<const SIMD_LANES: usize> = std::simd::Simd<f32, SIMD_LANES>;

#[multiversion(targets("x86_64+avx512f", "x86_64+avx2", "x86_64+sse2"))]
pub fn scalar_par(
    in_buf: &[f32],
    in_shape: (usize, usize),
    out_buf: &mut [utils::pixel::RGB],
    tiled: &[f32],
    config: &ThresholdConfig,
) {
    let (width, _) = in_shape;
    let fallback_color = config.map.last().unwrap().color;
    out_buf
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let tiled_row_idx = y & config.side_mask;
            let tiled_start = tiled_row_idx << config.order;
            let tiled_row = &tiled[tiled_start..tiled_start + width];

            let idx_offset = y * width;
            for (x, col) in row.iter_mut().enumerate() {
                let pixel_idx = idx_offset + x;
                let value = tiled_row[x];

                let mut color: utils::pixel::RGB = fallback_color;
                for map in &config.map {
                    if in_buf[pixel_idx] < value * map.scale + map.offset {
                        color = map.color;
                        break;
                    }
                }

                *col = color;
            }
        });
}

#[multiversion(targets("x86_64+avx512f", "x86_64+avx2", "x86_64+sse2"))]
pub fn fixed_par<const LANES: usize>(
    in_buf: &[f32],
    in_shape: (usize, usize),
    out_buf: &mut [utils::pixel::RGB],
    tiled: &[f32],
    config: &ThresholdConfig,
    scale: Simd<LANES>,
    offset: Simd<LANES>,
) where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    let (width, _) = in_shape;
    let fallback_color = config.map.last().unwrap().color;
    out_buf
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let tiled_row_idx = y & config.side_mask;
            let tiled_start = tiled_row_idx << config.order;
            let tiled_row = &tiled[tiled_start..tiled_start + width];

            let idx_offset = y * width;
            for (x, col) in row.iter_mut().enumerate() {
                let pixel_idx = idx_offset + x;

                let pixel = Simd::<LANES>::splat(in_buf[pixel_idx]);
                let value = Simd::<LANES>::splat(tiled_row[x]);

                let threshold = value * scale + offset;
                let result = pixel.simd_lt(threshold).to_bitmask();

                let mut color: utils::pixel::RGB = fallback_color;
                for (lane, map) in config.map.iter().enumerate() {
                    if result & (1 << lane) != 0 {
                        color = map.color;
                        break;
                    }
                }

                *col = color;
            }
        });
}

#[multiversion(targets("x86_64+avx512f", "x86_64+avx2", "x86_64+sse2"))]
pub fn fit_par<const LANES: usize>(
    in_buf: &[f32],
    in_shape: (usize, usize),
    out_buf: &mut [utils::pixel::RGB],
    tiled: &[f32],
    config: &ThresholdConfig,
    // this is not ok, most likely not aligned
    pass_data: &[SimdFitPassData<LANES>],
    compute_iters: usize,
) where
    std::simd::LaneCount<LANES>: std::simd::SupportedLaneCount,
{
    let (width, _) = in_shape;
    let fallback_color = config.map.last().unwrap().color;
    out_buf
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let tiled_row_idx = y & config.side_mask;
            let tiled_start = tiled_row_idx << config.order;
            let tiled_row = &tiled[tiled_start..tiled_start + width];

            let idx_offset = y * width;
            for (x, col) in row.iter_mut().enumerate() {
                let pixel_idx = idx_offset + x;

                let pixel = Simd::<LANES>::splat(in_buf[pixel_idx]);
                let value = Simd::<LANES>::splat(tiled_row[x]);

                let mut color: utils::pixel::RGB = fallback_color;
                'compute: for iter in 0..compute_iters {
                    let pool = &pass_data[iter];
                    let threshold = value * pool.scale + pool.offset;
                    let result = pixel.simd_lt(threshold).to_bitmask();

                    let color_offset = iter * LANES;
                    for lane in 0..pool.size {
                        if result & (1 << lane) != 0 {
                            color = config.map[color_offset + lane].color;
                            break 'compute;
                        }
                    }
                }

                *col = color;
            }
        });
}
