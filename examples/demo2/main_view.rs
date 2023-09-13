use std::rc::Rc;

use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{chart, colors, gauges, modifiers, table, text};

pub struct MainView {
    pub selected_tab: usize,
    pub selected_row: usize,
}

impl Widget for MainView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![1, 0, 1]);
        self.render_title_bar(areas[0], buf);
        match self.selected_tab {
            0 => self.render_tab1(areas[1], buf),
            1 => self.render_tab2(areas[1], buf),
            2 => self.render_tab3(areas[1], buf),
            3 => self.render_tab4(areas[1], buf),
            _ => unreachable!(),
        }
        self.render_bottom_bar(areas[2], buf);
    }
}

impl MainView {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Horizontal, vec![15, 0]);
        Paragraph::new("Ratatui")
            .italic()
            .bold()
            .white()
            .render(areas[0], buf);

        Tabs::new(vec!["Table", "Text Style", "Gauges", "Chart"])
            .style(Style::new().blue())
            .highlight_style(Style::new().bold().underlined().light_blue())
            .select(self.selected_tab)
            .render(areas[1], buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().bold().white().on_blue();
        Paragraph::new(Line::from(vec![
            // █ characters here are a hack around the VHS bug that swallows style resets for
            // whitespace characters
            "█".into(),
            "Q".set_style(key_style),
            "█ Quit █".into(),
            "←".set_style(key_style),
            "█ Previous Tab █".into(),
            "→".set_style(key_style),
            "█ Next Tab █".into(),
            "↑".set_style(key_style),
            "█ Previous Row █".into(),
            "↓".set_style(key_style),
            "█ Next Row".into(),
        ]))
        .blue()
        .on_black()
        .render(area, buf);
    }

    fn render_tab1(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![0]);
        table::render(self.selected_row, areas[0], buf);
    }

    fn render_tab2(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![6, 2, 0]);
        colors::render(areas[0], buf);
        modifiers::render(areas[1], buf);
        text::render(self.selected_row, areas[2], buf);
    }

    fn render_tab3(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![0]);
        gauges::render(self.selected_row, areas[0], buf);
    }

    fn render_tab4(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![0]);
        chart::render(areas[0], buf);
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
