use ratatui::prelude::*;

mod about;
mod bars;
mod email;
mod traceroute;

pub use about::AboutTab;
pub use bars::BarsTab;
pub use email::EmailTab;
pub use traceroute::TracerouteTab;

pub trait Tab {
    fn title(&self) -> String;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn select(&mut self, _row: usize) {}
}
