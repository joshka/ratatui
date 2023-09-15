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
        let area = layout(area, Direction::Horizontal, vec![32, 0]);
        render_logo(area[0], buf);
        render_crate_description(area[1], buf);
    }
}

pub fn render_logo(area: Rect, buf: &mut Buffer) {
    let area = area.inner(&Margin {
        vertical: 0,
        horizontal: 1,
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
    let margin = Margin {
        vertical: 1,
        horizontal: 2,
    };
    let area = area.inner(&margin);

    // intentionally draw a paragraph inside a block instead of using Paragraph::block()
    // so that the block's border is not drawn with the paragraph's background color
    let block = Block::new()
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(styles::BORDERS);
    let inner = block.inner(area);
    block.render(area, buf);

    let offset = (0, 0);
    Clear.render(inner, buf); // necessary in order to clear out the color swatches
    let text = "Ratatui
    - cooking up terminal user interfaces -

    Ratatui is a Rust crate that provides widgets (e.g. Pargraph, Table) and draws them to the screen efficiently every frame.";
    Paragraph::new(text)
        .style(styles::DESCRIPTION)
        .block(Block::new().padding(Padding::new(2, 2, 1, 1)))
        .wrap(Wrap { trim: true })
        .scroll(offset)
        .render(inner, buf);

    // let scroll_area = area.inner(&Margin {
    //     vertical: 1,
    //     horizontal: 0,
    // });
    // let mut scroll_state = ScrollbarState::new(14)
    //     .viewport_content_length(scroll_area.height as usize)
    //     .position(scroll);
    // Scrollbar::new(ScrollbarOrientation::VerticalRight)
    //     .style(Style::new().fg(color))
    //     .begin_symbol(None)
    //     .end_symbol(None)
    //     .render(scroll_area, buf, &mut scroll_state);;
}
