use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

use super::Tab;
use crate::{colors, styles, tui::layout};

#[derive(Debug, Default)]
pub struct Email {
    from: &'static str,
    subject: &'static str,
    body: &'static str,
}

#[derive(Debug, Default)]
pub struct EmailTab {
    selected_index: usize,
}

impl Tab for EmailTab {
    fn title(&self) -> String {
        "Email".to_string()
    }

    fn select(&mut self, row: usize) {
        self.selected_index = row;
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }
}

const EMAILS: &[Email] = &[
    Email {
        from: "Alice <alice@example.com>",
        subject: "Hello",
        body: "Hi Bob,\n\nHow are you?\n\nAlice",
    },
    Email {
        from: "Bob <bob@example.com>",
        subject: "Re: Hello",
        body: "Hi Alice,\nI'm fine, thanks!\n\nBob",
    },
    Email {
        from: "Charlie <charlie@example.com>",
        subject: "Re: Hello",
        body: "Hi Alice,\nI'm fine, thanks!\n\nCharlie",
    },
    Email {
        from: "Dave <dave@example.com>",
        subject: "Re: Hello (STOP REPLYING TO ALL)",
        body: "Hi Everyone,\nPlease stop replying to all.\n\nDave",
    },
    Email {
        from: "Eve <eve@example.com>",
        subject: "Re: Hello (STOP REPLYING TO ALL)",
        body: "Hi Everyone,\nI'm reading all your emails.\n\nEve",
    },
];

impl EmailTab {
    pub fn new(selected_index: usize) -> Self {
        Self {
            selected_index: selected_index % EMAILS.len(),
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        Clear.render(area, buf);
        let area = layout(area, Direction::Vertical, vec![6, 0]);
        self.render_inbox(area[0], buf);
        self.render_email(area[1], buf);
    }

    fn render_inbox(&self, area: Rect, buf: &mut Buffer) {
        let area = layout(area, Direction::Vertical, vec![1, 0]);
        Tabs::new(vec![" Inbox ", " Sent ", " Drafts "])
            .style(Style::new().fg(Color::Indexed(244)).bg(Color::Indexed(232)))
            .highlight_style(
                Style::new()
                    .bold()
                    .fg(Color::Indexed(232))
                    .bg(Color::Rgb(64, 96, 192)),
            )
            .select(0)
            .divider("")
            .render(area[0], buf);

        let highlight_symbol = ">>";
        let from_width = EMAILS
            .iter()
            .map(|e| e.from.width())
            .max()
            .unwrap_or_default();
        let subject_width = area[1].width as usize - from_width - highlight_symbol.width() - 1;
        let items = EMAILS
            .iter()
            .map(|e| {
                let from = format!("{:width$}", e.from, width = from_width);
                let subject = format!("{:width$}", e.subject, width = subject_width);
                let text = [from, subject].join(" ");
                ListItem::new(text)
            })
            .collect_vec();
        let mut state = ListState::default().with_selected(Some(self.selected_index));
        StatefulWidget::render(
            List::new(items)
                .highlight_style(Style::new().bold().yellow())
                .highlight_symbol(highlight_symbol),
            area[1],
            buf,
            &mut state,
        );
    }

    fn render_email(&self, area: Rect, buf: &mut Buffer) {
        let email = EMAILS.get(self.selected_index);
        let block = Block::new().borders(Borders::TOP).style(styles::APP);
        let inner = block.inner(area);
        block.render(area, buf);
        if let Some(email) = email {
            let mut text = vec![
                Line::from(vec!["From: ".bold(), email.from.clone().into()]),
                Line::from(vec!["Subject: ".bold(), email.subject.clone().into()]),
                "-".repeat(inner.width as usize).dim().into(),
            ];
            text.extend(email.body.lines().map(Line::from));
            Paragraph::new(text).render(inner, buf);
        } else {
            Paragraph::new("No email selected").render(inner, buf);
        }
    }
}
