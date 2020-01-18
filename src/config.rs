//! Config types.

use std::fmt;

use crate::item_writer::{ItemState, ItemWriter};

/// Part of a prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrefixPart {
    /// Non-whitespace part of a prefix.
    Prefix,
    /// Padding, whitespace-only suffix part of a prefix.
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
    pub(crate) fn write_edge<W: fmt::Write>(
        &self,
        writer: &mut W,
        last_child: bool,
        first_line: bool,
        fragment: PrefixPart,
    ) -> fmt::Result {
        use PrefixPart::{Padding, Prefix};

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
    pub(crate) fn is_prefix_whitespace(&self, last_child: bool, first_line: bool) -> bool {
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

/// Item style.
#[derive(Debug, Clone)]
pub struct ItemStyle {
    /// Whether the item is the last child.
    is_last_child: bool,
    /// Edge config.
    edge: EdgeConfig,
}

impl ItemStyle {
    /// Creates a new `ItemStyle`.
    pub fn new(is_last_child: bool, edge: EdgeConfig) -> Self {
        Self {
            is_last_child,
            edge,
        }
    }

    /// Returns whether the item is the last child.
    pub(crate) fn is_last_child(&self) -> bool {
        self.is_last_child
    }

    /// Returns the edge config.
    pub(crate) fn edge(&self) -> &EdgeConfig {
        &self.edge
    }
}

/// `TreeConfig` builder.
#[derive(Default, Debug, Clone, Copy)]
pub struct TreeConfigBuilder {
    /// Current config.
    config: TreeConfig,
}

impl TreeConfigBuilder {
    /// Creates a new `TreeConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Let the writer emit trailing whitespace if the line has no content.
    ///
    /// # Examples
    ///
    /// ```
    /// use plaintextree::{EdgeConfig, ItemStyle, TreeConfigBuilder, TreePrinter};
    /// let opts = {
    ///     let mut opts = TreeConfigBuilder::new();
    ///     opts.emit_trailing_whitespace();
    ///     opts.build()
    /// };
    /// let mut writer = TreePrinter::new(String::new(), opts);
    /// let style = ItemStyle::new(true, EdgeConfig::Ascii);
    /// writer.open_node(style, "foo\n\nbar")?;
    /// let buf = writer.finalize()?;
    ///
    /// // Note that `"    "` is emited for an empty line between "foo" and "bar".
    /// assert_eq!(buf, "`-- foo\n    \n    bar\n");
    /// # plaintextree::tree_printer::Result::Ok(())
    /// ```
    pub fn emit_trailing_whitespace(&mut self) -> &mut Self {
        self.config.emit_trailing_whitespace = true;
        self
    }

    /// Builds a `TreeConfig`.
    pub fn build(self) -> TreeConfig {
        self.config
    }
}

/// Options common for a tree.
#[derive(Default, Debug, Clone, Copy)]
pub struct TreeConfig {
    /// Whether to emit trailing whitespace.
    emit_trailing_whitespace: bool,
}

impl TreeConfig {
    /// Creates a new default `TreeConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the writer should emit trailing whitespace if the line has no content.
    pub fn emit_trailing_whitespace(self) -> bool {
        self.emit_trailing_whitespace
    }

    /// Creates a new `ItemWriter`.
    pub fn writer<'a, W: fmt::Write>(
        self,
        writer: &'a mut W,
        states: &'a mut [ItemState],
    ) -> ItemWriter<'a, W> {
        ItemWriter::with_options(writer, states, self)
    }
}
