use bevy::prelude::*;

use super::resources::{CityPopulation, CityServices};

#[derive(Component)]
pub struct CityStatsDisplayText;

pub fn setup_city_stats_display(mut commands: Commands) {
    commands.spawn((
        Text::new("Pop: 0 | Housing: 0 | Jobs: 0 | Happy: 1.00\nH_Dem: 0 | J_Dem: 0 | E_Dem: 0"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        CityStatsDisplayText,
    ));
}

pub fn update_city_stats_display(
    population: Res<CityPopulation>,
    services: Res<CityServices>,
    mut query: Query<&mut Text, With<CityStatsDisplayText>>,
) {
    if !population.is_changed() && !services.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    text.0 = format!(
        "Pop: {} | Housing: {} | Jobs: {} | Happy: {:.2}\nH_Dem: {} | J_Dem: {} | E_Dem: {}",
        population.population,
        services.housing_capacity,
        services.job_capacity,
        population.happiness,
        services.housing_demand,
        services.job_demand,
        services.entertainment_demand,
    );
}
