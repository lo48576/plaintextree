//! Tree node writer.

use std::{fmt, mem};

/// Prefix or padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrefixOrPadding {
    /// Prefix.
    Prefix,
    /// Padding.
    Padding,
}

/// Edge config.
#[derive(Debug, Clone)]
pub enum EdgeConfig {
    /// Standard ASCII tree.
    ///
    /// The same style as [`tree` command][unix-tree] with `LANG=C` for UNIX.
    ///
    /// ```text
    /// .
    /// |-- foo
    /// |   |-- bar
    /// |   |   `-- baz
    /// |   |
    /// |   |       baz2
    /// |   `-- qux
    /// |       `-- quux
    /// |-- corge
    /// `-- grault
    /// ```
    ///
    /// [unix-tree]: http://mama.indstate.edu/users/ice/tree/
    Ascii,
    /// Unicode assuming ruled line characters are single width (half width).
    ///
    /// The same style as [`tree` command][unix-tree] with `LANG=(lang).utf8` for UNIX.
    ///
    /// This won't be shown correctly in CJK fonts, because they usually have double-width glyphs
    /// for ruled lines.
    /// Consider using [`UnicodeDoubleWidth`] for East Asian environment.
    ///
    /// About ambiguous width characters, see [UAX #11: East Asian Width][UAX-11].
    ///
    /// ```text
    /// .
    /// ├── foo
    /// │   ├── bar
    /// │   │   └── baz
    /// │   │
    /// │   │       baz2
    /// │   └── qux
    /// │       └── quux
    /// ├── corge
    /// └── grault
    /// ```
    ///
    /// [UAX-11]: https://unicode.org/reports/tr11/
    /// [unix-tree]: http://mama.indstate.edu/users/ice/tree/
    /// [`UnicodeDoubleWidth`]: #variant.UnicodeDoubleWidth
    UnicodeSingleWidth,
    /// Unicode assuming ruled line characters are double width (full width).
    ///
    /// This would be useful for **East Asian** environment.
    ///
    /// This won't be shown correctly in non-east-asian fonts, because they usually have
    /// single-width glyphs for ruled lines.
    ///
    /// About ambiguous width characters, see [UAX #11: East Asian Width][UAX-11].
    ///
    /// ```text
    /// .
    /// ├─ foo
    /// │   ├─ bar
    /// │   │   └─ baz
    /// │   │
    /// │   │        baz2
    /// │   └─ qux
    /// │        └─ quux
    /// ├─ corge
    /// └─ grault
    /// ```
    ///
    /// Note that the single indent depth has the width of 5 spaces, not 4 spaces.
    ///
    /// [UAX-11]: https://unicode.org/reports/tr11/
    UnicodeDoubleWidth,
}

impl EdgeConfig {
    /// Writes the prefix or padding with the given config.
    fn write_edge<W: fmt::Write>(
        &self,
        writer: &mut W,
        last_child: bool,
        first_line: bool,
        fragment: PrefixOrPadding,
    ) -> fmt::Result {
        use PrefixOrPadding::{Padding, Prefix};

        match self {
            Self::Ascii => match (first_line, last_child, fragment) {
                (true, true, Prefix) => writer.write_str("`--"),
                (true, false, Prefix) => writer.write_str("|--"),
                (true, _, Padding) => writer.write_str(" "),
                (false, true, Prefix) => writer.write_str(""),
                (false, true, Padding) => writer.write_str("    "),
                (false, false, Prefix) => writer.write_str("|"),
                (false, false, Padding) => writer.write_str("   "),
            },
            Self::UnicodeSingleWidth => match (first_line, last_child, fragment) {
                (true, true, Prefix) => writer.write_str("\u{2514}\u{2500}\u{2500}"),
                (true, false, Prefix) => writer.write_str("\u{251C}\u{2500}\u{2500}"),
                (true, _, Padding) => writer.write_str(" "),
                (false, true, Prefix) => writer.write_str(""),
                (false, true, Padding) => writer.write_str("    "),
                (false, false, Prefix) => writer.write_str("\u{2502}"),
                (false, false, Padding) => writer.write_str("   "),
            },
            Self::UnicodeDoubleWidth => match (first_line, last_child, fragment) {
                (true, true, Prefix) => writer.write_str("\u{2514}\u{2500}"),
                (true, false, Prefix) => writer.write_str("\u{251C}\u{2500}"),
                (true, _, Padding) => writer.write_str(" "),
                (false, true, Prefix) => writer.write_str(""),
                (false, true, Padding) => writer.write_str("     "),
                (false, false, Prefix) => writer.write_str("\u{2502}"),
                (false, false, Padding) => writer.write_str("   "),
            },
        }
    }
}

