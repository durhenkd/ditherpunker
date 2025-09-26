use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

use json::{object, JsonValue};

use crate::{
    color_palette::{ColorMapElement, DEFAULT_COLOR_MAP},
    dithering::{error_diffusion::ErrorDiffusionType, threshold::ThresholdType, DitheringType},
    pixel_util::RGB,
};

#[derive(Debug)]
pub struct ProcessConfig {
    pub brigthness_delta: i32,
    pub constrast_delta: f32,
    pub dithering_type: DitheringType,
    pub color_map: Vec<ColorMapElement>,
    pub processing_width: u32,
    pub processing_height: u32,
    pub output_scale: u32,
}

impl ProcessConfig {
    fn to_config(json_string: String) -> Result<ProcessConfig, Box<dyn std::error::Error>> {
        let json = json::parse(json_string.as_str())?;

        let brigthness_delta = match json["brigthness_delta"].as_i32() {
            Some(val) => val,
            None => return ConfigError::get("Couldn't parse brigthness_delta"),
        };
        let constrast_delta: f32 = match json["constrast_delta"].as_f32() {
            Some(val) => val,
            None => return ConfigError::get("Couldn't parse constrast_delta"),
        };
        let processing_width: u32 = match json["processing_width"].as_u32() {
            Some(val) => val,
            None => return ConfigError::get("Couldn't parse processing_width"),
        };
        let processing_height: u32 = match json["processing_height"].as_u32() {
            Some(val) => val,
            None => return ConfigError::get("Couldn't parse processing_height"),
        };
        let output_scale: u32 = match json["output_scale"].as_u32() {
            Some(val) => val,
            None => return ConfigError::get("Couldn't parse output_scale"),
        };

        let dithering_type: DitheringType = match json["dithering_type"].as_str() {
            Some(s) => match s {
                "rand" => DitheringType::Ordered(ThresholdType::Rand),
                "bayer_0" => DitheringType::Ordered(ThresholdType::Bayer0),
                "bayer_1" => DitheringType::Ordered(ThresholdType::Bayer1),
                "bayer_2" => DitheringType::Ordered(ThresholdType::Bayer2),
                "bayer_3" => DitheringType::Ordered(ThresholdType::Bayer3),
                "blue_noise" => DitheringType::Ordered(ThresholdType::BlueNoise),
                "atkinson" => DitheringType::ErrorDifusion(ErrorDiffusionType::Atkinson),
                "jarvis" => DitheringType::ErrorDifusion(ErrorDiffusionType::JarvisJudiceNinke),
                "floyd" => DitheringType::ErrorDifusion(ErrorDiffusionType::FloydSteinberg),
                _ => return ConfigError::get("Not recognized dithering_type"),
            },
            None => return ConfigError::get("Couldn't parse dithering_type"),
        };

        let color_map = if json["color_map"].is_null() {
            DEFAULT_COLOR_MAP.to_vec()
        } else if json["color_map"].len() <= 1 {
            return ConfigError::get("color_map should be an array of 2 or more colors objects");
        } else {
            let mut index = 0;
            let mut color_map: Vec<ColorMapElement> = Vec::new();
            while index < json["color_map"].len() {
                let color = match json["color_map"][index]["color"].as_str() {
                    Some(val) => val.to_string(),
                    None => match json["color_map"][index].as_str() {
                        Some(val) => val.to_string(),
                        None => return ConfigError::get("Couldn't parse color_map.*.color"),
                    },
                };
                let scale = match json["color_map"][index]["scale"].as_f64() {
                    Some(val) => val,
                    None => 1.0,
                };
                let offset = match json["color_map"][index]["offset"].as_f64() {
                    Some(val) => val,
                    None => 0.0,
                };

                color_map.push(ColorMapElement {
                    color: RGB::from_hex(color)?,
                    scale: scale,
                    offset: offset,
                });

                index += 1;
            }
            color_map
        };

        Ok(ProcessConfig {
            brigthness_delta: brigthness_delta,
            constrast_delta: constrast_delta,
            dithering_type: dithering_type,
            color_map: color_map,
            processing_width: processing_width,
            processing_height: processing_height,
            output_scale: output_scale,
        })
    }

    fn to_json(config: &ProcessConfig) -> String {
        let mut data = json::JsonValue::new_object();

        data["brigthness_delta"] = config.brigthness_delta.into();
        data["constrast_delta"] = config.constrast_delta.into();
        data["dithering_type"] = config.dithering_type.into();
        data["color_map"] = config.color_map.clone().into();
        data["processing_width"] = config.processing_width.into();
        data["processing_height"] = config.processing_height.into();
        data["output_scale"] = config.output_scale.into();

        data.to_string()
    }

    pub fn read_config(path: &String) -> Result<ProcessConfig, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buff: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut buff)?;

        let json_string = String::from_utf8(buff)?;

        ProcessConfig::to_config(json_string)
    }

    pub fn write_config(&self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        let string = ProcessConfig::to_json(self);
        let mut file = File::create(path)?;
        file.write_all(string.as_bytes())?;
        Ok(())
    }
}

impl From<DitheringType> for JsonValue {
    fn from(dtype: DitheringType) -> Self {
        match dtype {
            DitheringType::Ordered(ordered_type) => ordered_type.into(),
            DitheringType::ErrorDifusion(diffusion_type) => diffusion_type.into(),
        }
    }
}

impl From<RGB> for JsonValue {
    fn from(rgb: RGB) -> Self {
        rgb.to_hex().into()
    }
}

impl From<ColorMapElement> for JsonValue {
    fn from(cme: ColorMapElement) -> Self {
        object! { color: cme.color, offset: cme.offset, scale: cme.scale }
    }
}

impl From<ThresholdType> for JsonValue {
    fn from(ttype: ThresholdType) -> Self {
        match ttype {
            ThresholdType::Rand => JsonValue::String(String::from("rand")),
            ThresholdType::Bayer0 => JsonValue::String(String::from("bayer_0")),
            ThresholdType::Bayer1 => JsonValue::String(String::from("bayer_1")),
            ThresholdType::Bayer2 => JsonValue::String(String::from("bayer_2")),
            ThresholdType::Bayer3 => JsonValue::String(String::from("bayer_3")),
            ThresholdType::BlueNoise => JsonValue::String(String::from("blue_noise")),
        }
    }
}

impl From<ErrorDiffusionType> for JsonValue {
    fn from(ttype: ErrorDiffusionType) -> Self {
        match ttype {
            ErrorDiffusionType::Atkinson => JsonValue::String(String::from("atkinson")),
            ErrorDiffusionType::JarvisJudiceNinke => JsonValue::String(String::from("jarvis")),
            ErrorDiffusionType::FloydSteinberg => JsonValue::String(String::from("floyd")),
        }
    }
}

#[derive(Debug)]
pub struct ConfigError {
    msg: String,
}

impl ConfigError {
    fn get(msg: &str) -> Result<ProcessConfig, Box<dyn std::error::Error>> {
        Err(Box::new(ConfigError {
            msg: String::from(msg),
        }))
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ConfigParseError {}", self.msg))
    }
}
impl Error for ConfigError {}
