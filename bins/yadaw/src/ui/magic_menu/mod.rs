use {
    crate::{audio_file::AudioFile, ui::components::text_input},
    kui::elements::{div, flex},
    std::{path::PathBuf, sync::Arc},
};

struct AudioFileResult {
    /// The path to the audio file.
    path: PathBuf,
    /// If the file has successfully been loaded, this will be `Some`.
    audio: Option<Arc<AudioFile>>,
}

/// A possible search result.
#[derive(Debug, Clone)]
enum SearchResult {
    /// An audio file.
    AudioFile(PathBuf),
}

/// Contains the state of the magic menu.
///
/// This is not shared between threads.
#[derive(Default)]
struct MagicMenu {
    /// The search results.
    results: Vec<SearchResult>,
    /// The previous query that was searched for.
    pervious_query: String,
}

impl MagicMenu {
    /// Notifies the state that the search query has changed.
    pub fn search(&mut self, query: &str) {
        if query == self.pervious_query {
            return;
        }

        self.pervious_query = query.to_owned();
        self.results.clear();
    }
}

/// Builds the magic menu element.
pub fn magic_menu() -> impl kui::Element {
    let mut state = MagicMenu::default();

    kui::elem! {
        div {
            radius: 8px;
            padding: 8px;
            brush: "#111";
            width: 400px;
            height: 500px;

            flex {
                gap: 8px;
                vertical;

                text_input {
                    placeholder: "What are you looking for?";
                    on_change: move |s| state.search(s);
                }

                flex {
                    gap: 8px;
                    vertical;
                }
            }
        }
    }
}
