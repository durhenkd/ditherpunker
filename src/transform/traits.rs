use crate::texture::{Shape, TextureMutSlice, TextureRef, TextureSlice};

/// Core trait for applying a transform to data.
///
/// Uses associated types for Input/Output to ensure type safety when chaining.
/// Lifetimes are method-local, allowing flexible borrowing without lifetime hell.
pub trait TextureTransform: Sized {
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
    fn prepare(&mut self, in_shape: Shape, out_shape: Shape);

    /// Apply once. Alias for [TextureTransform::prepare] followed by [TextureTransform::apply].
    fn once<'i, 'o>(
        mut self,
        input: TextureSlice<'i, Self::Input>,
        output: TextureMutSlice<'o, Self::Output>,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    ) {
        self.prepare(input.shape(), output.shape());
        self.apply(input, output)
    }
}
