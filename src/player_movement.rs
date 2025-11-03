use avian2d::prelude::*;
use bevy::prelude::*;
use crate::components::{Battery, Player, SystemToggles, PlayerSprite, PlayerDirection};

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut LinearVelocity, &mut Battery), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_movement { return; }
    if let Ok((mut velocity, mut battery)) = player_query.single_mut() {
        let mut force = Vec2::ZERO;
        let move_speed = 300.0;
        let mut is_moving = false;

        // Map keys to isometric movement
        // A/D moves along X axis
        // W/S moves along Z axis (depth)
        if keyboard_input.pressed(KeyCode::KeyA) {
            force.x -= move_speed;
            is_moving = true;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            force.x += move_speed;
            is_moving = true;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            // Move "forward" in isometric view (negative Z)
            // Since we're using 2D physics, we map this to Y
            force.y += move_speed;
            is_moving = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            // Move "backward" in isometric view (positive Z)
            force.y -= move_speed;
            is_moving = true;
        }

        // Drain battery when moving
        if is_moving && battery.current_charge > 0.0 {
            let drain_rate = 10.0; // Charge per second
            battery.current_charge = (battery.current_charge - drain_rate * time.delta_secs()).max(0.0);
        }

        // Only allow movement if battery has charge
        if battery.current_charge > 0.0 {
            velocity.0 = force;
        } else {
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn update_player_sprite_direction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut PlayerSprite, &mut Sprite), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_rotation { return; }
    
    for (mut player_sprite, mut sprite) in player_query.iter_mut() {
        // Calculate input direction
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
        
        // Only update direction if there's input
        if input_direction.length() > 0.1 {
            // Determine primary direction
            let new_direction = if input_direction.x.abs() > input_direction.y.abs() {
                if input_direction.x > 0.0 {
                    PlayerDirection::Right
                } else {
                    PlayerDirection::Left
                }
            } else {
                if input_direction.y > 0.0 {
                    PlayerDirection::Up
                } else {
                    PlayerDirection::Down
                }
            };
            
            // Only update if direction changed
            if player_sprite.current_direction != new_direction {
                player_sprite.current_direction = new_direction;
                
                // Update sprite atlas index based on direction
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = match new_direction {
                        PlayerDirection::Up => 0,
                        PlayerDirection::Down => 1,
                        PlayerDirection::Left => 2,
                        PlayerDirection::Right => 3,
                    };
                }
            }
        }
    }
}
