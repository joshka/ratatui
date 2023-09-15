use crate::main_view::{layout, render_title};
use ratatui::{prelude::*, widgets::*};

fn render_chart_tab(&self, area: Rect, buf: &mut Buffer) {
    chart::render(area, buf);
}

pub fn render(area: Rect, buf: &mut Buffer) {
    let layout = layout(area, Direction::Vertical, vec![1, 0]);
    render_title("Chart", layout[0], buf);
    let area = layout[1];
    let data = (0..area.width * 2)
        .map(f64::from)
        .map(|x| (x / area.width as f64) * 10.0 - 5.0)
        .map(|x| {
            (
                x,
                x.powi(5) + 3.5 * x.powi(4) - 2.5 * x.powi(3) - 12.5 * x.powi(2) + 1.5 * x + 9.0,
            )
        })
        .collect::<Vec<_>>();
    let datasets = vec![Dataset::default()
        .name("data1")
        .marker(Marker::Braille)
        .style(Style::new().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&data[..])];
    Chart::new(datasets)
        .x_axis(Axis::default().title("x").bounds([-4.0, 4.0]).labels(vec![
            "-4.0".into(),
            "0".into(),
            "4.0".into(),
        ]))
        .y_axis(Axis::default().title("y").bounds([-8.0, 8.0]).labels(vec![
            "-8".into(),
            "0".into(),
            "8".into(),
        ]))
        .render(layout[1], buf);
}
