use bevy::prelude::*;

use super::resources::{CurrentTileType, UiClickBlocker};
use crate::budget::BuildingType;
use crate::time::HelpOverlayState;

type TileSelectInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static TileSelectButton),
    (Changed<Interaction>, With<Button>),
>;

#[derive(Component)]
pub struct SelectedTileDisplayText;

#[derive(Component)]
pub struct TileSelectButton {
    building_type: BuildingType,
}

fn button_base_color(building_type: BuildingType) -> Color {
    match building_type {
        BuildingType::Residential => Color::srgba(0.6, 0.8, 1.0, 1.0),
        BuildingType::Commercial => Color::srgba(0.6, 1.0, 0.6, 1.0),
        BuildingType::Industry => Color::srgba(1.0, 0.9, 0.6, 1.0),
        BuildingType::Road => Color::srgba(0.8, 0.8, 0.8, 1.0),
        BuildingType::Decorative => Color::srgba(1.0, 0.6, 0.9, 1.0),
    }
}

pub fn setup_selected_tile_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

    // Full-width top bar with the selected text centered inside it
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Selected: None"),
                TextFont {
                    font,
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                SelectedTileDisplayText,
            ));
        });
}

pub fn setup_tile_select_buttons(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Silkscreen/Silkscreen-Regular.ttf");

    let container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .id();

    let spawn_button = |parent: Entity,
                        building_type: BuildingType,
                        label: &str,
                        font: &Handle<Font>,
                        commands: &mut Commands| {
        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(button_base_color(building_type)),
                    TileSelectButton { building_type },
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new(label),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        });
    };

    spawn_button(
        container,
        BuildingType::Residential,
        "R",
        &font,
        &mut commands,
    );
    spawn_button(
        container,
        BuildingType::Commercial,
        "C",
        &font,
        &mut commands,
    );
    spawn_button(container, BuildingType::Industry, "I", &font, &mut commands);
    spawn_button(container, BuildingType::Road, "O", &font, &mut commands);
    spawn_button(container, BuildingType::Decorative, "B", &font, &mut commands);
}

pub fn update_selected_tile_display(
    current_tile_type: Res<CurrentTileType>,
    mut query: Query<&mut Text, With<SelectedTileDisplayText>>,
) {
    if !current_tile_type.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    let label = match BuildingType::from_selection_index(current_tile_type.texture_index) {
        Some(BuildingType::Residential) => "Residential (R)",
        Some(BuildingType::Commercial) => "Commercial (C)",
        Some(BuildingType::Industry) => "Industry (I)",
        Some(BuildingType::Road) => "Road (O)",
        Some(BuildingType::Decorative) => "Decorative (B)",
        None => "None",
    };

    text.0 = format!("Selected: {}", label);
}

pub fn handle_tile_select_button_presses(
    mut interaction_q: TileSelectInteractionQuery<'_, '_>,
    mut current_tile_type: ResMut<CurrentTileType>,
    mut ui_click_blocker: ResMut<UiClickBlocker>,
    help_state: Option<Res<HelpOverlayState>>,
) {
    if let Some(state) = help_state {
        if state.active {
            return;
        }
    }
    for (interaction, button) in interaction_q.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                ui_click_blocker.just_clicked_ui = true;
                current_tile_type.texture_index = match button.building_type {
                    BuildingType::Residential => 2,
                    BuildingType::Commercial => 3,
                    BuildingType::Industry => 4,
                    BuildingType::Road => 5,
                    BuildingType::Decorative => 6,
                };
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub fn update_tile_select_button_colors(
    current_tile_type: Res<CurrentTileType>,
    mut query: Query<(&TileSelectButton, &mut BackgroundColor)>,
) {
    if !current_tile_type.is_changed() {
        return;
    }

    let active_type = BuildingType::from_selection_index(current_tile_type.texture_index);

    for (button, mut bg) in query.iter_mut() {
        if Some(button.building_type) == active_type {
            // Highlight currently selected tile type
            bg.0 = Color::srgba(1.0, 1.0, 1.0, 1.0);
        } else {
            // Reset to base color for other buttons
            bg.0 = button_base_color(button.building_type);
        }
    }
}
