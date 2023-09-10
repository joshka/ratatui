use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

use crate::main_view::layout;

struct Email {
    from: String,
    subject: String,
    body: String,
}

pub fn render(selected_row: usize, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Vertical, vec![6, 0]);
    let emails = vec![
        Email {
            from: "Alice <alice@example.com>".into(),
            subject: "Hello".into(),
            body: "Hi Bob,\n\nHow are you?\n\nAlice".into(),
        },
        Email {
            from: "Bob <bob@example.com>".into(),
            subject: "Re: Hello".into(),
            body: "Hi Alice,\n\nI'm fine, thanks!\n\nBob".into(),
        },
        Email {
            from: "Charlie <charlie@example.com>".into(),
            subject: "Re: Hello".into(),
            body: "Hi Alice,\n\nI'm fine, thanks!\n\nCharlie".into(),
        },
        Email {
            from: "Dave <dave@example.com>".into(),
            subject: "Re: Hello (STOP REPLYING TO ALL)".into(),
            body: "Hi Everyone,\n\nPlease stop replying to all.\n\nDave".into(),
        },
        Email {
            from: "Eve <eve@example.com>".into(),
            subject: "Re: Hello (STOP REPLYING TO ALL)".into(),
            body: "Hi Everyone,\n\nI'm reading all of your emails.\n\nEve".into(),
        },
    ];
    let email = emails.get(selected_row);
    render_inbox(&emails, selected_row, area[0], buf);
    render_email(email, area[1], buf);
}

fn render_inbox(emails: &[Email], selected_row: usize, area: Rect, buf: &mut Buffer) {
    let block = Block::new().title("Inbox").borders(Borders::ALL);
    let inner = block.inner(area);
    block.render(area, buf);

    let highlight_symbol = ">>";
    let from_width = emails
        .iter()
        .map(|e| e.from.width())
        .max()
        .unwrap_or_default();
    let subject_width = inner.width as usize - from_width - highlight_symbol.width() - 1;
    let items = emails
        .iter()
        .map(|e| {
            let from = format!("{:width$}", e.from, width = from_width);
            let subject = format!("{:width$}", e.subject, width = subject_width);
            let text = [from, subject].join(" ");
            ListItem::new(text)
        })
        .collect_vec();
    let mut state = ListState::default().with_selected(Some(selected_row));
    StatefulWidget::render(
        List::new(items)
            .highlight_style(Style::new().bold().yellow())
            .highlight_symbol(highlight_symbol),
        inner,
        buf,
        &mut state,
    );
}

fn render_email(email: Option<&Email>, area: Rect, buf: &mut Buffer) {
    let block = Block::new().title("Email").borders(Borders::ALL);
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
