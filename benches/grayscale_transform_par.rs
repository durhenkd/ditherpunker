#![feature(portable_simd)]
use std::hint::black_box;

use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
pub(crate) mod utils;

use ditherpunker::{
    prelude::{GrayscaleTransform, TextureTransform},
    texture::{Shape, Shape2D, Texture, TextureMutSlice, TextureRef, TextureSlice},
};
use ditherpunker_macros::simd_targets;
use rayon::prelude::*;

const SRGB_LUMA_F32: [f32; 3] = [0.2126, 0.7152, 0.0722];

#[simd_targets]
fn scalar_impl(in_buf: &[u8], out_buf: &mut [f32]) {
    in_buf.chunks_exact(3).enumerate().for_each(|(idx, pixel)| {
        let pix_value = (pixel[0] as f32 * SRGB_LUMA_F32[0]
            + pixel[1] as f32 * SRGB_LUMA_F32[1]
            + pixel[2] as f32 * SRGB_LUMA_F32[2])
            / 255.0;
        out_buf[idx] = pix_value;
    });
}

#[simd_targets]
fn scalar_par_impl(in_buf: &[u8], out_buf: &mut [f32], shape: Shape2D) {
    let (width, _) = shape;
    out_buf
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(row_idx, out_row)| {
            let row_start = row_idx * width;
            in_buf[row_start..row_start + width]
                .chunks_exact(3)
                .enumerate()
                .for_each(|(idx, pixel)| {
                    let pix_value = (pixel[0] as f32 * SRGB_LUMA_F32[0]
                        + pixel[1] as f32 * SRGB_LUMA_F32[1]
                        + pixel[2] as f32 * SRGB_LUMA_F32[2])
                        / 255.0;
                    out_row[idx] = pix_value;
                });
        });
}

struct ScalarGrayscale {}

impl TextureTransform for ScalarGrayscale {
    type Input = u8;
    type Output = f32;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        mut output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        scalar_impl(input.as_ref(), output.as_mut());
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

struct ScalarParGrayscale {}

impl TextureTransform for ScalarParGrayscale {
    type Input = u8;
    type Output = f32;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        mut output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        scalar_par_impl(input.as_ref(), output.as_mut(), input.shape_2d());
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

fn bench_transform(
    group: &mut BenchmarkGroup<'_, WallTime>,
    transform: &mut impl TextureTransform<Input = u8, Output = f32>,
    name: &str,
    size: u32,
) {
    let image = utils::read_test_image(size);
    let image = image.to_rgb8();

    let mut grayscale = black_box(Texture::<f32>::new(size, size, 1));

    group.bench_with_input(BenchmarkId::new(name, size), &size, |b, _| {
        let input = TextureSlice::from_image_buffer(&image);

        b.iter(|| {
            let res = transform.apply(input, grayscale.as_texture_mut_slice());
            black_box(res);
        });
    });
}

fn bench_by_param(
    group: &mut BenchmarkGroup<'_, WallTime>,
    transform: &mut impl TextureTransform<Input = u8, Output = f32>,
    name: &str,
    sizes: &[u32],
) {
    for size in sizes {
        bench_transform(group, transform, name, *size);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("grayscale_transform_par");

    let sizes = [100u32, 400, 450, 500, 600];
    bench_by_param(&mut group, &mut ScalarGrayscale {}, "scalar", &sizes);
    bench_by_param(&mut group, &mut ScalarParGrayscale {}, "scalar_par", &sizes);
    bench_by_param(
        &mut group,
        &mut GrayscaleTransform::Par.build(),
        "scalar_par_zip",
        &sizes,
    );

    group.finish();
}

criterion_group!(grayscale_transform_par, criterion_benchmark);
criterion_main!(grayscale_transform_par);
