/// Text that does not include any styling whatsoever.
struct UnstyledText {
    /// The text to render.
    text: String,
}

/// Allows running a function that will be used to style a [`Text`] element.
pub trait TextStyle {
    /// Styles the provided text.
    fn style(&self, text: &str);
}

/// An element responsible for rendering text.
pub struct Text<S: TextStyle + ?Sized> {
    /// The unstyled text to render.
    unstyled: UnstyledText,
    /// The instance responsible for adding style to the text.
    styler: S,
}
