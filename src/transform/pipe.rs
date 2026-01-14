use crate::prelude::TextureTransform;
use crate::texture::{Shape, Texture, TextureMutSlice, TextureRef, TextureSlice};

/// Extension trait that enables pipeline chaining
pub trait PipeableTransform: TextureTransform + Sized {
    /// Chain this transform with another, creating a pipeline
    ///
    /// The intermediate buffer between the two transforms is allocated automatically.
    ///
    /// # Example
    /// ```ignore
    ///
    /// // Create a pipeline: RGB -> Grayscale -> Dithered
    /// let mut pipeline = grayscale.pipe(dither, 800, 600);
    ///
    /// // Use it as a single transform
    /// pipeline.apply(input_texture, output_texture);
    /// ```
    fn pipe<T>(
        self,
        next: T,
        width: u32,
        height: u32,
        planes: u32,
    ) -> Pipeline<Self::Input, Self::Output, T::Output, Self, T>
    where
        T: TextureTransform<Input = Self::Output>,
        Self::Output: Default + Copy,
    {
        Pipeline::new(self, next, width, height, planes)
    }

    /// Chain this transform with another using a pre-allocated intermediate buffer
    fn pipe_with_buffer<T>(
        self,
        next: T,
        intermediate: Texture<Self::Output>,
    ) -> Pipeline<Self::Input, Self::Output, T::Output, Self, T>
    where
        T: TextureTransform<Input = Self::Output>,
    {
        Pipeline::with_buffer(self, next, intermediate)
    }

    fn pipe_with_shape<T>(
        self,
        next: T,
        shape: Shape,
    ) -> Pipeline<Self::Input, Self::Output, T::Output, Self, T>
    where
        T: TextureTransform<Input = Self::Output>,
        Self::Output: Default + Copy,
    {
        Pipeline::with_buffer(self, next, Texture::with_shape(shape))
    }
}

// Blanket implementation: all TextureTransforms are automatically pipeable
impl<T: TextureTransform> PipeableTransform for T {}

/// A pipeline that chains two transforms: A -> B -> C
///
/// The intermediate buffer B is owned by this struct and reused across invocations.
/// This allows efficient chaining without allocating intermediate buffers on each call.
///
/// Exposes only A -> C, hiding the intermediate type B.
pub struct Pipeline<A, B, C, T1, T2>
where
    T1: TextureTransform<Input = A, Output = B>,
    T2: TextureTransform<Input = B, Output = C>,
{
    t1: T1,
    t2: T2,
    b: Texture<B>,
}

impl<A, B, C, T1, T2> Pipeline<A, B, C, T1, T2>
where
    T1: TextureTransform<Input = A, Output = B>,
    T2: TextureTransform<Input = B, Output = C>,
{
    /// Create a new transform pipeline with a pre-allocated intermediate buffer
    pub fn with_buffer(t1: T1, t2: T2, intermediate: Texture<B>) -> Self {
        Self {
            t1,
            t2,
            b: intermediate,
        }
    }
}

impl<A, B, C, T1, T2> Pipeline<A, B, C, T1, T2>
where
    T1: TextureTransform<Input = A, Output = B>,
    T2: TextureTransform<Input = B, Output = C>,
    B: Default + Copy,
{
    /// Create a new transform pipeline with automatic intermediate buffer allocation
    pub fn new(t1: T1, t2: T2, width: u32, height: u32, planes: u32) -> Self {
        Self {
            t1,
            t2,
            b: Texture::new(width, height, planes),
        }
    }
}

impl<A, B, C, T1, T2> TextureTransform for Pipeline<A, B, C, T1, T2>
where
    T1: TextureTransform<Input = A, Output = B>,
    T2: TextureTransform<Input = B, Output = C>,
{
    type Input = A;
    type Output = C;

    #[inline(always)]
    fn apply<'i, 'o>(
        &mut self,
        input: TextureSlice<'i, A>,
        output: TextureMutSlice<'o, C>,
    ) -> (TextureSlice<'i, A>, TextureMutSlice<'o, C>) {
        let (input, _) = self.t1.apply(input, self.b.as_texture_mut_slice());
        let (_, output) = self.t2.apply(self.b.as_texture_slice(), output);
        (input, output)
    }

    #[inline(always)]
    fn prepare<'i>(&mut self, in_shape: Shape, out_shape: Shape) {
        let b_shape = self.b.shape();
        self.t1.prepare(in_shape, b_shape);
        self.t2.prepare(b_shape, out_shape);
    }
}

