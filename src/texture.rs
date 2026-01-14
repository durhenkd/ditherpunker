use image::{DynamicImage, ImageBuffer};
use std::{ops::Deref, path::Path};

use crate::{
    error::{DitherpunkerError, Result},
    utils::{self, buffer::uninitialized_buffer, image::write_png_buf},
};

pub type Shape2D = (usize, usize);
pub type Shape = (usize, usize, usize);

/// Trait defining ops available on Textures with
/// lendable inner buffer
pub trait TextureRef: AsRef<[Self::Inner]> {
    type Inner;

    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn planes(&self) -> u32;

    #[inline]
    fn shape_2d(&self) -> Shape2D {
        (self.width() as usize, self.height() as usize)
    }

    #[inline]
    fn shape(&self) -> Shape {
        (
            self.width() as usize,
            self.height() as usize,
            self.planes() as usize,
        )
    }

    fn split_by_planes(self) -> Vec<Texture<Self::Inner>>
    where
        Self: Sized,
        Self::Inner: Copy,
    {
        let planes = self.planes() as usize;
        let mut textures = Vec::<Texture<Self::Inner>>::with_capacity(planes);
        for plane in 0..planes {
            // SAFETY: texture is immediately initialized via deinterleave write
            let mut texture = unsafe {
                Texture::<Self::Inner>::new_uninitialized(self.width(), self.height(), 1)
            };
            self.as_ref()
                .iter()
                .skip(plane)
                .step_by(planes)
                .zip(texture.as_mut().iter_mut())
                .for_each(|(src, dst)| *dst = *src);
            textures.push(texture);
        }
        textures
    }

    fn merge_planes(planes: Vec<Texture<Self::Inner>>) -> Texture<Self::Inner>
    where
        Self: Sized,
        Self::Inner: Copy,
    {
        assert!(!planes.is_empty(), "Cannot merge empty plane vector");

        let width = planes[0].width();
        let height = planes[0].height();
        let num_planes = planes.len() as u32;

        // Verify all planes have the same dimensions
        assert!(
            planes
                .iter()
                .all(|p| p.width() == width && p.height() == height && p.planes() == 1),
            "All planes must have the same dimensions and be single-plane textures"
        );

        // SAFETY: texture is immediately initialized via interleaved write
        let mut merged =
            unsafe { Texture::<Self::Inner>::new_uninitialized(width, height, num_planes) };

        let dst = merged.as_mut();
        for (plane_idx, plane) in planes.iter().enumerate() {
            plane
                .as_ref()
                .iter()
                .enumerate()
                .for_each(|(pixel_idx, &value)| {
                    let dst_idx = pixel_idx * planes.len() + plane_idx;
                    dst[dst_idx] = value;
                });
        }

        merged
    }

    /// Interpret buffer as arbitrary pixel of Self::Inner and write it to png image.
    fn write_png_with_pixel<Pixel>(
        &self,
        path: &Path,
        compression: image::codecs::png::CompressionType,
        filtering: image::codecs::png::FilterType,
    ) -> Result
    where
        Pixel: image::Pixel<Subpixel = Self::Inner> + image::PixelWithColorType,
        Self::Inner: image::Primitive + image::Enlargeable,
        [Self::Inner]: image::EncodableLayout,
    {
        write_png_buf::<Pixel, Self::Inner>(
            self.as_ref(),
            self.shape_2d(),
            path,
            compression,
            filtering,
        )
    }

