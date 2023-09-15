use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use super::Tab;
use crate::{colors, styles, tui::layout};

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
        colors::render_rgb_colors(area, buf);
        let area = layout(area, Direction::Horizontal, vec![34, 0]);
        render_crate_description(area[1], buf);
        render_logo(area[0], buf);
    }
}

pub fn render_logo(area: Rect, buf: &mut Buffer) {
    let area = area.inner(&Margin {
        vertical: 0,
        horizontal: 2,
    });
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

fn render_crate_description(area: Rect, buf: &mut Buffer) {
    let area = area.inner(
        &(Margin {
            vertical: 4,
            horizontal: 2,
        }),
    );
    Clear.render(area, buf); // clear out the color swatches

    let area = area.inner(
        &(Margin {
            vertical: 1,
            horizontal: 2,
        }),
    );
    let text = "- cooking up terminal user interfaces -

    Ratatui is a Rust crate that provides widgets (e.g. Pargraph, Table) and draws them to the \
    screen efficiently every frame.";
    Paragraph::new(text)
        .style(styles::DESCRIPTION)
        .block(
            Block::new()
                .title("Ratatui")
                .title_alignment(Alignment::Center)
                .borders(Borders::TOP)
                .border_style(styles::BORDERS)
                .padding(Padding::new(0, 0, 0, 0)),
        )
        .wrap(Wrap { trim: true })
        .scroll((0, 0))
        .render(area, buf);
}
