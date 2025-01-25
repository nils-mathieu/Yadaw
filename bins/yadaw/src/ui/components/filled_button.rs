use kui::{
    elem,
    elements::{Length, button, div, label, make_appearance},
    peniko::Color,
    winit::window::CursorIcon,
};

/// A button element that can be clicked.
#[derive(Debug, Clone, Default)]
pub struct Builder<F> {
    text: String,
    act_on_press: bool,
    on_click: F,
    width: Option<Length>,
}

impl<F> Builder<F> {
    /// The text of the button.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// The width of the button.
    pub fn width(mut self, width: impl Into<Option<Length>>) -> Self {
        self.width = width.into();
        self
    }

    /// Whether to call the callback on press rather than on release.
    pub fn act_on_press(mut self, act_on_press: bool) -> Self {
        self.act_on_press = act_on_press;
        self
    }

    /// Sets the function that will be called when this button is clicked.
    pub fn on_click<F2>(self, on_click: F2) -> Builder<F2>
    where
        F2: FnMut(),
    {
        Builder {
            text: self.text,
            width: self.width,
            act_on_press: self.act_on_press,
            on_click,
        }
    }
}

impl<F> kui::IntoElement for Builder<F>
where
    F: FnMut(),
{
    type Element = impl kui::Element;

    fn into_element(mut self) -> Self::Element {
        let has_width = self.width.is_some();

        elem! {
            button {
                child: make_appearance(
                    elem! {
                        div {
                            radius: 4px;
                            padding_top: 8px;
                            padding_bottom: 8px;
                            padding_left: 16px;
                            padding_right: 16px;
                            brush: "#fff";
                            width: self.width;

                            label {
                                text: self.text;
                                font_stack: "Funnel Sans";
                                brush: "#000";
                                align_middle;
                                inline: !has_width;
                            }
                        }
                    },
                    move |el, cx, state, _| {
                        if state.hover() {
                            el.style.brush = Some(Color::from_rgb8(222, 222, 222).into());
                        } else {
                            el.style.brush = Some(Color::from_rgb8(255, 255, 255).into());
                        }
                        if state.just_entered() {
                            cx.window.set_cursor(CursorIcon::Pointer);
                        }
                        if state.just_left() {
                            cx.window.set_cursor(CursorIcon::Default);
                        }
                        if state.value_changed() {
                            (self.on_click)();
                        }
                        cx.window.request_redraw();
                    }
                );
                act_on_press: self.act_on_press;
            }
        }
    }
}
