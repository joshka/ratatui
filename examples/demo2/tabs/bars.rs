#![allow(dead_code)]

use ratatui::{prelude::*, widgets::*};

use super::Tab;
use crate::{colors, tui::layout};

pub struct BarsTab {
    pub selected_row: usize,
}

impl Tab for BarsTab {
    fn title(&self) -> String {
        "Bars".to_string()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render_bars_tab(area, buf);
    }
}

impl BarsTab {
    pub fn new() -> Self {
        Self { selected_row: 0 }
    }

    fn render_bars_tab(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        let area = layout(area, Direction::Vertical, vec![0, 6]);
        Clear.render(area[0], buf);
        Clear.render(area[1], buf);
        // bars::render(area[0], buf);
        // gauges::render(self.selected_row, area[1], buf);
    }

    pub fn render(area: Rect, buf: &mut Buffer) {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        render_simple_barchart(area[0], buf);
        render_horizontal_barchart(area[1], buf);
    }
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
