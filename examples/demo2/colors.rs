#![allow(dead_code)]
use palette::{
    convert::{FromColorUnclamped, IntoColorUnclamped},
    Okhsv, Srgb,
};
use ratatui::{prelude::*, widgets::*};

// use crate::view::layout;

// fn render_colors_tab(&self, area: Rect, buf: &mut Buffer) {
//     let area = layout(area, Direction::Vertical, vec![5, 1, 0]);
//     colors::render(area[0], buf);
//     modifiers::render(area[1], buf);
//     text::render(self.selected_row, area[2], buf);
// }

// pub fn render(area: Rect, buf: &mut Buffer) {
//     let area = layout(area, Direction::Horizontal, vec![8, 1, 36, 1, 0]);

//     render_16_colors(area[0], buf);
//     render_256_colors(area[2], buf);
//     render_rgb_colors(area[4], buf);
// }

fn render_16_colors(area: Rect, buf: &mut Buffer) {
    let sym = "██";
    Paragraph::new(vec![
        Line::from(vec![sym.black(), sym.red(), sym.green(), sym.yellow()]),
        Line::from(vec![sym.blue(), sym.magenta(), sym.cyan(), sym.gray()]),
        Line::from(vec![
            sym.dark_gray(),
            sym.light_red(),
            sym.light_green(),
            sym.light_yellow(),
        ]),
        Line::from(vec![
            sym.light_blue(),
            sym.light_magenta(),
            sym.light_cyan(),
            sym.white(),
        ]),
    ])
    .render(area, buf);
}

fn render_256_colors(area: Rect, buf: &mut Buffer) {
    for (xi, x) in (16..52).zip(area.left()..area.right()) {
        for (yi, y) in (0..3).zip(area.top()..area.bottom()) {
            let fg = Color::Indexed(yi * 72 + xi);
            let bg = Color::Indexed(yi * 72 + xi + 36);
            buf.get_mut(x, y).set_char('▀').set_fg(fg).set_bg(bg);
        }
        let fg = Color::Indexed(xi.saturating_add(216));
        buf.get_mut(x, area.bottom() - 1).set_char('█').set_fg(fg);
    }
}

pub fn render_rgb_colors(area: Rect, buf: &mut Buffer) {
    for (xi, x) in (area.left()..area.right()).enumerate() {
        for (yi, y) in (area.top()..area.bottom()).enumerate() {
            let yi = area.height as usize - yi - 1;
            let hue = xi as f32 * 360.0 / area.width as f32;
            let value_bg = (yi as f32 - 0.0) / (area.height as f32);
            let value_fg = (yi as f32 + 0.5) / (area.height as f32);
            let fg = Okhsv::<f32>::new(hue, Okhsv::max_saturation(), value_fg);
            let fg: Srgb<f32> = fg.into_color_unclamped();
            let fg: Srgb<u8> = fg.into_format();
            let fg = Color::Rgb(fg.red, fg.green, fg.blue);
            let bg = Okhsv::new(hue, Okhsv::max_saturation(), value_bg);
            let bg = Srgb::<f32>::from_color_unclamped(bg);
            let bg: Srgb<u8> = bg.into_format();
            let bg = Color::Rgb(bg.red, bg.green, bg.blue);
            buf.get_mut(x, y).set_char('▀').set_fg(fg).set_bg(bg);
        }
    }
}
