use crate::utils::pixel::RGB;

#[derive(Debug, Clone, Copy)]
pub struct ColorMapElement {
    pub color: RGB,
    pub scale: f32,  // only takes in consideration for error diffusion dithering
    pub offset: f32, // only takes in consideration for ordered dithering
}

pub const DEFAULT_COLOR_MAP: [ColorMapElement; 2] = [
    ColorMapElement {
        color: RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        scale: 1.0,
        offset: 0.0,
    },
    ColorMapElement {
        color: RGB {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        scale: 1.0,
        offset: 0.0,
    },
];
