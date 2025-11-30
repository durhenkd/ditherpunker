use std::env;

use ditherpunker::{config::ProcessConfig, image_utils, run};

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_image_path = &args[1];
    let output_image_path = &args[2];
    let process_config_path = &args[3];

    let image = image_utils::read_image(input_image_path).unwrap();
    let config: ProcessConfig = ProcessConfig::read_config(process_config_path).unwrap();
    let processed_image = run(config, image).unwrap();

    image_utils::write_image(&processed_image, output_image_path, image::ImageFormat::Png).unwrap();
}
