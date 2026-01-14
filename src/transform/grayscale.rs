use ditherpunker_macros::simd_targets;

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

#[simd_targets]
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

#[simd_targets]
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

#[cfg(test)]
mod tests {
    use super::{SRGB_LUMA_F32, scalar_impl, scalar_par_impl};

    #[test]
    fn test_scalar_impl_rgb() {
        // Test with RGB (3 planes)
        let in_buf = vec![
            255, 0, 0, // Red pixel
            0, 255, 0, // Green pixel
            0, 0, 255, // Blue pixel
            128, 128, 128, // Gray pixel
        ];
        let mut out_buf = vec![0.0; 4];

        scalar_impl(&in_buf, &mut out_buf, 3);

        // Expected values based on SRGB_LUMA_F32 weights
        let expected = [
            SRGB_LUMA_F32[0], // Red
            SRGB_LUMA_F32[1], // Green
            SRGB_LUMA_F32[2], // Blue
            (128.0 * SRGB_LUMA_F32[0] + 128.0 * SRGB_LUMA_F32[1] + 128.0 * SRGB_LUMA_F32[2])
                / 255.0, // Gray
        ];

        for (i, (&result, &expected)) in out_buf.iter().zip(expected.iter()).enumerate() {
            assert!(
                (result - expected).abs() < 1e-6,
                "Pixel {} mismatch: got {}, expected {}",
                i,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_scalar_impl_rgba() {
        // Test with RGBA (4 planes) - alpha should be ignored
        let in_buf = vec![
            255, 0, 0, 255, // Red pixel with full alpha
            0, 255, 0, 128, // Green pixel with half alpha
            0, 0, 255, 0, // Blue pixel with zero alpha
        ];
        let mut out_buf = vec![0.0; 3];

        scalar_impl(&in_buf, &mut out_buf, 4);

        // Alpha should not affect the grayscale calculation
        let expected = [
            SRGB_LUMA_F32[0], // Red
            SRGB_LUMA_F32[1], // Green
            SRGB_LUMA_F32[2], // Blue
        ];

        for (i, (&result, &expected)) in out_buf.iter().zip(expected.iter()).enumerate() {
            assert!(
                (result - expected).abs() < 1e-6,
                "Pixel {} mismatch: got {}, expected {}",
                i,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_scalar_par_impl_rgb() {
        // Test with RGB (3 planes) in parallel mode
        let in_buf = vec![
            255, 0, 0, 0, 255, 0, // Row 1: Red, Green
            0, 0, 255, 128, 128, 128, // Row 2: Blue, Gray
        ];
        let mut out_buf = vec![0.0; 4];

        scalar_par_impl(&in_buf, &mut out_buf, (2, 2, 3));

        let expected = [
            SRGB_LUMA_F32[0], // Red
            SRGB_LUMA_F32[1], // Green
            SRGB_LUMA_F32[2], // Blue
            (128.0 * SRGB_LUMA_F32[0] + 128.0 * SRGB_LUMA_F32[1] + 128.0 * SRGB_LUMA_F32[2])
                / 255.0, // Gray
        ];

        for (i, (&result, &expected)) in out_buf.iter().zip(expected.iter()).enumerate() {
            assert!(
                (result - expected).abs() < 1e-6,
                "Pixel {} mismatch: got {}, expected {}",
                i,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_scalar_par_impl_rgba() {
        // Test with RGBA (4 planes) in parallel mode
        let in_buf = vec![
            255, 0, 0, 255, 0, 255, 0, 128, // Row 1: Red, Green
            0, 0, 255, 0, 64, 64, 64, 255, // Row 2: Blue, Dark gray
        ];
        let mut out_buf = vec![0.0; 4];

        scalar_par_impl(&in_buf, &mut out_buf, (2, 2, 4));

        let expected = [
            SRGB_LUMA_F32[0], // Red
            SRGB_LUMA_F32[1], // Green
            SRGB_LUMA_F32[2], // Blue
            (64.0 * SRGB_LUMA_F32[0] + 64.0 * SRGB_LUMA_F32[1] + 64.0 * SRGB_LUMA_F32[2]) / 255.0, // Dark gray
        ];

        for (i, (&result, &expected)) in out_buf.iter().zip(expected.iter()).enumerate() {
            assert!(
                (result - expected).abs() < 1e-6,
                "Pixel {} mismatch: got {}, expected {}",
                i,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_scalar_impl_vs_scalar_par_impl() {
        // Verify both implementations produce identical results
        let in_buf = vec![
            255, 128, 64, 192, 96, 48, // Row 1
            32, 160, 224, 80, 176, 112, // Row 2
            144, 208, 16, 240, 120, 200, // Row 3
        ];
        let mut out_buf_seq = vec![0.0; 6];
        let mut out_buf_par = vec![0.0; 6];

        scalar_impl(&in_buf, &mut out_buf_seq, 3);
        scalar_par_impl(&in_buf, &mut out_buf_par, (2, 3, 3));

        for (i, (&seq, &par)) in out_buf_seq.iter().zip(out_buf_par.iter()).enumerate() {
            assert!(
                (seq - par).abs() < 1e-6,
                "Pixel {} mismatch between seq and par: seq={}, par={}",
                i,
                seq,
                par
            );
        }
    }
}
