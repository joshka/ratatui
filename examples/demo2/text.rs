use ratatui::{prelude::*, widgets::*};

use crate::main_view::{layout, render_title};

pub fn render(scroll: usize, area: Rect, buf: &mut Buffer) {
    let areas = layout(area, Direction::Vertical, vec![1, 0]);
    render_title("Text", areas[0], buf);
    let area = areas[1];

    let (area1, area2, area3) = split_thirds(area);

    let wrap = Wrap { trim: true };
    let offset = (scroll as u16, 0);
    Paragraph::new(lipsum::lipsum(40))
        .light_red()
        .alignment(Alignment::Left)
        .wrap(wrap)
        .scroll(offset)
        .render(area1, buf);
    Paragraph::new(lipsum::lipsum(40))
        .light_yellow()
        .alignment(Alignment::Center)
        .wrap(wrap)
        .scroll(offset)
        .render(area2, buf);
    Paragraph::new(lipsum::lipsum(40))
        .light_green()
        .alignment(Alignment::Right)
        .wrap(wrap)
        .scroll(offset)
        .render(area3, buf);
}

fn split_thirds(area: Rect) -> (Rect, Rect, Rect) {
    let width = (area.width as f32 - 2.0) / 3.0;
    let x1 = width;
    let x2 = x1 + width + 1.0;
    let x3 = x2 + width + 1.0;
    let x1 = f32::round(x1) as u16;
    let x2 = f32::round(x2) as u16;
    let x3 = f32::round(x3) as u16;
    let area1 = Rect::new(area.x, area.y, x1, area.height);
    let area2 = Rect::new(area.x + x1 + 1, area.y, x2 - x1 - 1, area.height);
    let area3 = Rect::new(area.x + x2 + 1, area.y, x3 - x2 - 1, area.height);
    (area1, area2, area3)
}
