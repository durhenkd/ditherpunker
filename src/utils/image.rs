use std::fs::File;

use crate::utils::pixel::RGB;
use image::{
    ConvertColorOptions, DynamicImage, ImageBuffer, ImageFormat, ImageReader, Rgba, metadata::Cicp,
};

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
