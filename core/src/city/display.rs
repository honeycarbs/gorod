use bevy::prelude::*;

use super::resources::{CityPopulation, CityServices};

#[derive(Component)]
pub enum CityStatKind {
    Pop,
    Housing,
    Jobs,
    Happy,
    HousingDemand,
    JobDemand,
    EntertainmentDemand,
}

pub fn setup_city_stats_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

    // Root container: two cards laid out horizontally
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            ..default()
        },))
        .with_children(|parent| {
            // Left card: population and capacities
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|card| {
                    // Title
                    card.spawn((
                        Text::new("Statistics"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Stats as a vertical column
                    for (label, kind) in [
                        ("Population: 0", CityStatKind::Pop),
                        ("Housing: 0", CityStatKind::Housing),
                        ("Jobs: 0", CityStatKind::Jobs),
                        ("Happiness: 1.00", CityStatKind::Happy),
                    ] {
                        card.spawn((
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            kind,
                        ));
                    }
                });

            // Right card: demands
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|card| {
                    // Title
                    card.spawn((
                        Text::new("Demand"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    for (label, kind) in [
                        ("Housing: 0", CityStatKind::HousingDemand),
                        ("Jobs: 0", CityStatKind::JobDemand),
                        ("Entertainment: 0", CityStatKind::EntertainmentDemand),
                    ] {
                        card.spawn((
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            kind,
                        ));
                    }
                });
        });
}

pub fn update_city_stats_display(
    population: Res<CityPopulation>,
    services: Res<CityServices>,
    mut stats_q: Query<(&mut Text, &CityStatKind)>,
) {
    if !population.is_changed() && !services.is_changed() {
        return;
    }

    for (mut text, kind) in stats_q.iter_mut() {
        text.0 = match kind {
            CityStatKind::Pop => format!("Population: {}", population.population),
            CityStatKind::Housing => format!("Housing: {}", services.housing_capacity),
            CityStatKind::Jobs => format!("Jobs: {}", services.job_capacity),
            CityStatKind::Happy => format!("Happiness: {:.2}", population.happiness),
            CityStatKind::HousingDemand => format!("Housing: {}", services.housing_demand),
            CityStatKind::JobDemand => format!("Jobs: {}", services.job_demand),
            CityStatKind::EntertainmentDemand => {
                format!("Entertainment: {}", services.entertainment_demand)
            }
        };
    }
}
