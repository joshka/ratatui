use std::rc::Rc;

use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{bars, chart, colors, email, gauges, modifiers, recipe, text, traceroute};

pub struct MainView {
    pub selected_tab: usize,
    pub selected_row: usize,
}

impl Widget for MainView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![1, 0, 1]);
        self.render_title_bar(area[0], buf);
        match self.selected_tab {
            0 => self.render_tab1(area[1], buf),
            1 => self.render_tab2(area[1], buf),
            2 => self.render_tab3(area[1], buf),
            3 => self.render_tab4(area[1], buf),
            4 => self.render_tab5(area[1], buf),
            5 => self.render_tab6(area[1], buf),
            _ => unreachable!(),
        }
        self.render_bottom_bar(area[2], buf);
    }
}

impl MainView {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Horizontal, vec![17, 0]);
        Paragraph::new("Ratatui v0.23.0 ".italic().bold().white().on_red()).render(area[0], buf);

        Tabs::new(vec![
            "Recipe",
            "Words",
            "Bars",
            "Chart",
            "Email",
            "Traceroute",
        ])
        .style(Style::new().blue())
        .highlight_style(Style::new().bold().underlined().light_blue())
        .select(self.selected_tab)
        .render(area[1], buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().black().on_dark_gray().not_dim();
        Paragraph::new(Line::from(vec![
            " Q ".set_style(key_style),
            " Quit ".into(),
            " ←/h ".set_style(key_style),
            " Previous Tab ".into(),
            " →/l ".set_style(key_style),
            " Next Tab ".into(),
            " ↑/k ".set_style(key_style),
            " Previous Row ".into(),
            " ↓/j ".set_style(key_style),
            " Next Row".into(),
        ]))
        .dim()
        .render(area, buf);
    }

    fn render_tab1(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![0]);
        recipe::render(self.selected_row, area[0], buf);
    }

    fn render_tab2(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![5, 1, 0]);
        colors::render(area[0], buf);
        modifiers::render(area[1], buf);
        text::render(self.selected_row, area[2], buf);
    }

    fn render_tab3(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![4, 0]);
        gauges::render(self.selected_row, area[0], buf);
        bars::render(area[1], buf);
    }

    fn render_tab4(&self, area: Rect, buf: &mut Buffer) {
        chart::render(area, buf);
    }

    fn render_tab5(&self, area: Rect, buf: &mut Buffer) {
        email::render(self.selected_row, area, buf);
    }

    fn render_tab6(&self, area: Rect, buf: &mut Buffer) {
        traceroute::render(self.selected_row, area, buf);
    }
}

pub fn render_title(title: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(title)
        .dim()
        .render(Rect { height: 1, ..area }, buf);
}

/// helper method to split an area into multiple sub-areas
pub fn layout(area: Rect, direction: Direction, heights: Vec<u16>) -> Rc<[Rect]> {
    let constraints = heights
        .iter()
        .map(|&h| {
            if h > 0 {
                Constraint::Length(h)
            } else {
                Constraint::Min(0)
            }
        })
        .collect_vec();
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
}