    /// Runtime dispather of write_png_with_pixel for appropiate pixel representations
    /// depending on the number of planes.
    ///
    /// ```
    /// 1 -> Luma
    /// 2 -> LumaA
    /// 3 -> Rgb
    /// 4 -> Rgba
    /// ```
    ///
    /// Traits constraints ensure only the subset implemented by image crate allow
    /// this implementation. (i.e. Luma<f32> cannot be as image directly, while Rgb<f32> can).
    fn write_png(
        &self,
        path: &Path,
        compression: image::codecs::png::CompressionType,
        filtering: image::codecs::png::FilterType,
    ) -> Result
    where
        Self::Inner: image::Primitive + image::Enlargeable,
        [Self::Inner]: image::EncodableLayout,
        // per pixel type asserts
        image::Luma<Self::Inner>: image::Pixel<Subpixel = Self::Inner> + image::PixelWithColorType,
        image::LumaA<Self::Inner>: image::Pixel<Subpixel = Self::Inner> + image::PixelWithColorType,
        image::Rgb<Self::Inner>: image::Pixel<Subpixel = Self::Inner> + image::PixelWithColorType,
        image::Rgba<Self::Inner>: image::Pixel<Subpixel = Self::Inner> + image::PixelWithColorType,
    {
        match self.planes() {
            1 => {
                self.write_png_with_pixel::<image::Luma<Self::Inner>>(path, compression, filtering)
            }
            2 => {
                self.write_png_with_pixel::<image::LumaA<Self::Inner>>(path, compression, filtering)
            }
            3 => self.write_png_with_pixel::<image::Rgb<Self::Inner>>(path, compression, filtering),
            4 => {
                self.write_png_with_pixel::<image::Rgba<Self::Inner>>(path, compression, filtering)
            }
            _ => Err(DitherpunkerError::ImageEncode(image::ImageError::Encoding(
                image::error::EncodingError::new(
                    image::error::ImageFormatHint::Exact(image::ImageFormat::Png),
                    DitherpunkerError::String(format!(
                        "unsupported plane count: {}",
                        self.planes()
                    )),
                ),
            ))),
        }
    }
}

/// Trait defining ops available on mutable Textures
pub trait TextureMut: TextureRef + AsMut<[Self::Inner]> {}

/// Texture with owned buffer.
#[derive(Debug, Clone)]
pub struct Texture<T> {
    buffer: Vec<T>,
    width: u32,
    height: u32,
    planes: u32,
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

    #[inline]
    fn planes(&self) -> u32 {
        self.planes
    }
}

impl<T> TextureMut for Texture<T> {}

impl<T> Texture<T> {
    /// # Safety
    ///
    /// Make sure the texture is initialized before usage.
    ///
    /// > planes is assumed to be 1
    pub unsafe fn new_uninitialized(width: u32, height: u32, planes: u32) -> Self {
        Self {
            width,
            height,
            buffer: unsafe { uninitialized_buffer((width * height * planes) as usize) },
            planes: 1,
        }
    }

    pub fn as_texture_slice<'s>(&'s self) -> TextureSlice<'s, T> {
        TextureSlice {
            buffer: &self.buffer,
            width: self.width,
            height: self.height,
            planes: self.planes,
        }
    }

    pub fn as_texture_mut_slice<'s>(&'s mut self) -> TextureMutSlice<'s, T> {
        TextureMutSlice {
            buffer: &mut self.buffer,
            width: self.width,
            height: self.height,
            planes: self.planes,
        }
    }
}

impl<T: std::clone::Clone> Texture<T> {
    pub fn from_slice(width: u32, height: u32, planes: u32, slice: &[T]) -> Self {
        assert_eq!(
            slice.len(),
            (width * height * planes) as usize,
            "buffers don't match sizes"
        );
        Texture {
            buffer: slice.to_owned(),
            width,
            height,
            planes,
        }
    }
}

impl<T: Default + Copy> Texture<T> {
    pub fn new(width: u32, height: u32, planes: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![T::default(); (width * height * planes) as usize],
            planes,
        }
    }
}

impl<T: Copy> Texture<T> {
    /// # Panics
    ///
    /// This will panic if the two slices have different lengths.
    pub fn copy_from_slice(&mut self, slice: &[T]) {
        self.buffer.copy_from_slice(slice);
    }

    /// Copy an image buffer into a new Texture
    pub fn from_image_buffer<P, Container>(image_buffer: &image::ImageBuffer<P, Container>) -> Self
    where
        P: image::Pixel<Subpixel = T>,
        Container: Deref<Target = [P::Subpixel]>,
    {
        let layout = image_buffer.sample_layout();
        // SAFETY: data is immediately copied over the uninitialized buffer
        let mut texture =
            unsafe { Self::new_uninitialized(layout.width, layout.height, layout.channels as u32) };
        texture.copy_from_slice(image_buffer.as_raw());
        texture
    }