impl Default for EdgeConfig {
    fn default() -> Self {
        EdgeConfig::Ascii
    }
}

/// Options for `ItemWriter`.
#[derive(Default, Debug, Clone)]
pub struct ItemWriterOptions {
    /// Whether to emit trailing whitespace.
    emit_trailing_whitespace: bool,
    /// Edge (prefix and padding) config.
    edge: EdgeConfig,
}

impl ItemWriterOptions {
    /// Creates a new `ItemWriterOptions`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Let the writer emit trailing whitespace if the line has no content.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt::Write;
    /// # use plaintextree::ItemWriterOptions;
    /// let mut buf = String::new();
    /// let mut writer = {
    ///     let mut opts = ItemWriterOptions::new();
    ///     opts.emit_trailing_whitespace();
    ///     opts.build(&mut buf, true)
    /// };
    /// writer.write_str("foo\n\nbar")?;
    ///
    /// // Note that "    " is emited for an empty line between "foo" and "bar".
    /// assert_eq!(buf, "`-- foo\n    \n    bar");
    /// # std::fmt::Result::Ok(())
    /// ```
    pub fn emit_trailing_whitespace(&mut self) -> &mut Self {
        self.emit_trailing_whitespace = true;
        self
    }

    /// Sets the edge style.
    pub fn edge(&mut self, edge: EdgeConfig) -> &mut Self {
        self.edge = edge;
        self
    }

    /// Creates a new `ItemWriter`.
    pub fn build<W: fmt::Write>(self, writer: W, is_last_child: bool) -> ItemWriter<W> {
        ItemWriter::with_options(writer, is_last_child, self)
    }
}

/// A sink to write single item.
pub struct ItemWriter<W> {
    /// Writer.
    writer: W,
    /// Item writer state.
    state: ItemWriterState,
}

impl<W: fmt::Write> ItemWriter<W> {
    /// Creates a new `ItemWriter`.
    pub fn new(writer: W, is_last_child: bool) -> Self {
        Self::with_options(writer, is_last_child, Default::default())
    }

    /// Creates a new `ItemWriter` with the given options.
    fn with_options(writer: W, is_last_child: bool, opts: ItemWriterOptions) -> Self {
        Self {
            writer,
            state: ItemWriterState {
                is_last_child,
                opts,
                at_first_line: true,
                edge_status: LineEdgeStatus::LineStart,
            },
        }
    }

    /// Writes a line prefix (and padding if possible) for the current line.
    fn write_single_prefix(&mut self) -> fmt::Result {
        assert_eq!(
            self.state.edge_status,
            LineEdgeStatus::LineStart,
            "Prefix should be emitted only once for each line"
        );
        self.state.edge_status = LineEdgeStatus::PrefixEmitted;

        self.state.opts.edge.write_edge(
            &mut self.writer,
            self.state.is_last_child,
            self.state.at_first_line,
            PrefixOrPadding::Prefix,
        )?;

        if self.state.opts.emit_trailing_whitespace {
            // Padding is always necessary.
            self.write_single_padding()?;
        }

        Ok(())
    }

    /// Writes a padding after the line prefix.
    fn write_single_padding(&mut self) -> fmt::Result {
        assert_eq!(
            self.state.edge_status,
            LineEdgeStatus::PrefixEmitted,
            "Prefix should be emitted only once after each line prefix"
        );
        self.state.edge_status = LineEdgeStatus::PaddingEmitted;

        self.state.opts.edge.write_edge(
            &mut self.writer,
            self.state.is_last_child,
            self.state.at_first_line,
            PrefixOrPadding::Padding,
        )
    }

    /// Writes a line prefix and padding if necessary.
    fn write_single_prefix_for_line(&mut self, line: &str) -> fmt::Result {
        // Write a line prefix if necessary.
        if self.state.edge_status == LineEdgeStatus::LineStart {
            self.write_single_prefix()?;
        }

        // Write a padding if necessary.
        // Delay the emission of the padding until the line content is given.
        if self.state.edge_status == LineEdgeStatus::PrefixEmitted
            && (self.state.opts.emit_trailing_whitespace || !line.is_empty())
        {
            self.write_single_padding()?;
        }

        Ok(())
    }

    /// Resets the writer status for the next new line.
    fn reset_line_state(&mut self) {
        self.state.at_first_line = false;
        self.state.edge_status = LineEdgeStatus::LineStart;
    }
}

impl<W: fmt::Write> fmt::Write for ItemWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (line, at_last_line) in lines_with_last_line_flag(s) {
            // Delay the emission of the prefix (and padding) until the line content is given.
            if at_last_line && line.is_empty() {
                break;
            }

            // Write a line prefix and padding if necessary.
            self.write_single_prefix_for_line(line)?;

            // Write the line content.
            self.writer.write_str(line)?;

