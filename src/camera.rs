use bevy::prelude::*;
use crate::components::{Player, SystemToggles};

pub fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.camera_follow { return; }
    if let (Ok(player_transform), Ok(mut camera_transform)) = (player_query.single(), camera_query.single_mut()) {
        // Smoothly follow the player with some offset for better view
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
        // Keep camera Z position fixed for 2D view
    }
}

pub fn camera_zoom(
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.camera_zoom { return; }
    for event in scroll_events.read() {
        if let Ok(mut projection) = camera_query.single_mut() {
            if let Projection::Orthographic(ref mut ortho) = projection.as_mut() {
                // Zoom in/out based on scroll direction
                let zoom_factor = 1.1;
                match event.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => {
                        if event.y > 0.0 {
                            // Scroll up - zoom in
                            ortho.scale /= zoom_factor;
                        } else {
                            // Scroll down - zoom out
                            ortho.scale *= zoom_factor;
                        }
                    }
                    bevy::input::mouse::MouseScrollUnit::Pixel => {
                        let zoom_change = -event.y * 0.001;
                        ortho.scale *= (1.0 + zoom_change).max(0.1);
                    }
                }
                
                // Clamp zoom to reasonable limits
                ortho.scale = ortho.scale.clamp(0.1, 10.0);
            }
        }
    }
}
