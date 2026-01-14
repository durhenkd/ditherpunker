use rand::Rng;

use crate::{
    color_palette::{ColorMapElement, DEFAULT_COLOR_MAP},
    utils::pixel::RGB,
};

pub const BENCH_IMAGE_SIZE: usize = 300;

pub fn rand_color(rng: &mut rand::rngs::ThreadRng) -> u8 {
    rng.random::<u8>().clamp(0, 255)
}

pub fn rand_rgb(rng: &mut rand::rngs::ThreadRng) -> RGB {
    RGB::from_u8(
        rand_color(rng),
        rand_color(rng),
        rand_color(rng),
        rand_color(rng),
    )
}

pub fn gen_random_image(size: usize) -> Vec<RGB> {
    let mut rng = rand::rng();
    (0..(size * size)).map(|_| rand_rgb(&mut rng)).collect()
}

pub fn default_color_map() -> Vec<ColorMapElement> {
    Vec::from(DEFAULT_COLOR_MAP)
}

pub fn random_color_map(size: usize) -> Vec<ColorMapElement> {
    let mut rng = rand::rng();
    (0..size)
        .map(|_| ColorMapElement {
            color: rand_rgb(&mut rng),
            offset: rng.random::<f32>(),
            scale: rng.random::<f32>(),
        })
        .collect()
}
