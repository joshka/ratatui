use ratatui::{prelude::*, widgets::*};

pub fn render(scroll: usize, area: Rect, buf: &mut Buffer) {
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);
    render_paragraph(Alignment::Left, Color::LightRed, scroll, area[0], buf);
    render_paragraph(Alignment::Center, Color::LightGreen, scroll, area[1], buf);
    render_paragraph(Alignment::Right, Color::LightBlue, scroll, area[2], buf);
}

fn render_paragraph(
    alignment: Alignment,
    color: Color,
    scroll: usize,
    area: Rect,
    buf: &mut Buffer,
) {
    let block = Block::new()
        .title(format!("{} Paragraph", alignment))
        .title_alignment(alignment)
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .padding(Padding::new(0, 1, 0, 0)); // for scrollbar
    let offset = (scroll as u16, 0);
    Paragraph::new(lipsum::lipsum(40))
        .style(Style::new().fg(color))
        .alignment(alignment)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll(offset)
        .render(area, buf);

    let scroll_area = area.inner(&Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut scroll_state = ScrollbarState::new(14)
        .viewport_content_length(scroll_area.height as usize)
        .position(scroll);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(Style::new().fg(color))
        .begin_symbol(None)
        .end_symbol(None)
        .render(scroll_area, buf, &mut scroll_state);
}
