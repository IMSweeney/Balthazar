use avian2d::prelude::*;
use bevy::prelude::*;

use crate::components::{Player, CordSegment, CordSystem, Pole, SystemToggles, CordMaterial, AttachmentPoint};
use crate::setup::Z_GAME_LAYER;

// Catmull-Rom spline interpolation between four control points
fn catmull_rom_spline(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    0.5 * (
        (2.0 * p1) +
        (-p0 + p2) * t +
        (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 +
        (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
    )
}

// Generate smooth spline points from control points
fn generate_spline_points(control_points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if control_points.len() < 2 {
        return Vec::new();
    }
    
    if control_points.len() == 2 {
        return control_points.to_vec();
    }
    
    let mut spline_points = Vec::new();
    
    // For each segment between control points
    for i in 0..control_points.len() - 1 {
        // Get the four control points needed for Catmull-Rom
        let p0 = if i == 0 { control_points[i] } else { control_points[i - 1] };
        let p1 = control_points[i];
        let p2 = control_points[i + 1];
        let p3 = if i + 2 < control_points.len() { control_points[i + 2] } else { control_points[i + 1] };
        
        // Sample points along this segment
        for j in 0..samples_per_segment {
            let t = j as f32 / samples_per_segment as f32;
            let point = catmull_rom_spline(p0, p1, p2, p3, t);
            spline_points.push(point);
        }
    }
    
    // Add the final point
    spline_points.push(*control_points.last().unwrap());
    
    spline_points
}

// Render cord as textured meshes connecting segments
pub fn render_cord_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cord_system: ResMut<CordSystem>,
    cord_material: Res<CordMaterial>,
    segment_query: Query<&Transform, With<CordSegment>>,
    player_query: Query<&Transform, With<Player>>,
    attachment_query: Query<&Transform, With<AttachmentPoint>>,
    mesh_query: Query<Entity, With<CordMeshSegment>>,
) {
    // Collect all positions along the cord
    let mut control_points = Vec::new();
    
    // Start from attachment point if attached
    if let Some(attached_pole) = cord_system.attached_pole {
        if let Ok(attachment_transform) = attachment_query.get(attached_pole) {
            control_points.push(attachment_transform.translation.truncate());
        }
    }
    
    // Add all segment positions
    for &segment_entity in &cord_system.segments {
        if let Ok(transform) = segment_query.get(segment_entity) {
            control_points.push(transform.translation.truncate());
        }
    }
    
    // Add player backpack position
    if let Ok(player_transform) = player_query.single() {
        let backpack_pos = player_transform.translation.truncate() + Vec2::new(0.0, -12.0);
        control_points.push(backpack_pos);
    }
    
    // Remove old mesh entities
    for entity in cord_system.visual_meshes.drain(..) {
        if let Ok(entity) = mesh_query.get(entity) {
            commands.entity(entity).despawn();
        }
    }
    
    // Generate smooth spline points
    let samples_per_segment = 8; // Number of points to sample between each control point
    let spline_points = generate_spline_points(&control_points, samples_per_segment);
    
    // Create mesh segments between consecutive spline points
    let cord_width = 8.0; // Width of the cord visual
    if spline_points.len() >= 2 {
        for i in 0..spline_points.len() - 1 {
            let start = spline_points[i];
            let end = spline_points[i + 1];
            
            // Calculate the midpoint, length, and rotation for the segment
            let midpoint = (start + end) / 2.0;
            let diff = end - start;
            let length = diff.length();
            let angle = diff.y.atan2(diff.x);
            
            // Create a rectangular mesh for this segment
            let mesh_entity = commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(length, cord_width))),
                MeshMaterial2d(cord_material.material.clone()), // Use textured material
                Transform::from_translation(midpoint.extend(Z_GAME_LAYER)) // Same Z as game objects
                    .with_rotation(Quat::from_rotation_z(angle)),
                CordMeshSegment,
            )).id();
            
            cord_system.visual_meshes.push(mesh_entity);
        }
    }
}

// Component to mark visual cord mesh segments
#[derive(Component)]
pub struct CordMeshSegment;


