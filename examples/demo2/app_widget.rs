use ratatui::{prelude::*, widgets::*};

use crate::{styles, tabs::Tab, tui};

pub struct AppWidget {
    tab: Box<dyn Tab>,
    tab_index: usize,
    titles: Vec<String>,
}

impl AppWidget {
    pub fn new(tab: Box<dyn Tab>, tab_index: usize, titles: Vec<String>) -> Self {
        AppWidget {
            tab,
            tab_index,
            titles,
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

        Tabs::new(self.titles.clone())
            .style(styles::TABS)
            .highlight_style(styles::TABS_SELECTED)
            .select(self.tab_index)
            .render(area[1], buf);
    }

    fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        self.tab.render(area, buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().fg(Color::Indexed(232)).bg(Color::Indexed(236));
        Paragraph::new(Line::from(vec![
            " Q/Esc ".set_style(key_style),
            " Quit  ".into(),
            " Tab ".set_style(key_style),
            " Next Tab  ".into(),
            " ←/h ".set_style(key_style),
            " Left  ".into(),
            " →/l ".set_style(key_style),
            " Right  ".into(),
            " ↑/k ".set_style(key_style),
            " Up  ".into(),
            " ↓/j ".set_style(key_style),
            " Down".into(),
        ]))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232))
        .render(area, buf);
    }
}
