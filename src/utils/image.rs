use crate::{
    texture::{TextureRef, TextureSlice},
    utils::pixel::RGB,
};
use image::{
    ConvertColorOptions, DynamicImage, ImageBuffer, ImageFormat, ImageReader, Rgba,
    codecs::png::{CompressionType, FilterType, PngEncoder},
    metadata::Cicp,
};
use std::{fs::File, path::Path};

pub fn read_image(path: &String) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut image = ImageReader::open(path)?.decode()?;
    image.convert_color_space(
        Cicp::SRGB_LINEAR,
        ConvertColorOptions::default(),
        image::ColorType::Rgba8,
    )?;
    Ok(image)
}

pub fn write_image(
    image: &DynamicImage,
    path: &String,
    image_format: ImageFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    image.write_to(&mut File::create(path).unwrap(), image_format)?;
    Ok(())
}

pub fn dynimg_to_rgb(image: &DynamicImage) -> Vec<RGB> {
    image
        .to_rgba8()
        .chunks(4)
        .map(|list| RGB::from_u8(list[0], list[1], list[2], list[3]))
        .collect::<Vec<RGB>>()
}

pub fn rgb_to_dynimg(rgbs: &[RGB], width: u32, height: u32) -> DynamicImage {
    let raw_data = rgbs
        .iter()
        .flat_map(|p| {
            [
                (p.r * 255.0) as u8,
                (p.g * 255.0) as u8,
                (p.b * 255.0) as u8,
                (p.a * 255.0) as u8,
            ]
        })
        .collect::<Vec<u8>>();

    DynamicImage::ImageRgba8(
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, raw_data)
            .expect("Could construct an image"),
    )
}

/// From a TextureRef construct an ImageBuffer without copying
/// and write to file using png encoder.
pub(crate) fn write_png_texture<Pixel, T>(
    texture: TextureSlice<'_, T>,
    path: &Path,
    compression: CompressionType,
    filtering: FilterType,
) -> crate::error::Result
where
    Pixel: image::Pixel + image::PixelWithColorType,
    T: image::Primitive + image::Enlargeable,
    // ImageBuffer::from_raw
    for<'a> &'a [T]: std::ops::Deref<Target = [Pixel::Subpixel]>,
    // write_with_encoder
    [<Pixel as image::Pixel>::Subpixel]: image::EncodableLayout,
{
    let image_buf =
        ImageBuffer::<Pixel, &[T]>::from_raw(texture.width(), texture.height(), texture.as_ref())
            .expect("image buffers don't match");
    let file = &mut std::io::BufWriter::new(std::fs::File::create(path)?);
    let encoder = PngEncoder::new_with_quality(file, compression, filtering);
    image_buf.write_with_encoder(encoder)?;
    Ok(())
}
