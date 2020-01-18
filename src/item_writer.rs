//! Tree node writer.

use std::{
    fmt::{self, Write},
    mem,
};

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
#[non_exhaustive]
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

    /// Returns whether the prefix and padding consist of whitespaces.
    ///
    /// When both of prefix and padding are empty, this should return `true` (i.e. an empty string
    /// should be considered as "whitespaces").
    fn is_prefix_whitespace(&self, last_child: bool, first_line: bool) -> bool {
        match self {
            Self::Ascii | Self::UnicodeSingleWidth | Self::UnicodeDoubleWidth => {
                last_child && !first_line
            }
        }
    }
}

impl Default for EdgeConfig {
    fn default() -> Self {
        EdgeConfig::Ascii
    }
}

/// Options for `ItemWriter`.
#[derive(Default, Debug, Clone, Copy)]
pub struct ItemWriterOptions {
    /// Whether to emit trailing whitespace.
    emit_trailing_whitespace: bool,
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
    /// use plaintextree::{EdgeConfig, ItemWriterState};
    /// let mut buf = String::new();
    /// let mut states = &mut [ItemWriterState::new(true, EdgeConfig::Ascii)];
    /// let mut writer = {
    ///     let mut opts = ItemWriterOptions::new();
    ///     opts.emit_trailing_whitespace();
    ///     opts.build(&mut buf, states)
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

    /// Creates a new `ItemWriter`.
    pub fn build<'a, W: fmt::Write>(
        self,
        writer: &'a mut W,
        states: &'a mut [ItemWriterState],
    ) -> ItemWriter<'a, W> {
        ItemWriter::with_options(writer, states, self)
    }
}

/// A sink to write single item.
pub struct ItemWriter<'a, W> {
    /// Writer.
    writer: &'a mut W,
    /// Writer options.
    opts: ItemWriterOptions,
    /// Item writer state.
    states: &'a mut [ItemWriterState],
}

impl<'a, W: fmt::Write> ItemWriter<'a, W> {
    /// Creates a new `ItemWriter` with the given node writer states.
    pub fn new(writer: &'a mut W, states: &'a mut [ItemWriterState]) -> Self {
        Self::with_options(writer, states, Default::default())
    }

    /// Creates a new `ItemWriter` with the given node writer states and options.
    fn with_options(
        writer: &'a mut W,
        states: &'a mut [ItemWriterState],
        opts: ItemWriterOptions,
    ) -> Self {
        Self {
            writer,
            states,
            opts,
        }
    }

    /// Writes line prefixes and paddings if necessary.
    fn write_prefix_and_padding(&mut self, line_is_empty: bool) -> fmt::Result {
        if self.states.is_empty() {
            return Ok(());
        }

        // Delay the emission of the prefixes and paddings in some cases.
        let emit_last_padding = self.opts.emit_trailing_whitespace || !line_is_empty;
        let last_non_omissible_prefix_index = if emit_last_padding {
            assert!(!self.states.is_empty(), "Decrement should never overflow");
            Some(self.states.len() - 1)
        } else {
            self.states.iter().rposition(|state| {
                !state
                    .edge
                    .is_prefix_whitespace(state.is_last_child, state.at_first_line)
            })
        };
        if let Some(last_non_omissible_prefix_index) = last_non_omissible_prefix_index {
            let Self {
                writer,
                states,
                opts,
                ..
            } = self;
            let writer: &mut W = *writer;
            states
                .iter_mut()
                .take(last_non_omissible_prefix_index)
                .try_for_each(|state| {
                    if state.edge_status == LineEdgeStatus::LineStart {
                        state.write_prefix(writer, opts.emit_trailing_whitespace)?;
                    }
                    if state.edge_status == LineEdgeStatus::PrefixEmitted {
                        state.write_padding(writer)?;
                    }
                    debug_assert_eq!(state.edge_status, LineEdgeStatus::PaddingEmitted);
                    Ok(())
                })?;

            let last_state = &mut states[last_non_omissible_prefix_index];
            if last_state.edge_status == LineEdgeStatus::LineStart {
                last_state.write_prefix(writer, opts.emit_trailing_whitespace)?;
            }
            if last_state.edge_status == LineEdgeStatus::PrefixEmitted && emit_last_padding {
                last_state.write_padding(writer)?;
            }
        }

        Ok(())
    }

    /// Resets the writer status for the next new line.
    fn reset_line_state(&mut self) {
        self.states
            .iter_mut()
            .for_each(|state| state.reset_line_state());
    }

    /// Writes a newline character if necessary, and moves the cursor to the head of the next line.
    pub(crate) fn go_to_next_line(&mut self) -> fmt::Result {
        let last_state = self
            .states
            .last()
            .expect("Should never fail: `states` must not be empty");
        if !last_state.is_at_line_head() {
            self.opts
                .build(&mut self.writer, &mut self.states)
                .write_str("\n")?;
        }
        debug_assert!(self
            .states
            .last()
            .expect("Should never fail: `states` must not be empty")
            .is_at_line_head());

        Ok(())
    }
}

impl<'a, W: fmt::Write> fmt::Write for ItemWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (line, at_last_line) in lines_with_last_line_flag(s) {
            // Delay the emission of the prefix (and padding) until the line content is given.
            if at_last_line && line.is_empty() {
                break;
            }

            // Write line prefixes and paddings if necessary.
            self.write_prefix_and_padding(line.is_empty())?;

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
pub struct ItemWriterState {
    /// Whether the item is the last child.
    is_last_child: bool,
    /// Edge config.
    edge: EdgeConfig,
    /// Whether the current line is the first line.
    at_first_line: bool,
    /// Edge emission status.
    edge_status: LineEdgeStatus,
}

