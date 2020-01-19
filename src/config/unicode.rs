//! Unicode edge configs.
//  Character roles are named as `{{item_pos}}_{{line_pos}}_{{char_pos}}`.
//
//  * item_pos:
//      + `preceding`: Non-last item.
//      + `last`: Last item.
//      + `any`: Any item.
//  * line_pos:
//      + `first`: First line in an item.
//      + `succeeding`: Non-first line in an item.
//      + `any`: Any line in an item.
//  * char_pos:
//      + `first`: First character in a line.
//      + `succeeding`: Non-first character in a line.
//      + `any`: Any character in a line.
//
//  ```
//  root
//  |-- foo <- `|` is preceding_first_first, `-` is any_first_succeeding
//  |   foo2 <- `|` is preceding_succeeding_first
//  `-- bar <- `` ` `` is last_first_first, `-` is any_first_succeeding
//      bar2
//  ```

use std::fmt;

use crate::config::PrefixPart;

/// Dash level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DashLevel {
    /// Double,
    Double,
    /// Triple.
    Triple,
    /// Quadruple.
    Quadruple,
}

/// Edge width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeWidth {
    /// Narrow.
    Narrow,
    /// Bold.
    Bold,
}

impl Default for EdgeWidth {
    fn default() -> Self {
        Self::Narrow
    }
}

/// Unicode edge style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EdgeStyle {
    /// Solid (single line).
    Solid(EdgeWidth),
    /// Dashed line.
    Dashed(EdgeWidth, DashLevel),
    /// Double line.
    Double,
}

impl EdgeStyle {
    /// Collapses the `Dashed` variant into `Solid`.
    fn dashed_to_solid(self) -> EdgeStyleWithoutDashed {
        self.into()
    }
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self::Solid(EdgeWidth::default())
    }
}

/// Unicode edge style without `Dashed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EdgeStyleWithoutDashed {
    /// Solid (single line).
    Solid(EdgeWidth),
    /// Double line.
    Double,
}

impl From<EdgeStyle> for EdgeStyleWithoutDashed {
    fn from(v: EdgeStyle) -> Self {
        match v {
            EdgeStyle::Solid(width) | EdgeStyle::Dashed(width, _) => Self::Solid(width),
            EdgeStyle::Double => Self::Double,
        }
    }
}

/// Unicode corner style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerStyle {
    /// Angle.
    Angle,
    /// Round.
    Round,
}

impl Default for CornerStyle {
    fn default() -> Self {
        Self::Angle
    }
}

/// Width of ambiguous width characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbiWidth {
    /// Use single width (half width).
    ///
    /// Usually use this for non-CJK environment.
    Single,
    /// Use double width.
    ///
    /// Usually use this for CJK environment.
    Double,
}

/// Returns a character for the non-last item, first line, first character.
fn preceding_first_first(
    vertical_backward: EdgeStyle,
    vertical_forward: EdgeStyle,
    horizontal: EdgeStyle,
) -> Option<char> {
    use EdgeStyleWithoutDashed::{Double, Solid};
    use EdgeWidth::{Bold, Narrow};

    match (
        vertical_backward.dashed_to_solid(),
        vertical_forward.dashed_to_solid(),
        horizontal.dashed_to_solid(),
    ) {
        // ├
        (Solid(Narrow), Solid(Narrow), Solid(Narrow)) => Some('\u{251c}'),
        // ┝
        (Solid(Narrow), Solid(Narrow), Solid(Bold)) => Some('\u{251d}'),
        // ╞
        (Solid(Narrow), Solid(Narrow), Double) => Some('\u{255e}'),
        // ┟
        (Solid(Narrow), Solid(Bold), Solid(Narrow)) => Some('\u{251f}'),
        // ┢
        (Solid(Narrow), Solid(Bold), Solid(Bold)) => Some('\u{2522}'),
        (Solid(Narrow), Solid(Bold), Double) => None,
        (Solid(Narrow), Double, _) => None,
        // ┞
        (Solid(Bold), Solid(Narrow), Solid(Narrow)) => Some('\u{251e}'),
        // ┡
        (Solid(Bold), Solid(Narrow), Solid(Bold)) => Some('\u{2521}'),
        (Solid(Bold), Solid(Narrow), Double) => None,
        // ┠
        (Solid(Bold), Solid(Bold), Solid(Narrow)) => Some('\u{2520}'),
        // ┣
        (Solid(Bold), Solid(Bold), Solid(Bold)) => Some('\u{2523}'),
        (Solid(Bold), Solid(Bold), Double) => None,
        (Solid(Bold), Double, _) => None,
        (Double, Solid(Narrow), _) | (Double, Solid(Bold), _) => None,
        // ╟
        (Double, Double, Solid(Narrow)) => Some('\u{255f}'),
        (Double, Double, Solid(Bold)) => None,
        // ╠
        (Double, Double, Double) => Some('\u{2560}'),
    }
}