            // Write the newline if there are next lines to be written.
            if !at_last_line {
                self.writer.write_char('\n')?;
                self.reset_line_state();
            }
        }

        Ok(())
    }
}

/// Item writer state for single nest level.
#[derive(Debug, Clone)]
struct ItemWriterState {
    /// Whether the item is the last child.
    is_last_child: bool,
    /// Options.
    opts: ItemWriterOptions,
    /// Whether the current line is the first line.
    at_first_line: bool,
    /// Edge emission status.
    edge_status: LineEdgeStatus,
}

/// Line prefix emission status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LineEdgeStatus {
    /// At the beginning of the current line.
    LineStart,
    /// Emitted the prefix for the current line, but padding is not yet written.
    PrefixEmitted,
    /// Emitted both prefix and padding for the current line.
    PaddingEmitted,
}

/// Returns an iterator of lines with "last line" flag.
fn lines_with_last_line_flag(s: &str) -> impl Iterator<Item = (&str, bool)> {
    let mut lines_raw = s.lines();
    let mut current = lines_raw.next();
    // `<str>::lines()` treats the trailing "\n" as a line ending, but does not consider it as a
    // beginning of a new line.
    // This flag is necessary to emit extra line if the string has a trailing newline.
    let mut emit_extra_line = s.bytes().last() == Some(b'\n');

    std::iter::from_fn(move || match lines_raw.next() {
        Some(next) => Some((
            current
                .replace(next)
                .expect("Should never fail: previous item must be emitted by the iterator"),
            false,
        )),
        None => match current.take() {
            Some(current) => Some((current, !emit_extra_line)),
            None => {
                if mem::replace(&mut emit_extra_line, false) {
                    Some(("", true))
                } else {
                    None
                }
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Write;

    #[test]
    fn empty_tree() {
        let mut buf = String::new();
        let _writer = ItemWriter::new(&mut buf, false);
        assert!(
            buf.is_empty(),
            "Writer should write nothing until it is told to write something"
        );
    }

    /// Emits the tree for testing.
    ///
    /// ```text
    /// .
    /// |-- foo
    /// |   |-- bar
    /// |   |   `-- baz
    /// |   |
    /// |   |       baz2
    /// |   `-- qux
    /// |       `-- quux
    /// |-- corge
    /// `-- grault
    /// ```
    fn emit_test_tree(opts: ItemWriterOptions) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        buf.write_str(".\n")?;
        {
            let mut foo = opts.clone().build(&mut buf, false);
            foo.write_str("foo\n")?;
            {
                let mut bar = opts.clone().build(&mut foo, false);
                bar.write_str("bar\n")?;
                opts.clone()
                    .build(&mut bar, true)
                    .write_str("baz\n\nbaz2\n")?;
            }
            {
                let mut qux = opts.clone().build(&mut foo, true);
                qux.write_str("qux\n")?;
                opts.clone().build(&mut qux, true).write_str("quux\n")?;
            }
        }
        opts.clone().build(&mut buf, false).write_str("corge\n")?;
        opts.clone().build(&mut buf, true).write_str("grault\n")?;

        Ok(buf)
    }

    #[test]
    fn ascii_tree() -> fmt::Result {
        let opts = {
            let mut opts = ItemWriterOptions::new();
            opts.edge(EdgeConfig::Ascii);
            opts
        };
        let got = emit_test_tree(opts)?;

        let expected = "\
                        .\n\
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
    fn unicode_single_width_tree() -> fmt::Result {
        let opts = {
            let mut opts = ItemWriterOptions::new();
            opts.edge(EdgeConfig::UnicodeSingleWidth);
            opts
        };
        let got = emit_test_tree(opts)?;

        let expected = "\
                        .\n\
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
    fn unicode_double_width_tree() -> fmt::Result {
        let opts = {
            let mut opts = ItemWriterOptions::new();
            opts.edge(EdgeConfig::UnicodeDoubleWidth);
            opts
        };
        let got = emit_test_tree(opts)?;

        let expected = "\
                        .\n\
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
    fn non_last_item_single_line() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, false);
        writer.write_str("foo")?;

        assert_eq!(buf, "|-- foo");
        Ok(())
    }

    #[test]
    fn last_item_single_line() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, true);
        writer.write_str("foo")?;

        assert_eq!(buf, "`-- foo");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, false);
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|\n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, true);
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n\n    bar");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = {
            let mut opts = ItemWriterOptions::new();
            opts.emit_trailing_whitespace();
            opts.build(&mut buf, false)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|   \n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = {
            let mut opts = ItemWriterOptions::new();
            opts.emit_trailing_whitespace();
            opts.build(&mut buf, true)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n    \n    bar");
        Ok(())
    }
}
