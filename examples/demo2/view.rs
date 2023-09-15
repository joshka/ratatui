use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{
    tabs::{self, Tab},
    tui,
};

pub struct MainView {
    pub selected_tab: usize,
    pub selected_row: usize,
    pub tabs: Vec<Box<dyn tabs::Tab>>,
}

impl Widget for MainView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::new().bg(Color::Indexed(233)).render(area, buf);
        let area = tui::layout(area, Direction::Vertical, vec![1, 0, 1]);
        self.render_title_bar(area[0], buf);
        if let Some(tab) = self.tabs.get(self.selected_tab) {
            tab.render(area[1], buf);
        }
        self.render_bottom_bar(area[2], buf);
    }
}

impl MainView {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let area = tui::layout(area, Direction::Horizontal, vec![17, 0]);
        Paragraph::new(
            "Ratatui v0.23.0 "
                .bold()
                .fg(Color::Indexed(252))
                .bg(Color::Indexed(232)),
        )
        .render(area[0], buf);

        let titles = self.tabs.iter().map(|tab| tab.title()).collect_vec();
        Tabs::new(titles)
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

    pub fn new(selected_tab: usize, selected_row: usize) -> Self {
        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(tabs::AboutTab),
            Box::new(tabs::EmailTab::new()),
            Box::new(tabs::TracerouteTab::new()),
            // Box::new(tabs::TextTab),
            // Box::new(tabs::BarsTab),
        ];

        MainView {
            selected_tab,
            selected_row,
            tabs,
        }
    }
}
