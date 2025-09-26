use crate::{
    color_palette::ColorMapElement,
    dithering::{error_diffusion::ErrorDiffusionType, threshold::ThresholdType},
    pixel_util::RGB,
};

pub mod error_diffusion;
pub mod threshold;

#[derive(Debug, Clone, Copy)]
pub enum DitheringType {
    Ordered(ThresholdType),
    ErrorDifusion(ErrorDiffusionType),
}

pub fn dither(
    data: &mut Vec<RGB>,
    width: u32,
    _height: u32,
    ditheringt: DitheringType,
    color_map: &Vec<ColorMapElement>,
) {
    match ditheringt {
        DitheringType::Ordered(ttype) => dither_ordered(data, width, _height, ttype, color_map),
        DitheringType::ErrorDifusion(_) => panic!("This is not implemented yet"),
    };
}

fn dither_ordered(
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
        if value < ttype.get_threshold(x, y) * color_map[index].scale + color_map[index].offset
        {
            return color_map[index].color;
        }
        index += 1;
    }
    return color_map.last().unwrap().color;
}
