use super::resources::{GameClock, GameTime, HelpOverlayState, HelpPage, TimeSpeed};
use bevy::prelude::*;

#[derive(Component)]
pub struct TimeDisplayText;

#[derive(Component)]
pub struct HelpButton;

#[derive(Component)]
pub struct HelpOverlay;

#[derive(Component)]
pub struct HelpCloseButton;

#[derive(Component)]
pub struct HelpNextButton;

#[derive(Component)]
pub struct HelpPageOne;

#[derive(Component)]
pub struct HelpPageTwo;

pub fn setup_time_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut help_state: ResMut<HelpOverlayState>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

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
                ..default()
            },
            Visibility::Visible,
            HelpOverlay,
        ))
        .with_children(|parent| {
            // First page wrapper: full-screen, centers its card like a slide
            parent
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
                    HelpPageOne,
                    Visibility::Visible,
                ))
                .with_children(|page| {
                    page.spawn((
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
                                "This is a little game inspired by all the cool city builders out there.\n\
\n\
Your goal is to make people happy and attract them to the city by\n\
providing housing, jobs and entertainment with commercial and\n\
industrial buildings.\n\
\n\
Happiness is a scale from 0 to 1. Buildings contribute to stats if they are\n\
connected by a road.\n\
\n\
Your town will grow if people are happy.",
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
                                HelpNextButton,
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("oh ok"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    });
                });

            // Second page wrapper: full-screen, centers its card like a slide
            parent
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
                    HelpPageTwo,
                    Visibility::Hidden,
                ))
                .with_children(|page| {
                    page.spawn((
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
                            Text::new(
                                "You start in the middle of the map and only tiles near the center are\n\
placeable at the beginning.\n\
\n\
Press R/C/I/O or use the buttons to select Residential, Commercial,\n\
Industry or Roads.\n\
\n\
Use ',' and '.' to change building or road variants.\n\
\n\
Use Click+Shift to demolish the building.\n\
\n\
W,A,S,D to move, +- to zoom.\n\
\n\
Space pauses time, 1/2/3 change game speed.\n\
\n\
If you want to see this window again, press the \"?\" button.",
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
        });
    help_state.active = true;
    help_state.previous_speed = None;
    help_state.page = HelpPage::First;
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
    mut vis_q: Query<
        (
            &mut Visibility,
            Option<&HelpOverlay>,
            Option<&HelpPageOne>,
            Option<&HelpPageTwo>,
        ),
    >,
    mut button_q: Query<
        (
            &Interaction,
            Option<&HelpButton>,
            Option<&HelpNextButton>,
            Option<&HelpCloseButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_time: ResMut<GameTime>,
    mut help_state: ResMut<HelpOverlayState>,
    mut current_tile_type: Option<ResMut<crate::map::CurrentTileType>>,
    mut ui_click_blocker: Option<ResMut<crate::map::UiClickBlocker>>,
) {
    let mut show_first_page = false;
    let mut show_second_page = false;
    let mut hide_overlay = false;

    for (interaction, is_help_button, is_next_button, is_close_button) in button_q.iter_mut() {
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
            help_state.page = HelpPage::First;
            game_time.speed = TimeSpeed::Paused;
            show_first_page = true;

            if let Some(mut current) = current_tile_type.take() {
                current.texture_index = 0;
            }
        } else if is_next_button.is_some() {
            if let Some(ref mut blocker) = ui_click_blocker {
                blocker.just_clicked_ui = true;
            }

            help_state.page = HelpPage::Second;
            show_second_page = true;
        } else if is_close_button.is_some() {
            if let Some(ref mut blocker) = ui_click_blocker {
                blocker.just_clicked_ui = true;
            }

            help_state.active = false;
            if let Some(prev) = help_state.previous_speed.take() {
                game_time.speed = prev;
            }
            hide_overlay = true;
        }
    }

    // Apply visibility changes based on which button was pressed this frame.
    if show_first_page {
        for (mut vis, overlay, page_one, page_two) in vis_q.iter_mut() {
            if overlay.is_some() {
                *vis = Visibility::Visible;
            } else if page_one.is_some() {
                *vis = Visibility::Visible;
            } else if page_two.is_some() {
                *vis = Visibility::Hidden;
            }
        }
    }

    if show_second_page {
        for (mut vis, _overlay, page_one, page_two) in vis_q.iter_mut() {
            if page_one.is_some() {
                *vis = Visibility::Hidden;
            } else if page_two.is_some() {
                *vis = Visibility::Visible;
            }
        }
    }

    if hide_overlay {
        for (mut vis, overlay, page_one, page_two) in vis_q.iter_mut() {
            if overlay.is_some() || page_one.is_some() || page_two.is_some() {
                *vis = Visibility::Hidden;
            }
        }
    }
}
