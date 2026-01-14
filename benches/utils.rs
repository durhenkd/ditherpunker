use std::str::FromStr;

use ditherpunker::{
    color_palette::ColorMapElement,
    dithering::threshold::{matrices, threshold_transform::ThresholdConfig},
    texture::Texture,
    utils::pixel::RGB,
};
use image::DynamicImage;
use itertools::Itertools;
use rand::Rng;

pub const BENCH_IMAGE_SIZE: u32 = 300;

pub fn read_test_image(size: u32) -> DynamicImage {
    ditherpunker::utils::image::read_image(&"./assets/bench_asset.png".to_string())
        .unwrap()
        .resize(size, size, image::imageops::FilterType::Gaussian)
}

/// Default config for bayer transform benchmarks
pub fn threshold_config(colors: usize) -> ThresholdConfig {
    std::hint::black_box(ThresholdConfig::new(
        1,
        matrices::BAYER0.to_vec(),
        color_map_preset(colors),
    ))
}

/// Get owned data to perform bayer transformations
pub fn threshold_data(size: u32) -> (Texture<f32>, Texture<ditherpunker::utils::pixel::RGB>) {
    (
        std::hint::black_box(
            read_test_image(size)
                .grayscale()
                .brighten(60)
                .adjust_contrast(10_f32)
                .to_luma32f()
                .into(),
        ),
        std::hint::black_box(Texture::new(size, size, 1)),
    )
}

pub fn rand_color(rng: &mut rand::rngs::ThreadRng) -> u8 {
    rng.random::<u8>().clamp(0, 255)
}

pub fn rand_rgb(rng: &mut rand::rngs::ThreadRng) -> RGB {
    RGB::from_u8(
        rand_color(rng),
        rand_color(rng),
        rand_color(rng),
        rand_color(rng),
    )
}

pub fn random_color_map(size: usize) -> Vec<ColorMapElement> {
    let mut rng = rand::rng();
    (0..size)
        .map(|_| ColorMapElement {
            color: rand_rgb(&mut rng),
            offset: rng.random::<f32>(),
            scale: rng.random::<f32>(),
        })
        .collect()
}

