use std::borrow::Cow;

use super::{Span, Style, StyledGrapheme};
use crate::layout::Alignment;

/// A line of text, consisting of one or more [`Span`]s.
///
/// [`Line`]s are used wherever text is displayed in the terminal and represent a single line of
/// text. When a [`Line`] is rendered, it is rendered as a single line of text, with each [`Span`]
/// being rendered in order (left to right). The [`Style`] of each [`Span`] will be combined with
/// the [`Style`] of the [`Line`], so that each [`Span`] can be styled individually, and also
/// inherit the style of the [`Line`].
///
/// [`Line`]s are created from [`Span`]s, [`String`]s, and [`&str`]s. They can be styled with a
/// [`Style`], and can have a target [`Alignment`].
///
/// # Line Style Compatibility
///
/// The [`Style`] of a [`Line`] was added in v0.25.0. Prior to that, the style of a line was
/// determined only by the style of each [`Span`] contained in the line. For this reason, this
/// field may not be supported yet by all widgets (outside of the `ratatui` crate itself).
///
/// # Examples
///
/// ```rust
/// # use ratatui::prelude::*;
/// Line::raw("test content");
/// Line::styled("test content", Style::new().yellow());
/// Line::from(vec![
///     Span::styled("My", Style::default().fg(Color::Yellow)),
///     Span::raw(" text"),
/// ]);
/// ```
///
/// Newlines are ignored when creating a [`Line`] from a [`String`] or [`&str`]:
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub struct Line<'a> {
    pub spans: Vec<Span<'a>>,
    pub style: Style,
    pub alignment: Option<Alignment>,
}

/// Constructor methods
impl<'a> Line<'a> {
    /// Create a line with the default style.
    ///
    /// `content` can be any type that is convertible to [`Cow<str>`] (e.g. [`&str`], [`String`],
    /// [`Cow<str>`], or your own type that implements [`Into<Cow<str>>`]).
    ///
    /// A [`Line`] can specify a [`Style`], which will be applied before the style of each [`Span`]
    /// in the line.
    ///
    /// Any newlines in the content are removed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// # use std::borrow::Cow;
    ///
    /// Line::raw("test content");
    /// Line::raw(String::from("test content"));
    /// Line::raw(Cow::from("test content"));
    /// ```
    pub fn raw<T>(content: T) -> Line<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        Line {
            spans: content
                .into()
                .lines()
                .map(|v| Span::raw(v.to_string()))
                .collect(),
            ..Default::default()
        }
    }

    /// Create a line with the given style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// Line::styled("My text", style);
    /// Line::styled(String::from("My text"), style);
    /// ```
    pub fn styled<T>(content: T, style: Style) -> Line<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        Line {
            spans: content
                .into()
                .lines()
                .map(|v| Span::raw(v.to_string()))
                .collect(),
            style,
            ..Default::default()
        }
    }
}

/// # Builder methods
///
/// These methods can be used to construct or modify an existing Line. These can be chained.
///
/// # Examples
///
/// ```
/// # use ratatui::prelude::*;
/// let line = Line::default()
///     .spans(vec!["hello".red(), "world".green()])
///     .alignment(Alignment::Center)
///     .style(Style::new().bold());
/// ````
impl<'a> Line<'a> {
    /// Sets the spans of this line of text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let line = Line::default().spans(vec![
    ///     Span::styled("My", Style::default().fg(Color::Yellow)),
    ///     Span::raw(" text"),
    /// ]);
    /// ```
    pub fn spans<T: Into<Vec<Span<'a>>>>(mut self, spans: T) -> Self {
        self.spans = spans.into();
        self
    }

    /// Sets the target alignment for this line of text.
    ///
    /// Defaults to [`None`], meaning the alignment is determined by the rendering widget.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let mut line = Line::from("Hi, what's up?");
    /// assert_eq!(None, line.alignment);
    /// assert_eq!(Some(Alignment::Right), line.alignment(Alignment::Right).alignment)
    /// ```
    pub fn alignment<T: Into<Alignment>>(mut self, alignment: T) -> Self {
        self.alignment = Some(alignment.into());
        self
    }

    /// Sets the style of this line of text.
    ///
    /// Defaults to [`Style::default()`].
    ///
    /// Note: This field was added in v0.25.0. Prior to that, the style of a line was determined
    /// only by the style of each [`Span`] contained in the line. For this reason, this field may
    /// not be supported by all widgets (outside of the `ratatui` crate itself).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let mut line = Line::from("foo").style(Style::new().red());
    /// ```
    pub fn style<T: Into<Style>>(mut self, style: T) -> Self {
        self.style = style.into();
        self
    }
}

