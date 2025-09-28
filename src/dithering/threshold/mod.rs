mod matrices;

use crate::{
    color_palette::ColorMapElement,
    dithering::threshold::{
        matrices::{BAYER0, BAYER1, BAYER2, BAYER3, BLUE_NOISE},
    },
    pixel_util::RGB,
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
    pub fn dither(
        self,
        data: &mut Vec<RGB>,
        width: u32,
        _height: u32,
        color_map: &Vec<ColorMapElement>,
    ) {
        let mut index = 0;
        while index < data.len() {
            data[index] = self.dither_helper(
                data[index].r,
                color_map,
                index % width as usize,
                index / width as usize,
            );

            index += 1;
        }
    }

    fn dither_helper(
        self,
        value: f64,
        color_map: &Vec<ColorMapElement>,
        x: usize,
        y: usize,
    ) -> RGB {
        let mut index = 0;
        while index < color_map.len() {
            if value < self.get_threshold(x, y) * color_map[index].scale + color_map[index].offset {
                return color_map[index].color;
            }
            index += 1;
        }
        return color_map.last().unwrap().color;
    }

    fn get_threshold(self, x: usize, y: usize) -> f64 {
        match self {
            ThresholdType::Rand => rand::rng().random::<f64>(),
            ThresholdType::Bayer0 => 1.0 - BAYER0[y % 2 * 2 + x % 2],
            ThresholdType::Bayer1 => 1.0 - BAYER1[y % 4 * 4 + x % 4],
            ThresholdType::Bayer2 => 1.0 - BAYER2[y % 8 * 8 + x % 8],
            ThresholdType::Bayer3 => 1.0 - BAYER3[y % 16 * 16 + x % 16],
            ThresholdType::BlueNoise => BLUE_NOISE[y % 128 * 128 + x % 128],
        }
    }
}
