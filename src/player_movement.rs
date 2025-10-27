use avian2d::prelude::*;
use bevy::prelude::*;
use crate::components::{Player, SystemToggles};

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_movement { return; }
    if let Ok(mut velocity) = player_query.single_mut() {
        let mut force = Vec2::ZERO;
        let move_speed = 300.0;

        // Map keys to isometric movement
        // A/D moves along X axis
        // W/S moves along Z axis (depth)
        if keyboard_input.pressed(KeyCode::KeyA) {
            force.x -= move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            force.x += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            // Move "forward" in isometric view (negative Z)
            // Since we're using 2D physics, we map this to Y
            force.y += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            // Move "backward" in isometric view (positive Z)
            force.y -= move_speed;
        }

        // Apply force as velocity (since we have no gravity, this works well)
        velocity.0 = force;
    }
}

pub fn rotate_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &LinearVelocity), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_rotation { return; }
    for (mut transform, velocity) in player_query.iter_mut() {
        let vel = velocity.0;
        
        // Only rotate if the player is moving with significant velocity
        if vel.length() > 10.0 {
            // Calculate player input direction
            let mut input_direction = Vec2::ZERO;
            if keyboard_input.pressed(KeyCode::KeyA) {
                input_direction.x -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                input_direction.x += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyW) {
                input_direction.y += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                input_direction.y -= 1.0;
            }
            
            let is_player_controlled = input_direction.length() > 0.1;
            
            let target_angle = if is_player_controlled {
                // Check if input direction roughly matches velocity direction
                let input_normalized = input_direction.normalize();
                let velocity_normalized = vel.normalize();
                let dot_product = input_normalized.dot(velocity_normalized);
                
                if dot_product > 0.3 { // Player is moving in the direction they're trying to go
                    // Face the direction of movement when successfully controlling motion
                    vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2
                } else {
                    // Player is fighting against physics (like being pulled by cord while trying to move away)
                    // Face the direction they're trying to go (input direction)
                    input_direction.y.atan2(input_direction.x) - std::f32::consts::FRAC_PI_2
                }
            } else {
                // Face opposite to movement direction when being pulled by cord/physics
                // Add π to reverse the direction
                vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2 + std::f32::consts::PI
            };
            
            // Get current rotation angle
            let current_rotation = transform.rotation;
            let current_angle = current_rotation.to_euler(bevy::math::EulerRot::ZYX).0;
            
            // Calculate the shortest angular distance
            let mut angle_diff = target_angle - current_angle;
            // Wrap to [-π, π] range for shortest rotation
            while angle_diff > std::f32::consts::PI {
                angle_diff -= 2.0 * std::f32::consts::PI;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += 2.0 * std::f32::consts::PI;
            }
            
            // Rotation speed (radians per second)
            let rotation_speed = if is_player_controlled { 4.0 } else { 2.0 }; // Slower when transitioning to physics motion
            let max_rotation_this_frame = rotation_speed * time.delta_secs();
            
            // Clamp the rotation change to the maximum allowed this frame
            let rotation_change = angle_diff.clamp(-max_rotation_this_frame, max_rotation_this_frame);
            let new_angle = current_angle + rotation_change;
            
            // Apply the smoothed rotation
            transform.rotation = Quat::from_rotation_z(new_angle);
        }
    }
}
