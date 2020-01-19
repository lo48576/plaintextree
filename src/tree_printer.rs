//! Tree printer.

use std::{
    error,
    fmt::{self, Write},
};

use crate::{
    config::{ItemStyle, TreeConfig},
    item_writer::ItemState,
};

/// Tree print result.
pub type Result<T> = std::result::Result<T, Error>;

/// Tree print error.
#[derive(Debug, Clone)]
pub enum Error {
    /// Attempt to close a node when there are no open nodes.
    ExtraNodeClose,
    /// Backend formatter error.
    Format(fmt::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExtraNodeClose => {
                f.write_str("Attempt to close a node but there are no open nodes")
            }
            Self::Format(e) => write!(f, "Backend formatter error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Format(e) => Some(e),
            _ => None,
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Self {
        Self::Format(e)
    }
}

/// Tree printer.
pub struct TreePrinter<W> {
    /// Writer.
    writer: W,
    /// Options.
    opts: TreeConfig,
    /// Item writer states for each nest level.
    states: Vec<ItemState>,
}

impl<W: fmt::Write> TreePrinter<W> {
    /// Creates a new `TreePrinter`.
    pub fn new(writer: W, opts: TreeConfig) -> Self {
        Self {
            writer,
            opts,
            states: Vec::new(),
        }
    }

    /// Opens a new node with the given content.
    pub fn open_node(&mut self, style: ItemStyle, content: impl fmt::Display) -> Result<()> {
        // Go to newline before emitting new node.
        if !self.states.is_empty() {
            self.opts
                .writer(&mut self.writer, &mut self.states)
                .go_to_next_line()?;
        }

        self.states.push(style.into());
        self.opts
            .writer(&mut self.writer, &mut self.states)
            .write_fmt(format_args!("{}", content))?;

        Ok(())
    }

    /// Closes a node.
    pub fn close_node(&mut self) -> Result<()> {
        if self.states.is_empty() {
            // Too much close!
            return Err(Error::ExtraNodeClose);
        }

        if self.opts.emit_trailing_newline() {
            // Go to newline automatically at the end of a node.
            self.opts
                .writer(&mut self.writer, &mut self.states)
                .go_to_next_line()?;
        }

        self.states.pop();

        Ok(())
    }

    /// Finishes writing the tree and returns the inner writer.
    pub fn finalize(mut self) -> Result<W> {
        for _ in 0..self.states.len() {
            self.close_node()?;
        }
        assert!(self.states.is_empty());

        Ok(self.writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::EdgeConfig;

    fn emit_test_tree(edge: EdgeConfig) -> Result<String> {
        let mut buf = String::new();
        buf.write_str(".\n")?;
        let mut printer = TreePrinter::new(&mut buf, TreeConfig::new());

        printer.open_node(ItemStyle::non_last(edge.clone()), "foo")?;
        printer.open_node(ItemStyle::non_last(edge.clone()), "bar")?;
        printer.open_node(ItemStyle::last(edge.clone()), "baz\n\nbaz2")?;
        printer.close_node()?;
        printer.close_node()?;
        printer.open_node(ItemStyle::last(edge.clone()), "qux")?;
        printer.open_node(ItemStyle::last(edge.clone()), "quux")?;
        printer.close_node()?;
        printer.close_node()?;
        printer.close_node()?;
        printer.open_node(ItemStyle::non_last(edge.clone()), "corge\n")?;
        printer.close_node()?;
        printer.open_node(ItemStyle::last(edge.clone()), "grault")?;
        printer.close_node()?;

        printer.finalize()?;
        Ok(buf)
    }

    #[test]
    fn ascii() -> Result<()> {
        let got = emit_test_tree(EdgeConfig::Ascii)?;

        let expected = ".\n\
                        |-- foo\n\
                        |   |-- bar\n\
                        |   |   `-- baz\n\
                        |   |\n\
                        |   |       baz2\n\
                        |   `-- qux\n\
                        |       `-- quux\n\
                        |-- corge\n\
                        `-- grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_single_width() -> Result<()> {
        let got = emit_test_tree(EdgeConfig::UnicodeSingleWidth)?;

        let expected = ".\n\
                        ├── foo\n\
                        │   ├── bar\n\
                        │   │   └── baz\n\
                        │   │\n\
                        │   │       baz2\n\
                        │   └── qux\n\
                        │       └── quux\n\
                        ├── corge\n\
                        └── grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_double_width() -> Result<()> {
        use crate::unicode::{self, UnicodeEdgeConfigBuilder};

        let uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Double)
            .build()
            .expect("Should never fail: Valid styles combination");
        let got = emit_test_tree(EdgeConfig::Unicode(uniconf))?;

        let expected = ".\n\
                        ├─ foo\n\
                        │   ├─ bar\n\
                        │   │   └─ baz\n\
                        │   │\n\
                        │   │        baz2\n\
                        │   └─ qux\n\
                        │        └─ quux\n\
                        ├─ corge\n\
                        └─ grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_custom1() -> Result<()> {
        use crate::unicode::{self, UnicodeEdgeConfigBuilder};

        let uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
            .vertical(unicode::EdgeStyle::Double)
            .build()
            .expect("Should never fail: Valid styles combination");
        let got = emit_test_tree(EdgeConfig::Unicode(uniconf))?;

        let expected = ".\n\
                        ╟── foo\n\
                        ║   ╟── bar\n\
                        ║   ║   ╙── baz\n\
                        ║   ║\n\
                        ║   ║       baz2\n\
                        ║   ╙── qux\n\
                        ║       ╙── quux\n\
                        ╟── corge\n\
                        ╙── grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_custom2() -> Result<()> {
        use crate::unicode::{self, UnicodeEdgeConfigBuilder};

        let uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
            .horizontal(unicode::EdgeStyle::Double)
            .build()
            .expect("Should never fail: Valid styles combination");
        let got = emit_test_tree(EdgeConfig::Unicode(uniconf))?;

        let expected = ".\n\
                        ╞══ foo\n\
                        │   ╞══ bar\n\
                        │   │   ╘══ baz\n\
                        │   │\n\
                        │   │       baz2\n\
                        │   ╘══ qux\n\
                        │       ╘══ quux\n\
                        ╞══ corge\n\
                        ╘══ grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_custom3() -> Result<()> {
        use crate::unicode::{self, UnicodeEdgeConfigBuilder};

        let uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
            .vertical(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
            .horizontal(unicode::EdgeStyle::Dashed(
                unicode::EdgeWidth::Narrow,
                unicode::DashLevel::Triple,
            ))
            .build()
            .expect("Should never fail: Valid styles combination");
        let got = emit_test_tree(EdgeConfig::Unicode(uniconf))?;

        let expected = ".\n\
                        ┠┄┄ foo\n\
                        ┃   ┠┄┄ bar\n\
                        ┃   ┃   ┖┄┄ baz\n\
                        ┃   ┃\n\
                        ┃   ┃       baz2\n\
                        ┃   ┖┄┄ qux\n\
                        ┃       ┖┄┄ quux\n\
                        ┠┄┄ corge\n\
                        ┖┄┄ grault\n";
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn unicode_custom4() -> Result<()> {
        use crate::unicode::{self, UnicodeEdgeConfigBuilder};

        let got = {
            let mut printer = TreePrinter::new(".\n".to_owned(), TreeConfig::new());

            let foo_uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                .build()
                .expect("Should never fail: Valid styles combination");
            printer.open_node(ItemStyle::non_last(EdgeConfig::Unicode(foo_uniconf)), "foo")?;
            printer.close_node()?; // foo
            let bar_uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                .vertical_forward(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                .horizontal(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                .build()
                .expect("Should never fail: Valid styles combination");
            printer.open_node(
                ItemStyle::non_last(EdgeConfig::Unicode(bar_uniconf)),
                "bar\nbar2",
            )?;
            printer.close_node()?; // bar
            let baz_uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                .vertical_backward(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                .horizontal(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                .build()
                .expect("Should never fail: Valid styles combination");
            printer.open_node(ItemStyle::non_last(EdgeConfig::Unicode(baz_uniconf)), "baz")?;
            let qux_uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                .vertical(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                .build()
                .expect("Should never fail: Valid styles combination");
            printer.open_node(ItemStyle::non_last(EdgeConfig::Unicode(qux_uniconf)), "qux")?;
            let quux_uniconf = UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                .horizontal(unicode::EdgeStyle::Dashed(
                    unicode::EdgeWidth::Narrow,
                    unicode::DashLevel::Triple,
                ))
                .build()
                .expect("Should never fail: Valid styles combination");
            printer.open_node(ItemStyle::last(EdgeConfig::Unicode(quux_uniconf)), "quux")?;
            printer.close_node()?; // quux
            printer.close_node()?; // qux
            let corge_uniconf =
                UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                    .vertical(unicode::EdgeStyle::Solid(unicode::EdgeWidth::Bold))
                    .horizontal(unicode::EdgeStyle::Dashed(
                        unicode::EdgeWidth::Bold,
                        unicode::DashLevel::Double,
                    ))
                    .build()
                    .expect("Should never fail: Valid styles combination");
            printer.open_node(ItemStyle::last(EdgeConfig::Unicode(corge_uniconf)), "corge")?;
            printer.close_node()?; // corge
            printer.close_node()?; // baz
            let grault_uniconf =
                UnicodeEdgeConfigBuilder::with_ambiwidth(unicode::AmbiWidth::Single)
                    .corner(unicode::CornerStyle::Round)
                    .build()
                    .expect("Should never fail: Valid styles combination");
            printer.open_node(
                ItemStyle::last(EdgeConfig::Unicode(grault_uniconf)),
                "grault",
            )?;
            printer.close_node()?; // grault

            printer.finalize()?
        };

        let expected = ".\n\
                        ├── foo\n\
                        ┢━━ bar\n\
                        ┃   bar2\n\
                        ┡━━ baz\n\
                        │   ┠── qux\n\
                        │   ┃   └┄┄ quux\n\
                        │   ┗╍╍ corge\n\
                        ╰── grault\n";
        assert_eq!(got, expected);
        Ok(())
    }
}