/// Other methods
impl Line<'_> {
    /// Returns the width of the underlying string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let line = Line::from(vec![
    ///     Span::styled("My", Style::default().fg(Color::Yellow)),
    ///     Span::raw(" text"),
    /// ]);
    /// assert_eq!(7, line.width());
    /// ```
    pub fn width(&self) -> usize {
        self.spans.iter().map(Span::width).sum()
    }

    /// Patches the style of each Span in an existing Line, adding modifiers from the given style.
    ///
    /// This is useful for when you want to apply a style to a line that already has some styling.
    /// In contrast to [`Line::style`], this method will not overwrite the existing style, but
    /// instead will add the given style's modifiers to the existing style of each `Span`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// let mut raw_line = Line::from(vec![
    ///     Span::raw("My"),
    ///     Span::raw(" text"),
    /// ]);
    /// let mut styled_line = Line::from(vec![
    ///     Span::styled("My", style),
    ///     Span::styled(" text", style),
    /// ]);
    ///
    /// assert_ne!(raw_line, styled_line);
    ///
    /// raw_line.patch_style(style);
    /// assert_eq!(raw_line, styled_line);
    /// ```
    pub fn patch_style(&mut self, style: Style) {
        for span in &mut self.spans {
            span.patch_style(style);
        }
    }

    /// Resets the style of each Span in the Line.
    /// Equivalent to calling `patch_style(Style::reset())`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let mut line = Line::from(vec![
    ///     Span::styled("My", Style::default().fg(Color::Yellow)),
    ///     Span::styled(" text", Style::default().add_modifier(Modifier::BOLD)),
    /// ]);
    ///
    /// line.reset_style();
    /// assert_eq!(Style::reset(), line.spans[0].style);
    /// assert_eq!(Style::reset(), line.spans[1].style);
    /// ```
    pub fn reset_style(&mut self) {
        for span in &mut self.spans {
            span.reset_style();
        }
    }

    /// Returns an iterator over the graphemes held by this line.
    ///
    /// `base_style` is the [`Style`] that will be patched with each grapheme [`Style`] to get
    /// the resulting [`Style`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::iter::Iterator;
    /// use ratatui::{prelude::*, text::StyledGrapheme};
    ///
    /// let style = Style::new().green().on_black();
    /// let line = Line::styled("Text", Style::new().yellow());
    /// let graphemes = line.styled_graphemes(style).collect::<Vec<StyledGrapheme>>();
    /// ```
    pub fn styled_graphemes(&self, base_style: Style) -> impl Iterator<Item = StyledGrapheme> {
        self.spans
            .iter()
            .flat_map(move |span| span.styled_graphemes(base_style))
    }

    pub fn styled_spans(&self) -> impl Iterator<Item = Span> {
        self.spans.iter().cloned().map(|s| s.style(self.style))
    }
}

impl<'a> IntoIterator for Line<'a> {
    type Item = Span<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.into_iter()
    }
}

impl<'a> From<String> for Line<'a> {
    fn from(s: String) -> Self {
        Self::from(vec![Span::from(s)])
    }
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(vec![Span::from(s)])
    }
}

impl<'a> From<Vec<Span<'a>>> for Line<'a> {
    fn from(spans: Vec<Span<'a>>) -> Self {
        Self {
            spans,
            ..Default::default()
        }
    }
}

impl<'a> From<Span<'a>> for Line<'a> {
    fn from(span: Span<'a>) -> Self {
        Self::from(vec![span])
    }
}