/// Trait for types that can be piped through a TextureTransform
pub trait PipeableTextures<'i, 'o> {
    type Input;
    type Output;

    /// Pipe these textures through a transform
    ///
    /// ## Example
    ///
    /// Transform chain:
    ///
    /// ```ignore
    /// transform
    ///     .apply(input.as_texture_slice(), output.as_texture_mut_slice())
    ///     .pipe(&mut transform)
    ///     .pipe(&mut transform);
    /// ```
    ///
    /// Bundle texture and pipe:
    ///
    /// ```ignore
    /// (input.as_texture_slice(), output.as_texture_mut_slice())
    ///     .pipe(&mut transform)
    ///     .pipe(&mut transform);
    /// ```
    fn pipe<T>(
        self,
        transform: &mut T,
    ) -> (
        TextureSlice<'i, Self::Input>,
        TextureMutSlice<'o, Self::Output>,
    )
    where
        T: TextureTransform<Input = Self::Input, Output = Self::Output>;
}

// Implement for the tuple of (TextureSlice, TextureMutSlice)
impl<'i, 'o, A, B> PipeableTextures<'i, 'o> for (TextureSlice<'i, A>, TextureMutSlice<'o, B>) {
    type Input = A;
    type Output = B;

    #[inline(always)]
    fn pipe<T>(self, transform: &mut T) -> (TextureSlice<'i, A>, TextureMutSlice<'o, B>)
    where
        T: TextureTransform<Input = A, Output = B>,
    {
        transform.apply(self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::{PipeableTransform, TextureTransform},
        texture::{Shape, Texture},
    };

    #[test]
    fn test_pipeline_cascades_apply() {
        let shape: Shape = (4, 4, 1);
        let input = Texture::<u8>::with_shape(shape);
        let mut output = Texture::<u8>::with_shape(shape);

        let a = IncTransform::default();
        let b = IncTransform::default();

        let mut pipeline = a.pipe_with_shape(b, shape);
        pipeline.apply(input.as_texture_slice(), output.as_texture_mut_slice());

        assert!(output.as_ref().iter().all(|p| *p == 2));
    }

    #[test]
    fn test_pipeline_cascades_prepare() {
        let mut pipeline =
            IncTransform::default().pipe_with_shape(IncTransform::default(), (4, 4, 1));

        pipeline.prepare((2, 2, 1), (8, 8, 1));

        assert_eq!(pipeline.t1.in_shape, Some((2, 2, 1)));
        assert_eq!(pipeline.t1.out_shape, Some((4, 4, 1)));
        assert_eq!(pipeline.t2.in_shape, Some((4, 4, 1)));
        assert_eq!(pipeline.t2.out_shape, Some((8, 8, 1)));
    }

    #[derive(Default)]
    struct IncTransform {
        in_shape: Option<Shape>,
        out_shape: Option<Shape>,
    }
    impl TextureTransform for IncTransform {
        type Input = u8;
        type Output = u8;

        fn apply<'i, 'o>(
            &mut self,
            input: crate::prelude::TextureSlice<'i, Self::Input>,
            mut output: crate::prelude::TextureMutSlice<'o, Self::Output>,
        ) -> (
            crate::prelude::TextureSlice<'i, Self::Input>,
            crate::prelude::TextureMutSlice<'o, Self::Output>,
        ) {
            output
                .as_mut()
                .iter_mut()
                .zip(input.as_ref().iter())
                .for_each(|(dst, src)| *dst = *src + 1);
            (input, output)
        }

        fn prepare(&mut self, in_shape: crate::texture::Shape, out_shape: crate::texture::Shape) {
            self.in_shape = Some(in_shape);
            self.out_shape = Some(out_shape);
        }
    }
}
