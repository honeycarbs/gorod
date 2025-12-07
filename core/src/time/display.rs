use super::resources::{GameClock, GameTime, TimeSpeed};
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeDisplayText;

pub fn setup_time_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> =
        asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

    commands.spawn((
        Text::new("Day 1, 00:00:00 [NORMAL]"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        TextFont {
            font,
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent black
        TimeDisplayText,
    ));
}

pub fn update_time_display(
    clock: Res<GameClock>,
    time: Res<GameTime>,
    mut query: Query<&mut Text, With<TimeDisplayText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return; // Exit if no entity or multiple entities
    };

    let speed_str = match time.speed {
        TimeSpeed::Paused => "PAUSED",
        TimeSpeed::Normal => "1x",
        TimeSpeed::Fast => "2x",
        TimeSpeed::UltraFast => "3x",
    };

    text.0 = format!(
        "Day {}, {:02}:{:02}:{:02} [{}]",
        clock.day, clock.hour, clock.minute, clock.second, speed_str
    );
}
