// use ratatui::{prelude::*, widgets::*};

// pub fn render_text_tab(&self, area: Rect, buf: &mut Buffer) {
//     colors::render_rgb_colors(area, buf);
//     let area = area.inner(&Margin {
//         vertical: 0,
//         horizontal: 1,
//     });
//     let area = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(vec![Constraint::Ratio(1, 3); 3])
//         .split(area);
//     let margin = Margin {
//         vertical: 1,
//         horizontal: 1,
//     };
//     text::render_paragraph(
//         Alignment::Left,
//         Color::Rgb(192, 92, 64),
//         0,
//         area[0].inner(&margin),
//         buf,
//     );
//     text::render_paragraph(
//         Alignment::Center,
//         Color::LightGreen,
//         0,
//         area[1].inner(&margin),
//         buf,
//     );
//     text::render_paragraph(
//         Alignment::Right,
//         Color::LightBlue,
//         0,
//         area[2].inner(&margin),
//         buf,
//     );
// }

// pub fn render(scroll: usize, area: Rect, buf: &mut Buffer) {
//     let area = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(vec![
//             Constraint::Ratio(1, 3),
//             Constraint::Ratio(1, 3),
//             Constraint::Ratio(1, 3),
//         ])
//         .split(area);
//     render_paragraph(Alignment::Left, Color::LightRed, scroll, area[0], buf);
//     render_paragraph(Alignment::Center, Color::LightGreen, scroll, area[1], buf);
//     render_paragraph(Alignment::Right, Color::LightBlue, scroll, area[2], buf);
// }

// pub fn render_paragraph(
//     alignment: Alignment,
//     _color: Color,
//     scroll: usize,
//     area: Rect,
//     buf: &mut Buffer,
// ) {
//     let block = Block::new()
//         .title(format!("{} aligned", alignment))
//         .title_alignment(alignment)
//         .title_style(
//             Style::new()
//                 .fg(Color::Indexed(255))
//                 .bg(Color::Rgb(16, 24, 48)),
//         )
//         .border_type(BorderType::Rounded)
//         .borders(Borders::ALL)
//         .border_style(Style::new().fg(Color::Indexed(252)));
//     // .padding(Padding::new(1, 1, 0, 0)); // for scrollbar
//     let inner = block.inner(area);
//     block.render(area, buf);
//     let offset = (scroll as u16, 0);
//     Clear.render(inner, buf);
//     Paragraph::new(lipsum::lipsum(40))
//         .style(Style::new().fg(Color::Gray).bg(Color::Rgb(16, 24, 48)))
//         .alignment(alignment)
//         .wrap(Wrap { trim: true })
//         .scroll(offset)
//         .render(inner, buf);

//     // let scroll_area = area.inner(&Margin {
//     //     vertical: 1,
//     //     horizontal: 0,
//     // });
//     // let mut scroll_state = ScrollbarState::new(14)
//     //     .viewport_content_length(scroll_area.height as usize)
//     //     .position(scroll);
//     // Scrollbar::new(ScrollbarOrientation::VerticalRight)
//     //     .style(Style::new().fg(color))
//     //     .begin_symbol(None)
//     //     .end_symbol(None)
//     //     .render(scroll_area, buf, &mut scroll_state);
// }
