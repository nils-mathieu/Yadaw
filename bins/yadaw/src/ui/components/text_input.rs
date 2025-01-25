use kui::{
    IntoElement, elem,
    elements::{Length, div, label, make_appearance, text_input},
    peniko::Color,
    winit::window::CursorIcon,
};

/// A text input element.
#[derive(Debug, Clone, Default)]
pub struct Builder<F> {
    placeholder: String,
    on_change: F,
    width: Option<Length>,
}

impl<F> Builder<F> {
    /// Sets the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the width of the text input.
    pub fn width(mut self, width: impl Into<Option<Length>>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the function that will be called when the text changes.
    pub fn on_change<F2>(self, on_change: F2) -> Builder<F2>
    where
        F2: FnMut(&str),
    {
        Builder {
            placeholder: self.placeholder,
            width: self.width,
            on_change,
        }
    }
}

impl<F> IntoElement for Builder<F>
where
    F: FnMut(&str),
{
    type Element = impl kui::Element;

    fn into_element(mut self) -> Self::Element {
        elem! {
            text_input {
                appearance: make_appearance(
                    elem!{
                        div {
                            border_brush: "#555";
                            border_thickness: 2upx;
                            padding_top: 8px;
                            padding_bottom: 8px;
                            padding_left: 16px;
                            padding_right: 16px;
                            radius: 4px;
                            width: self.width;

                            label {
                                text: self.placeholder.as_str();
                                font_stack: "Funnel Sans";
                                brush: "#555";
                            }
                        }
                    },
                    move |elem, cx, state, text: &str| {
                        if state.value_changed() {
                            if text.is_empty() {
                                elem.child.set_text(self.placeholder.clone());
                                elem.child.style_mut().brush = Color::from_rgb8(0x55, 0x55, 0x55).into();
                                cx.window.request_redraw();
                            } else {
                                elem.child.set_text(text);
                                elem.child.style_mut().brush = Color::from_rgb8(0xff, 0xff, 0xff).into();
                                cx.window.request_redraw();
                            }

                            (self.on_change)(text);
                        }
                        if state.just_entered() {
                            cx.window.set_cursor(CursorIcon::Text);
                        }
                        if state.just_left() {
                            cx.window.set_cursor(CursorIcon::Default);
                        }
                        if state.just_focused() {
                            elem.style.border_brush = Some(Color::from_rgb8(0xff, 0xff, 0xff).into());
                        }
                        if state.just_unfocused() {
                            elem.style.border_brush = Some(Color::from_rgb8(0x55, 0x55, 0x55).into());
                        }
                    }
                );
            }
        }
    }
}
