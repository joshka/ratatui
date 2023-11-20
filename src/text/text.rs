use std::borrow::Cow;

use super::{Line, Span};
use crate::style::Style;

/// A string split over multiple lines where each line is composed of several clusters, each with
/// their own style.
///
/// A [`Text`], like a [`Span`], can be constructed using one of the many `From` implementations
/// or via the [`Text::raw`] and [`Text::styled`] methods. Helpfully, [`Text`] also implements
/// [`core::iter::Extend`] which enables the concatenation of several [`Text`] blocks.
///
/// A [`Text`] can also be styled, which applies the given style before each line. This is useful
/// for styling the entire block of text at once, but having the ability to override the style of
/// individual lines.
///
/// ```rust
/// use ratatui::prelude::*;
///
/// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
///
/// // An initial two lines of `Text` built from a `&str`
/// let mut text = Text::from("The first line\nThe second line");
/// assert_eq!(2, text.height());
///
/// // Adding two more unstyled lines
/// text.extend(Text::raw("These are two\nmore lines!"));
/// assert_eq!(4, text.height());
///
/// // Adding a final two styled lines
/// text.extend(Text::styled("Some more lines\nnow with more style!", style));
/// assert_eq!(6, text.height());
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub struct Text<'a> {
    pub lines: Vec<Line<'a>>,
    pub style: Style,
}

/// # Constructors
///
/// [`Text`] can be constructed using one of the many `From` implementations or via the
/// [`Text::raw`] and [`Text::styled`] methods.
///
/// # Examples
///
/// ```rust
/// # use ratatui::prelude::*;
/// Text::raw("The first line\nThe second line");
/// Text::raw(String::from("The first line\nThe second line"));
/// Text::styled("The first line\nThe second line", Style::new().red().bold());
/// ```
impl<'a> Text<'a> {
    /// Create some text (potentially multiple lines) with no style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// Text::raw("The first line\nThe second line");
    /// Text::raw(String::from("The first line\nThe second line"));
    /// ```
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let lines: Vec<_> = match content.into() {
            Cow::Borrowed("") => vec![Line::from("")],
            Cow::Borrowed(s) => s.lines().map(Line::from).collect(),
            Cow::Owned(s) if s.is_empty() => vec![Line::from("")],
            Cow::Owned(s) => s.lines().map(|l| Line::from(l.to_owned())).collect(),
        };

        Text::from(lines)
    }

    /// Create some text (potentially multiple lines) with a style.
    ///
    /// Note: this method was created before Text had a style field. As such, the style is stored
    /// in both the [`Text`] and in each [`Line`]. This means that if you change the style of the
    /// [`Text`] after construction, the style of each [`Line`] will not be updated. To update the
    /// style of each [`Line`], use [`Text::patch_style`] or [`Text::reset_style`], or construct
    /// the [`Text`] with the lines already styled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// Text::styled("The first line\nThe second line", style);
    /// Text::styled(String::from("The first line\nThe second line"), style);
    /// ```
    pub fn styled<T>(content: T, style: Style) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let mut raw = Text::raw(content);
        raw.patch_style(style);
        raw.style(style)
    }
}

/// # Builder methods
///
/// These methods can be used to construct or modify an existing [`Text`]. These can be chained.
impl<'a> Text<'a> {
    /// Sets the lines of the [`Text`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let text = Text::default().lines(vec![
    ///     Line::from("The first line"),
    ///     Line::from("The second line"),
    /// ]);
    /// ```
    pub fn lines<T>(mut self, lines: T) -> Self
    where
        T: IntoIterator<Item = Line<'a>>,
    {
        self.lines = lines.into_iter().collect();
        self
    }

    /// Sets the style of the [`Text`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let text = Text::default().style(Style::new().red().bold());
    /// ```
    pub fn style<T>(mut self, style: T) -> Self
    where
        T: Into<Style>,
    {
        self.style = style.into();
        self
    }
}

impl Text<'_> {
    /// Patches the style of each line in an existing Text, adding modifiers from the given style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// let mut raw_text = Text::raw("The first line\nThe second line");
    /// let styled_text = Text::styled(String::from("The first line\nThe second line"), style);
    /// assert_ne!(raw_text, styled_text);
    ///
    /// raw_text.patch_style(style);
    /// assert_eq!(raw_text, styled_text);
    /// ```
    pub fn patch_style(&mut self, style: Style) {
        for line in &mut self.lines {
            line.patch_style(style);
        }
    }

    /// Resets the style of the Text.
    /// Equivalent to calling `patch_style(Style::reset())`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// let mut text = Text::styled("The first line\nThe second line", style);
    ///
    /// text.reset_style();
    /// for line in &text.lines {
    ///     for span in &line.spans {
    ///         assert_eq!(Style::reset(), span.style);
    ///     }
    /// }
    /// ```
    pub fn reset_style(&mut self) {
        for line in &mut self.lines {
            line.reset_style();
        }
    }

    /// Returns the max width of all the lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let text = Text::from("The first line\nThe second line");
    /// assert_eq!(15, text.width());
    /// ```
    pub fn width(&self) -> usize {
        self.lines.iter().map(Line::width).max().unwrap_or_default()
    }

    /// Returns the height of the text, i.e. the number of lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let text = Text::from("The first line\nThe second line");
    /// assert_eq!(2, text.height());
    /// ```
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    /// Returns an iterator over the lines of the text.
    ///
    /// Each line is styled with the style of the text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::prelude::*;
    /// let style = Style::new().yellow().italic();
    /// let text = Text::styled("The first line\nThe second line", style);
    /// let styled_lines = text.styled_lines().collect::<Vec<_>>();
    /// ````
    pub fn styled_lines(&self) -> impl Iterator<Item = Line> {
        self.lines
            .iter()
            .cloned()
            .map(|line| line.style(self.style))
    }
}