impl<'a> From<Line<'a>> for String {
    fn from(line: Line<'a>) -> String {
        line.spans.iter().fold(String::new(), |mut acc, s| {
            acc.push_str(s.content.as_ref());
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        layout::Alignment,
        style::{Color, Modifier, Style},
        text::{Line, Span, StyledGrapheme},
    };

    #[test]
    fn test_width() {
        let line = Line::from(vec![
            Span::styled("My", Style::default().fg(Color::Yellow)),
            Span::raw(" text"),
        ]);
        assert_eq!(7, line.width());

        let empty_line = Line::default();
        assert_eq!(0, empty_line.width());
    }

    #[test]
    fn test_patch_style() {
        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC);
        let mut raw_line = Line::from(vec![Span::raw("My"), Span::raw(" text")]);
        let styled_line = Line::from(vec![
            Span::styled("My", style),
            Span::styled(" text", style),
        ]);

        assert_ne!(raw_line, styled_line);

        raw_line.patch_style(style);
        assert_eq!(raw_line, styled_line);
    }

    #[test]
    fn test_reset_style() {
        let mut line = Line::from(vec![
            Span::styled("My", Style::default().fg(Color::Yellow)),
            Span::styled(" text", Style::default().add_modifier(Modifier::BOLD)),
        ]);

        line.reset_style();
        assert_eq!(Style::reset(), line.spans[0].style);
        assert_eq!(Style::reset(), line.spans[1].style);
    }

    #[test]
    fn test_from_string() {
        let s = String::from("Hello, world!");
        let line = Line::from(s);
        assert_eq!(vec![Span::from("Hello, world!")], line.spans);
    }

    #[test]
    fn test_from_str() {
        let s = "Hello, world!";
        let line = Line::from(s);
        assert_eq!(vec![Span::from("Hello, world!")], line.spans);
    }

    #[test]
    fn test_from_vec() {
        let spans = vec![
            Span::styled("Hello,", Style::default().fg(Color::Red)),
            Span::styled(" world!", Style::default().fg(Color::Green)),
        ];
        let line = Line::from(spans.clone());
        assert_eq!(spans, line.spans);
    }

    #[test]
    fn test_from_span() {
        let span = Span::styled("Hello, world!", Style::default().fg(Color::Yellow));
        let line = Line::from(span.clone());
        assert_eq!(vec![span], line.spans);
    }

    #[test]
    fn test_into_string() {
        let line = Line::from(vec![
            Span::styled("Hello,", Style::default().fg(Color::Red)),
            Span::styled(" world!", Style::default().fg(Color::Green)),
        ]);
        let s: String = line.into();
        assert_eq!("Hello, world!", s);
    }

    #[test]
    fn test_alignment() {
        let line = Line::from("This is left").alignment(Alignment::Left);
        assert_eq!(Some(Alignment::Left), line.alignment);

        let line = Line::from("This is default");
        assert_eq!(None, line.alignment);
    }

    #[test]
    fn styled_graphemes() {
        const RED: Style = Style::new().fg(Color::Red);
        const GREEN: Style = Style::new().fg(Color::Green);
        const BLUE: Style = Style::new().fg(Color::Blue);
        const RED_ON_WHITE: Style = Style::new().fg(Color::Red).bg(Color::White);
        const GREEN_ON_WHITE: Style = Style::new().fg(Color::Green).bg(Color::White);
        const BLUE_ON_WHITE: Style = Style::new().fg(Color::Blue).bg(Color::White);

        let line = Line::from(vec![
            Span::styled("He", RED),
            Span::styled("ll", GREEN),
            Span::styled("o!", BLUE),
        ]);
        let styled_graphemes = line
            .styled_graphemes(Style::new().bg(Color::White))
            .collect::<Vec<StyledGrapheme>>();
        assert_eq!(
            styled_graphemes,
            vec![
                StyledGrapheme::new("H", RED_ON_WHITE),
                StyledGrapheme::new("e", RED_ON_WHITE),
                StyledGrapheme::new("l", GREEN_ON_WHITE),
                StyledGrapheme::new("l", GREEN_ON_WHITE),
                StyledGrapheme::new("o", BLUE_ON_WHITE),
                StyledGrapheme::new("!", BLUE_ON_WHITE),
            ],
        );
    }

    #[test]
    fn raw_str() {
        let line = Line::raw("test content");
        assert_eq!(line.spans, vec![Span::raw("test content")]);
        assert_eq!(line.alignment, None);

        let line = Line::raw("a\nb");
        assert_eq!(line.spans, vec![Span::raw("a"), Span::raw("b")]);
        assert_eq!(line.alignment, None);
    }
}
