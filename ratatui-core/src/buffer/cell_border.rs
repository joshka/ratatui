use bitflags::bitflags;

use crate::style::Color;

bitflags! {
    /// The segments of a cell's border, as defined by the cell border protocol.
    ///
    /// Each outer edge is split into two halves, and the centre has four arms. Bit values match
    /// the protocol's `MASK` field.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
    pub struct BorderSegments: u16 {
        /// Left half of the top edge.
        const TOP_LEFT     = 1 << 0;
        /// Right half of the top edge.
        const TOP_RIGHT    = 1 << 1;
        /// Upper half of the right edge.
        const RIGHT_TOP    = 1 << 2;
        /// Lower half of the right edge.
        const RIGHT_BOTTOM = 1 << 3;
        /// Left half of the bottom edge.
        const BOTTOM_LEFT  = 1 << 4;
        /// Right half of the bottom edge.
        const BOTTOM_RIGHT = 1 << 5;
        /// Upper half of the left edge.
        const LEFT_TOP     = 1 << 6;
        /// Lower half of the left edge.
        const LEFT_BOTTOM  = 1 << 7;
        /// Centre arm from the middle to the left edge.
        const H_LEFT       = 1 << 8;
        /// Centre arm from the middle to the right edge.
        const H_RIGHT      = 1 << 9;
        /// Centre arm from the middle to the top edge.
        const V_TOP        = 1 << 10;
        /// Centre arm from the middle to the bottom edge.
        const V_BOTTOM     = 1 << 11;

        /// The whole top edge.

        const TOP        = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits();
        /// The whole right edge.
        const RIGHT      = Self::RIGHT_TOP.bits() | Self::RIGHT_BOTTOM.bits();
        /// The whole bottom edge.
        const BOTTOM     = Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        /// The whole left edge.
        const LEFT       = Self::LEFT_TOP.bits() | Self::LEFT_BOTTOM.bits();
        /// All four outer edges.
        const OUTLINE    = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
        /// The horizontal centreline.
        const HORIZONTAL = Self::H_LEFT.bits() | Self::H_RIGHT.bits();
        /// The vertical centreline.
        const VERTICAL   = Self::V_TOP.bits() | Self::V_BOTTOM.bits();
    }
}

bitflags! {
    /// Joints of a cell's border that are drawn rounded rather than square, as defined by the
    /// cell border protocol.
    ///
    /// A corner rounds only when the cell draws both of the lines that meet there. Bit values
    /// match the protocol's `JOINTS` field.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
    pub struct BorderJoints: u8 {
        /// Where the top and left edges meet.
        const CORNER_TOP_LEFT     = 1 << 0;
        /// Where the top and right edges meet.
        const CORNER_TOP_RIGHT    = 1 << 1;
        /// Where the bottom and right edges meet.
        const CORNER_BOTTOM_RIGHT = 1 << 2;
        /// Where the bottom and left edges meet.
        const CORNER_BOTTOM_LEFT  = 1 << 3;
        /// Where the centre arms to the top and right meet.
        const CENTER_TOP_RIGHT    = 1 << 4;
        /// Where the centre arms to the bottom and right meet.
        const CENTER_BOTTOM_RIGHT = 1 << 5;
        /// Where the centre arms to the bottom and left meet.
        const CENTER_BOTTOM_LEFT  = 1 << 6;
        /// Where the centre arms to the top and left meet.
        const CENTER_TOP_LEFT     = 1 << 7;
    }
}

/// How a cell's border lines are drawn.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LineStyle {
    /// One continuous line.
    #[default]
    Solid,
    /// Two parallel lines. Reads best at wider stroke widths.
    Double,
    /// A dashed line.
    Dashed,
    /// A dotted line.
    Dotted,
}

impl LineStyle {
    /// The protocol's name for the style.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Double => "double",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
        }
    }
}

/// Thin lines drawn on the edges and centrelines of a cell without using any character positions.
///
/// This is an experimental cell attribute for terminals that support the cell border protocol.
/// Backends that know the protocol send it as a pen before writing the cell; others ignore it, so
/// applications should check for support and fall back to box-drawing characters.
///
/// All segments of one cell share a single stroke.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct CellBorder {
    /// Which segments are drawn. Empty means no border.
    pub segments: BorderSegments,
    /// The stroke colour. [`Color::Reset`] means the terminal's default foreground colour.
    pub color: Color,
    /// The stroke thickness in 1/256 of the cell width (`1..=64`).
    pub width: u8,
    /// Whether the stroke is centred on the cell edge (`true`) or drawn inside the cell.
    ///
    /// Centred strokes on a shared edge between two cells merge into one line.
    pub centered: bool,
    /// Which stroke wins when two centred strokes share an edge (`0..=15`, higher wins).
    ///
    /// Use this to keep a highlighted outline on top of its neighbours regardless of draw order.
    pub priority: u8,
    /// How the lines are drawn.
    pub style: LineStyle,
    /// Which joints are rounded.
    pub rounded: BorderJoints,
    /// The colour to fill a rounded corner cell with outside its arc, usually whatever is behind
    /// the rounded shape. [`Color::Reset`] means the terminal's default background. `None`
    /// leaves the cell's own background there.
    pub outside: Option<Color>,
    /// The radius of rounded corners in 1/256 of the cell width (`1..=256`). `None` uses the
    /// terminal's default.
    pub radius: Option<u16>,
}

impl CellBorder {
    /// No border.
    pub const NONE: Self = Self {
        segments: BorderSegments::empty(),
        color: Color::Reset,
        width: 16,
        centered: true,
        priority: 0,
        style: LineStyle::Solid,
        rounded: BorderJoints::empty(),
        outside: None,
        radius: None,
    };

    /// Creates a centred border on the given segments with the default width.
    pub const fn new(segments: BorderSegments, color: Color) -> Self {
        Self {
            segments,
            color,
            ..Self::NONE
        }
    }

    /// Returns true if no segments are drawn.
    pub const fn is_empty(self) -> bool {
        self.segments.is_empty()
    }
}

impl Default for CellBorder {
    fn default() -> Self {
        Self::NONE
    }
}
