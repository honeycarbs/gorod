use bevy::prelude::*;

#[derive(Component)]
pub struct CameraController {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 400.0,
            zoom_speed: 1.0,
            min_zoom: 0.1,
            max_zoom: 5.0,
        }
    }
}

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_movement, camera_zoom));
    }
}

fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Transform, &CameraController), With<Camera>>,
) {
    for (mut transform, controller) in camera_q.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * controller.move_speed * time.delta_secs();
        }
    }
}

/// System to handle camera zoom with +/- keys
fn camera_zoom(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Projection, With<CameraController>>,
) {
    for mut projection in camera_q.iter_mut() {
        let mut zoom_delta = 0.0;

        // Plus/Equals key to zoom in
        if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
            zoom_delta -= 1.0;
        }
        // Minus key to zoom out
        if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
            zoom_delta += 1.0;
        }

        if zoom_delta != 0.0 {
            // Match on the projection enum to access OrthographicProjection
            if let Projection::Orthographic(ortho) = projection.as_mut() {
                let zoom_change = zoom_delta * time.delta_secs();
                ortho.scale = (ortho.scale + zoom_change).clamp(0.1, 5.0);
            }
        }
    }
}

/// System to reset camera to default position and zoom
pub fn reset_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<CameraController>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (mut transform, mut projection) in camera_q.iter_mut() {
            transform.translation = Vec3::ZERO;

            // Reset projection scale
            if let Projection::Orthographic(ortho) = projection.as_mut() {
                ortho.scale = 1.0;
            }

            info!("Camera reset to origin");
        }
    }
}