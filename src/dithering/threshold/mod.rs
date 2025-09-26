mod bayer;
mod blue_noise;

use crate::dithering::threshold::{
    bayer::{BAYER0, BAYER1, BAYER2, BAYER3},
    blue_noise::BLUE_NOISE,
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
    pub fn get_threshold(self, x: usize, y: usize) -> f64 {
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
