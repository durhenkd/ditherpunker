#![feature(portable_simd)]
use std::hint::black_box;

use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main, measurement::WallTime,
};

pub(crate) mod utils;
use ditherpunker::{
    prelude::{GrayscaleTransform, TextureTransform},
    texture::{Shape, Texture, TextureMutSlice, TextureRef, TextureSlice},
};
use ditherpunker_macros::simd_targets;
use image::buffer::ConvertBuffer;

// compute grayscale using the image::ImageBuffer struct.
// this creates new vec, so it's not suitable for repeated use
struct ImBufGrayscale {}

impl TextureTransform for ImBufGrayscale {
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
        let im_buf = image::ImageBuffer::<image::Rgb<u8>, &[u8]>::from_raw(
            input.width(),
            input.height(),
            input.as_ref(),
        )
        .unwrap();

        // this allocates a vec for each transform
        let grayscale: image::ImageBuffer<image::Luma<f32>, Vec<f32>> = im_buf.convert();
        output.as_mut().copy_from_slice(grayscale.as_raw());

        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

// simple grayscale computation by averaging each plane.
// this is not a correct luminance representation since
// all planes are weighted the same.
struct SimpleScalarGraysale {}

impl TextureTransform for SimpleScalarGraysale {
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
        let out_buf = output.as_mut();
        input
            .as_ref()
            .chunks_exact(3)
            .enumerate()
            .for_each(|(idx, pixel)| {
                out_buf[idx] = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
            });
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

// targetted dispatcher of SimpleScalarGrayscale for ScalarTargetGraysale
#[simd_targets]
fn scalar_impl(in_buf: &[u8], out_buf: &mut [f32]) {
    in_buf.chunks_exact(3).enumerate().for_each(|(idx, pixel)| {
        out_buf[idx] = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
    });
}

// targetted version of SimpleScalarGrayscale
struct SimpleScalarTargetGraysale {}

impl TextureTransform for SimpleScalarTargetGraysale {
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

/// Coefficients to transform from sRGB to a CIE Y (luminance) value.
const SRGB_LUMA: [u32; 3] = [2126, 7152, 722];
const SRGB_LUMA_DIV: u32 = 10000;

/// F32 coefficients for faster computation (same values as integer version)
const SRGB_LUMA_F32: [f32; 3] = [0.2126, 0.7152, 0.0722];

// luminance correct version of SimpleScalarGrayscale
struct ScalarUpperGrayscale {}

impl TextureTransform for ScalarUpperGrayscale {
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
        let out_buf = output.as_mut();
        input
            .as_ref()
            .chunks_exact(3)
            .enumerate()
            .for_each(|(idx, pixel)| {
                let pix_value = (pixel[0] as u32 * SRGB_LUMA[0]
                    + pixel[1] as u32 * SRGB_LUMA[1]
                    + pixel[2] as u32 * SRGB_LUMA[2])
                    / SRGB_LUMA_DIV;
                out_buf[idx] = pix_value as f32 / 255.0;
            });
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

// targetted dispatcher of SimpleUpperGrayscale for ScalarUpperTargetGrayscale
#[simd_targets]
fn upper_impl(in_buf: &[u8], out_buf: &mut [f32]) {
    in_buf.chunks_exact(3).enumerate().for_each(|(idx, pixel)| {
        let pix_value = (pixel[0] as u32 * SRGB_LUMA[0]
            + pixel[1] as u32 * SRGB_LUMA[1]
            + pixel[2] as u32 * SRGB_LUMA[2])
            / SRGB_LUMA_DIV;
        out_buf[idx] = pix_value as f32 / 255.0;
    });
}

// targetted, luminance correct version of SimpleScalarTargetGrayscale
struct ScalarUpperTargetGrayscale {}

impl TextureTransform for ScalarUpperTargetGrayscale {
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
        upper_impl(input.as_ref(), output.as_mut());
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

// ScalarUpperGrayscale using f32 types instead of u32
struct ScalarRFCF32Grayscale {}

impl TextureTransform for ScalarRFCF32Grayscale {
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
        let out_buf = output.as_mut();
        input
            .as_ref()
            .chunks_exact(3)
            .enumerate()
            .for_each(|(idx, pixel)| {
                let pix_value = (pixel[0] as f32 * SRGB_LUMA_F32[0]
                    + pixel[1] as f32 * SRGB_LUMA_F32[1]
                    + pixel[2] as f32 * SRGB_LUMA_F32[2])
                    / 255.0;
                out_buf[idx] = pix_value;
            });
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
    let image: image::DynamicImage = utils::read_test_image(size);
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
    let mut group = c.benchmark_group("grayscale_transform");

    let sizes = [100u32, 300, 500];
    bench_by_param(&mut group, &mut ImBufGrayscale {}, "imbuf", &sizes);
    bench_by_param(&mut group, &mut SimpleScalarGraysale {}, "scalar", &sizes);
    bench_by_param(
        &mut group,
        &mut SimpleScalarTargetGraysale {},
        "scalar-impl",
        &sizes,
    );
    bench_by_param(
        &mut group,
        &mut ScalarUpperGrayscale {},
        "scalar-upper",
        &sizes,
    );
    bench_by_param(
        &mut group,
        &mut ScalarUpperTargetGrayscale {},
        "scalar-upper-impl",
        &sizes,
    );
    bench_by_param(
        &mut group,
        &mut ScalarRFCF32Grayscale {},
        "scalar-rfc-f32",
        &sizes,
    );
    bench_by_param(
        &mut group,
        &mut GrayscaleTransform::Seq.build(),
        "scalar-rfc-f32-impl",
        &sizes,
    );

    group.finish();
}

criterion_group!(grayscale_transform, criterion_benchmark);
criterion_main!(grayscale_transform);
