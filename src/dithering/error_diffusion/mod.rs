use crate::{color_palette::ColorMapElement, pixel_util::RGB};
use matrices::{
    ATKINSON, ATKINSON_SIZE, FLOYD_STEINBERG, FLOYD_STEINBERG_SIZE, JARVIS_JUDICE_NINKE,
    JARVIS_JUDICE_NINKE_SIZE,
};

mod matrices;

#[derive(Debug, Clone, Copy)]
pub enum ErrorDiffusionType {
    FloydSteinberg,
    JarvisJudiceNinke,
    Atkinson,
}

impl ErrorDiffusionType {
    pub fn dither(
        self,
        data: &mut Vec<RGB>,
        width: u32,
        height: u32,
        color_map: &Vec<ColorMapElement>,
    ) {

        let mut n_color_map = color_map.clone();
        normalize_color_map(&mut n_color_map);

        match self {
            ErrorDiffusionType::Atkinson => ErrorDiffusionType::dither_helper(
                ATKINSON.to_vec(),
                ATKINSON_SIZE,
                data,
                width,
                height,
                &n_color_map,
            ),
            ErrorDiffusionType::JarvisJudiceNinke => ErrorDiffusionType::dither_helper(
                JARVIS_JUDICE_NINKE.to_vec(),
                JARVIS_JUDICE_NINKE_SIZE,
                data,
                width,
                height,
                &n_color_map,
            ),
            ErrorDiffusionType::FloydSteinberg => ErrorDiffusionType::dither_helper(
                FLOYD_STEINBERG.to_vec(),
                FLOYD_STEINBERG_SIZE,
                data,
                width,
                height,
                &n_color_map,
            ),
        }
    }

    fn dither_helper(
        matrix: Vec<f64>,
        matrix_dimenisons: [usize; 2],
        data: &mut Vec<RGB>,
        width: u32,
        _height: u32,
        color_map: &Vec<ColorMapElement>,
    ) {
        todo!();
    }
}

pub fn normalize_color_map(color_map: &mut Vec<ColorMapElement>) {
    let sum = color_map.iter().map(|x| x.scale).reduce(|acc, e| acc+e).unwrap_or(0.0);
    let mut rolling_sum = 0.0;
    let mut index = 0;

    while index < color_map.len() {
        color_map[index].scale /= sum;
        color_map[index].scale += rolling_sum;
        color_map[index].scale = color_map[index].scale.max(color_map[index].offset);
        rolling_sum = color_map[index].scale;

        index += 1;
    }
}