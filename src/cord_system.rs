use bevy::prelude::*;

use crate::components::{Player, CordSystem, CordMaterial, IsometricGrid};
use crate::setup::Z_CORD_LAYER;

// Component to mark visual cord mesh segments
#[derive(Component)]
pub struct CordMeshSegment;

// Render cord trail path on ground
pub fn render_cord_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    cord_system: Option<ResMut<CordSystem>>,
    cord_material: Res<CordMaterial>,
    player_query: Query<&Transform, With<Player>>,
    mesh_query: Query<Entity, With<CordMeshSegment>>,
) {
    let Some(mut cord_system) = cord_system else { return; };
    // Remove old mesh entities
    for entity in cord_system.visual_meshes.drain(..) {
        if let Ok(entity) = mesh_query.get(entity) {
            commands.entity(entity).despawn();
        }
    }
    
    // Only render cord if attached to a pole
    if cord_system.attached_pole.is_some() {
        if let Ok(player_transform) = player_query.single() {
            let player_pos = player_transform.translation.truncate();
            
            // Build the complete path from attachment through trail to player
            let mut full_path = cord_system.trail_path.clone();
            
            // If trail is empty, still need to render direct line from attachment to player
            // The trail should at least have the attachment point
            if full_path.is_empty() {
                // This shouldn't happen if update_cord_trail runs first, but handle it anyway
                // Just render nothing for now - next frame will have the attachment point
                return;
            }
            
            full_path.push(player_pos);
            
            // Render cord segments between consecutive points in the trail
            let cord_width = 8.0;
            for i in 0..full_path.len() - 1 {
                let start = full_path[i];
                let end = full_path[i + 1];
                
                // Calculate midpoint, length, and rotation
                let midpoint = (start + end) / 2.0;
                let diff = end - start;
                let length = diff.length();
                let angle = diff.y.atan2(diff.x);
                
                // Create cord segment above tilemap but below player
                let mesh_entity = commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(length, cord_width))),
                    MeshMaterial2d(cord_material.material.clone()),
                    Transform::from_translation(midpoint.extend(Z_CORD_LAYER))
                        .with_rotation(Quat::from_rotation_z(angle)),
                    CordMeshSegment,
                )).id();
                cord_system.visual_meshes.push(mesh_entity);

                // If this is a corner (direction change at the end of this segment), draw a square cap
                if i + 1 < full_path.len() - 1 {
                    let next = full_path[i + 2];
                    let dir1 = (end - start).normalize_or_zero();
                    let dir2 = (next - end).normalize_or_zero();
                    // If not (nearly) colinear, place a square cap
                    let dot = dir1.dot(dir2);
                    if dot < 0.99 { // direction changed
                        let cap_entity = commands.spawn((
                            Mesh2d(meshes.add(Rectangle::new(cord_width, cord_width))),
                            MeshMaterial2d(cord_material.material.clone()),
                            Transform::from_translation(end.extend(Z_CORD_LAYER)),
                            CordMeshSegment,
                        )).id();
                        cord_system.visual_meshes.push(cap_entity);
                    }
                }
            }
        }
    }
}


// Cord retraction is now just visual - no physics to manage
pub fn handle_cord_retraction(
    _keyboard_input: Res<ButtonInput<KeyCode>>,
    _cord_system: Option<ResMut<CordSystem>>,
) {
    // Cord is purely visual now, no retraction needed
}

pub fn handle_cord_attachment(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cord_system: Option<ResMut<CordSystem>>,
    player_query: Query<&Transform, With<Player>>,
    pole_query: Query<(Entity, &Transform), With<crate::components::Pole>>,
) {
    let Some(mut cord_system) = cord_system else { return; };
    // Check for spacebar press
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.single() {
            match cord_system.attached_pole {
                Some(_current_pole) => {
                    // Cord is attached, disconnect it
                    disconnect_cord_from_pole(&mut cord_system);
                    println!("Cord disconnected from pole!");
                },
                None => {
                    // Cord is not attached, try to attach to closest pole
                    if let Some((closest_pole, _pole_transform)) = find_closest_pole(
                        player_transform.translation,
                        &pole_query,
                        cord_system.attachment_range
                    ) {
                        // Attach directly to pole entity
                        attach_cord_to_pole(&mut cord_system, closest_pole);
                        println!("Cord attached to pole!");
                    } else {
                        println!("No poles within attachment range!");
                    }
                }
            }
        }
    }
}