pub fn color_map_preset(colors: usize) -> Vec<ColorMapElement> {
    let mut rng = rand::rng();
    let cmap = |hex: &str, offset: f32, scale: f32| -> ColorMapElement {
        ColorMapElement {
            color: RGB::from_hex(String::from_str(hex).unwrap()).unwrap(),
            offset,
            scale,
        }
    };

    match colors {
        2 => vec![cmap("#020217", 0.0, 0.8), cmap("#e6e2c2", 0.15, 0.85)],
        3 => {
            vec![
                cmap("#020217", 0.0, 0.8),
                cmap("#2e2627", 0.05, 0.9),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        4 => {
            vec![
                cmap("#020217", 0.0, 0.8),
                cmap("#2e2627", 0.05, 0.9),
                cmap("#60594b", 0.1, 0.7),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        5 => {
            vec![
                cmap("#020217", 0.0, 0.6),
                cmap("#2e2627", 0.05, 0.8),
                cmap("#60594b", 0.1, 0.7),
                cmap("#4a5b53", 0.2, 0.8),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        6 => {
            vec![
                cmap("#020217", 0.0, 0.6),
                cmap("#2e2627", 0.05, 0.8),
                cmap("#60594b", 0.1, 0.7),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        7 => {
            vec![
                cmap("#020217", 0.0, 0.6),
                cmap("#2e2627", 0.05, 0.4),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#60594b", 0.1, 0.7),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        8 => {
            vec![
                cmap("#020217", 0.0, 0.6),
                cmap("#2e2627", 0.05, 0.4),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#60594b", 0.1, 0.7),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#8a9a7b", 0.25, 0.8),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        12 => {
            vec![
                cmap("#020217", 0.0, 0.5),
                cmap("#1a1520", 0.02, 0.4),
                cmap("#2e2627", 0.05, 0.4),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#4e4540", 0.12, 0.65),
                cmap("#60594b", 0.1, 0.7),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#3d5048", 0.18, 0.82),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#6a7a5e", 0.22, 0.78),
                cmap("#a8b89d", 0.28, 0.82),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        16 => {
            vec![
                cmap("#020217", 0.0, 0.5),
                cmap("#12101a", 0.01, 0.38),
                cmap("#1a1520", 0.02, 0.4),
                cmap("#2e2627", 0.05, 0.42),
                cmap("#352d30", 0.08, 0.55),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#4e4540", 0.12, 0.65),
                cmap("#60594b", 0.1, 0.7),
                cmap("#564e46", 0.14, 0.72),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#3d5048", 0.18, 0.82),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#567061", 0.24, 0.76),
                cmap("#7a8a75", 0.26, 0.8),
                cmap("#b5c0a8", 0.3, 0.84),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        24 => {
            vec![
                cmap("#020217", 0.0, 0.48),
                cmap("#0d0a14", 0.01, 0.36),
                cmap("#12101a", 0.01, 0.38),
                cmap("#1a1520", 0.02, 0.4),
                cmap("#251e24", 0.04, 0.45),
                cmap("#2e2627", 0.05, 0.42),
                cmap("#352d30", 0.08, 0.55),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#463f3c", 0.11, 0.62),
                cmap("#4e4540", 0.12, 0.65),
                cmap("#574f46", 0.13, 0.68),
                cmap("#60594b", 0.1, 0.7),
                cmap("#564e46", 0.14, 0.72),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#445550", 0.19, 0.78),
                cmap("#3d5048", 0.18, 0.82),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#3e6055", 0.21, 0.74),
                cmap("#567061", 0.24, 0.76),
                cmap("#6a7a6b", 0.25, 0.78),
                cmap("#7a8a75", 0.26, 0.8),
                cmap("#96a58d", 0.28, 0.82),
                cmap("#b5c0a8", 0.3, 0.84),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        32 => {
            vec![
                cmap("#020217", 0.0, 0.48),
                cmap("#090712", 0.005, 0.35),
                cmap("#0d0a14", 0.01, 0.36),
                cmap("#12101a", 0.01, 0.38),
                cmap("#16121d", 0.015, 0.39),
                cmap("#1a1520", 0.02, 0.4),
                cmap("#201a22", 0.03, 0.43),
                cmap("#251e24", 0.04, 0.45),
                cmap("#2e2627", 0.05, 0.42),
                cmap("#322a2c", 0.07, 0.52),
                cmap("#352d30", 0.08, 0.55),
                cmap("#3a3538", 0.09, 0.58),
                cmap("#3e3a3e", 0.3, 0.6),
                cmap("#463f3c", 0.11, 0.62),
                cmap("#4e4540", 0.12, 0.65),
                cmap("#574f46", 0.13, 0.68),
                cmap("#60594b", 0.1, 0.7),
                cmap("#5d544a", 0.135, 0.71),
                cmap("#564e46", 0.14, 0.72),
                cmap("#4f5851", 0.17, 0.74),
                cmap("#4a5b53", 0.2, 0.75),
                cmap("#445550", 0.19, 0.78),
                cmap("#3d5048", 0.18, 0.82),
                cmap("#385448", 0.16, 0.86),
                cmap("#2e544a", 0.15, 0.9),
                cmap("#3e6055", 0.21, 0.74),
                cmap("#4e6d5e", 0.23, 0.75),
                cmap("#567061", 0.24, 0.76),
                cmap("#6a7a6b", 0.25, 0.78),
                cmap("#7a8a75", 0.26, 0.8),
                cmap("#96a58d", 0.28, 0.82),
                cmap("#e6e2c2", 0.15, 0.85),
            ]
        }
        _ => vec![
            cmap("#030117", 0.0, 0.6),
            cmap("#22191c", 0.05, 0.4),
            cmap("#2D2423", 0.3, 0.6),
            cmap("#435148", 0.1, 0.7),
            cmap("#54655a", 0.2, 0.75),
            cmap("#5e7367", 0.15, 0.9),
            cmap("#c5caad", 0.15, 0.85),
        ]
        .into_iter()
        .chain((0..colors - 7).map(|_| ColorMapElement {
            color: rand_rgb(&mut rng),
            offset: rng.random(),
            scale: rng.random(),
        }))
        .take(colors)
        .collect_vec(),
    }
}
