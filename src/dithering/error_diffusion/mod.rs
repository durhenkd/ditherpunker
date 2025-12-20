use crate::{color_palette::ColorMapElement, utils::pixel::RGB};
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
    pub fn dither(self, data: &mut [RGB], width: u32, height: u32, color_map: &[ColorMapElement]) {
        let mut n_color_map = color_map.to_owned();
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
        data: &mut [RGB],
        width: u32,
        _height: u32,
        color_map: &[ColorMapElement],
    ) {
        /*
        prepare utils and variables
        */
        let mut factor = matrix.clone();
        let origin = factor.iter().position(|x| *x == -1.0).unwrap();
        let offsets = calculate_offset_matrix(matrix_dimenisons, width, origin);
        factor[origin] = 0.0;

        let mut index_data: usize = 0;
        while index_data < data.len() {
            /*
            give the pixel a color and calculate the difference
            */
            let error: f64 = discrete_and_calculate_error(&mut data[index_data], color_map);

            /*
            distribute the difference to nearby pixels
            */
            let mut index_matrix: usize = 0;
            while index_matrix < factor.len() {
                let index = index_data as isize + offsets[index_matrix];

                if index < 0 || index >= data.len() as isize {
                    index_matrix += 1;
                    continue;
                }

                data[index as usize].add_luminosity(error * factor[index_matrix]);

                index_matrix += 1;
            }

            index_data += 1;
        }
    }
}

fn normalize_color_map(color_map: &mut [ColorMapElement]) {
    let sum = color_map[1..]
        .iter()
        .map(|x| x.scale)
        .reduce(|acc, e| acc + e)
        .unwrap_or(0.0);
    let mut rolling_sum = 0.0;
    let mut index = 1;

    while index < color_map.len() {
        color_map[index].scale /= sum;
        color_map[index].scale += rolling_sum;
        rolling_sum = color_map[index].scale;

        index += 1;
    }

    color_map[0].scale = 0.0;
}

fn discrete_and_calculate_error(pixel: &mut RGB, color_map: &[ColorMapElement]) -> f64 {
    let mut index_map = 0;
    let mut min_index = 0;
    let mut min_diff = f64::MAX;
    while index_map < color_map.len() {
        let diff = (pixel.grayscale() - color_map[index_map].scale).abs();
        if diff < min_diff {
            min_index = index_map;
            min_diff = diff;
        }

        index_map += 1;
    }

    let last_element = color_map[min_index];
    let error = pixel.grayscale() - last_element.scale;
    (*pixel) = last_element.color;

    error
}

fn calculate_offset_matrix(matrix_dimenisons: [usize; 2], width: u32, origin: usize) -> Vec<isize> {
    let mut offsets: Vec<isize> = Vec::new();

    let mut index_i: usize = 0;
    while index_i < matrix_dimenisons[1] {
        let mut index_j: usize = 0;
        while index_j < matrix_dimenisons[0] {
            offsets.push(width as isize * index_i as isize + index_j as isize - origin as isize);
            index_j += 1;
        }

        index_i += 1;
    }

    offsets
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_calculate_offset_matrix() {
        let error_diffusion_matrix = [0.20, -1.0, 0.15, 0.10, 0.10, 0.20, 0.20, 0.05];
        let origin_index = error_diffusion_matrix
            .iter()
            .position(|x| *x == -1.0)
            .unwrap();

        assert_eq!(origin_index, 1);

        let matrix = calculate_offset_matrix([4, 2], 300, origin_index);
        assert_eq!(matrix, vec![-1, 0, 1, 2, 299, 300, 301, 302]);
    }

    #[test]
    fn test_calculate_offset_matrix_2() {
        let error_diffusion_matrix = [-1.0, 0.0, 0.15, 0.10, 0.10, 0.20, 0.20, 0.05];
        let origin_index = error_diffusion_matrix
            .iter()
            .position(|x| *x == -1.0)
            .unwrap();

        assert_eq!(origin_index, 0);

        let matrix = calculate_offset_matrix([4, 2], 300, origin_index);
        assert_eq!(matrix, vec![0, 1, 2, 3, 300, 301, 302, 303]);
    }

    #[test]
    fn test_normalize_color_map() {
        let mut color_map: Vec<ColorMapElement> = vec![
            ColorMapElement {
                color: RGB {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.00,
                },
                scale: 1.0,
                offset: 0.0,
            },
            ColorMapElement {
                color: RGB {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.00,
                },
                scale: 1.0,
                offset: 0.0,
            },
        ];

        normalize_color_map(&mut color_map);

        assert_eq!(color_map[0].scale, 0.5);
        assert_eq!(color_map[1].scale, 1.0);
    }
}
