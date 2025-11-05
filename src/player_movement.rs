use bevy::prelude::*;
use crate::components::{Battery, Player, SystemToggles, PlayerSprite, PlayerDirection, GridMovement, IsometricGrid};

pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    grid: Res<IsometricGrid>,
    mut player_query: Query<(&Transform, &mut GridMovement, &mut Battery), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_movement { return; }
    
    for (transform, mut grid_movement, mut battery) in player_query.iter_mut() {
        // Check if we're currently moving to a target
        if grid_movement.is_moving {
            continue; // Let the grid_movement_system handle it
        }
        
        // Check for new movement input (only one key press at a time for grid movement)
        // Isometric movement: keys move along diagonal axes
        // W: up-right (NE), S: down-left (SW), A: up-left (NW), D: down-right (SE)
        let mut target_offset = None;
        
        if keys.just_pressed(KeyCode::KeyW) {
            // Move up-right in isometric space (NE)
            target_offset = Some(Vec2::new(grid.tile_size / 2.0, grid.tile_size / 4.0));
        } else if keys.just_pressed(KeyCode::KeyS) {
            // Move down-left in isometric space (SW)
            target_offset = Some(Vec2::new(-grid.tile_size / 2.0, -grid.tile_size / 4.0));
        } else if keys.just_pressed(KeyCode::KeyA) {
            // Move up-left in isometric space (NW)
            target_offset = Some(Vec2::new(-grid.tile_size / 2.0, grid.tile_size / 4.0));
        } else if keys.just_pressed(KeyCode::KeyD) {
            // Move down-right in isometric space (SE)
            target_offset = Some(Vec2::new(grid.tile_size / 2.0, -grid.tile_size / 4.0));
        }
        
        // Start moving to target if input detected and battery has charge
        if let Some(offset) = target_offset {
            if battery.current_charge > 0.0 {
                let current_pos = transform.translation.truncate();
                grid_movement.target_position = Some(current_pos + offset);
                grid_movement.is_moving = true;
                
                // Drain some battery for initiating movement
                let drain_per_move = 2.0;
                battery.current_charge = (battery.current_charge - drain_per_move).max(0.0);
            }
        }
    }
}

pub fn grid_movement_system(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut GridMovement), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_movement { return; }
    
    for (mut transform, mut grid_movement) in player_query.iter_mut() {
        if let Some(target) = grid_movement.target_position {
            let current_pos = transform.translation.truncate();
            let direction = target - current_pos;
            let distance = direction.length();
            
            // Check if we've reached the target
            if distance < 2.0 {
                // Snap to exact position
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                grid_movement.target_position = None;
                grid_movement.is_moving = false;
            } else {
                // Move towards target using direct transform manipulation
                let direction_normalized = direction.normalize();
                let move_delta = direction_normalized * grid_movement.move_speed * time.delta_secs();
                
                // Make sure we don't overshoot
                if move_delta.length() >= distance {
                    transform.translation.x = target.x;
                    transform.translation.y = target.y;
                    grid_movement.target_position = None;
                    grid_movement.is_moving = false;
                } else {
                    transform.translation.x += move_delta.x;
                    transform.translation.y += move_delta.y;
                }
            }
        } else if grid_movement.is_moving {
            grid_movement.is_moving = false;
        }
    }
}

pub fn update_player_sprite_direction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut PlayerSprite, &mut Sprite, &GridMovement), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_rotation { return; }
    
    for (mut player_sprite, mut sprite, _grid_movement) in player_query.iter_mut() {
        // Check for key just pressed to update direction immediately
        let mut new_direction = None;
        
        if keyboard_input.just_pressed(KeyCode::KeyW) {
            new_direction = Some(PlayerDirection::Up);
        } else if keyboard_input.just_pressed(KeyCode::KeyS) {
            new_direction = Some(PlayerDirection::Down);
        } else if keyboard_input.just_pressed(KeyCode::KeyA) {
            new_direction = Some(PlayerDirection::Left);
        } else if keyboard_input.just_pressed(KeyCode::KeyD) {
            new_direction = Some(PlayerDirection::Right);
        }
        
        // Update sprite direction if changed
        if let Some(dir) = new_direction {
            if player_sprite.current_direction != dir {
                player_sprite.current_direction = dir;
                
                // Update sprite atlas index based on direction
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = match dir {
                        PlayerDirection::Up => 3,    // Swapped with Right
                        PlayerDirection::Down => 1,
                        PlayerDirection::Left => 2,
                        PlayerDirection::Right => 0, // Swapped with Up
                    };
                }
            }
        }
    }
}
