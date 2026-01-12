use crate::texture::{TextureMutSlice, TextureShape, TextureSlice};

/// Core trait for applying a transform to data.
///
/// Uses associated types for Input/Output to ensure type safety when chaining.
/// Lifetimes are method-local, allowing flexible borrowing without lifetime hell.
pub trait TextureTransform {
    type Input;
    type Output;

    /// Apply the transform from input texture to output texture
    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, Self::Input>,
        output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    );

    /// Preparation step that can inspect data shape before transformation
    #[allow(unused_variables)]
    fn prepare(&mut self, in_shape: TextureShape, out_shape: TextureShape);
}
