//! Tree node writer.

use std::{fmt, mem};

/// Options for `ItemWriter`.
#[derive(Default, Debug, Clone)]
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

    /// Creates a new `ItemWriter`.
    pub fn build<W: fmt::Write>(self, writer: W, is_last_child: bool) -> ItemWriter<W> {
        ItemWriter::with_options(writer, is_last_child, self)
    }
}

/// A sink to write single item.
pub struct ItemWriter<W> {
    /// Writer.
    writer: W,
    /// Whether the item is the last child.
    is_last_child: bool,
    /// Options.
    opts: ItemWriterOptions,
    /// Whether the current line is the first line.
    at_first_line: bool,
    /// Prefix emission status.
    prefix_status: LinePrefixStatus,
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
            is_last_child,
            opts,
            at_first_line: true,
            prefix_status: LinePrefixStatus::LineStart,
        }
    }

    /// Writes a line prefix (and padding if possible) for the current line.
    fn write_prefix(&mut self) -> fmt::Result {
        assert_eq!(
            self.prefix_status,
            LinePrefixStatus::LineStart,
            "Prefix should be emitted only once for each line"
        );
        self.prefix_status = LinePrefixStatus::PrefixEmitted;

        match (self.at_first_line, self.is_last_child) {
            (true, true) => self.writer.write_str("`--"),
            (true, false) => self.writer.write_str("|--"),
            (false, true) => self.writer.write_str(""),
            (false, false) => self.writer.write_str("|"),
        }?;

        if self.opts.emit_trailing_whitespace {
            // Padding is always necessary.
            self.write_padding()?;
        }

        Ok(())
    }

    /// Writes a padding after the line prefix.
    fn write_padding(&mut self) -> fmt::Result {
        assert_eq!(
            self.prefix_status,
            LinePrefixStatus::PrefixEmitted,
            "Prefix should be emitted only once after each line prefix"
        );
        self.prefix_status = LinePrefixStatus::PaddingEmitted;

        match (self.at_first_line, self.is_last_child) {
            (true, _) => self.writer.write_str(" "),
            (false, true) => self.writer.write_str("    "),
            (false, false) => self.writer.write_str("   "),
        }
    }

    /// Resets the writer status for the next new line.
    fn reset_line_state(&mut self) {
        self.at_first_line = false;
        self.prefix_status = LinePrefixStatus::LineStart;
    }
}

impl<W: fmt::Write> fmt::Write for ItemWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (line, at_last_line) in lines_with_last_line_flag(s) {
            // Delay the emission of the prefix (and padding) until the line content is given.
            if at_last_line && line.is_empty() {
                break;
            }

            // Write a line prefix if necessary.
            if self.prefix_status == LinePrefixStatus::LineStart {
                self.write_prefix()?;
            }

            // Write a padding if necessary.
            // Delay the emission of the padding until the line content is given.
            if self.prefix_status == LinePrefixStatus::PrefixEmitted
                && (self.opts.emit_trailing_whitespace || !line.is_empty())
            {
                self.write_padding()?;
            }

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

/// Line prefix emission status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LinePrefixStatus {
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
    fn non_last_item_single_line_with_trailing_newline() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, false);
        writer.write_str("foo\n")?;

        assert_eq!(buf, "|-- foo\n");
        Ok(())
    }

    #[test]
    fn last_item_single_line_with_trailing_newline() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, true);
        writer.write_str("foo\n")?;

        assert_eq!(buf, "`-- foo\n");
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
    fn non_last_item_multi_line_with_trailing_newline() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, false);
        writer.write_str("foo\n\nbar\n")?;

        assert_eq!(buf, "|-- foo\n|\n|   bar\n");
        Ok(())
    }

    #[test]
    fn last_item_multi_line_with_trailing_newline() -> fmt::Result {
        let mut buf = String::new();
        let mut writer = ItemWriter::new(&mut buf, true);
        writer.write_str("foo\n\nbar\n")?;

        assert_eq!(buf, "`-- foo\n\n    bar\n");
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