    fn from_image_impl<P, Container, ImageMap>(path: &Path, map: ImageMap) -> Result<Self>
    where
        P: image::Pixel<Subpixel = T>,
        Container: Deref<Target = [P::Subpixel]>,
        ImageMap: FnOnce(DynamicImage) -> ImageBuffer<P, Container>,
    {
        let image = image::ImageReader::open(path)?.decode()?;
        let image = map(image);
        Ok(Self::from_image_buffer(&image))
    }

    /// Read a DynamicImage, map to ImageBuffer and copy into a new Texture
    ///
    /// ## Example
    ///
    /// Read image as f32 luma:
    ///
    /// ```ignore
    /// let texture =
    ///    Texture::<f32>::from_image("path/to/image.png", |image| image.to_luma32f());
    /// ```
    pub fn from_image<P, Container, ImageMap>(path: impl AsRef<Path>, map: ImageMap) -> Result<Self>
    where
        P: image::Pixel<Subpixel = T>,
        Container: Deref<Target = [P::Subpixel]>,
        ImageMap: FnOnce(DynamicImage) -> ImageBuffer<P, Container>,
    {
        Self::from_image_impl(path.as_ref(), map)
    }
}

impl Texture<utils::pixel::RGB> {
    // TODO: maybe remove, not particularly useful
    pub fn from_image_as_rgb<P: AsRef<Path>>(path: P) -> Result<Self> {
        let image = image::ImageReader::open(path)?.decode()?.to_rgba8();
        let mut texture = unsafe { Self::new_uninitialized(image.width(), image.height(), 1) };
        image.pixels().enumerate().for_each(|(idx, pixel)| {
            texture.buffer[idx] = utils::pixel::RGB::from_u8_array(&pixel.0);
        });
        Ok(texture)
    }
}

impl<T: image::Primitive> From<ImageBuffer<image::Luma<T>, Vec<T>>> for Texture<T> {
    fn from(value: ImageBuffer<image::Luma<T>, Vec<T>>) -> Self {
        let layout = value.sample_layout();
        Texture::from_slice(
            layout.width,
            layout.height,
            layout.channels as u32,
            value.as_raw(),
        )
    }
}

/// Texture with borrowed internal buffer
#[derive(Debug, Copy, Clone)]
pub struct TextureSlice<'a, T> {
    buffer: &'a [T],
    width: u32,
    height: u32,
    planes: u32,
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

    fn planes(&self) -> u32 {
        self.planes
    }
}

impl<'a, T> TextureSlice<'a, T> {
    pub fn new(width: u32, height: u32, planes: u32, buffer: &'a [T]) -> Self {
        Self {
            width,
            height,
            buffer,
            planes,
        }
    }

    /// Copy an image buffer into a new Texture
    pub fn from_image_buffer<P, Container>(
        image_buffer: &'a image::ImageBuffer<P, Container>,
    ) -> Self
    where
        P: image::Pixel<Subpixel = T>,
        Container: Deref<Target = [P::Subpixel]>,
    {
        let layout = image_buffer.sample_layout();
        Self::new(
            layout.width,
            layout.height,
            layout.channels as u32,
            image_buffer.as_raw(),
        )
    }
}

#[derive(Debug)]
pub struct TextureMutSlice<'a, T> {
    buffer: &'a mut [T],
    width: u32,
    height: u32,
    planes: u32,
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

    fn planes(&self) -> u32 {
        self.planes
    }
}

impl<T> TextureMut for TextureMutSlice<'_, T> {}

impl<'a, T> TextureMutSlice<'a, T> {
    pub fn new(width: u32, height: u32, planes: u32, buffer: &'a mut [T]) -> Self {
        Self {
            buffer,
            width,
            height,
            planes,
        }
    }
}

pub mod prelude {
    pub use super::{Texture, TextureMut, TextureMutSlice, TextureRef, TextureSlice};
}
