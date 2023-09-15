#![allow(dead_code)]

use ratatui::{prelude::*, widgets::*};

use super::Tab;
use crate::{colors, tui::layout};

pub struct MiscWidgetsTab {
    pub selected_row: usize,
}

impl MiscWidgetsTab {
    pub fn new() -> Self {
        Self { selected_row: 0 }
    }
}

impl Tab for MiscWidgetsTab {
    fn title(&self) -> String {
        "Misc Widgets".to_string()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        let area = layout(area, Direction::Vertical, vec![0, 6]);
        render_bars(area[0], buf);
        render_gauges(self.selected_row, area[1], buf);
    }
}

fn render_bars(area: Rect, buf: &mut Buffer) {
    Clear.render(area, buf);
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(area);

    render_simple_barchart(area[0], buf);
    render_horizontal_barchart(area[1], buf);
}

fn render_simple_barchart(area: Rect, buf: &mut Buffer) {
    let data = vec![
        ("Jan", 10),
        ("Feb", 20),
        ("Mar", 30),
        ("Apr", 40),
        ("May", 50),
        ("Jun", 60),
        ("Jul", 70),
    ];
    let block = Block::default()
        .title("BarChart")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    BarChart::default()
        .data(&data)
        .block(block)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        )
        .label_style(Style::default().fg(Color::Yellow))
        .bar_style(Style::default().fg(Color::Green))
        .render(area, buf);
}

fn render_horizontal_barchart(area: Rect, buf: &mut Buffer) {
    // https://www.videocardbenchmark.net/high_end_gpus.html
    let nvidia = Style::new().bg(Color::Green);
    let amd = Style::new().bg(Color::Red);
    let data = [
        Bar::default()
            .text_value("GeForce RTX 4090 (38,978)".into())
            .value_style(nvidia)
            .value(38978),
        Bar::default()
            .text_value("GeForce RTX 4080 (34,879)".into())
            .value_style(nvidia)
            .value(34879),
        Bar::default()
            .text_value("Radeon PRO W7800 (32,146)".into())
            .value_style(amd)
            .value(32146),
        Bar::default()
            .text_value("GeForce RTX 4070 Ti (31,659)".into())
            .value_style(nvidia)
            .value(31659),
        Bar::default()
            .text_value("Radeon RX 7900 XTX (31,180)".into())
            .value_style(amd)
            .value(31180),
    ];
    let group = BarGroup::default().label("GPU".into()).bars(&data);
    let block = Block::default()
        .title("Passmark")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    BarChart::default()
        .direction(Direction::Horizontal)
        .block(block)
        .data(group)
        .bar_gap(0)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .label_style(Style::default().fg(Color::Yellow))
        .bar_style(Style::default().light_blue())
        .render(area, buf);
}

pub fn render_gauges(progress: usize, area: Rect, buf: &mut Buffer) {
    Clear.render(area, buf);
    let block = Block::new()
        .title("Gauges")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    block.render(area, buf);
    let area = layout(inner, Direction::Vertical, vec![1, 1, 1, 1, 0]);

    let percent = (progress * 2 + 20).min(100) as f64;
    let progress_label = if percent < 100.0 {
        format!("{}%", percent)
    } else {
        "Done!".into()
    };

    render_gauge(percent, &progress_label, area[0], buf);
    render_line_gauge_1(percent, &progress_label, area[1], buf);
    render_line_gauge_2(percent, &progress_label, area[2], buf);
    render_sparkline(progress, "Sparkline", area[3], buf);
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

pub fn render_sparkline(progress: usize, title: &str, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Horizontal, vec![10, 0]);
    Paragraph::new(title)
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
