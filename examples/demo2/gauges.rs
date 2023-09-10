use ratatui::{prelude::*, widgets::*};

use crate::main_view::layout;

pub fn render(progress: usize, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Vertical, vec![1, 1, 1, 1, 0]);

    let percent = (progress * 2 + 20).min(100) as f64;
    let progress_label = if percent < 100.0 {
        format!("{}%", percent)
    } else {
        "Done!".into()
    };

    render_gauge(percent, &progress_label, area[0], buf);
    render_line_gauge_1(percent, &progress_label, area[1], buf);
    render_line_gauge_2(percent, &progress_label, area[2], buf);
    render_sparkline(progress, area[3], buf);
}

fn render_gauge(percent: f64, label: &str, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Horizontal, vec![10, 0]);
    Paragraph::new("Gauge")
        .style(Style::new().light_green())
        .render(area[0], buf);
    Gauge::default()
        .ratio(percent / 100.0)
        .label(format!("Processing: {}", label))
        .style(Style::new().black())
        .gauge_style(Style::new().green().on_light_green())
        .use_unicode(false)
        .render(area[1], buf);
}

fn render_line_gauge_1(percent: f64, label: &str, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Horizontal, vec![10, 0]);
    Paragraph::new("LineGauge")
        .style(Style::new().light_blue())
        .render(area[0], buf);
    LineGauge::default()
        .ratio(percent / 100.0)
        .label(format!("Upload: {}", label))
        .style(Style::new().light_blue())
        .gauge_style(Style::new().blue().on_light_blue())
        .line_set(symbols::line::NORMAL)
        .render(area[1], buf);
}

fn render_line_gauge_2(percent: f64, label: &str, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Horizontal, vec![10, 0]);
    LineGauge::default()
        .ratio(1.0 - percent / 100.0)
        .label(format!("Download: {}", label))
        .style(Style::new().light_yellow())
        .gauge_style(Style::new().light_red().on_yellow())
        .line_set(symbols::line::THICK)
        .render(area[1], buf);
}

fn render_sparkline(progress: usize, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Horizontal, vec![10, 0]);
    Paragraph::new("Sparkline")
        .style(Style::new().white())
        .render(area[0], buf);
    let mut data = [
        8, 8, 8, 8, 7, 7, 7, 6, 6, 5, 4, 3, 3, 2, 2, 1, 1, 1, 2, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 7,
        7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 4, 6, 7, 8, 8, 8, 8, 6, 4, 2, 1, 1, 1, 1, 2, 2, 2, 3,
        3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7,
    ];
    let mid = progress % data.len();
    data.rotate_left(mid);
    Sparkline::default()
        .data(&data)
        .style(Style::new().white())
        .render(area[1], buf);
}
