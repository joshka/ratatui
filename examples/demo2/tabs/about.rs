use itertools::Itertools;
use ratatui::prelude::*;

use super::Tab;
use crate::{colors, text::render_paragraph, tui::layout};

const RATATUI_LOGO: [&str; 32] = [
    "               ███              ",
    "              █████             ",
    "            ███████             ",
    "           ████████             ",
    "          █████████             ",
    "         ██████████             ",
    "        ████████████            ",
    "        █████████████           ",
    "        █████████████     ██████",
    "         ███████████    ████████",
    "              █████ ███████████ ",
    "               ███ ██xx████████ ",
    "                █ ███xx████████ ",
    "            ████ █████████████  ",
    "           █████████████████    ",
    "           ████████████████     ",
    "           ████████████████     ",
    "            ███ ██████████      ",
    "          ██    █████████       ",
    "         █xx█   █████████       ",
    "        █xxxx█ ██████████       ",
    "       █xx█xxx█ █████████       ",
    "      █xx██xxxx█ ████████       ",
    "     █xxxxxxxxxx█ ██████████    ",
    "    █xxxxxxxxxxxx█ ██████████   ",
    "   █xxxxxxx██xxxxx█ █████████   ",
    "  █xxxxxxxxx██xxxxx█ ████  ███  ",
    " █xxxxxxxxxxxxxxxxxx█ ██   ███  ",
    "█xxxxxxxxxxxxxxxxxxxx█ █   ███  ",
    " █xxxxxxxxxxxxxxxxxxxx█   ███   ",
    "  █xxxxxxxxxxxxxxxxxxxx█ ██     ",
    "   █xxxxxxxxxxxxxxxxxxxx█ █     ",
];

pub struct AboutTab;

impl AboutTab {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tab for AboutTab {
    fn title(&self) -> String {
        "About".to_string()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render(area, buf);
    }
}

pub fn render(area: Rect, buf: &mut Buffer) {
    colors::render_rgb_colors(area, buf);
    let area = layout(area, Direction::Horizontal, vec![32, 0]);
    let margin = Margin {
        vertical: 0,
        horizontal: 1,
    };
    render_logo(area[0].inner(&margin), buf);
    let margin = Margin {
        vertical: 1,
        horizontal: 2,
    };
    render_paragraph(
        Alignment::Left,
        Color::LightBlue,
        0,
        area[1].inner(&margin),
        buf,
    );
}

pub fn render_logo(area: Rect, buf: &mut Buffer) {
    for (y, (line1, line2)) in RATATUI_LOGO.iter().tuples().enumerate() {
        for (x, (ch1, ch2)) in line1.chars().zip(line2.chars()).enumerate() {
            let x = area.left() + x as u16;
            let y = area.top() + y as u16;
            let cell = buf.get_mut(x, y);
            match (ch1, ch2) {
                ('█', '█') => {
                    cell.set_char('█');
                    cell.fg = Color::Indexed(255);
                }
                ('█', ' ') => {
                    cell.set_char('▀');
                    cell.fg = Color::Indexed(255);
                }
                (' ', '█') => {
                    cell.set_char('▄');
                    cell.fg = Color::Indexed(255);
                }
                ('█', 'x') => {
                    cell.set_char('▀');
                    cell.fg = Color::Indexed(255);
                    cell.bg = Color::Black;
                }
                ('x', '█') => {
                    cell.set_char('▄');
                    cell.fg = Color::Indexed(255);
                    cell.bg = Color::Black;
                }
                ('x', 'x') => {
                    cell.set_char(' ');
                    cell.fg = Color::Indexed(255);
                    cell.bg = Color::Black;
                }
                (_, _) => {}
            };
        }
    }
}
