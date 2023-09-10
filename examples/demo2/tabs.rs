use ratatui::prelude::*;

mod about;
mod email;
mod misc;
mod recipe;
mod traceroute;

pub use about::AboutTab;
pub use email::EmailTab;
pub use misc::MiscWidgetsTab;
pub use recipe::RecipeTab;
pub use traceroute::TracerouteTab;

pub trait Tab {
    fn title(&self) -> String;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn select(&mut self, _row: usize) {}
}
