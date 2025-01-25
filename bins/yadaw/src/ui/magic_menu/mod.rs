use {
    crate::ui::components::{filled_button, text_input},
    kui::elements::{Length, div, flex, label},
};

/// Builds the magic menu element.
pub fn magic_menu() -> impl kui::Element {
    kui::elem! {
        div {
            radius: 8px;
            padding: 8px;
            brush: "#111";
            width: 200px;
            height: 300px;

            flex {
                gap: 8px;
                vertical;

                label {
                    text: "Test Title";
                    brush: "#fff";
                    align_middle;
                    font_stack: "Funnel Sans";
                    font_size: 16px;
                    font_weight: 500.0;
                }

                filled_button {
                    text: "Click me!";
                    on_click: || println!("Clicked!");
                    width: Length::ParentWidth(1.0);
                }

                text_input {
                    placeholder: "Write something...";
                    on_change: |text| println!("Text changed: {}", text);
                }
            }
        }
    }
}
