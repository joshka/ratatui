use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use super::Tab;
use crate::{colors, styles, tui};

struct Ingredient {
    quantity: &'static str,
    name: &'static str,
    notes: &'static str,
}

const INGREDIENTS: &[Ingredient] = &[
    Ingredient {
        quantity: "4 tbsp",
        name: "olive oil",
        notes: "",
    },
    Ingredient {
        quantity: "1",
        name: "onion",
        notes: "thinly sliced",
    },
    Ingredient {
        quantity: "4",
        name: "cloves garlic",
        notes: "peeled and sliced",
    },
    Ingredient {
        quantity: "1",
        name: "small bay leaf",
        notes: "",
    },
    Ingredient {
        quantity: "1",
        name: "small eggplant",
        notes: "cut into 1/2 inch cubes",
    },
    Ingredient {
        quantity: "1",
        name: "small zucchini",
        notes: "halved lengthwise and cut into thin slices",
    },
    Ingredient {
        quantity: "1",
        name: "red bell pepper",
        notes: "cut into slivers",
    },
    Ingredient {
        quantity: "4",
        name: "plum tomatoes",
        notes: "coarsely chopped",
    },
    Ingredient {
        quantity: "1 tsp",
        name: "kosher salt",
        notes: "",
    },
    Ingredient {
        quantity: "1/4 cup",
        name: "shredded fresh basil leaves",
        notes: "",
    },
    Ingredient {
        quantity: "",
        name: "freshly ground black pepper",
        notes: "",
    },
];

#[derive(Debug)]
pub struct RecipeTab {
    selected_row: usize,
}

impl RecipeTab {
    pub fn new(selected_row: usize) -> Self {
        Self {
            selected_row: selected_row % INGREDIENTS.len(),
        }
    }
}

impl Tab for RecipeTab {
    fn title(&self) -> String {
        "Recipe".to_string()
    }

    fn select(&mut self, row: usize) {
        self.selected_row = row;
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        colors::render_rgb_colors(area, buf);
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });
        Clear.render(area, buf);
        Block::new()
            .style(styles::APP)
            .borders(Borders::ALL)
            .border_line_set(ratatui::symbols::line::QUADRANT_OUTSIDE)
            .border_style(
                Style::new()
                    .bg(styles::APP.bg.unwrap())
                    .fg(Color::Indexed(250)),
            )
            .render(area, buf);

        let area = area.inner(&Margin {
            horizontal: 2,
            vertical: 1,
        });
        let area = tui::layout(area, Direction::Vertical, vec![8, 0]);

        let lines: Vec<Line> = vec![
            Line::from(vec![
            "Step 1: ".white().bold(),
            "Over medium-low heat, add the oil to a large skillet with the onion, garlic, and bay \
            leaf, stirring occasionally, until the onion has softened."
                .into(),
        ]),
            Line::from(vec![
                "Step 2: ".white().bold(),
                "Add the eggplant and cook, stirring occasionally, for 8 minutes or until the \
            eggplant has softened. Stir in the zucchini, red bell pepper, tomatoes, and salt, and \
            cook over medium heat, stirring occasionally, for 5 to 7 minutes or until the \
            vegetables are tender. Stir in the basil and few grinds of pepper to taste."
                    .into(),
            ]),
            Line::from(vec!["Ingredients:".white().bold()]),
        ];
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(area[0], buf);

        let mut state = TableState::default().with_selected(Some(self.selected_row));
        // https://www.realsimple.com/food-recipes/browse-all-recipes/ratatouille

        let rows = INGREDIENTS
            .iter()
            .map(|i| Row::new(vec![i.quantity, i.name, i.notes]))
            .collect_vec();
        StatefulWidget::render(
            Table::new(rows)
                .header(
                    Row::new(vec!["Qty", "Ingredient", "Notes"])
                        .style(Style::new().white().underlined()),
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
}
