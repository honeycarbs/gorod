use super::resources::{GameClock, GameTime, HelpOverlayState, TimeSpeed};
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeDisplayText;

#[derive(Component)]
pub struct HelpButton;

#[derive(Component)]
pub struct HelpOverlay;

#[derive(Component)]
pub struct HelpCloseButton;

pub fn setup_time_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut help_state: ResMut<HelpOverlayState>,
) {
    let font: Handle<Font> =
        asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

    // Container for time text and help button in the top-right
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Day 1, 00:00:00 [NORMAL]"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimeDisplayText,
            ));

            // Small "?" button that reopens the help overlay
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    HelpButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("?"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        });

    // Central help overlay, initially visible on first load
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Visible,
            HelpOverlay,
        ))
        .with_children(|parent| {
            parent
                .spawn((
        Node {
            width: Val::Px(520.0),
            margin: UiRect::all(Val::Px(16.0)),
            padding: UiRect::all(Val::Px(24.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("WELCOME TO GOROD"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    card.spawn((
                        Text::new(
                            "This is a little game inspired by all the cool city builders out there\n\
\n\
Your goal is to make people happy and attract them to the city by\n\
providing housing, jobs and entertainment with commercial and\n\
industrial buildings.\n\
\n\
Happiness is a scale from 0 to 1.\n\
\n\
You start in the middle of the map and only tiles near the center\n\
are placeable at the beginning.\n\
\n\
Press R/C/I/O or use the buttons to select Residential, Commercial,\n\
Industry or Roads.\n\
Use ',' and '.' to change building or road variants.\n\
\n\
Space pauses time, 1/2/3 change game speed.",
                        ),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    card
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                            HelpCloseButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("got it, let me play"),
                                TextFont {
                                    font,
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::BLACK),
                            ));
                        });
                });
        });
    // Start with help active; keep the game paused until the player changes speed
    help_state.active = true;
    help_state.previous_speed = None;
}

pub fn update_time_display(
    clock: Res<GameClock>,
    time: Res<GameTime>,
    mut query: Query<&mut Text, With<TimeDisplayText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
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

pub fn handle_help_ui(
    mut overlay_q: Query<&mut Visibility, With<HelpOverlay>>,
    mut button_q: Query<
        (&Interaction, Option<&HelpButton>, Option<&HelpCloseButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_time: ResMut<GameTime>,
    mut help_state: ResMut<HelpOverlayState>,
    mut current_tile_type: Option<ResMut<crate::map::CurrentTileType>>,
    mut ui_click_blocker: Option<ResMut<crate::map::UiClickBlocker>>,
) {
    let Ok(mut overlay_vis) = overlay_q.single_mut() else {
        return;
    };

    for (interaction, is_help_button, is_close_button) in button_q.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if is_help_button.is_some() {
            if let Some(ref mut blocker) = ui_click_blocker {
                blocker.just_clicked_ui = true;
            }
            if !help_state.active {
                help_state.previous_speed = Some(game_time.speed);
                help_state.active = true;
            }
            game_time.speed = TimeSpeed::Paused;
            *overlay_vis = Visibility::Visible;

            if let Some(mut current) = current_tile_type.take() {
                current.texture_index = 0;
            }
        } else if is_close_button.is_some() {
            if let Some(ref mut blocker) = ui_click_blocker {
                blocker.just_clicked_ui = true;
            }

            help_state.active = false;
            if let Some(prev) = help_state.previous_speed.take() {
                game_time.speed = prev;
            }
            *overlay_vis = Visibility::Hidden;
        }
    }
}
