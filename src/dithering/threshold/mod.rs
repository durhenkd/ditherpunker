mod bayer;
mod blue_noise;

use crate::{
    color_palette::ColorMapElement,
    dithering::threshold::{
        bayer::{BAYER0, BAYER1, BAYER2, BAYER3},
        blue_noise::BLUE_NOISE,
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

pub fn dither_ordered(
    data: &mut Vec<RGB>,
    width: u32,
    _height: u32,
    ttype: ThresholdType,
    color_map: &Vec<ColorMapElement>,
) {
    let mut index = 0;
    while index < data.len() {
        data[index] = dither_ordered_helper(
            data[index].r,
            color_map,
            ttype,
            index % width as usize,
            index / width as usize,
        );

        index += 1;
    }
}

fn dither_ordered_helper(
    value: f64,
    color_map: &Vec<ColorMapElement>,
    ttype: ThresholdType,
    x: usize,
    y: usize,
) -> RGB {
    let mut index = 0;
    while index < color_map.len() {
        if value < ttype.get_threshold(x, y) * color_map[index].scale + color_map[index].offset {
            return color_map[index].color;
        }
        index += 1;
    }
    return color_map.last().unwrap().color;
}
