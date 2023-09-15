use ratatui::prelude::*;

pub const DARK_BLUE: Color = Color::Rgb(16, 24, 48);
pub const LIGHT_BLUE: Color = Color::Rgb(64, 96, 192);
pub const APP_BACKGROUND: Color = DARK_BLUE;
pub const APP_TITLE: Style = Style::new()
    .add_modifier(Modifier::BOLD)
    .fg(Color::Indexed(252))
    .bg(Color::Indexed(232));
pub const TABS: Style = Style::new().fg(Color::Indexed(244)).bg(Color::Indexed(232));
pub const TABS_SELECTED: Style = Style::new().add_modifier(Modifier::BOLD).fg(LIGHT_BLUE);
pub const BORDERS: Style = Style::new().fg(Color::Indexed(252));
pub const DESCRIPTION: Style = Style::new().fg(Color::Gray).bg(DARK_BLUE);
