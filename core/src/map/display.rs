use bevy::prelude::*;

use super::resources::CurrentTileType;
use crate::budget::BuildingType;

#[derive(Component)]
pub struct SelectedTileDisplayText;

pub fn setup_selected_tile_display(mut commands: Commands) {
    commands.spawn((
        Text::new("Selected: Road (O)"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        SelectedTileDisplayText,
    ));
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

    let label = match BuildingType::from_texture_index(current_tile_type.texture_index) {
        Some(BuildingType::Residential) => "Residential (R)",
        Some(BuildingType::Commercial) => "Commercial (C)",
        Some(BuildingType::Industry) => "Industry (I)",
        Some(BuildingType::Road) => "Road (O)",
        None => "None",
    };

    text.0 = format!("Selected: {}", label);
}


