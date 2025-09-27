use crate::{color_palette::ColorMapElement, pixel_util::RGB};

mod error_diffusion;
mod threshold;

pub use error_diffusion::ErrorDiffusionType;
pub use threshold::ThresholdType;

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
        DitheringType::Ordered(ttype) => {
            threshold::dither_ordered(data, width, _height, ttype, color_map)
        }
        DitheringType::ErrorDifusion(_) => panic!("This is not implemented yet"),
    };
}
