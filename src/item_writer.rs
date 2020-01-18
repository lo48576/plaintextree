//! Tree node writer.

use std::{
    fmt::{self, Write},
    mem,
};

use crate::config::{EdgeConfig, ItemStyle, PrefixPart, TreeConfig};

/// A sink to write single item.
pub(crate) struct ItemWriter<'a, W> {
    /// Writer.
    writer: &'a mut W,
    /// Writer options.
    opts: TreeConfig,
    /// Item writer state.
    states: &'a mut [ItemState],
}

impl<'a, W: fmt::Write> ItemWriter<'a, W> {
    /// Creates a new `ItemWriter`.
    pub(crate) fn new(writer: &'a mut W, states: &'a mut [ItemState], opts: TreeConfig) -> Self {
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
        let emit_last_padding = self.opts.emit_trailing_whitespace() || !line_is_empty;
        let last_non_omissible_prefix_index = if emit_last_padding {
            assert!(!self.states.is_empty(), "Decrement should never overflow");
            Some(self.states.len() - 1)
        } else {
            self.states.iter().rposition(|state| {
                !state
                    .edge()
                    .is_prefix_whitespace(state.is_last_child(), state.at_first_line)
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
                        state.write_prefix(writer, opts.emit_trailing_whitespace())?;
                    }
                    if state.edge_status == LineEdgeStatus::PrefixEmitted {
                        state.write_padding(writer)?;
                    }
                    debug_assert_eq!(state.edge_status, LineEdgeStatus::PaddingEmitted);
                    Ok(())
                })?;

            let last_state = &mut states[last_non_omissible_prefix_index];
            if last_state.edge_status == LineEdgeStatus::LineStart {
                last_state.write_prefix(writer, opts.emit_trailing_whitespace())?;
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
                .writer(&mut self.writer, &mut self.states)
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
pub(crate) struct ItemState {
    /// Item style.
    style: ItemStyle,
    /// Whether the current line is the first line.
    at_first_line: bool,
    /// Edge emission status.
    edge_status: LineEdgeStatus,
}

impl ItemState {
    /// Returns whether the cursor is at the beginning of the line.
    pub(crate) fn is_at_line_head(&self) -> bool {
        self.edge_status == LineEdgeStatus::LineStart
    }

    /// Returns whether the item is the last child.
    fn is_last_child(&self) -> bool {
        self.style.is_last_child()
    }

    /// Returns the edge config.
    fn edge(&self) -> &EdgeConfig {
        self.style.edge()
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

        self.edge().write_edge(
            writer,
            self.is_last_child(),
            self.at_first_line,
            PrefixPart::Prefix,
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

        self.edge().write_edge(
            writer,
            self.is_last_child(),
            self.at_first_line,
            PrefixPart::Padding,
        )
    }

    /// Resets the writer status for the next new line.
    fn reset_line_state(&mut self) {
        self.at_first_line = false;
        self.edge_status = LineEdgeStatus::LineStart;
    }
}

impl From<ItemStyle> for ItemState {
    fn from(style: ItemStyle) -> Self {
        Self {
            style,
            at_first_line: true,
            edge_status: LineEdgeStatus::LineStart,
        }
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

    use crate::config::TreeConfigBuilder;

    #[test]
    fn empty_tree() {
        let mut buf = String::new();
        let _writer = ItemWriter::new(
            &mut buf,
            &mut [ItemStyle::non_last(EdgeConfig::Ascii).into()],
            TreeConfig::new(),
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
    fn emit_test_tree(edge: EdgeConfig, opts: TreeConfig) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        let mut states = Vec::new();
        buf.write_str(".\n")?;

        {
            states.push(ItemStyle::non_last(edge.clone()).into());
            opts.writer(&mut buf, &mut states).write_str("foo\n")?;
            {
                states.push(ItemStyle::non_last(edge.clone()).into());
                opts.writer(&mut buf, &mut states).write_str("bar\n")?;
                {
                    states.push(ItemStyle::last(edge.clone()).into());
                    opts.writer(&mut buf, &mut states)
                        .write_str("baz\n\nbaz2\n")?;
                    states.pop();
                }
                states.pop();
            }
            {
                states.push(ItemStyle::last(edge.clone()).into());
                opts.writer(&mut buf, &mut states).write_str("qux\n")?;
                {
                    states.push(ItemStyle::last(edge.clone()).into());
                    opts.writer(&mut buf, &mut states).write_str("quux\n")?;
                    states.pop();
                }
                states.pop();
            }
            states.pop();
        }
        {
            states.push(ItemStyle::non_last(edge.clone()).into());
            opts.writer(&mut buf, &mut states).write_str("corge\n")?;
            states.pop();
        }
        {
            states.push(ItemStyle::last(edge.clone()).into());
            opts.writer(&mut buf, &mut states).write_str("grault\n")?;
            states.pop();
        }

        Ok(buf)
    }

    #[test]
    fn ascii_tree() -> fmt::Result {
        let got = emit_test_tree(EdgeConfig::Ascii, TreeConfig::new())?;

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
    fn unicode_single_width_tree() -> fmt::Result {
        let got = emit_test_tree(EdgeConfig::UnicodeSingleWidth, TreeConfig::new())?;

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
    fn unicode_double_width_tree() -> fmt::Result {
        let got = emit_test_tree(EdgeConfig::UnicodeDoubleWidth, TreeConfig::new())?;

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
    fn non_last_item_single_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::non_last(EdgeConfig::Ascii).into()];
        let mut writer = ItemWriter::new(&mut buf, states, TreeConfig::new());
        writer.write_str("foo")?;

        assert_eq!(buf, "|-- foo");
        Ok(())
    }

    #[test]
    fn last_item_single_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::last(EdgeConfig::Ascii).into()];
        let mut writer = ItemWriter::new(&mut buf, states, TreeConfig::new());
        writer.write_str("foo")?;

        assert_eq!(buf, "`-- foo");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::non_last(EdgeConfig::Ascii).into()];
        let mut writer = ItemWriter::new(&mut buf, states, TreeConfig::new());
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|\n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::last(EdgeConfig::Ascii).into()];
        let mut writer = ItemWriter::new(&mut buf, states, TreeConfig::new());
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n\n    bar");
        Ok(())
    }

    #[test]
    fn non_last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::non_last(EdgeConfig::Ascii).into()];
        let mut writer = {
            let mut opts = TreeConfigBuilder::new();
            opts.emit_trailing_whitespace(true);
            opts.build().writer(&mut buf, states)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "|-- foo\n|   \n|   bar");
        Ok(())
    }

    #[test]
    fn last_item_multi_line_with_trailing_spaces() -> fmt::Result {
        let mut buf = String::new();
        let states = &mut [ItemStyle::last(EdgeConfig::Ascii).into()];
        let mut writer = {
            let mut opts = TreeConfigBuilder::new();
            opts.emit_trailing_whitespace(true);
            opts.build().writer(&mut buf, states)
        };
        writer.write_str("foo\n\nbar")?;

        assert_eq!(buf, "`-- foo\n    \n    bar");
        Ok(())
    }
}
