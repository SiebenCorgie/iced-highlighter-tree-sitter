//! A [Highlighter](https://docs.rs/iced/latest/iced/advanced/text/trait.Highlighter.html) for Iced's [TextEditor](https://docs.rs/iced/latest/iced/widget/struct.TextEditor.html)
//! based on [TreeSitterHighlight](https://docs.rs/tree-sitter-highlight/latest/tree_sitter_highlight/).
//!
//!
//! Use [to_format] and [TSSettings] to configure how/what is higlighted. Works out of the box with the rust-syntax. Have a look at the `rusteditor` example on how to
//! include this in your project. The gist is:
//!
//! ```rust ignore
//!struct YourState{
//!   //..other stuff
//!
//!   //safe the _settings_ in you application state. This prevents reloading
//!   // TS for each highlighting.
//!   ts: TSSettings
//!}
//! //in your state's view function:
//! fn view(&self) -> Element<Message>{
//!     text_editor(&self.content)
//!         .highlight_with::<iced_highlighter_tree_sitter::Highlighter>(
//!             self.ts.clone(),
//!             iced_highlighter_tree_sitter::to_format
//!     )
//! }
//! ```

use std::{ops::Range, sync::Arc};

use tree_sitter_highlight::HighlightEvent;

///Standard formating function. Assumes that you use the `highlight_names` defined [here](https://crates.io/crates/tree-sitter-highlight).
///
///If you want to use other names (because of a different TSQuery setup, or theme), consider building your own [TSSettings] and `to_format` function.
///
///The highlighter itself will still work 🙂.
pub fn to_format(
    highlight: &tree_sitter_highlight::Highlight,
    theme: &iced::Theme,
) -> iced::advanced::text::highlighter::Format<iced::Font> {
    let palette = theme.extended_palette();

    //sort out _what_ is being highlighted, and come up with color and font
    match highlight.0 {
        //comment
        1 => iced::advanced::text::highlighter::Format {
            color: Some(palette.secondary.weak.color),
            font: Some(iced::Font::DEFAULT),
        },
        //constant
        2 | 3 => iced::advanced::text::highlighter::Format {
            color: Some(palette.danger.weak.color),
            font: Some(iced::Font::MONOSPACE),
        },
        //strings
        18 | 19 => iced::advanced::text::highlighter::Format {
            color: Some(palette.success.base.color),
            font: Some(iced::Font::DEFAULT),
        },
        //functions
        6 | 7 => iced::advanced::text::highlighter::Format {
            color: Some(palette.success.strong.color),
            font: Some(iced::Font::MONOSPACE),
        },
        //types
        21 | 22 => iced::advanced::text::highlighter::Format {
            color: Some(palette.primary.weak.color),
            font: Some(iced::Font::MONOSPACE),
        },
        //variables
        24 => iced::advanced::text::highlighter::Format {
            color: Some(palette.danger.weak.color),
            font: Some(iced::Font::MONOSPACE),
        },
        //anything _special_
        8 | 9 => iced::advanced::text::highlighter::Format {
            color: Some(palette.danger.strong.color),
            font: Some(iced::Font::MONOSPACE),
        },
        _ => iced::advanced::text::highlighter::Format {
            color: None,
            font: Some(iced::Font::MONOSPACE),
        },
    }
}

#[derive(Clone)]
pub struct TSSettings {
    ///Internally used highlighting configuration. Build your own, if you want to use
    ///some
    pub tsconfig: Arc<tree_sitter_highlight::HighlightConfiguration>,
}

impl TSSettings {
    ///Configures `config` to recognize the standard names (defined in the tree-sitter-highlight [README](https://crates.io/crates/tree-sitter-highlight)).
    ///
    ///
    ///If you want to roll your own highlighting, consider setting up the config yourself, and combining it with a custom `to_format` function when
    ///applying the highlighter to a text-edit.
    pub fn new(mut config: tree_sitter_highlight::HighlightConfiguration) -> Self {
        let highlight_names = [
            "attribute",
            "comment",
            "constant",
            "constant.builtin",
            "constructor",
            "embedded",
            "function",
            "function.builtin",
            "keyword",
            "module",
            "number",
            "operator",
            "property",
            "property.builtin",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "punctuation.special",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
        ];

        config.configure(&highlight_names);

        Self {
            //wrap into something clonabel, so we don't have to load TS
            //whenever the highlighter is created.
            tsconfig: Arc::new(config),
        }
    }
}

impl PartialEq for TSSettings {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.tsconfig, &other.tsconfig)
    }
}

pub struct Highlighter {
    highlighter: tree_sitter_highlight::Highlighter,
    settings: TSSettings,
    line: usize,
}

impl iced::advanced::text::Highlighter for Highlighter {
    type Highlight = tree_sitter_highlight::Highlight;
    type Settings = TSSettings;
    type Iterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        let highlighter = tree_sitter_highlight::Highlighter::new();
        Self {
            settings: settings.clone(),
            highlighter,
            line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
    }

    fn change_line(&mut self, line: usize) {
        self.line = line;
    }

    fn current_line(&self) -> usize {
        self.line
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        //NOTE: we ignore anything that _fails_.
        //      In the future one might want to tag those areas.
        let events = match self.highlighter.highlight(
            &self.settings.tsconfig.as_ref(),
            line.as_bytes(),
            None,
            |_| None,
        ) {
            Ok(events) => events,
            Err(_e) => return Box::new([].into_iter()),
        };
        let alloc_size = {
            let (lower, upper) = events.size_hint();
            if let Some(upper) = upper {
                upper
            } else {
                lower
            }
        };
        let mut format_instructions = Vec::with_capacity(alloc_size);

        let mut current_style = None;
        for event in events {
            let event = if let Ok(ev) = event {
                ev
            } else {
                continue;
            };
            match event {
                HighlightEvent::Source { start, end } => {
                    if let Some(style) = current_style {
                        format_instructions.push((start..end, style));
                    }
                }
                HighlightEvent::HighlightStart(styleid) => {
                    current_style = Some(styleid);
                }
                HighlightEvent::HighlightEnd => {
                    current_style = None;
                }
            }
        }

        //iterate throuht the event chain, and transform them into a list of _formats_ + their range.
        Box::new(format_instructions.into_iter())
    }
}
