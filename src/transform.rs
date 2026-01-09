/// Core trait for applying a transform to data
///
/// Generic over the right-hand side type `Rhs`, similar to `Add`, `Sub`, etc.
pub trait Transform<Rhs = ()> {
    /// Apply the transform to the given data
    fn apply(&mut self, rhs: &mut Rhs);
}
