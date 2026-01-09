use std::fmt;

/// The main error type for the ditherpunker crate
#[derive(Debug)]
pub enum DitherpunkerError {
    /// Error occurred while reading or decoding an image
    ImageDecode(image::ImageError),

    /// Error occurred while writing or encoding an image
    ImageEncode(image::ImageError),

    /// Error occurred during I/O operations (file read/write)
    Io(std::io::Error),
}

impl fmt::Display for DitherpunkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DitherpunkerError::ImageDecode(e) => write!(f, "Image decode error: {}", e),
            DitherpunkerError::ImageEncode(e) => write!(f, "Image encode error: {}", e),
            DitherpunkerError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for DitherpunkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DitherpunkerError::ImageDecode(e) | DitherpunkerError::ImageEncode(e) => Some(e),
            DitherpunkerError::Io(e) => Some(e),
        }
    }
}

// From implementations for automatic conversion from common error types

impl From<image::ImageError> for DitherpunkerError {
    fn from(err: image::ImageError) -> Self {
        // Distinguish between decode and encode errors based on the error kind
        match &err {
            image::ImageError::Encoding(_) => DitherpunkerError::ImageEncode(err),
            _ => DitherpunkerError::ImageDecode(err),
        }
    }
}

impl From<std::io::Error> for DitherpunkerError {
    fn from(err: std::io::Error) -> Self {
        DitherpunkerError::Io(err)
    }
}

// Convenience type alias for Results using DitherpunkerError
pub type Result<T = ()> = std::result::Result<T, DitherpunkerError>;
