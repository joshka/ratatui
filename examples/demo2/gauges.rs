use crate::main_view::{layout, render_title};
use ratatui::{prelude::*, widgets::*};

pub fn render(progress: usize, area: Rect, buf: &mut Buffer) {
    let areas = layout(area, Direction::Vertical, vec![1, 0]);
    render_title("Gauges", areas[0], buf);

    let areas = layout(areas[1], Direction::Vertical, vec![1, 1, 0]);
    let percent = format!("{}%", progress);
    Gauge::default()
        .ratio(progress as f64 / 100.0)
        .label(percent.clone())
        .style(Style::new().blue().on_black())
        .gauge_style(Style::new().red().on_black())
        .use_unicode(true)
        .render(areas[0], buf);

    LineGauge::default()
        .ratio(progress as f64 / 100.0)
        .label(percent.clone())
        .style(Style::new().blue().on_black())
        .gauge_style(Style::new().red().on_black())
        .render(areas[1], buf);
}
