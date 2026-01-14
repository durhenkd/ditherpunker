pub mod matrices;
mod multi_impl;
pub mod threshold_transform;

use crate::{
    color_palette::ColorMapElement,
    dithering::threshold::{
        matrices::{BAYER0, BAYER1, BAYER2, BAYER3 /*BLUE_NOISE*/},
        threshold_transform::{ThresholdConfig, ThresholdImpl},
    },
    prelude::TextureTransform,
    texture::{TextureMutSlice, TextureSlice},
    utils::pixel::RGB,
};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

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
    /// Quickly dither 1 thing. Prefer using [ThresholdType::to_transform] for other purposes.
    pub fn dither(self, data: &mut [RGB], width: u32, height: u32, color_map: &[ColorMapElement]) {
        let mut transform = self.to_transform(color_map.to_vec());

        let grayscale: Vec<f32> = data
            .par_chunks(width as usize)
            .map(|row| row.iter().map(|pixel| pixel.grayscale()))
            .flatten_iter()
            .collect();

        let input = TextureSlice::new(width, height, 1, grayscale.as_slice());
        let output = TextureMutSlice::new(width, height, 1, data);

        transform.apply(input, output);
    }

    pub fn to_transform(
        &self,
        color_map: Vec<ColorMapElement>,
    ) -> impl TextureTransform<Input = f32, Output = RGB> {
        let config = self.to_transform_config(color_map);
        ThresholdImpl::auto(&config).build(config)
    }

    fn to_transform_config(&self, color_map: Vec<ColorMapElement>) -> ThresholdConfig {
        match self {
            ThresholdType::Rand => unimplemented!("requires pre-processing"),
            ThresholdType::Bayer0 => ThresholdConfig::new(1, BAYER0.to_vec(), color_map),
            ThresholdType::Bayer1 => ThresholdConfig::new(2, BAYER1.to_vec(), color_map),
            ThresholdType::Bayer2 => ThresholdConfig::new(3, BAYER2.to_vec(), color_map),
            ThresholdType::Bayer3 => ThresholdConfig::new(3, BAYER3.to_vec(), color_map),
            ThresholdType::BlueNoise => unimplemented!("generate blue noise"),
        }
    }
}
