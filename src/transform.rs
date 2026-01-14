pub mod grayscale;
pub mod pipe;
pub mod traits;

pub mod prelude {
    pub use super::{grayscale::GrayscaleTransform, pipe::*, traits::*};
}