impl ItemWriterState {
    /// Creates a new `ItemWriterState`.
    pub fn new(is_last_child: bool, edge: EdgeConfig) -> Self {
        Self {
            is_last_child,
            edge,
            at_first_line: true,
            edge_status: LineEdgeStatus::LineStart,
        }
    }

    /// Returns whether the cursor is at the beginning of the line.
    pub(crate) fn is_at_line_head(&self) -> bool {
        self.edge_status == LineEdgeStatus::LineStart
    }

    /// Writes a line prefix (and padding if possible) for the current line.
    fn write_prefix<W: fmt::Write>(
        &mut self,
        writer: &mut W,
        emit_trailing_whitespace: bool,
    ) -> fmt::Result {
        assert_eq!(
            self.edge_status,
            LineEdgeStatus::LineStart,
            "Prefix should be emitted only once for each line"
        );
        self.edge_status = LineEdgeStatus::PrefixEmitted;

        self.edge.write_edge(
            writer,
            self.is_last_child,
            self.at_first_line,
            PrefixOrPadding::Prefix,
        )?;

        if emit_trailing_whitespace {
            // Padding is always necessary.
            self.write_padding(writer)?;
        }

        Ok(())
    }

    /// Writes a padding after the line prefix.
    fn write_padding<W: fmt::Write>(&mut self, writer: &mut W) -> fmt::Result {
        assert_eq!(
            self.edge_status,
            LineEdgeStatus::PrefixEmitted,
            "Prefix should be emitted only once after each line prefix"
        );
        self.edge_status = LineEdgeStatus::PaddingEmitted;

        self.edge.write_edge(
            writer,
            self.is_last_child,
            self.at_first_line,
            PrefixOrPadding::Padding,
        )
    }

    /// Resets the writer status for the next new line.
    fn reset_line_state(&mut self) {
        self.at_first_line = false;
        self.edge_status = LineEdgeStatus::LineStart;
    }
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
        let _writer = ItemWriter::new(
            &mut buf,
            &mut [ItemWriterState::new(false, EdgeConfig::Ascii)],
        );
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
    fn emit_test_tree(edge: EdgeConfig, opts: ItemWriterOptions) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        let mut states = Vec::new();
        buf.write_str(".\n")?;

        {
            states.push(ItemWriterState::new(false, edge.clone()));
            opts.build(&mut buf, &mut states).write_str("foo\n")?;
            {
                states.push(ItemWriterState::new(false, edge.clone()));
                opts.build(&mut buf, &mut states).write_str("bar\n")?;
                {
                    states.push(ItemWriterState::new(true, edge.clone()));
                    opts.build(&mut buf, &mut states)
                        .write_str("baz\n\nbaz2\n")?;
                    states.pop();
                }
                states.pop();
            }
            {
                states.push(ItemWriterState::new(true, edge.clone()));
                opts.build(&mut buf, &mut states).write_str("qux\n")?;
                {
                    states.push(ItemWriterState::new(true, edge.clone()));
                    opts.build(&mut buf, &mut states).write_str("quux\n")?;
                    states.pop();
                }
                states.pop();
            }
            states.pop();
        }
        {
            states.push(ItemWriterState::new(false, edge.clone()));
            opts.build(&mut buf, &mut states).write_str("corge\n")?;
            states.pop();
        }
        {
            states.push(ItemWriterState::new(true, edge.clone()));
            opts.build(&mut buf, &mut states).write_str("grault\n")?;
            states.pop();
        }

        Ok(buf)
    }

    #[test]
    fn ascii_tree() -> fmt::Result {
        let got = emit_test_tree(EdgeConfig::Ascii, ItemWriterOptions::new())?;

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
        let got = emit_test_tree(EdgeConfig::UnicodeSingleWidth, ItemWriterOptions::new())?;

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
        let got = emit_test_tree(EdgeConfig::UnicodeDoubleWidth, ItemWriterOptions::new())?;

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
        let states = &mut [ItemWriterState::new(false, EdgeConfig::Ascii)];
        let mut writer = ItemWriter::new(&mut buf, states);
        writer.write_str("foo")?;

        assert_eq!(buf, "|-- foo");
        Ok(())
    }

    #[test]
    fn last_item_single_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemWriterState::new(true, EdgeConfig::Ascii)];
        let mut writer = ItemWriter::new(&mut buf, states);
        writer.write_str("foo")?;

        assert_eq!(buf, "`-- foo");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemWriterState::new(false, EdgeConfig::Ascii)];
        let mut writer = ItemWriter::new(&mut buf, states);
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|\n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemWriterState::new(true, EdgeConfig::Ascii)];
        let mut writer = ItemWriter::new(&mut buf, states);
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n\n    bar");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemWriterState::new(false, EdgeConfig::Ascii)];
        let mut writer = {
            let mut opts = ItemWriterOptions::new();
            opts.emit_trailing_whitespace();
            opts.build(&mut buf, states)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|   \n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemWriterState::new(true, EdgeConfig::Ascii)];
        let mut writer = {
            let mut opts = ItemWriterOptions::new();
            opts.emit_trailing_whitespace();
            opts.build(&mut buf, states)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n    \n    bar");
        Ok(())
    }
}