pub fn handle_cord_retraction(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, (With<Player>, Without<CordSegment>)>,
    attachment_query: Query<&Transform, With<AttachmentPoint>>,
    segment_query: Query<&Transform, With<CordSegment>>,
    time: Res<Time>,
) {
    let retraction_speed = 300.0; // Much faster retraction
    let extension_speed = 80.0;
    
    // Check if shift is pressed
    let should_retract = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    
    if should_retract && cord_system.current_length > cord_system.min_length {
        // Retract cord
        cord_system.is_retracting = true;
        cord_system.current_length -= retraction_speed * time.delta_secs();
        cord_system.current_length = cord_system.current_length.max(cord_system.min_length);
        
        // Remove segments if we have too many
        let new_target_segments = (cord_system.current_length / cord_system.segment_length) as usize;
        while cord_system.segments.len() > new_target_segments.max(2) {
            remove_cord_segment(&mut commands, &mut cord_system);
        }
        
    } else if !should_retract {
        // Not retracting - check if player is pulling to extend cord
        if let (Ok(player_transform), Some(attached_point)) = (player_query.single(), cord_system.attached_pole) {
            // Find the attached attachment point's position (already at pole base)
            if let Ok(attachment_transform) = attachment_query.get(attached_point) {
                // Use 2D distance (XY plane) since attachment point is at same z-layer as player
                let distance = player_transform.translation.truncate().distance(attachment_transform.translation.truncate());
                
                // If player is pulling at current cord length, extend the cord
                if distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                    cord_system.current_length += extension_speed * time.delta_secs();
                    cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
                    
                    // Add segments if we need more
                    let new_target_segments = (cord_system.current_length / cord_system.segment_length) as usize;
                    while cord_system.segments.len() < new_target_segments {
                        add_cord_segment(&mut commands, &mut cord_system, &segment_query);
                    }
                }
            }
        }
        
        cord_system.is_retracting = false;
    }
}

pub fn add_cord_segment(
    commands: &mut Commands,
    cord_system: &mut CordSystem,
    segment_query: &Query<&Transform, With<CordSegment>>,
) {
    if cord_system.segments.is_empty() {
        return;
    }

    // Get the position of the last segment
    let last_segment_pos = if let Ok(last_transform) = segment_query.get(*cord_system.segments.last().unwrap()) {
        last_transform.translation
    } else {
        return;
    };

    // Calculate position for new segment (extend towards player)
    let player_pos = Vec3::new(cord_system.current_length, 0.0, 0.0);
    let direction = (player_pos - last_segment_pos).normalize_or_zero();
    let new_pos = last_segment_pos + direction * cord_system.segment_length;

    // Create new segment (invisible - we render lines instead)
    let new_segment = commands.spawn((
        Sprite {
            color: Color::NONE, // Invisible
            custom_size: Some(Vec2::new(cord_system.segment_size, cord_system.segment_size)),
            ..default()
        },
        Transform::from_translation(new_pos),
        RigidBody::Dynamic,
        Collider::circle(cord_system.segment_size / 2.0),
        LinearDamping(1.2),
        AngularDamping(1.2),
        CordSegment,
    )).id();

    // Remove the old last-to-player joint
    if let Some(old_joint) = cord_system.joints.pop() {
        commands.entity(old_joint).despawn();
    }

    // Add joint connecting the previous last segment to new segment
    let prev_last_segment = *cord_system.segments.last().unwrap();
    let new_joint = commands.spawn(
        DistanceJoint::new(prev_last_segment, new_segment)
            .with_limits(cord_system.segment_length * 0.95, cord_system.segment_length * 1.05)
    ).id();
    cord_system.joints.push(new_joint);

    // Add joint connecting new segment to player (at backpack)
    let player_joint = commands.spawn(
        FixedJoint::new(new_segment, cord_system.player_entity)
            .with_local_anchor2(Vec2::new(0.0, -12.0))
    ).id();
    cord_system.joints.push(player_joint);

    // Add the new segment to our list
    cord_system.segments.push(new_segment);
}