/// Returns a character for the last item, first line, first character.
fn last_first_first(
    vertical_backward: EdgeStyle,
    horizontal: EdgeStyle,
    corner: CornerStyle,
) -> Option<char> {
    use CornerStyle::{Angle, Round};
    use EdgeStyleWithoutDashed::{Double, Solid};
    use EdgeWidth::{Bold, Narrow};

    match (
        vertical_backward.dashed_to_solid(),
        horizontal.dashed_to_solid(),
        corner,
    ) {
        // └
        (Solid(Narrow), Solid(Narrow), Angle) => Some('\u{2514}'),
        // ╰
        (Solid(Narrow), Solid(Narrow), Round) => Some('\u{2570}'),
        // ┕
        (Solid(Narrow), Solid(Bold), Angle) => Some('\u{2515}'),
        (Solid(Narrow), Solid(Bold), Round) => None,
        // ╘
        (Solid(Narrow), Double, Angle) => Some('\u{2558}'),
        (Solid(Narrow), Double, Round) => None,
        // ┖
        (Solid(Bold), Solid(Narrow), Angle) => Some('\u{2516}'),
        (Solid(Bold), Solid(Narrow), Round) => None,
        // ┗
        (Solid(Bold), Solid(Bold), Angle) => Some('\u{2517}'),
        (Solid(Bold), Solid(Bold), Round) => None,
        (Solid(Bold), Double, _) => None,
        // ╙
        (Double, Solid(Narrow), Angle) => Some('\u{2559}'),
        (Double, Solid(Narrow), Round) => None,
        (Double, Solid(Bold), _) => None,
        // ╚
        (Double, Double, Angle) => Some('\u{255a}'),
        (Double, Double, Round) => None,
    }
}

/// Returns a character for the non-last item, succeeding line, first character.
fn preceding_succeeding_first(vertical_forward: EdgeStyle) -> Option<char> {
    use DashLevel::{Double as DoubleDash, Quadruple, Triple};
    use EdgeStyle::{Dashed, Double as DoubleLine, Solid};
    use EdgeWidth::{Bold, Narrow};

    match vertical_forward {
        // │
        Solid(Narrow) => Some('\u{2502}'),
        // ┃
        Solid(Bold) => Some('\u{2503}'),
        // ╎
        Dashed(Narrow, DoubleDash) => Some('\u{254e}'),
        // ┆
        Dashed(Narrow, Triple) => Some('\u{2506}'),
        // ┊
        Dashed(Narrow, Quadruple) => Some('\u{250a}'),
        // ╏
        Dashed(Bold, DoubleDash) => Some('\u{254f}'),
        // ┇
        Dashed(Bold, Triple) => Some('\u{2507}'),
        // ┋
        Dashed(Bold, Quadruple) => Some('\u{250b}'),
        // ║
        DoubleLine => Some('\u{2551}'),
    }
}

/// Returns a character for any item, first line, succeeding character.
fn any_first_succeeding(horizontal: EdgeStyle) -> Option<char> {
    use DashLevel::{Double as DoubleDash, Quadruple, Triple};
    use EdgeStyle::{Dashed, Double as DoubleLine, Solid};
    use EdgeWidth::{Bold, Narrow};

    match horizontal {
        // ─
        Solid(Narrow) => Some('\u{2500}'),
        // ━
        Solid(Bold) => Some('\u{2501}'),
        // ╌
        Dashed(Narrow, DoubleDash) => Some('\u{254c}'),
        // ┄
        Dashed(Narrow, Triple) => Some('\u{2504}'),
        // ┈
        Dashed(Narrow, Quadruple) => Some('\u{2508}'),
        // ╍
        Dashed(Bold, DoubleDash) => Some('\u{254d}'),
        // ┅
        Dashed(Bold, Triple) => Some('\u{2505}'),
        // ┉
        Dashed(Bold, Quadruple) => Some('\u{2509}'),
        // ═
        DoubleLine => Some('\u{2550}'),
    }
}

/// Unicode edge style.
#[derive(Debug, Clone, Copy)]
pub struct UnicodeEdgeConfigBuilder {
    /// Width of ambiguous width characters.
    ambiwidth: AmbiWidth,
    /// Vertical backward edge style.
    vertical_backward: EdgeStyle,
    /// Vertical forward edge style.
    vertical_forward: EdgeStyle,
    /// Horizontal edge style.
    horizontal: EdgeStyle,
    /// Corner style.
    corner: CornerStyle,
}

