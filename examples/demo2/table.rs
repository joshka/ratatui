use ratatui::{prelude::*, widgets::*};

use crate::main_view::{layout, render_title};

pub fn render(selected_row: usize, area: Rect, buf: &mut Buffer) {
    let layout = layout(area, Direction::Vertical, vec![1, 0]);
    render_title("Soup Ingredients", layout[0], buf);
    let mut state = TableState::default().with_selected(Some(selected_row));

    // https://www.realsimple.com/food-recipes/browse-all-recipes/ratatouille
    StatefulWidget::render(
        Table::new(vec![
            Row::new(vec!["01", "4 tbsp", "olive oil", ""]),
            Row::new(vec!["02", "1", "onion", "thinly sliced"]),
            Row::new(vec!["03", "4", "cloves garlic", "peeled and sliced"]),
            Row::new(vec!["04", "1", "small bay leaf", ""]),
            Row::new(vec!["05", "1", "small eggplant", "cut into 1/2 inch cubes"]),
            Row::new(vec!["06", "1", "small zucchini", "halved and sliced"]),
            Row::new(vec!["07", "1", "red bell pepper", "cut into slivers"]),
            Row::new(vec!["08", "4", "plum tomatoes", "coarsely chopped"]),
            Row::new(vec!["09", "1 tsp", "kosher salt", ""]),
            Row::new(vec!["10", "1/4 cup", "shredded fresh basil leaves", ""]),
            Row::new(vec!["11", "", "freshly ground black pepper", ""]),
        ])
        .header(
            Row::new(vec!["Item", "Qty", "Ingredient", "Notes"])
                .style(Style::new().black().on_light_blue()),
        )
        .widths(&[
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Length(30),
            Constraint::Length(450),
        ])
        .highlight_style(Style::new().black().on_light_yellow()),
        layout[1],
        buf,
        &mut state,
    );
}