impl<'a> From<String> for Text<'a> {
    fn from(s: String) -> Text<'a> {
        Text::raw(s)
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(s: &'a str) -> Text<'a> {
        Text::raw(s)
    }
}

impl<'a> From<Cow<'a, str>> for Text<'a> {
    fn from(s: Cow<'a, str>) -> Text<'a> {
        Text::raw(s)
    }
}

impl<'a> From<Span<'a>> for Text<'a> {
    fn from(span: Span<'a>) -> Text<'a> {
        Text {
            lines: vec![Line::from(span)],
            ..Default::default()
        }
    }
}

impl<'a> From<Line<'a>> for Text<'a> {
    fn from(line: Line<'a>) -> Text<'a> {
        Text {
            lines: vec![line],
            ..Default::default()
        }
    }
}

impl<'a> From<Vec<Line<'a>>> for Text<'a> {
    fn from(lines: Vec<Line<'a>>) -> Text<'a> {
        Text {
            lines,
            ..Default::default()
        }
    }
}

impl<'a> IntoIterator for Text<'a> {
    type Item = Line<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl<'a, T> Extend<T> for Text<'a>
where
    T: Into<Line<'a>>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let lines = iter.into_iter().map(Into::into);
        self.lines.extend(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Stylize;

    #[test]
    fn raw() {
        let text = Text::raw("The first line\nThe second line");
        assert_eq!(
            text.lines,
            vec![Line::from("The first line"), Line::from("The second line")]
        );
    }

    #[test]
    fn styled() {
        let style = Style::new().yellow().italic();
        let text = Text::styled("The first line\nThe second line", style);
        assert_eq!(
            text.lines,
            vec![
                Line::from(Span::styled("The first line", style)),
                Line::from(Span::styled("The second line", style))
            ]
        );
    }

    #[test]
    fn width() {
        let text = Text::from("The first line\nThe second line");
        assert_eq!(15, text.width());
    }

    #[test]
    fn height() {
        let text = Text::from("The first line\nThe second line");
        assert_eq!(2, text.height());
    }

    #[test]
    fn patch_style() {
        let style = Style::new().yellow().italic();
        let style2 = Style::new().red().underlined();
        let mut text = Text::styled("The first line\nThe second line", style);

        text.patch_style(style2);
        let expected_style = Style::new().red().italic().underlined();
        assert_eq!(
            text.lines,
            vec![
                Line::from(Span::styled("The first line", expected_style)),
                Line::from(Span::styled("The second line", expected_style))
            ]
        );
    }

    #[test]
    fn reset_style() {
        let style = Style::new().yellow().italic();
        let mut text = Text::styled("The first line\nThe second line", style);

        text.reset_style();
        assert_eq!(
            text.lines,
            vec![
                Line::from(Span::styled("The first line", Style::reset())),
                Line::from(Span::styled("The second line", Style::reset()))
            ]
        );
    }

    #[test]
    fn from_string() {
        let text = Text::from(String::from("The first line\nThe second line"));
        assert_eq!(
            text.lines,
            vec![Line::from("The first line"), Line::from("The second line")]
        );
    }

    #[test]
    fn from_str() {
        let text = Text::from("The first line\nThe second line");
        assert_eq!(
            text.lines,
            vec![Line::from("The first line"), Line::from("The second line")]
        );
    }

    #[test]
    fn from_cow() {
        let text = Text::from(Cow::Borrowed("The first line\nThe second line"));
        assert_eq!(
            text.lines,
            vec![Line::from("The first line"), Line::from("The second line")]
        );
    }

    #[test]
    fn from_span() {
        let style = Style::new().yellow().italic();
        let text = Text::from(Span::styled("The first line\nThe second line", style));
        assert_eq!(
            text.lines,
            vec![Line::from(Span::styled(
                "The first line\nThe second line",
                style
            ))]
        );
    }

    #[test]
    fn from_line() {
        let text = Text::from(Line::from("The first line"));
        assert_eq!(text.lines, vec![Line::from("The first line")]);
    }

    #[test]
    fn from_vec_line() {
        let text = Text::from(vec![
            Line::from("The first line"),
            Line::from("The second line"),
        ]);
        assert_eq!(
            text.lines,
            vec![Line::from("The first line"), Line::from("The second line")]
        );
    }

    #[test]
    fn into_iter() {
        let text = Text::from("The first line\nThe second line");
        let mut iter = text.into_iter();
        assert_eq!(iter.next(), Some(Line::from("The first line")));
        assert_eq!(iter.next(), Some(Line::from("The second line")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn extend() {
        let mut text = Text::from("The first line\nThe second line");
        text.extend(vec![
            Line::from("The third line"),
            Line::from("The fourth line"),
        ]);
        assert_eq!(
            text.lines,
            vec![
                Line::from("The first line"),
                Line::from("The second line"),
                Line::from("The third line"),
                Line::from("The fourth line"),
            ]
        );
    }

    #[test]
    fn extend_from_iter() {
        let mut text = Text::from("The first line\nThe second line");
        text.extend(vec![
            Line::from("The third line"),
            Line::from("The fourth line"),
        ]);
        assert_eq!(
            text.lines,
            vec![
                Line::from("The first line"),
                Line::from("The second line"),
                Line::from("The third line"),
                Line::from("The fourth line"),
            ]
        );
    }

    #[test]
    fn extend_from_iter_str() {
        let mut text = Text::from("The first line\nThe second line");
        text.extend(vec!["The third line", "The fourth line"]);
        assert_eq!(
            text.lines,
            vec![
                Line::from("The first line"),
                Line::from("The second line"),
                Line::from("The third line"),
                Line::from("The fourth line"),
            ]
        );
    }
}
