use std::fmt::{Debug, Display};

// values are defined in a range [0.0, 1.0]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct RGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl RGB {
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> RGB {
        RGB {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        }
    }

    pub fn from_hex(string: String) -> Result<RGB, Box<dyn std::error::Error>> {
        let clean_string = String::from(string.trim().to_lowercase()).replace("#", "");
        let r_str = &clean_string[0..2];
        let g_str = &clean_string[2..4];
        let b_str = &clean_string[4..6];

        let r = u32::from_str_radix(r_str, 16)? as f64 / 255.0;
        let g = u32::from_str_radix(g_str, 16)? as f64 / 255.0;
        let b = u32::from_str_radix(b_str, 16)? as f64 / 255.0;

        Ok(RGB {
            r: r,
            g: g,
            b: b,
            a: 1.0,
        })
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        
        format!("{:X}{:X}{:X}", r, g, b)
    }

    fn grayscale(&self) -> f64 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }

    pub fn to_grayscale(&self) -> RGB {
        let l = self.grayscale();
        RGB {
            r: l,
            g: l,
            b: l,
            a: self.a,
        }
    }

    pub fn add_luminosity(&mut self, amount: f64) {
        self.r = (self.r + amount).clamp(0.0, 1.0);
        self.g = (self.g + amount).clamp(0.0, 1.0);
        self.b = (self.b + amount).clamp(0.0, 1.0);
    }

    pub fn set_value(&mut self, value: f64) {
        self.r = (value).clamp(0.0, 1.0);
        self.g = (value).clamp(0.0, 1.0);
        self.b = (value).clamp(0.0, 1.0);
    }

    pub fn set_rgba(&mut self, amount: RGB) {
        self.r = (amount.r).clamp(0.0, 1.0);
        self.g = (amount.g).clamp(0.0, 1.0);
        self.b = (amount.b).clamp(0.0, 1.0);
        self.a = (amount.a).clamp(0.0, 1.0);
    }
}

impl Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}