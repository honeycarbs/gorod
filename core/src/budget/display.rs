use super::resources::Budget;
use bevy::prelude::*;

#[derive(Component)]
pub struct BudgetDisplayText;

pub fn setup_budget_display(mut commands: Commands) {
    commands.spawn((
        Text::new("Budget: $50,000"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        BudgetDisplayText,
    ));
}

pub fn update_budget_display(
    budget: Res<Budget>,
    mut query: Query<&mut Text, With<BudgetDisplayText>>,
) {
    if !budget.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    text.0 = format!("Budget: ${}", format_money(budget.money));
}

fn format_money(amount: i64) -> String {
    let abs_amount = amount.abs();
    let formatted = if abs_amount >= 1_000_000 {
        format!("{:.1}M", amount as f64 / 1_000_000.0)
    } else if abs_amount >= 1_000 {
        format!("{:.1}K", amount as f64 / 1_000.0)
    } else {
        format!("{}", amount)
    };

    if amount < 0 {
        format!("-{}", formatted)
    } else {
        formatted
    }
}