pub fn find_closest_pole(
    player_pos: Vec3,
    pole_query: &Query<(Entity, &Transform), With<crate::components::Pole>>,
    max_range: f32,
) -> Option<(Entity, Transform)> {
    let mut closest_pole = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform) in pole_query.iter() {
        // Use 2D distance (XY plane only)
        let distance = player_pos.truncate().distance(transform.translation.truncate());
        if distance <= max_range && distance < closest_distance {
            closest_distance = distance;
            closest_pole = Some((entity, *transform));
        }
    }
    
    closest_pole
}

pub fn disconnect_cord_from_pole(
    cord_system: &mut CordSystem,
) {
    // Simply mark as not attached and clear trail
    cord_system.attached_pole = None;
    cord_system.trail_path.clear();
}

pub fn attach_cord_to_pole(
    cord_system: &mut CordSystem,
    attachment_entity: Entity,
) {
    // Mark as attached to this attachment point and clear any existing trail
    cord_system.attached_pole = Some(attachment_entity);
    cord_system.trail_path.clear();
}

fn snap_to_tile_center(world_pos: Vec2, tile_size: f32, ground_offset: f32) -> Vec2 {
    // Convert world to grid coordinates
    let y = world_pos.y - ground_offset;
    let grid_x = (world_pos.x / (tile_size / 2.0) + y / (tile_size / 4.0)) / 2.0;
    let grid_y = (y / (tile_size / 4.0) - world_pos.x / (tile_size / 2.0)) / 2.0;
    
    // Round to nearest grid cell
    let grid_x_rounded = grid_x.round();
    let grid_y_rounded = grid_y.round();
    
    // Convert back to world coordinates (tile center at ground plane)
    let x = (grid_x_rounded - grid_y_rounded) * tile_size / 2.0;
    let y = (grid_x_rounded + grid_y_rounded) * tile_size / 4.0;
    
    // Player is already at ground plane, no offset needed
    Vec2::new(x, y - 16.0)
}

// Track player movement and build cord trail
pub fn update_cord_trail(
    cord_system: Option<ResMut<CordSystem>>,
    grid: Res<IsometricGrid>,
    player_query: Query<&Transform, With<Player>>,
    pole_query: Query<&Transform, With<crate::components::Pole>>,
) {
    let Some(mut cord_system) = cord_system else { return; };
    // Only track trail if attached to a pole
    if let Some(attached_pole) = cord_system.attached_pole {
        if let (Ok(player_transform), Ok(pole_transform)) = 
            (player_query.single(), pole_query.get(attached_pole)) {
            
            let player_pos = player_transform.translation.truncate();
            let pole_pos = pole_transform.translation.truncate();
            
            // Snap both positions to tile centers at ground plane
            let player_tile_center = snap_to_tile_center(player_pos, grid.tile_size, 0.0);
            let pole_tile_center = snap_to_tile_center(pole_pos, grid.tile_size, 32.0);
            
            // Initialize trail with pole tile center if empty
            if cord_system.trail_path.is_empty() {
                cord_system.trail_path.push(pole_tile_center);
                return;
            }
            
            // Check if player has moved to a different tile
            let last_point = *cord_system.trail_path.last().unwrap();
            
            // Check if player moved back to the previous tile (backtracking)
            if cord_system.trail_path.len() >= 2 {
                let second_to_last = cord_system.trail_path[cord_system.trail_path.len() - 2];
                if (player_tile_center - second_to_last).length() < 0.1 {
                    // Player moved back to previous tile, remove last segment
                    cord_system.trail_path.pop();
                    return;
                }
            }
            
            // Only add new trail point if we're on a different tile center
            if (player_tile_center - last_point).length() > 0.1 {
                cord_system.trail_path.push(player_tile_center);
            }
        }
    }
}

// Wrapper functions that check system toggles
pub fn cord_retraction_wrapper(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cord_system: Option<ResMut<CordSystem>>,
    toggles: Res<crate::components::SystemToggles>,
) {
    if !toggles.cord_systems { return; }
    handle_cord_retraction(keyboard_input, cord_system);
}

pub fn cord_attachment_wrapper(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cord_system: Option<ResMut<CordSystem>>,
    player_query: Query<&Transform, With<Player>>,
    pole_query: Query<(Entity, &Transform), With<crate::components::Pole>>,
    toggles: Res<crate::components::SystemToggles>,
) {
    if !toggles.cord_systems { return; }
    handle_cord_attachment(keyboard_input, cord_system, player_query, pole_query);
}
