use crate::{
    color_palette::ColorMapElement,
    dithering::{error_diffusion::ErrorDiffusionType, threshold::ThresholdType},
    utils::pixel::RGB,
};

pub mod error_diffusion;
pub mod threshold;

#[derive(Debug, Clone, Copy)]
pub enum DitheringType {
    Rand,
    Bayer0,
    Bayer1,
    Bayer2,
    Bayer3,
    BlueNoise,
    FloydSteinberg,
    JarvisJudiceNinke,
    Atkinson,
}

impl DitheringType {
    pub fn dither(&self, data: &mut [RGB], width: u32, height: u32, color_map: &[ColorMapElement]) {
        match self {
            Self::Rand => ThresholdType::Rand.dither(data, width, height, color_map),
            Self::Bayer0 => ThresholdType::Bayer0.dither(data, width, height, color_map),
            Self::Bayer1 => ThresholdType::Bayer1.dither(data, width, height, color_map),
            Self::Bayer2 => ThresholdType::Bayer2.dither(data, width, height, color_map),
            Self::Bayer3 => ThresholdType::Bayer3.dither(data, width, height, color_map),
            Self::BlueNoise => ThresholdType::BlueNoise.dither(data, width, height, color_map),
            Self::FloydSteinberg => {
                ErrorDiffusionType::FloydSteinberg.dither(data, width, height, color_map)
            }
            Self::JarvisJudiceNinke => {
                ErrorDiffusionType::JarvisJudiceNinke.dither(data, width, height, color_map)
            }
            Self::Atkinson => ErrorDiffusionType::Atkinson.dither(data, width, height, color_map),
        };
    }
}
