use itertools::Itertools;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Map, Points},
        *,
    },
};

struct Hop {
    host: &'static str,
    address: &'static str,
    location: (f64, f64),
}

impl Hop {
    fn new(name: &'static str, address: &'static str, location: (f64, f64)) -> Self {
        Self {
            host: name,
            address,
            location,
        }
    }
}

pub fn render(selected_row: usize, area: Rect, buf: &mut Buffer) {
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);
    let hops = generate_hops();
    render_hops(&hops, selected_row, area[0], buf);
    let path: Option<(&Hop, &Hop)> = hops.iter().tuple_windows().nth(selected_row);
    render_map(path, area[1], buf);
}

fn generate_hops() -> Vec<Hop> {
    let canberra = (149.1, -35.3);
    let sydney = (151.1, -33.9);
    let melbourne = (144.9, -37.8);
    let perth = (115.9, -31.9);
    let darwin = (130.8, -12.4);
    let brisbane = (153.0, -27.5);
    let adelaide = (138.6, -34.9);
    // Go traceroute bad.horse some time, it's fun. these locations are made up and don't correspond
    // to the actual IP addresses (which are in Toronto, Canada).
    vec![
        Hop::new("home", "127.0.0.1", canberra),
        Hop::new("bad.horse", "162.252.205.130", sydney),
        Hop::new("bad.horse", "162.252.205.131", melbourne),
        Hop::new("bad.horse", "162.252.205.132", brisbane),
        Hop::new("bad.horse", "162.252.205.133", sydney),
        Hop::new("he.rides.across.the.nation", "162.252.205.134", perth),
        Hop::new("the.thoroughbred.of.sin", "162.252.205.135", darwin),
        Hop::new("he.got.the.application", "162.252.205.136", brisbane),
        Hop::new("that.you.just.sent.in", "162.252.205.137", adelaide),
        Hop::new("it.needs.evaluation", "162.252.205.138", darwin),
        Hop::new("so.let.the.games.begin", "162.252.205.139", perth),
        Hop::new("a.heinous.crime", "162.252.205.140", brisbane),
        Hop::new("a.show.of.force", "162.252.205.141", canberra),
        Hop::new("a.murder.would.be.nice.of.course", "162.252.205.142", perth),
        Hop::new("bad.horse", "162.252.205.143", melbourne),
        Hop::new("bad.horse", "162.252.205.144", darwin),
        Hop::new("bad.horse", "162.252.205.145", melbourne),
        Hop::new("he-s.bad", "162.252.205.146", perth),
        Hop::new("the.evil.league.of.evil", "162.252.205.147", brisbane),
        Hop::new("is.watching.so.beware", "162.252.205.148", darwin),
        Hop::new("the.grade.that.you.receive", "162.252.205.149", perth),
        Hop::new("will.be.your.last.we.swear", "162.252.205.150", adelaide),
        Hop::new("so.make.the.bad.horse.gleeful", "162.252.205.151", sydney),
        Hop::new("or.he-ll.make.you.his.mare", "162.252.205.152", melbourne),
        Hop::new("o_o", "162.252.205.153", brisbane),
        Hop::new("you-re.saddled.up", "162.252.205.154", darwin),
        Hop::new("there-s.no.recourse", "162.252.205.155", perth),
        Hop::new("it-s.hi-ho.silver", "162.252.205.156", sydney),
        Hop::new("signed.bad.horse", "162.252.205.157", canberra),
    ]
}

fn render_hops(hops: &[Hop], selected_row: usize, area: Rect, buf: &mut Buffer) {
    let mut state = TableState::default().with_selected(Some(selected_row));
    let rows = hops
        .iter()
        .map(|hop| Row::new(vec![hop.host, hop.address]))
        .collect_vec();
    let block = Block::default()
        .title("IP Addresses")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    StatefulWidget::render(
        Table::new(rows)
            .header(Row::new(vec!["Host", "Address"]).dark_gray().on_gray())
            .widths(&[Constraint::Max(100), Constraint::Length(15)])
            .highlight_style(Style::new().dark_gray().on_white())
            .block(block),
        area,
        buf,
        &mut state,
    );
}

fn render_map(path: Option<(&Hop, &Hop)>, area: Rect, buf: &mut Buffer) {
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
