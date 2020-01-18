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
    ///
    /// Prefer [`last()`][last] and [`non_last()`][non_last] when `is_last_child` argument is
    /// constant.
    ///
    /// [last]: #method.last
    /// [non_last]: #method.non_last
    pub fn new(is_last_child: bool, edge: EdgeConfig) -> Self {
        Self {
            is_last_child,
            edge,
        }
    }

    /// Creates a new `ItemStyle` for the last child.
    ///
    /// This is same as `ItemStyle::new(true, edge)`.
    pub fn last(edge: EdgeConfig) -> Self {
        Self::new(true, edge)
    }

    /// Creates a new `ItemStyle` for a non-last child.
    ///
    /// This is same as `ItemStyle::new(false, edge)`.
    pub fn non_last(edge: EdgeConfig) -> Self {
        Self::new(false, edge)
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
    /// The value is `false` by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use plaintextree::{EdgeConfig, ItemStyle, TreeConfigBuilder, TreePrinter};
    ///
    /// let opts = TreeConfigBuilder::new()
    ///     .emit_trailing_whitespace(true)
    ///     .build();
    ///
    /// let mut writer = TreePrinter::new(String::new(), opts);
    /// writer.open_node(ItemStyle::last(EdgeConfig::Ascii), "foo\n\nbar")?;
    /// let buf = writer.finalize()?;
    ///
    /// // Note that `"    "` is emitted for an empty line between "foo" and "bar".
    /// assert_eq!(buf, "`-- foo\n    \n    bar\n");
    /// # plaintextree::Result::Ok(())
    /// ```
    pub fn emit_trailing_whitespace(&mut self, v: bool) -> &mut Self {
        self.config.emit_trailing_whitespace = v;
        self
    }

    /// Let the writer emit trailing newline automatically at the end of the tree.
    ///
    /// The value is `true` by default.
    ///
    /// # Examples
    ///
    /// By default, the tree printer will put a newline at the end of the tree, if there is not.
    ///
    /// ```
    /// use plaintextree::{EdgeConfig, ItemStyle, TreeConfig, TreePrinter};
    ///
    /// let opts = TreeConfig::new();
    ///
    /// let mut writer = TreePrinter::new(String::new(), opts);
    /// writer.open_node(ItemStyle::last(EdgeConfig::Ascii), "foo")?;
    /// let buf = writer.finalize()?;
    ///
    /// // Note that the newline character at the end of the output.
    /// assert_eq!(buf, "`-- foo\n");
    /// # plaintextree::Result::Ok(())
    /// ```
    ///
    /// If there are already a newline, the printer does not emit an additional newline.
    ///
    /// ```
    /// use plaintextree::{EdgeConfig, ItemStyle, TreeConfig, TreePrinter};
    ///
    /// let opts = TreeConfig::new();
    ///
    /// let mut writer = TreePrinter::new(String::new(), opts);
    /// // Feed a trailing newline explicitly.
    /// writer.open_node(ItemStyle::last(EdgeConfig::Ascii), "foo\n")?;
    /// let buf = writer.finalize()?;
    ///
    /// // Note that there are only one newline character at the end of the output.
    /// assert_eq!(buf, "`-- foo\n");
    /// # plaintextree::Result::Ok(())
    /// ```
    ///
    /// With this flag unset, the tree printer does not emit an additional newline.
    ///
    /// ```
    /// use plaintextree::{EdgeConfig, ItemStyle, TreeConfigBuilder, TreePrinter};
    ///
    /// let opts = TreeConfigBuilder::new()
    ///     .emit_trailing_newline(false)
    ///     .build();
    ///
    /// let mut writer = TreePrinter::new(String::new(), opts);
    /// writer.open_node(ItemStyle::last(EdgeConfig::Ascii), "foo")?;
    /// let buf = writer.finalize()?;
    ///
    /// // Note that there are no newline characters at the end of the output.
    /// assert_eq!(buf, "`-- foo");
    /// # plaintextree::Result::Ok(())
    /// ```
    pub fn emit_trailing_newline(&mut self, v: bool) -> &mut Self {
        self.config.emit_trailing_newline = v;
        self
    }

    /// Builds a `TreeConfig`.
    pub fn build(self) -> TreeConfig {
        self.config
    }
}

/// Options common for a tree.
#[derive(Debug, Clone, Copy)]
pub struct TreeConfig {
    /// Whether to emit trailing whitespace.
    ///
    /// Default is `false`.
    emit_trailing_whitespace: bool,
    /// Whether to emit a newline automatically at the tail of the tree.
    ///
    /// Default is `true`.
    emit_trailing_newline: bool,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            emit_trailing_whitespace: false,
            emit_trailing_newline: true,
        }
    }
}

impl TreeConfig {
    /// Creates a new default `TreeConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the writer should emit trailing whitespace if the line has no content.
    pub(crate) fn emit_trailing_whitespace(self) -> bool {
        self.emit_trailing_whitespace
    }

    /// Returns whether the writer should emit trailing newline at the tail of the tree.
    pub(crate) fn emit_trailing_newline(self) -> bool {
        self.emit_trailing_newline
    }

    /// Creates a new `ItemWriter`.
    pub(crate) fn writer<'a, W: fmt::Write>(
        self,
        writer: &'a mut W,
        states: &'a mut [ItemState],
    ) -> ItemWriter<'a, W> {
        ItemWriter::new(writer, states, self)
    }
}