pub fn remove_cord_segment(
    commands: &mut Commands,
    cord_system: &mut CordSystem,
) {
    if cord_system.segments.len() <= 2 {
        return; // Keep at least 2 segments
    }

    // Remove the last segment
    if let Some(last_segment) = cord_system.segments.pop() {
        commands.entity(last_segment).despawn();
    }

    // Remove the last two joints (segment-to-segment and segment-to-player)
    for _ in 0..2 {
        if let Some(joint) = cord_system.joints.pop() {
            commands.entity(joint).despawn();
        }
    }

    // Reconnect the new last segment to the player (at backpack)
    if let Some(&new_last_segment) = cord_system.segments.last() {
        let player_joint = commands.spawn(
            FixedJoint::new(new_last_segment, cord_system.player_entity)
                .with_local_anchor2(Vec2::new(0.0, -12.0))
        ).id();
        cord_system.joints.push(player_joint);
    }
}

pub fn handle_cord_attachment(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, With<Player>>,
    pole_query: Query<(Entity, &Transform), With<Pole>>,
    attachment_query: Query<(Entity, &AttachmentPoint)>,
) {
    // Check for spacebar press
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.single() {
            match cord_system.attached_pole {
                Some(_current_pole) => {
                    // Cord is attached, disconnect it
                    disconnect_cord_from_pole(&mut commands, &mut cord_system);
                    println!("Cord disconnected from pole!");
                },
                None => {
                    // Cord is not attached, try to attach to closest pole
                    if let Some((closest_pole, pole_transform)) = find_closest_pole(
                        player_transform.translation,
                        &pole_query,
                        cord_system.attachment_range
                    ) {
                        // Check if an attachment point already exists for this pole
                        let existing_attachment = attachment_query
                            .iter()
                            .find(|(_, ap)| ap.parent_pole == closest_pole)
                            .map(|(entity, _)| entity);
                        
                        let attachment_entity = if let Some(existing) = existing_attachment {
                            existing
                        } else {
                            // Create new attachment point at pole XY but player Z layer
                            commands.spawn((
                                Sprite {
                                    color: Color::NONE, // Invisible
                                    custom_size: Some(Vec2::new(1.0, 1.0)),
                                    ..default()
                                },
                                Transform::from_translation(Vec3::new(
                                    pole_transform.translation.x,
                                    pole_transform.translation.y,
                                    Z_GAME_LAYER, // Same z-layer as player
                                )),
                                RigidBody::Static,
                                Collider::circle(0.5),
                                AttachmentPoint {
                                    parent_pole: closest_pole,
                                },
                            )).id()
                        };
                        
                        attach_cord_to_pole(&mut commands, &mut cord_system, attachment_entity);
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
    pole_query: &Query<(Entity, &Transform), With<Pole>>,
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
    commands: &mut Commands,
    cord_system: &mut CordSystem,
) {
    // Remove the joint connecting the first cord segment to the pole
    if let Some(first_joint) = cord_system.joints.first() {
        commands.entity(*first_joint).despawn();
        cord_system.joints.remove(0);
    }
    
    // Mark as not attached to any pole
    cord_system.attached_pole = None;
}

pub fn attach_cord_to_pole(
    commands: &mut Commands,
    cord_system: &mut CordSystem,
    attachment_entity: Entity,
) {
    // Connect the first cord segment to the attachment point (at pole base)
    if let Some(&first_segment) = cord_system.segments.first() {
        let joint = commands.spawn(
            DistanceJoint::new(attachment_entity, first_segment)
                .with_limits(cord_system.segment_length * 0.9, cord_system.segment_length * 1.1)
        ).id();
        
        // Insert the new joint at the beginning
        cord_system.joints.insert(0, joint);
    }
    
    // Mark as attached to this attachment point
    cord_system.attached_pole = Some(attachment_entity);
}

// Wrapper functions that check system toggles
pub fn cord_retraction_wrapper(
    commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, (With<Player>, Without<CordSegment>)>,
    attachment_query: Query<&Transform, With<AttachmentPoint>>,
    segment_query: Query<&Transform, With<CordSegment>>,
    time: Res<Time>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.cord_systems { return; }
    handle_cord_retraction(commands, keyboard_input, cord_system, player_query, attachment_query, segment_query, time);
}

pub fn cord_attachment_wrapper(
    commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, With<Player>>,
    pole_query: Query<(Entity, &Transform), With<Pole>>,
    attachment_query: Query<(Entity, &AttachmentPoint)>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.cord_systems { return; }
    handle_cord_attachment(commands, keyboard_input, cord_system, player_query, pole_query, attachment_query);
}
