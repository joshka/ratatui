use std::iter;

use strum::{Display, EnumString};
use unicode_width::UnicodeWidthStr;

use super::ScrollbarState;
use crate::{
    prelude::*,
    symbols::scrollbar::{Set, DOUBLE_HORIZONTAL, DOUBLE_VERTICAL},
    widgets::StatefulWidget,
};

/// A widget to display a scrollbar
///
/// The following components of the scrollbar are customizable in symbol and style. Note the
/// scrollbar is represented horizontally but it can also be set vertically (which is actually the
/// default).
///
/// ```text
/// <--▮------->
/// ^  ^   ^   ^
/// │  │   │   └ end
/// │  │   └──── track
/// │  └──────── thumb
/// └─────────── begin
/// ```
///
/// # Examples
///
/// ```rust
/// use ratatui::{prelude::*, widgets::*};
///
/// # fn render_paragraph_with_scrollbar(frame: &mut Frame, area: Rect) {
/// let vertical_scroll = 0; // from app state
///
/// let items = vec![
///     Line::from("Item 1"),
///     Line::from("Item 2"),
///     Line::from("Item 3"),
/// ];
/// let paragraph = Paragraph::new(items.clone())
///     .scroll((vertical_scroll as u16, 0))
///     .block(Block::new().borders(Borders::RIGHT)); // to show a background for the scrollbar
///
/// let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
///     .begin_symbol(Some("↑"))
///     .end_symbol(Some("↓"));
///
/// let mut scrollbar_state = ScrollbarState::new(items.len()).position(vertical_scroll);
///
/// let area = frame.size();
/// // Note we render the paragraph
/// frame.render_widget(paragraph, area);
/// // and the scrollbar, those are separate widgets
/// frame.render_stateful_widget(
///     scrollbar,
///     area.inner(&Margin {
///         // using an inner vertical margin of 1 unit makes the scrollbar inside the block
///         vertical: 1,
///         horizontal: 0,
///     }),
///     &mut scrollbar_state,
/// );
/// # }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Scrollbar<'a> {
    orientation: ScrollbarOrientation,
    thumb_style: Style,
    thumb_symbol: &'a str,
    track_style: Style,
    track_symbol: Option<&'a str>,
    begin_symbol: Option<&'a str>,
    begin_style: Style,
    end_symbol: Option<&'a str>,
    end_style: Style,
}

/// This is the position of the scrollbar around a given area.
///
/// ```plain
///           HorizontalTop
///             ┌───────┐
/// VerticalLeft│       │VerticalRight
///             └───────┘
///          HorizontalBottom
/// ```
#[derive(Debug, Default, Display, EnumString, Clone, Eq, PartialEq, Hash)]
pub enum ScrollbarOrientation {
    /// Positions the scrollbar on the right, scrolling vertically
    #[default]
    VerticalRight,
    /// Positions the scrollbar on the left, scrolling vertically
    VerticalLeft,
    /// Positions the scrollbar on the bottom, scrolling horizontally
    HorizontalBottom,
    /// Positions the scrollbar on the top, scrolling horizontally
    HorizontalTop,
}

impl<'a> Default for Scrollbar<'a> {
    fn default() -> Self {
        Self {
            orientation: ScrollbarOrientation::default(),
            thumb_symbol: DOUBLE_VERTICAL.thumb,
            thumb_style: Style::default(),
            track_symbol: Some(DOUBLE_VERTICAL.track),
            track_style: Style::default(),
            begin_symbol: Some(DOUBLE_VERTICAL.begin),
            begin_style: Style::default(),
            end_symbol: Some(DOUBLE_VERTICAL.end),
            end_style: Style::default(),
        }
    }
}

impl<'a> Scrollbar<'a> {
    /// Creates a new scrollbar with the given position.
    ///
    /// Most of the time you'll want [`ScrollbarOrientation::VerticalLeft`] or
    /// [`ScrollbarOrientation::HorizontalBottom`]. See [`ScrollbarOrientation`] for more options.
    pub fn new(orientation: ScrollbarOrientation) -> Self {
        Self::default().orientation(orientation)
    }