impl UnicodeEdgeConfigBuilder {
    /// Creates a new Unicode edge config builder.
    pub fn with_ambiwidth(ambiwidth: AmbiWidth) -> Self {
        Self {
            ambiwidth,
            vertical_backward: Default::default(),
            vertical_forward: Default::default(),
            horizontal: Default::default(),
            corner: Default::default(),
        }
    }

    /// Sets the vertical ruled line style for both backward and forward.
    pub fn vertical(&mut self, style: EdgeStyle) -> &mut Self {
        self.vertical_backward = style;
        self.vertical_forward = style;
        self
    }

    /// Sets the vertical backward ruled line style.
    pub fn vertical_backward(&mut self, style: EdgeStyle) -> &mut Self {
        self.vertical_backward = style;
        self
    }

    /// Sets the vertical forward ruled line style.
    pub fn vertical_forward(&mut self, style: EdgeStyle) -> &mut Self {
        self.vertical_forward = style;
        self
    }

    /// Sets the horizontal ruled line style.
    pub fn horizontal(&mut self, style: EdgeStyle) -> &mut Self {
        self.horizontal = style;
        self
    }

    /// Sets the corner line style.
    pub fn corner(&mut self, corner: CornerStyle) -> &mut Self {
        self.corner = corner;
        self
    }

    /// Creates a `UnicodeEdgeConfig`.
    pub fn build(&self) -> Option<UnicodeEdgeConfig> {
        let preceding_first_first = preceding_first_first(
            self.vertical_backward,
            self.vertical_forward,
            self.horizontal,
        )?;
        let last_first_first =
            last_first_first(self.vertical_backward, self.horizontal, self.corner)?;
        let preceding_succeeding_first = preceding_succeeding_first(self.vertical_forward)?;
        let any_first_succeeding = any_first_succeeding(self.horizontal)?;

        Some(UnicodeEdgeConfig {
            ambiwidth: self.ambiwidth,
            preceding_first_first,
            last_first_first,
            any_first_succeeding,
            preceding_succeeding_first,
        })
    }
}

/// Unicode edge style.
#[derive(Debug, Clone, Copy)]
pub struct UnicodeEdgeConfig {
    /// Width of ambiguous width characters.
    ambiwidth: AmbiWidth,
    /// Preceding item, first line, first character.
    preceding_first_first: char,
    /// Last item, first line, first character.
    last_first_first: char,
    /// Any item, first line, succeeding character.
    any_first_succeeding: char,
    /// Preceding item, succeeding line, first character.
    preceding_succeeding_first: char,
}

impl UnicodeEdgeConfig {
    /// Returns a character for the first line, first char.
    fn first_line_first_char(&self, last_child: bool) -> char {
        if last_child {
            self.last_first_first
        } else {
            self.preceding_first_first
        }
    }

    /// Writes the prefix or padding with the given config.
    pub(crate) fn write_edge<W: fmt::Write>(
        &self,
        writer: &mut W,
        last_child: bool,
        first_line: bool,
        fragment: PrefixPart,
    ) -> fmt::Result {
        use PrefixPart::{Padding, Prefix};

        match (first_line, last_child, self.ambiwidth, fragment) {
            (true, _, AmbiWidth::Single, Prefix) => write!(
                writer,
                "{0}{1}{1}",
                self.first_line_first_char(last_child),
                self.any_first_succeeding
            ),
            (true, _, AmbiWidth::Double, Prefix) => write!(
                writer,
                "{0}{1}",
                self.first_line_first_char(last_child),
                self.any_first_succeeding
            ),
            (true, _, _, Padding) => writer.write_str(" "),
            (false, true, _, Prefix) => Ok(()),
            (false, true, AmbiWidth::Single, Padding) => writer.write_str("    "),
            (false, true, AmbiWidth::Double, Padding) => writer.write_str("     "),
            (false, false, _, Prefix) => writer.write_char(self.preceding_succeeding_first),
            (false, false, _, Padding) => writer.write_str("   "),
        }
    }

    /// Returns whether the prefix and padding consist of whitespaces.
    ///
    /// When both of prefix and padding are empty, this should return `true` (i.e. an empty string
    /// should be considered as "whitespaces").
    pub(crate) fn is_prefix_whitespace(&self, last_child: bool, first_line: bool) -> bool {
        last_child && !first_line
    }
}
