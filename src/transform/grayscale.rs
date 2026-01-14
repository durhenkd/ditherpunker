use multiversion::multiversion;

use crate::{
    prelude::TextureTransform,
    texture::{Shape, Shape2D, TextureMutSlice, TextureRef, TextureSlice},
};

pub enum GrayscaleTransform {
    Seq,
    Par,
}

impl GrayscaleTransform {
    pub fn auto(shape_hint: Shape2D) -> Self {
        let (width, height) = shape_hint;
        let count = width * height;

        if width < 450 || count < 202500 {
            return GrayscaleTransform::Seq;
        }
        GrayscaleTransform::Par
    }

    pub fn build(&self) -> impl TextureTransform<Input = u8, Output = f32> {
        match self {
            GrayscaleTransform::Seq => GrayscaleTransformImpl::Seq(GrayscaleSeq {}),
            GrayscaleTransform::Par => GrayscaleTransformImpl::Par(GrayscalePar {}),
        }
    }
}

enum GrayscaleTransformImpl {
    Seq(GrayscaleSeq),
    Par(GrayscalePar),
}

impl TextureTransform for GrayscaleTransformImpl {
    type Input = u8;
    type Output = f32;

    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        match self {
            GrayscaleTransformImpl::Seq(t) => t.apply(input, output),
            GrayscaleTransformImpl::Par(t) => t.apply(input, output),
        }
    }

    fn prepare(&mut self, in_shape: Shape, out_shape: Shape) {
        match self {
            GrayscaleTransformImpl::Seq(t) => t.prepare(in_shape, out_shape),
            GrayscaleTransformImpl::Par(t) => t.prepare(in_shape, out_shape),
        };
    }
}

struct GrayscaleSeq {}

impl TextureTransform for GrayscaleSeq {
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
        scalar_impl(input.as_ref(), output.as_mut(), input.planes() as usize);
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

struct GrayscalePar {}

impl TextureTransform for GrayscalePar {
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
        scalar_par_impl(input.as_ref(), output.as_mut(), input.shape());
        (input, output)
    }

    fn prepare(&mut self, _: Shape, _: Shape) {}
}

const SRGB_LUMA_F32: [f32; 3] = [0.2126, 0.7152, 0.0722];

#[multiversion(targets("x86_64+avx512f", "x86_64+avx2", "x86_64+sse2"))]
fn scalar_impl(in_buf: &[u8], out_buf: &mut [f32], planes: usize) {
    debug_assert!(planes == 3 || planes == 4);
    in_buf
        .chunks_exact(planes)
        .enumerate()
        .for_each(|(idx, pixel)| {
            let pix_value = (pixel[0] as f32 * SRGB_LUMA_F32[0]
                + pixel[1] as f32 * SRGB_LUMA_F32[1]
                + pixel[2] as f32 * SRGB_LUMA_F32[2])
                / 255.0;
            out_buf[idx] = pix_value;
        });
}

#[multiversion(targets("x86_64+avx512f", "x86_64+avx2", "x86_64+sse2"))]
fn scalar_par_impl(in_buf: &[u8], out_buf: &mut [f32], shape: Shape) {
    use rayon::prelude::*;

    let (width, _, planes) = shape;
    debug_assert!(planes == 3 || planes == 4);
    out_buf
        .par_chunks_exact_mut(width)
        .zip(in_buf.par_chunks_exact(width * planes))
        .for_each(|(out_row, in_row)| {
            out_row
                .iter_mut()
                .zip(in_row.chunks_exact(planes))
                .for_each(|(out_pixel, in_pixel)| {
                    *out_pixel = (in_pixel[0] as f32 * SRGB_LUMA_F32[0]
                        + in_pixel[1] as f32 * SRGB_LUMA_F32[1]
                        + in_pixel[2] as f32 * SRGB_LUMA_F32[2])
                        / 255.0;
                });
        });
}
