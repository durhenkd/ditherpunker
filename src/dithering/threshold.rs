pub mod threshold_transform;
pub mod matrices;
mod multi_impl;

use crate::{
    color_palette::ColorMapElement,
    dithering::threshold::matrices::{BAYER0, BAYER1, BAYER2, BAYER3 /*BLUE_NOISE*/},
    utils::pixel::RGB,
};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum ThresholdType {
    Rand,
    Bayer0,
    Bayer1,
    Bayer2,
    Bayer3,
    BlueNoise,
}

impl ThresholdType {
    pub fn dither(self, data: &mut [RGB], width: u32, _height: u32, color_map: &[ColorMapElement]) {
        let mut index = 0;
        while index < data.len() {
            data[index] = self.dither_helper(
                data[index].grayscale(),
                color_map,
                index % width as usize,
                index / width as usize,
            );

            index += 1;
        }
    }

    fn dither_helper(self, value: f32, color_map: &[ColorMapElement], x: usize, y: usize) -> RGB {
        let mut index = 0;
        while index < color_map.len() {
            if value < self.get_threshold(x, y) * color_map[index].scale + color_map[index].offset {
                return color_map[index].color;
            }
            index += 1;
        }

        color_map.last().unwrap().color
    }

    fn get_threshold(self, x: usize, y: usize) -> f32 {
        match self {
            ThresholdType::Rand => rand::rng().random::<f32>(),
            ThresholdType::Bayer0 => 1.0 - BAYER0[y % 2 * 2 + x % 2],
            ThresholdType::Bayer1 => 1.0 - BAYER1[y % 4 * 4 + x % 4],
            ThresholdType::Bayer2 => 1.0 - BAYER2[y % 8 * 8 + x % 8],
            ThresholdType::Bayer3 => 1.0 - BAYER3[y % 16 * 16 + x % 16],
            ThresholdType::BlueNoise => todo!("adjust precision"),
            // ThresholdType::BlueNoise => BLUE_NOISE[y % 128 * 128 + x % 128],
        }
    }
}
