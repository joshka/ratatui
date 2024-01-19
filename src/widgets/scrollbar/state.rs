use strum::{Display, EnumString};

/// A struct representing the state of a Scrollbar widget.
///
/// # Important
///
/// It's essential to set the `content_length` field when using this struct. This field
/// represents the total length of the scrollable content. The default value is zero
/// which will result in the Scrollbar not rendering.
///
/// For example, in the following list, assume there are 4 bullet points:
///
/// - the `content_length` is 4
/// - the `position` is 0
/// - the `viewport_content_length` is 2
///
/// ```text
/// ┌───────────────┐
/// │1. this is a   █
/// │   single item █
/// │2. this is a   ║
/// │   second item ║
/// └───────────────┘
/// ```
///
/// If you don't have multi-line content, you can leave the `viewport_content_length` set to the
/// default of 0 and it'll use the track size as a `viewport_content_length`.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScrollbarState {
    /// The total length of the scrollable content.
    pub(crate) content_length: usize,
    /// The current position within the scrollable content.
    pub(crate) position: usize,
    /// The length of content in current viewport.
    pub(crate) viewport_content_length: usize,
}

/// An enum representing a scrolling direction.
///
/// This is used with [`ScrollbarState::scroll`].
///
/// It is useful for example when you want to store in which direction to scroll.
#[derive(Debug, Default, Display, EnumString, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ScrollDirection {
    /// Forward scroll direction, usually corresponds to scrolling downwards or rightwards.
    #[default]
    Forward,
    /// Backward scroll direction, usually corresponds to scrolling upwards or leftwards.
    Backward,
}

impl ScrollbarState {
    /// Constructs a new ScrollbarState with the specified content length.
    ///
    /// `content_length` is the total number of element, that can be scrolled. See
    /// [`ScrollbarState`] for more details.
    pub fn new(content_length: usize) -> Self {
        Self {
            content_length,
            ..Default::default()
        }
    }

    /// Sets the scroll position of the scrollbar.
    ///
    /// This represents the number of scrolled items.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    /// Sets the length of the scrollable content.
    ///
    /// This is the number of scrollable items. If items have a length of one, then this is the
    /// same as the number of scrollable cells.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn content_length(mut self, content_length: usize) -> Self {
        self.content_length = content_length;
        self
    }

    /// Sets the items' size.
    ///
    /// This is a fluent setter method which must be chained or used as it consumes self
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn viewport_content_length(mut self, viewport_content_length: usize) -> Self {
        self.viewport_content_length = viewport_content_length;
        self
    }

    /// Decrements the scroll position by one, ensuring it doesn't go below zero.
    pub fn prev(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    /// Increments the scroll position by one, ensuring it doesn't exceed the length of the content.
    pub fn next(&mut self) {
        self.position = self
            .position
            .saturating_add(1)
            .min(self.content_length.saturating_sub(1))
    }

    /// Sets the scroll position to the start of the scrollable content.
    pub fn first(&mut self) {
        self.position = 0;
    }

    /// Sets the scroll position to the end of the scrollable content.
    pub fn last(&mut self) {
        self.position = self.content_length.saturating_sub(1)
    }

    /// Changes the scroll position based on the provided [`ScrollDirection`].
    pub fn scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Forward => {
                self.next();
            }
            ScrollDirection::Backward => {
                self.prev();
            }
        }
    }
}
