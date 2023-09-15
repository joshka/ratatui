use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{
    styles,
    tabs::{self, Tab},
    tui,
};

pub struct AppWidget {
    pub tab_index: usize,
    pub tabs: Vec<Box<dyn tabs::Tab>>,
}

impl AppWidget {
    pub fn new(selected_tab: usize) -> Self {
        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(tabs::AboutTab),
            Box::new(tabs::EmailTab::new()),
            Box::new(tabs::TracerouteTab::new()),
            // Box::new(tabs::TextTab),
            // Box::new(tabs::BarsTab),
        ];

        AppWidget {
            tab_index: selected_tab,
            tabs,
        }
    }
}

impl Widget for AppWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::new().bg(styles::APP_BACKGROUND).render(area, buf);
        let area = tui::layout(area, Direction::Vertical, vec![1, 0, 1]);
        self.render_title_bar(area[0], buf);
        self.render_selected_tab(area[1], buf);
        self.render_bottom_bar(area[2], buf);
    }
}

impl AppWidget {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let area = tui::layout(area, Direction::Horizontal, vec![17, 0]);

        Paragraph::new(Span::styled("Ratatui v0.23.0 ", styles::APP_TITLE)).render(area[0], buf);

        let titles = self.tabs.iter().map(|tab| tab.title()).collect_vec();
        Tabs::new(titles)
            .style(styles::TABS)
            .highlight_style(styles::TABS_SELECTED)
            .select(self.tab_index)
            .render(area[1], buf);
    }

    fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        if let Some(tab) = self.tabs.get(self.tab_index) {
            tab.render(area, buf);
        }
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().fg(Color::Indexed(232)).bg(Color::Indexed(244));
        Paragraph::new(Line::from(vec![
            " Q ".set_style(key_style),
            " Quit ".into(),
            " Tab ".set_style(key_style),
            " Next Tab ".into(),
            " Shift+Tab ".set_style(key_style),
            " Prev Tab ".into(),
            " ← ".set_style(key_style),
            " Left ".into(),
            " → ".set_style(key_style),
            " Right ".into(),
            " ↑ ".set_style(key_style),
            " Up ".into(),
            " ↓ ".set_style(key_style),
            " Down ".into(),
        ]))
        .fg(Color::Indexed(244))
        .bg(Color::Indexed(232))
        .render(area, buf);
    }
}
