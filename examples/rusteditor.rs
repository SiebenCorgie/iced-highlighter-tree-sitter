use iced::widget::{column, horizontal_space, row, text, text_editor, toggler};
use iced::{Center, Element, Fill, Font, Task, Theme};
use iced_highlighter_tree_sitter::TSSettings;

use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application("Editor - Iced", Editor::update, Editor::view)
        .theme(Editor::theme)
        .default_font(Font::MONOSPACE)
        .run()
}

struct Editor {
    file: Option<PathBuf>,
    ts: TSSettings,
    content: text_editor::Content,
    word_wrap: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    WordWrapToggled(bool),
}

impl Default for Editor {
    fn default() -> Self {
        let config = tree_sitter_highlight::HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .unwrap();

        Self {
            file: None,
            content: text_editor::Content::with_text(include_str!("rusteditor.rs")),
            ts: TSSettings::new(config),
            word_wrap: true,
            is_dirty: false,
        }
    }
}

impl Editor {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            horizontal_space(),
            toggler(self.word_wrap)
                .label("Word Wrap")
                .on_toggle(Message::WordWrapToggled)
        ]
        .spacing(10)
        .align_y(Center);

        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        column![
            controls,
            text_editor(&self.content)
                .height(Fill)
                .on_action(Message::ActionPerformed)
                .wrapping(if self.word_wrap {
                    text::Wrapping::Word
                } else {
                    text::Wrapping::None
                })
                .highlight_with::<iced_highlighter_tree_sitter::Highlighter>(
                    self.ts.clone(),
                    iced_highlighter_tree_sitter::to_format
                ),
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
