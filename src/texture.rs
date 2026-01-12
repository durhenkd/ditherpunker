use image::ImageBuffer;
use std::path::Path;

use crate::utils::buffer::uninitialized_buffer;

pub type TextureShape = (usize, usize);

/// Trait defining ops available on Textures with
/// lendable inner buffer
pub trait TextureRef: AsRef<[Self::Inner]> {
    type Inner;

    fn width(&self) -> u32;
    fn height(&self) -> u32;

    #[inline]
    fn shape(&self) -> TextureShape {
        (self.width() as usize, self.height() as usize)
    }
}

/// Trait defining ops available on mutable
/// Textures
pub trait TextureMut: TextureRef + AsMut<[Self::Inner]> {}

/// Texture with owned buffer.
#[derive(Debug, Clone)]
pub struct Texture<T> {
    width: u32,
    height: u32,
    buffer: Vec<T>,
}

impl<T> AsRef<[T]> for Texture<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.buffer
    }
}

impl<T> AsMut<[T]> for Texture<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.buffer
    }
}

impl<T> TextureRef for Texture<T> {
    type Inner = T;

    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }
}

impl<T> TextureMut for Texture<T> {}

impl<T> Texture<T> {
    /// # Safety
    ///
    /// Make sure the texture is initialized before usage.
    pub unsafe fn new_uninitialized(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: unsafe { uninitialized_buffer((width * height) as usize) },
        }
    }

    pub fn as_texture_slice<'s>(&'s self) -> TextureSlice<'s, T> {
        TextureSlice {
            width: self.width,
            height: self.height,
            buffer: &self.buffer,
        }
    }

    pub fn as_texture_mut_slice<'s>(&'s mut self) -> TextureMutSlice<'s, T> {
        TextureMutSlice {
            width: self.width,
            height: self.height,
            buffer: &mut self.buffer,
        }
    }
}

impl<T: std::clone::Clone> Texture<T> {
    pub fn from_slice(width: u32, height: u32, slice: &[T]) -> Self {
        assert_eq!(
            slice.len(),
            (width * height) as usize,
            "buffers don't match sizes"
        );
        Texture {
            width,
            height,
            buffer: slice.to_owned(),
        }
    }
}

impl<T: Default + Copy> Texture<T> {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![T::default(); (width * height) as usize],
        }
    }
}

impl<T: std::marker::Copy> Texture<T> {
    /// # Panics
    /// This function will panic if the two slices have different lengths.
    pub fn copy_from_slice(&mut self, slice: &[T]) {
        self.buffer.copy_from_slice(slice);
    }
}

impl Texture<f32> {
    pub fn from_luma32f_image<P: AsRef<Path>>(path: P) -> crate::error::Result<Self> {
        // pixel transformations from image crate assume linear RGB space
        // color space coefficients follow a newer color spec:
        // floats:      [0.2126, 0.7152, 0.0722]    image-0.25.9/src/images/buffer.rs:1577
        // integral:    [2126, 7152, 722]           image-0.25.9/src/color.rs:602
        let image = image::ImageReader::open(path)?.decode()?.to_luma32f();
        // SAFETY: data is immediately copied over the uninitialized buffer
        let mut texture = unsafe { Self::new_uninitialized(image.width(), image.height()) };
        texture.copy_from_slice(image.into_flat_samples().as_slice());
        Ok(texture)
    }
}

impl Texture<u8> {
    pub fn from_luma8_image<P: AsRef<Path>>(path: P) -> crate::error::Result<Self> {
        // pixel transformations from image crate assume linear RGB space
        // color space coefficients follow a newer color spec:
        // floats:      [0.2126, 0.7152, 0.0722]    image-0.25.9/src/images/buffer.rs:1577
        // integral:    [2126, 7152, 722]           image-0.25.9/src/color.rs:602
        let image = image::ImageReader::open(path)?.decode()?.to_luma8();
        // SAFETY: data is immediately copied over the uninitialized buffer
        let mut texture = unsafe { Self::new_uninitialized(image.width(), image.height()) };
        texture.copy_from_slice(image.into_flat_samples().as_slice());
        Ok(texture)
    }
}

impl Texture<crate::utils::pixel::RGB> {
    pub fn from_rgba8_image<P: AsRef<Path>>(path: P) -> crate::error::Result<Self> {
        let image = image::ImageReader::open(path)?.decode()?.to_rgba8();
        let mut texture = unsafe { Self::new_uninitialized(image.width(), image.height()) };
        image.pixels().enumerate().for_each(|(idx, pixel)| {
            texture.buffer[idx] = crate::utils::pixel::RGB::from_u8_array(&pixel.0);
        });
        Ok(texture)
    }
}

impl<T: image::Primitive> From<ImageBuffer<image::Luma<T>, Vec<T>>> for Texture<T> {
    fn from(value: ImageBuffer<image::Luma<T>, Vec<T>>) -> Self {
        Texture::from_slice(
            value.width(),
            value.height(),
            value.into_flat_samples().as_slice(),
        )
    }
}

/// Texture with borrowed internal buffer
#[derive(Debug, Copy, Clone)]
pub struct TextureSlice<'a, T> {
    width: u32,
    height: u32,
    buffer: &'a [T],
}

impl<T> AsRef<[T]> for TextureSlice<'_, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.buffer
    }
}

impl<T> TextureRef for TextureSlice<'_, T> {
    type Inner = T;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl<'a, T> TextureSlice<'a, T> {
    pub fn new(width: u32, height: u32, buffer: &'a [T]) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }
}

#[derive(Debug)]
pub struct TextureMutSlice<'a, T> {
    width: u32,
    height: u32,
    buffer: &'a mut [T],
}

impl<'a, T> AsRef<[T]> for TextureMutSlice<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.buffer
    }
}

impl<'a, T> AsMut<[T]> for TextureMutSlice<'a, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.buffer
    }
}

impl<T> TextureRef for TextureMutSlice<'_, T> {
    type Inner = T;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl<T> TextureMut for TextureMutSlice<'_, T> {}

impl<'a, T> TextureMutSlice<'a, T> {
    pub fn new(width: u32, height: u32, buffer: &'a mut [T]) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }
}

// trait WriteLumaPng<P, T>: TextureOps<Inner = T>
// where
//     P: image::Pixel + image::PixelWithColorType,
//     T: image::Primitive,
//     // ImageBuffer::from_raw
//     for<'a> &'a [T]: Deref<Target = [P::Subpixel]>,
//     // write_with_encoder
//     [<P as image::Pixel>::Subpixel]: image::EncodableLayout,
// {
//     fn write_luma_png(
//         &self,
//         path: impl AsRef<Path>,
//         compression: CompressionType,
//         filtering: FilterType,
//     ) -> crate::error::Result {
//         let image_buf =
//             ImageBuffer::<P, &[T]>::from_raw(self.width(), self.height(), self.as_ref())
//                 .expect("width and height do not match buffer size");
//         let file = &mut std::io::BufWriter::new(std::fs::File::create(path)?);
//         let encoder = PngEncoder::new_with_quality(file, compression, filtering);
//         image_buf.write_with_encoder(encoder)?;
//         Ok(())
//     }
// }

// impl WriteLumaPng<image::Luma<u8>, u8> for TextureRef<'_, u8> {}
// impl WriteLumaPng<image::Rgba<u8>, u8> for TextureRef<'_, u8> {}

pub mod prelude {
    pub use super::{Texture, TextureMut, TextureMutSlice, TextureRef, TextureSlice};
}
