use image::{DynamicImage, imageops::FilterType};

use crate::config::ProcessConfig;

pub mod color_palette;
pub mod config;
pub mod dithering;
pub mod image_utils;
pub mod pixel_util;

pub fn run(
    config: ProcessConfig,
    original_img: DynamicImage,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let image = original_img
        .resize(
            config.processing_width,
            config.processing_height,
            image::imageops::FilterType::Gaussian,
        )
        .grayscale()
        .brighten(config.brigthness_delta)
        .adjust_contrast(config.constrast_delta);

    let mut rgbs = image_utils::dynimg_to_rgb(&image);

    config
        .dithering_type
        .dither(&mut rgbs, image.width(), image.height(), &config.color_map);

    let new_image = image_utils::rgb_to_dynimg(&rgbs, image.width(), image.height());
    let new_image = new_image.resize(
        new_image.width() * config.output_scale,
        new_image.height() * config.output_scale,
        FilterType::Nearest,
    );

    Ok(new_image)
}