    /// Sets the position of the scrollbar.
    ///
    /// The orientation of the scrollbar is the position it will take around a [`Rect`]. See
    /// [`ScrollbarOrientation`] for more details.
    ///
    /// Resets the symbols to [`DOUBLE_VERTICAL`] or [`DOUBLE_HORIZONTAL`] based on orientation.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn orientation(mut self, orientation: ScrollbarOrientation) -> Self {
        self.orientation = orientation;
        let set = if self.orientation.is_vertical() {
            DOUBLE_VERTICAL
        } else {
            DOUBLE_HORIZONTAL
        };
        self.symbols(set)
    }

    /// Sets the orientation and symbols for the scrollbar from a [`Set`].
    ///
    /// This has the same effect as calling [`Scrollbar::orientation`] and then
    /// [`Scrollbar::symbols`]. See those for more details.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn orientation_and_symbol(mut self, orientation: ScrollbarOrientation, set: Set) -> Self {
        self.orientation = orientation;
        self.symbols(set)
    }

    /// Sets the symbol that represents the thumb of the scrollbar.
    ///
    /// The thumb is the handle representing the progression on the scrollbar. See [`Scrollbar`]
    /// for a visual example of what this represents.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn thumb_symbol(mut self, thumb_symbol: &'a str) -> Self {
        self.thumb_symbol = thumb_symbol;
        self
    }

    /// Sets the style on the scrollbar thumb.
    ///
    /// The thumb is the handle representing the progression on the scrollbar. See [`Scrollbar`]
    /// for a visual example of what this represents.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn thumb_style<S: Into<Style>>(mut self, thumb_style: S) -> Self {
        self.thumb_style = thumb_style.into();
        self
    }

    /// Sets the symbol that represents the track of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn track_symbol(mut self, track_symbol: Option<&'a str>) -> Self {
        self.track_symbol = track_symbol;
        self
    }

    /// Sets the style that is used for the track of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn track_style<S: Into<Style>>(mut self, track_style: S) -> Self {
        self.track_style = track_style.into();
        self
    }

    /// Sets the symbol that represents the beginning of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn begin_symbol(mut self, begin_symbol: Option<&'a str>) -> Self {
        self.begin_symbol = begin_symbol;
        self
    }

    /// Sets the style that is used for the beginning of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn begin_style<S: Into<Style>>(mut self, begin_style: S) -> Self {
        self.begin_style = begin_style.into();
        self
    }

    /// Sets the symbol that represents the end of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn end_symbol(mut self, end_symbol: Option<&'a str>) -> Self {
        self.end_symbol = end_symbol;
        self
    }

    /// Sets the style that is used for the end of the scrollbar.
    ///
    /// See [`Scrollbar`] for a visual example of what this represents.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn end_style<S: Into<Style>>(mut self, end_style: S) -> Self {
        self.end_style = end_style.into();
        self
    }

    /// Sets the symbols used for the various parts of the scrollbar from a [`Set`].
    ///
    /// ```text
    /// <--▮------->
    /// ^  ^   ^   ^
    /// │  │   │   └ end
    /// │  │   └──── track
    /// │  └──────── thumb
    /// └─────────── begin
    /// ```
    ///
    /// Only sets begin_symbol, end_symbol and track_symbol if they already contain a value.
    /// If they were set to `None` explicitly, this function will respect that choice. Use their
    /// respective setters to change their value.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn symbols(mut self, symbol: Set) -> Self {
        self.thumb_symbol = symbol.thumb;
        if self.track_symbol.is_some() {
            self.track_symbol = Some(symbol.track);
        }
        if self.begin_symbol.is_some() {
            self.begin_symbol = Some(symbol.begin);
        }
        if self.end_symbol.is_some() {
            self.end_symbol = Some(symbol.end);
        }
        self
    }

    /// Sets the style used for the various parts of the scrollbar from a [`Style`].
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// ```text
    /// <--▮------->
    /// ^  ^   ^   ^
    /// │  │   │   └ end
    /// │  │   └──── track
    /// │  └──────── thumb
    /// └─────────── begin
    /// ```
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.track_style = style;
        self.thumb_style = style;
        self.begin_style = style;
        self.end_style = style;
        self
    }
}

