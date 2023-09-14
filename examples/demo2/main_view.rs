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
        Block::new().bg(Color::Indexed(233)).render(area, buf);
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
        Paragraph::new(
            "Ratatui v0.23.0 "
                .bold()
                .fg(Color::Indexed(252))
                .bg(Color::Indexed(232)),
        )
        .render(area[0], buf);

        Tabs::new(vec![
            "Recipe",
            "Words",
            "Bars",
            "Chart",
            "Email",
            "Traceroute",
        ])
        .style(Style::new().fg(Color::Indexed(244)).bg(Color::Indexed(232)))
        .highlight_style(Style::new().bold().fg(Color::Rgb(64, 96, 192)))
        .select(self.selected_tab)
        .render(area[1], buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().fg(Color::Indexed(232)).bg(Color::Indexed(244));
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
        .fg(Color::Indexed(244))
        .bg(Color::Indexed(232))
        .render(area, buf);
    }

    fn render_tab1(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3); 3])
            .split(area);
        let margin = Margin {
            vertical: 1,
            horizontal: 2,
        };
        text::render_paragraph(
            Alignment::Left,
            Color::LightRed,
            0,
            area[0].inner(&margin),
            buf,
        );
        text::render_paragraph(
            Alignment::Center,
            Color::LightGreen,
            0,
            area[1].inner(&margin),
            buf,
        );
        text::render_paragraph(
            Alignment::Right,
            Color::LightBlue,
            0,
            area[2].inner(&margin),
            buf,
        );
    }

    fn render_tab2(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![5, 1, 0]);
        colors::render(area[0], buf);
        modifiers::render(area[1], buf);
        text::render(self.selected_row, area[2], buf);
    }

    fn render_tab3(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        let area = layout(area, Direction::Vertical, vec![0, 6]);
        Clear.render(area[0], buf);
        Clear.render(area[1], buf);
        bars::render(area[0], buf);
        gauges::render(self.selected_row, area[1], buf);
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
