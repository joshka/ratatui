use crate::main_view::layout;
use ratatui::{prelude::*, widgets::*};

pub fn render(selected_row: usize, area: Rect, buf: &mut Buffer) {
    let area = layout(area, Direction::Vertical, vec![8, 0]);

    let lines: Vec<Line> = vec![
        Line::from(vec![
            "Step 1: ".light_green().bold().italic(),
            "Over medium-low heat, add the oil to a large skillet with the onion, garlic, and bay \
            leaf, stirring occasionally, until the onion has softened."
                .into(),
        ]),
        Line::from(vec![
            "Step 2: ".light_green().bold().italic(),
            "Add the eggplant and cook, stirring occasionally, for 8 minutes or until the \
            eggplant has softened. Stir in the zucchini, red bell pepper, tomatoes, and salt, and \
            cook over medium heat, stirring occasionally, for 5 to 7 minutes or until the \
            vegetables are tender. Stir in the basil and few grinds of pepper to taste."
                .into(),
        ]),
        Line::from(vec!["Ingredients:".light_green().bold().italic()]),
    ];
    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .render(area[0], buf);

    let mut state = TableState::default().with_selected(Some(selected_row));
    // https://www.realsimple.com/food-recipes/browse-all-recipes/ratatouille
    StatefulWidget::render(
        Table::new(vec![
            Row::new(vec!["4 tbsp", "olive oil", ""]),
            Row::new(vec!["1", "onion", "thinly sliced"]),
            Row::new(vec!["4", "cloves garlic", "peeled and sliced"]),
            Row::new(vec!["1", "small bay leaf", ""]),
            Row::new(vec!["1", "small eggplant", "cut into 1/2 inch cubes"]),
            Row::new(vec![
                "1".into(),
                "small zucchini".into(),
                Text::raw("halved lengthwise and cut into\nthin slices"),
            ])
            .height(2),
            Row::new(vec!["1", "red bell pepper", "cut into slivers"]),
            Row::new(vec!["4", "plum tomatoes", "coarsely chopped"]),
            Row::new(vec!["1 tsp", "kosher salt", ""]),
            Row::new(vec!["1/4 cup", "shredded fresh basil leaves", ""]),
            Row::new(vec!["", "freshly ground black pepper", ""]),
        ])
        .header(
            Row::new(vec!["Qty", "Ingredient", "Notes"]).style(Style::new().white().underlined()),
        )
        .widths(&[
            Constraint::Length(7),
            Constraint::Length(30),
            Constraint::Length(450),
        ])
        .highlight_style(Style::new().light_yellow()),
        area[1],
        buf,
        &mut state,
    );
}
