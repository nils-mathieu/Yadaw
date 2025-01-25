use {
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        elements::{appearance::Appearance, interactive::InteractiveState},
        event::{Event, EventResult, KeyEvent},
    },
    vello::kurbo::{Point, Size},
    winit::keyboard::{ModifiersState, NamedKey},
};

/// Removes the last word of the provided string.
fn remove_last_word(s: &mut String) {
    let idx = s
        .trim_end_matches(|c: char| c.is_whitespace())
        .trim_end_matches(|c: char| !c.is_whitespace())
        .trim_end_matches(|c: char| c.is_whitespace())
        .len();
    s.truncate(idx);
}

/// An element that allows the user to input text.
///
/// # Remarks
///
/// This does not include any text rendering.
#[derive(Clone, Debug, Default)]
pub struct TextInput<A: ?Sized> {
    /// The value of the text input element.
    pub value: String,
    /// The state of the interactive element.
    pub state: InteractiveState,
    /// The appearance of the text input element.
    pub appearance: A,
}

impl<A> TextInput<A> {
    /// Sets the appearance of the text input element.
    pub fn appearance<A2>(self, appearance: A2) -> TextInput<A2> {
        TextInput {
            value: self.value,
            state: self.state,
            appearance,
        }
    }
}

impl<A: ?Sized + Appearance<str>> TextInput<A> {
    /// Handles a key event.
    fn handle_key_event(&mut self, modifiers: ModifiersState, event: &KeyEvent) -> bool {
        if !event.state.is_pressed() {
            return false;
        }

        if event.logical_key == NamedKey::Backspace {
            if cfg!(target_os = "macos") {
                if modifiers.control_key() {
                    // Ignored.
                    return false;
                }

                if modifiers.super_key() {
                    self.value.clear();
                } else if modifiers.alt_key() {
                    remove_last_word(&mut self.value);
                } else {
                    self.value.pop();
                }
            } else {
                #[allow(clippy::collapsible_if)]
                if modifiers.control_key() {
                    remove_last_word(&mut self.value);
                } else {
                    self.value.pop();
                }
            }

            self.state.insert(InteractiveState::VALUE_CHANGED);
            return true;
        }

        if event.logical_key == NamedKey::Enter || event.logical_key == NamedKey::Tab {
            return true;
        }

        if let Some(text) = event.text.as_ref() {
            self.value.push_str(text);
            self.state.insert(InteractiveState::VALUE_CHANGED);
            return true;
        }

        false
    }
}

impl<A> Element for TextInput<A>
where
    A: ?Sized + Appearance<str>,
{
    #[inline]
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        self.appearance
            .size_hint(elem_context, layout_context, space)
    }

    #[inline]
    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        self.appearance
            .place(elem_context, layout_context, pos, size);
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.appearance.hit_test(point)
    }

    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {
        self.appearance.draw(elem_context, scene);
    }

    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        self.state.remove_transient_states();

        let og_state = self.state;
        let mut event_result = self
            .state
            .handle_pointer_interactions(&mut |pt| self.appearance.hit_test(pt), event);
        if self.state.focused() {
            if let Some(ev) = event.downcast_ref::<KeyEvent>() {
                self.handle_key_event(elem_context.window.keyboard_modifiers(), ev);
                event_result = EventResult::Handled;
            }
        }
        if og_state != self.state {
            self.appearance
                .state_changed(elem_context, self.state, &self.value);
        }
        if event_result.is_handled() {
            return EventResult::Handled;
        }
        self.appearance.event(elem_context, event);
        EventResult::Continue
    }

    fn begin(&mut self, elem_context: &ElemContext) {
        self.appearance.begin(elem_context);
        self.appearance
            .state_changed(elem_context, self.state, &self.value);
    }
}
