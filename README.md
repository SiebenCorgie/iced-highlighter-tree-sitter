<div align="center">


# iced-highlighter-tree-sitter

[![dependency status](https://deps.rs/repo/github/SiebenCorgie/iced-highlighter-tree-sitter/status.svg)](https://deps.rs/repo/github/SiebenCorgie/iced-highlighter-tree-sitter)
[![MIT](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

[TreeSitterHighlighting](https://crates.io/crates/tree-sitter-highlight) based highlighter implementation for [Iced's](https://github.com/iced-rs/iced) `TextEditor`.

</div>


### Development notice

I came up with this implementation in about 20min, so it might not be _the most effective_ way to handle TreeSitter. My main pain-point atm. is that the internal TS `highlighter` needs to be recreated for each _view_. It _would_ be nice to keep it over several frame.


### Usage

Include the package in your `Cargo.toml`, then supply the _custom_ highlighter like this:
```rust
struct YourState{
//..other stuff
  //safe the _settings_ in you application state. This prevents reloading
  // TS for each highlighting.
  ts: TSSettings
}
//in your state's view function:
fn view(&self) -> Element<Message>{
    text_editor(&self.content)
        .highlight_with::<iced_highlighter_tree_sitter::Highlighter>(
            self.ts.clone(),
            iced_highlighter_tree_sitter::to_format
    )
}
```

Note that you can initialize `TSSettings` with _any_ tree-sitter language. Have a look at the `rusteditor` example on how that works.


### Example

```
cargo run --example rusteditor
```
