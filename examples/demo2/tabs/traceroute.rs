use itertools::Itertools;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Map, Points},
        *,
    },
};

use super::Tab;
use crate::{colors, styles, tui::layout};

#[derive(Debug)]
struct Hop {
    host: &'static str,
    address: &'static str,
    location: (f64, f64),
}

impl Hop {
    const fn new(name: &'static str, address: &'static str, location: (f64, f64)) -> Self {
        Self {
            host: name,
            address,
            location,
        }
    }
}

#[derive(Debug)]
pub struct TracerouteTab {
    selected_row: usize,
}

impl TracerouteTab {
    pub fn new(selected_row: usize) -> Self {
        Self {
            selected_row: selected_row % HOPS.len(),
        }
    }
}

impl Tab for TracerouteTab {
    fn title(&self) -> String {
        "Traceroute".to_string()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render_traceroute_tab(area, buf);
    }

    fn select(&mut self, row: usize) {
        self.selected_row = row;
    }
}

impl TracerouteTab {
    fn render_traceroute_tab(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        Clear.render(area, buf);
        Block::new().style(styles::APP).render(area, buf);
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(area);
        let left_area = layout(area[0], Direction::Vertical, vec![0, 4]);
        self.render_hops(left_area[0], buf);
        render_ping(self.selected_row, left_area[1], buf);
        self.render_map(area[1], buf);
    }

    fn render_hops(&self, area: Rect, buf: &mut Buffer) {
        let mut state = TableState::default().with_selected(Some(self.selected_row));
        let rows = HOPS
            .iter()
            .map(|hop| Row::new(vec![hop.host, hop.address]))
            .collect_vec();
        let block = Block::default()
            .title("Traceroute bad.horse")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        StatefulWidget::render(
            Table::new(rows)
                .header(Row::new(vec!["Host", "Address"]).bold().underlined())
                .widths(&[Constraint::Max(100), Constraint::Length(15)])
                .highlight_style(Style::new().dark_gray().on_white())
                .block(block),
            area,
            buf,
            &mut state,
        );
    }

    fn render_map(&self, area: Rect, buf: &mut Buffer) {
        let path: Option<(&Hop, &Hop)> = HOPS.iter().tuple_windows().nth(self.selected_row);
        let block = Block::new().title("Map").borders(Borders::ALL);
        let map = Map {
            resolution: canvas::MapResolution::High,
            color: Color::Gray,
        };
        Canvas::default()
            .marker(Marker::Dot)
            .x_bounds([113.0, 154.0]) // australia
            .y_bounds([-42.0, -11.0]) // australia
            .paint(|context| {
                context.draw(&map);
                if let Some(path) = path {
                    context.draw(&canvas::Line::new(
                        path.0.location.0,
                        path.0.location.1,
                        path.1.location.0,
                        path.1.location.1,
                        Color::Blue,
                    ));
                    context.draw(&Points {
                        color: Color::Green,
                        coords: &[path.0.location], // sydney
                    });
                    context.draw(&Points {
                        color: Color::Red,
                        coords: &[path.1.location], // perth
                    });
                }
            })
            .block(block)
            .render(area, buf);
    }
}

const CANBERRA: (f64, f64) = (149.1, -35.3);
const SYDNEY: (f64, f64) = (151.1, -33.9);
const MELBOURNE: (f64, f64) = (144.9, -37.8);
const PERTH: (f64, f64) = (115.9, -31.9);
const DARWIN: (f64, f64) = (130.8, -12.4);
const BRISBANE: (f64, f64) = (153.0, -27.5);
const ADELAIDE: (f64, f64) = (138.6, -34.9);

// Go traceroute bad.horse some time, it's fun. these locations are made up and don't correspond
// to the actual IP addresses (which are in Toronto, Canada).
const HOPS: &[Hop] = &[
    Hop::new("home", "127.0.0.1", CANBERRA),
    Hop::new("bad.horse", "162.252.205.130", SYDNEY),
    Hop::new("bad.horse", "162.252.205.131", MELBOURNE),
    Hop::new("bad.horse", "162.252.205.132", BRISBANE),
    Hop::new("bad.horse", "162.252.205.133", SYDNEY),
    Hop::new("he.rides.across.the.nation", "162.252.205.134", PERTH),
    Hop::new("the.thoroughbred.of.sin", "162.252.205.135", DARWIN),
    Hop::new("he.got.the.application", "162.252.205.136", BRISBANE),
    Hop::new("that.you.just.sent.in", "162.252.205.137", ADELAIDE),
    Hop::new("it.needs.evaluation", "162.252.205.138", DARWIN),
    Hop::new("so.let.the.games.begin", "162.252.205.139", PERTH),
    Hop::new("a.heinous.crime", "162.252.205.140", BRISBANE),
    Hop::new("a.show.of.force", "162.252.205.141", CANBERRA),
    Hop::new("a.murder.would.be.nice.of.course", "162.252.205.142", PERTH),
    Hop::new("bad.horse", "162.252.205.143", MELBOURNE),
    Hop::new("bad.horse", "162.252.205.144", DARWIN),
    Hop::new("bad.horse", "162.252.205.145", MELBOURNE),
    Hop::new("he-s.bad", "162.252.205.146", PERTH),
    Hop::new("the.evil.league.of.evil", "162.252.205.147", BRISBANE),
    Hop::new("is.watching.so.beware", "162.252.205.148", DARWIN),
    Hop::new("the.grade.that.you.receive", "162.252.205.149", PERTH),
    Hop::new("will.be.your.last.we.swear", "162.252.205.150", ADELAIDE),
    Hop::new("so.make.the.bad.horse.gleeful", "162.252.205.151", SYDNEY),
    Hop::new("or.he-ll.make.you.his.mare", "162.252.205.152", MELBOURNE),
    Hop::new("o_o", "162.252.205.153", BRISBANE),
    Hop::new("you-re.saddled.up", "162.252.205.154", DARWIN),
    Hop::new("there-s.no.recourse", "162.252.205.155", PERTH),
    Hop::new("it-s.hi-ho.silver", "162.252.205.156", SYDNEY),
    Hop::new("signed.bad.horse", "162.252.205.157", CANBERRA),
];

pub fn render_ping(progress: usize, area: Rect, buf: &mut Buffer) {
    let mut data = [
        8, 8, 8, 8, 7, 7, 7, 6, 6, 5, 4, 3, 3, 2, 2, 1, 1, 1, 2, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 7,
        7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 4, 6, 7, 8, 8, 8, 8, 6, 4, 2, 1, 1, 1, 1, 2, 2, 2, 3,
        3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7,
    ];
    let mid = progress % data.len();
    data.rotate_left(mid);
    Sparkline::default()
        .block(
            Block::new()
                .title("Ping")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .data(&data)
        .style(Style::new().white())
        .render(area, buf);
}
