/// A scheme is responsible for building an element tree, and rebuilding it when something
/// in the state changes.
pub trait Scheme<T: ?Sized> {
    /// The element type that this scheme builds.
    type Element;

    /// Builds the element tree for the first time.
    fn build(self, state: &mut T) -> Self::Element;

    /// Rebuilds the element tree when the state changes.
    fn rebuild(self, state: &mut T, target: &mut Self::Element);
}
