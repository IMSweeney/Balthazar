use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::gizmos::config::GizmoConfigStore;
use bevy::gizmos::gizmos::Gizmos;

use crate::components::{Player, CordSegment, CordSystem, PoleAttachment, SystemToggles};

pub fn set_gizmo_width(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 10.0;
}

// Render cord as thick splines connecting segments
pub fn render_cord_lines(
    mut gizmos: Gizmos,
    cord_system: Res<CordSystem>,
    segment_query: Query<&Transform, With<CordSegment>>,
    player_query: Query<&Transform, With<Player>>,
    attachment_query: Query<&Transform, With<PoleAttachment>>,
) {
    // Collect all positions along the cord
    let mut positions = Vec::new();
    
    // Start from attachment point if attached
    if let Some(attached_pole) = cord_system.attached_pole {
        if let Ok(attachment_transform) = attachment_query.get(attached_pole) {
            positions.push(attachment_transform.translation.truncate());
        }
    }
    
    // Add all segment positions
    for &segment_entity in &cord_system.segments {
        if let Ok(transform) = segment_query.get(segment_entity) {
            positions.push(transform.translation.truncate());
        }
    }
    
    // Add player backpack position
    if let Ok(player_transform) = player_query.single() {
        let backpack_pos = player_transform.translation.truncate() + Vec2::new(0.0, -12.0);
        positions.push(backpack_pos);
    }
    
    // Draw thick cubic Bézier curves between points for smooth spline
    if positions.len() >= 2 {
        for i in 0..positions.len() - 1 {
            let p0 = positions[i];
            let p3 = positions[i + 1];
            
            // Calculate control points for smooth curves
            let direction = (p3 - p0).normalize_or_zero();
            let distance = p0.distance(p3);
            let control_offset = distance * 0.25; // Control point offset
            
            // Create control points that follow the tangent direction
            let p1 = p0 + direction * control_offset;
            let p2 = p3 - direction * control_offset;
            
            // Calculate Bézier points once
            let points: Vec<Vec2> = (0..=64).map(|t| {
                let t = t as f32 / 64.0;
                // Cubic Bézier interpolation
                let one_minus_t = 1.0 - t;
                let one_minus_t_sq = one_minus_t * one_minus_t;
                let one_minus_t_cubed = one_minus_t_sq * one_minus_t;
                let t_sq = t * t;
                let t_cubed = t_sq * t;
                
                one_minus_t_cubed * p0
                    + 3.0 * one_minus_t_sq * t * p1
                    + 3.0 * one_minus_t * t_sq * p2
                    + t_cubed * p3
            }).collect();
            
            // Draw black cord (thicker)
            for j in 0..points.len() - 1 {
                gizmos.line_2d(points[j], points[j + 1], Color::BLACK);
            }
        }
    }
}


pub fn handle_cord_retraction(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, (With<Player>, Without<CordSegment>)>,
    attachment_query: Query<&Transform, With<PoleAttachment>>,
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
                let distance = player_transform.translation.distance(attachment_transform.translation);
                
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
    attachment_query: Query<(Entity, &Transform), With<PoleAttachment>>,
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
                    // Cord is not attached, try to attach to closest attachment point
                    if let Some((closest_attachment, _)) = find_closest_attachment_point(
                        player_transform.translation,
                        &attachment_query,
                        cord_system.attachment_range
                    ) {
                        attach_cord_to_pole(&mut commands, &mut cord_system, closest_attachment);
                        println!("Cord attached to pole!");
                    } else {
                        println!("No poles within attachment range!");
                    }
                }
            }
        }
    }
}

pub fn find_closest_attachment_point(
    player_pos: Vec3,
    attachment_query: &Query<(Entity, &Transform), With<PoleAttachment>>,
    max_range: f32,
) -> Option<(Entity, f32)> {
    let mut closest_attachment = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform) in attachment_query.iter() {
        // Attachment points are already positioned at pole bases, so use position directly
        let distance = player_pos.distance(transform.translation);
        if distance <= max_range && distance < closest_distance {
            closest_distance = distance;
            closest_attachment = Some((entity, distance));
        }
    }
    
    closest_attachment
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
    attachment_query: Query<&Transform, With<PoleAttachment>>,
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
    attachment_query: Query<(Entity, &Transform), With<PoleAttachment>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.cord_systems { return; }
    handle_cord_attachment(commands, keyboard_input, cord_system, player_query, attachment_query);
}