impl ScrollbarOrientation {
    fn is_vertical(&self) -> bool {
        self == &Self::VerticalLeft || self == &Self::VerticalRight
    }
}

impl<'a> StatefulWidget for Scrollbar<'a> {
    type State = ScrollbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.content_length == 0 {
            return;
        }

        let area = self.scollbar_area(area);
        let bars = self.bars(area, state);
        for (position, (symbol, style)) in area.rows().flat_map(Rect::columns).zip(bars) {
            buf.set_string(position.x, position.y, symbol, style);
        }
    }
}

impl Scrollbar<'_> {
    /// Returns that the scrollbar should be rendered in the given area
    fn scollbar_area(&self, area: Rect) -> Rect {
        match self.orientation {
            ScrollbarOrientation::VerticalLeft => Rect { width: 1, ..area },
            ScrollbarOrientation::VerticalRight => Rect {
                x: area.right().saturating_sub(1),
                width: 1,
                ..area
            },
            ScrollbarOrientation::HorizontalTop => Rect { height: 1, ..area },
            ScrollbarOrientation::HorizontalBottom => Rect {
                y: area.bottom().saturating_sub(1),
                height: 1,
                ..area
            },
        }
    }

    /// Returns an iterator over the symbols and styles of the parts of a scrollbar
    fn bars(&self, area: Rect, state: &mut ScrollbarState) -> impl Iterator<Item = (&str, Style)> {
        let (track_start_len, thumb_len, track_end_len) = self.part_lengths(area, state);

        let begin = self.begin_symbol.map(|s| (s, self.begin_style));
        let track = self.track_symbol.map(|s| (s, self.track_style));
        let thumb = Some((self.thumb_symbol, self.thumb_style));
        let end = self.end_symbol.map(|s| (s, self.end_style));

        iter::once(begin)
            .chain(iter::repeat(track).take(track_start_len))
            .chain(iter::repeat(thumb).take(thumb_len))
            .chain(iter::repeat(track).take(track_end_len))
            .chain(iter::once(end))
            .flatten()
    }

    /// Returns the lengths of the parts of a scrollbar
    ///
    /// ```plain
    /// <---------▮▮▮▮▮▮▮------->
    ///    start   thumb   end
    /// ```
    fn part_lengths(&self, area: Rect, state: &mut ScrollbarState) -> (usize, usize, usize) {
        let track_len = self.track_length_excluding_arrow_heads(area) as f64;
        let viewport_len = self.viewport_length(area) as f64;

        let content_length = state.content_length as f64;
        // if user passes in position > content_length, we shouldn't panic
        // this will prevent rendering outside of available area
        let position = state.position.min(state.content_length - 1) as f64;

        // vscode style scrolling behavior
        let scrollable_content_len = content_length + viewport_len - 1.0;
        let thumb_start = position * track_len / scrollable_content_len;
        let thumb_end = (position + viewport_len) * track_len / scrollable_content_len;

        // `.round() as usize` gives closest int, as opposed to `floor` or `ceil`
        //
        // we intentionally round just the positions, and sizes are calculated from integer
        // positions. Rounding the sizes can lead to subtle off by 1 errors.
        // e.g. 6.4 (position) +3.4 (length) = 9.8 which rounds to 10, but 6 (rounded position) + 3
        // (rounded length) = 9.
        let track_start_len = thumb_start.round() as usize;
        let thumb_end = thumb_end.round() as usize;

        let thumb_len = thumb_end.saturating_sub(track_start_len);
        let track_end_len = track_len as usize - track_start_len - thumb_len;

        (track_start_len, thumb_len, track_end_len)
    }

    fn track_length_excluding_arrow_heads(&self, area: Rect) -> u16 {
        let start_len = self.begin_symbol.map(|s| s.width() as u16).unwrap_or(0);
        let end_len = self.end_symbol.map(|s| s.width() as u16).unwrap_or(0);
        let arrows_len = start_len + end_len;
        if self.orientation.is_vertical() {
            area.height.saturating_sub(arrows_len)
        } else {
            area.width.saturating_sub(arrows_len)
        }
    }

    fn viewport_length(&self, area: Rect) -> u16 {
        if self.orientation.is_vertical() {
            area.height
        } else {
            area.width
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use unicode_width::UnicodeWidthStr;

    use super::*;

    #[rstest]
    #[case("█═", 0, 2, "position_0")]
    #[case("═█", 1, 2, "position_1")]
    fn render_scrollbar_simplest(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width() as u16;
        let mut buffer = Buffer::empty(Rect::new(0, 0, size, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(None)
            .end_symbol(None)
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message
        );
    }

    #[rstest]
    #[case("#####-----", 0, 10, "position_0")]
    #[case("-#####----", 1, 10, "position_1")]
    #[case("-#####----", 2, 10, "position_2")]
    #[case("--#####---", 3, 10, "position_3")]
    #[case("--#####---", 4, 10, "position_4")]
    #[case("---#####--", 5, 10, "position_5")]
    #[case("---#####--", 6, 10, "position_6")]
    #[case("----#####-", 7, 10, "position_7")]
    #[case("----#####-", 8, 10, "position_8")]
    #[case("-----#####", 9, 10, "position_9")]
    fn render_scrollbar_simple(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width();
        let mut buffer = Buffer::empty(Rect::new(0, 0, size as u16, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message,
        );
    }

    #[rstest]
    #[case("          ", 0, 0, "position_0")]
    fn render_scrollbar_nobar(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width();
        let mut buffer = Buffer::empty(Rect::new(0, 0, size as u16, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message,
        );
    }

    #[rstest]
    #[case("##########", 0, 1, "position_0")]
    fn render_scrollbar_fullbar(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width();
        let mut buffer = Buffer::empty(Rect::new(0, 0, size as u16, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message,
        );
    }

    #[rstest]
    #[case("#########-", 0, 2, "position_0")]
    #[case("-#########", 1, 2, "position_1")]
    fn render_scrollbar_almost_fullbar(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width();
        let mut buffer = Buffer::empty(Rect::new(0, 0, size as u16, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message,
        );
    }

    #[rstest]
    #[case("█████═════", 0, 10, "position_0")]
    #[case("═█████════", 1, 10, "position_1")]
    #[case("═█████════", 2, 10, "position_2")]
    #[case("══█████═══", 3, 10, "position_3")]
    #[case("══█████═══", 4, 10, "position_4")]
    #[case("═══█████══", 5, 10, "position_5")]
    #[case("═══█████══", 6, 10, "position_6")]
    #[case("════█████═", 7, 10, "position_7")]
    #[case("════█████═", 8, 10, "position_8")]
    #[case("═════█████", 9, 10, "position_9")]
    #[case("═════█████", 100, 10, "position_out_of_bounds")]
    fn render_scrollbar_without_symbols(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width() as u16;
        let mut buffer = Buffer::empty(Rect::new(0, 0, size, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(None)
            .end_symbol(None)
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message
        );
    }

    #[rstest]
    #[case("<####---->", 0, 10, "position_0")]
    #[case("<#####--->", 1, 10, "position_1")]
    #[case("<-####--->", 2, 10, "position_2")]
    #[case("<-####--->", 3, 10, "position_3")]
    #[case("<--####-->", 4, 10, "position_4")]
    #[case("<--####-->", 5, 10, "position_5")]
    #[case("<---####->", 6, 10, "position_6")]
    #[case("<---####->", 7, 10, "position_7")]
    #[case("<---#####>", 8, 10, "position_8")]
    #[case("<----####>", 9, 10, "position_9")]
    #[case("<----####>", 10, 10, "position_one_out_of_bounds")]
    #[case("<----####>", 15, 10, "position_few_out_of_bounds")]
    #[case("<----####>", 500, 10, "position_very_many_out_of_bounds")]
    fn render_scrollbar_with_symbols(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width() as u16;
        let mut buffer = Buffer::empty(Rect::new(0, 0, size, 1));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(Some("<"))
            .end_symbol(Some(">"))
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected]),
            "{}",
            assertion_message,
        );
    }

    #[rstest]
    #[case("█████═════", 0, 10, "position_0")]
    #[case("═█████════", 1, 10, "position_1")]
    #[case("═█████════", 2, 10, "position_2")]
    #[case("══█████═══", 3, 10, "position_3")]
    #[case("══█████═══", 4, 10, "position_4")]
    #[case("═══█████══", 5, 10, "position_5")]
    #[case("═══█████══", 6, 10, "position_6")]
    #[case("════█████═", 7, 10, "position_7")]
    #[case("════█████═", 8, 10, "position_8")]
    #[case("═════█████", 9, 10, "position_9")]
    #[case("═════█████", 100, 10, "position_out_of_bounds")]
    fn render_scrollbar_twoline_horizontal(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width() as u16;
        let mut buffer = Buffer::empty(Rect::new(0, 0, size, 2));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(None)
            .end_symbol(None)
            .render(buffer.area, &mut buffer, &mut state);
        let empty_string: String = " ".repeat(size as usize);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![&empty_string, expected]),
            "{}",
            assertion_message
        );

        let mut buffer = Buffer::empty(Rect::new(0, 0, size, 2));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalTop)
            .begin_symbol(None)
            .end_symbol(None)
            .render(buffer.area, &mut buffer, &mut state);
        let empty_string: String = " ".repeat(size as usize);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![expected, &empty_string]),
            "{}",
            assertion_message
        );
    }

    #[rstest]
    #[case("<####---->", 0, 10, "position_0")]
    #[case("<#####--->", 1, 10, "position_1")]
    #[case("<-####--->", 2, 10, "position_2")]
    #[case("<-####--->", 3, 10, "position_3")]
    #[case("<--####-->", 4, 10, "position_4")]
    #[case("<--####-->", 5, 10, "position_5")]
    #[case("<---####->", 6, 10, "position_6")]
    #[case("<---####->", 7, 10, "position_7")]
    #[case("<---#####>", 8, 10, "position_8")]
    #[case("<----####>", 9, 10, "position_9")]
    #[case("<----####>", 10, 10, "position_one_out_of_bounds")]
    fn render_scrollbar_twoline_vertical(
        #[case] expected: &str,
        #[case] position: usize,
        #[case] content_length: usize,
        #[case] assertion_message: &str,
    ) {
        let size = expected.width() as u16;
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, size));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("<"))
            .end_symbol(Some(">"))
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        let empty_string: String = " ".repeat(size as usize);
        let bars = empty_string
            .chars()
            .zip(expected.chars())
            .map(|(a, b)| format!("{a}{b}"));
        assert_eq!(buffer, Buffer::with_lines(bars), "{}", assertion_message);

        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, size));
        let mut state = ScrollbarState::default()
            .position(position)
            .content_length(content_length);
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalLeft)
            .begin_symbol(Some("<"))
            .end_symbol(Some(">"))
            .track_symbol(Some("-"))
            .thumb_symbol("#")
            .render(buffer.area, &mut buffer, &mut state);
        let empty_string: String = " ".repeat(size as usize);
        let bars = expected
            .chars()
            .zip(empty_string.chars())
            .map(|(a, b)| format!("{a}{b}"));
        assert_eq!(buffer, Buffer::with_lines(bars), "{}", assertion_message);
    }
}
