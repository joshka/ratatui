use itertools::Itertools;
use ratatui::prelude::*;

const LOGO: [&str; 32] = [
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

pub fn render(area: Rect, buf: &mut Buffer) {
    for (y, (line1, line2)) in LOGO.iter().tuples().enumerate() {
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
