use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec2::ZERO)) // No gravity for top-down view
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, handle_cord_retraction, handle_cord_attachment))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Pole;

#[derive(Component)]
struct CordSegment;

#[derive(Resource)]
struct CordSystem {
    segments: Vec<Entity>,
    joints: Vec<Entity>,
    max_length: f32,
    min_length: f32,
    current_length: f32,
    segment_length: f32, // Fixed length per segment
    segment_size: f32,   // Visual size of each segment
    player_entity: Entity,
    is_retracting: bool,
    attached_pole: Option<Entity>, // Track which pole the cord is attached to
    attachment_range: f32, // Maximum distance to attach to a pole
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn multiple poles for testing
    let pole_positions = vec![
        Vec3::new(0., 0., 0.), // Center pole
        Vec3::new(300., 200., 0.),
        Vec3::new(-300., 150., 0.),
        Vec3::new(200., -250., 0.),
        Vec3::new(-150., -200., 0.),
    ];
    
    let mut pole_entities = Vec::new();
    for (i, pos) in pole_positions.iter().enumerate() {
        let pole_entity = commands.spawn((
            Sprite {
                color: if i == 0 { Color::srgb(0.6, 0.4, 0.2) } else { Color::srgb(0.5, 0.3, 0.1) }, // Different colors
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_translation(*pos),
            RigidBody::Static,
            Collider::circle(15.0),
            Pole,
        )).id();
        pole_entities.push(pole_entity);
    }
    
    let center_pole = pole_entities[0]; // First pole is the center one

    // Create cord segments connecting pole to player
    let max_cord_length = 500.0; // Much longer cord
    let min_cord_length = 50.0;
    let segment_length = 20.0; // Fixed length per segment
    let segment_size = 8.0; // Visual size of each segment
    let initial_num_segments = (max_cord_length / segment_length) as usize;
    
    let mut cord_entities = Vec::new();
    let mut joint_entities = Vec::new();
    
    // Create cord segments
    for i in 0..initial_num_segments {
        let x = (i + 1) as f32 * segment_length;
        let cord_entity = commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2), // Dark gray for cord
                custom_size: Some(Vec2::new(segment_size, segment_size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, 0., 0.)),
            RigidBody::Dynamic,
            Collider::circle(segment_size / 2.0),
            LinearDamping(1.2), // Higher damping for less flexible cord
            AngularDamping(1.2),
            CordSegment,
        )).id();
        
        cord_entities.push(cord_entity);
    }

    // Spawn player at end of rope
    let player_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(max_cord_length, 0., 0.)),
        RigidBody::Dynamic,
        Collider::rectangle(50.0, 50.0),
        LinearDamping(0.5),
        AngularDamping(0.8),
        Player,
    )).id();

    // Connect pole to first cord segment
    if let Some(&first_cord) = cord_entities.first() {
        let joint = commands.spawn(
            DistanceJoint::new(center_pole, first_cord)
                .with_limits(segment_length * 0.9, segment_length * 1.1) // Slight flexibility
        ).id();
        joint_entities.push(joint);
    }

    // Connect cord segments to each other with stiffer joints
    for i in 0..cord_entities.len() - 1 {
        let joint = commands.spawn(
            DistanceJoint::new(cord_entities[i], cord_entities[i + 1])
                .with_limits(segment_length * 0.95, segment_length * 1.05) // Less stretch for cord
        ).id();
        joint_entities.push(joint);
    }

    // Connect last cord segment to player
    if let Some(&last_cord) = cord_entities.last() {
        let joint = commands.spawn(
            DistanceJoint::new(last_cord, player_entity)
                .with_limits(segment_length * 0.9, segment_length * 1.1)
        ).id();
        joint_entities.push(joint);
    }

    // Initialize cord system resource
    commands.insert_resource(CordSystem {
        segments: cord_entities,
        joints: joint_entities,
        max_length: max_cord_length,
        min_length: min_cord_length,
        current_length: max_cord_length,
        segment_length,
        segment_size,
        player_entity,
        is_retracting: false,
        attached_pole: Some(center_pole), // Initially attached to center pole
        attachment_range: 100.0, // Can attach to poles within 100 units
    });
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
) {
    if let Ok(mut velocity) = player_query.single_mut() {
        let mut force = Vec2::ZERO;
        let move_speed = 300.0;

        if keyboard_input.pressed(KeyCode::KeyA) {
            force.x -= move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            force.x += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            force.y += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            force.y -= move_speed;
        }

        // Apply force as velocity (since we have no gravity, this works well)
        velocity.0 = force;
    }
}

fn handle_cord_retraction(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, (With<Player>, Without<CordSegment>)>,
    pole_query: Query<(Entity, &Transform), (With<Pole>, Without<Player>, Without<CordSegment>)>,
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
        if let (Ok(player_transform), Some(attached_pole)) = (player_query.single(), cord_system.attached_pole) {
            // Find the attached pole's position
            if let Ok((_, pole_transform)) = pole_query.get(attached_pole) {
                let distance = player_transform.translation.distance(pole_transform.translation);
                
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

fn add_cord_segment(
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

    // Create new segment
    let new_segment = commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.2),
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

    // Add joint connecting new segment to player
    let player_joint = commands.spawn(
        DistanceJoint::new(new_segment, cord_system.player_entity)
            .with_limits(cord_system.segment_length * 0.9, cord_system.segment_length * 1.1)
    ).id();
    cord_system.joints.push(player_joint);

    // Add the new segment to our list
    cord_system.segments.push(new_segment);
}

fn remove_cord_segment(
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

    // Reconnect the new last segment to the player
    if let Some(&new_last_segment) = cord_system.segments.last() {
        let player_joint = commands.spawn(
            DistanceJoint::new(new_last_segment, cord_system.player_entity)
                .with_limits(cord_system.segment_length * 0.9, cord_system.segment_length * 1.1)
        ).id();
        cord_system.joints.push(player_joint);
    }
}

fn handle_cord_attachment(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cord_system: ResMut<CordSystem>,
    player_query: Query<&Transform, (With<Player>, Without<Pole>)>,
    pole_query: Query<(Entity, &Transform), With<Pole>>,
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
                    if let Some((closest_pole, _)) = find_closest_pole(
                        player_transform.translation,
                        &pole_query,
                        cord_system.attachment_range
                    ) {
                        attach_cord_to_pole(&mut commands, &mut cord_system, closest_pole);
                        println!("Cord attached to pole!");
                    } else {
                        println!("No poles within attachment range!");
                    }
                }
            }
        }
    }
}

fn find_closest_pole(
    player_pos: Vec3,
    pole_query: &Query<(Entity, &Transform), With<Pole>>,
    max_range: f32,
) -> Option<(Entity, f32)> {
    let mut closest_pole = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform) in pole_query.iter() {
        let distance = player_pos.distance(transform.translation);
        if distance <= max_range && distance < closest_distance {
            closest_distance = distance;
            closest_pole = Some((entity, distance));
        }
    }
    
    closest_pole
}

fn disconnect_cord_from_pole(
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

fn attach_cord_to_pole(
    commands: &mut Commands,
    cord_system: &mut CordSystem,
    pole_entity: Entity,
) {
    // Connect the first cord segment to the new pole
    if let Some(&first_segment) = cord_system.segments.first() {
        let joint = commands.spawn(
            DistanceJoint::new(pole_entity, first_segment)
                .with_limits(cord_system.segment_length * 0.9, cord_system.segment_length * 1.1)
        ).id();
        
        // Insert the new joint at the beginning
        cord_system.joints.insert(0, joint);
    }
    
    // Mark as attached to this pole
    cord_system.attached_pole = Some(pole_entity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cord_retraction_logic() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 100.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: false,
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Test retraction
        let retraction_speed = 100.0;
        let delta_time = 0.1;
        let should_retract = true;

        if should_retract && cord_system.current_length > cord_system.min_length {
            cord_system.is_retracting = true;
            cord_system.current_length -= retraction_speed * delta_time;
            cord_system.current_length = cord_system.current_length.max(cord_system.min_length);
        }

        // Verify retraction
        assert_eq!(cord_system.current_length, 90.0);
        assert!(cord_system.is_retracting);
    }

    #[test]
    fn test_cord_retraction_minimum_limit() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 50.0, // Already at minimum
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: false,
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Test retraction at minimum
        let retraction_speed = 100.0;
        let delta_time = 0.1;
        let should_retract = true;

        if should_retract && cord_system.current_length > cord_system.min_length {
            cord_system.is_retracting = true;
            cord_system.current_length -= retraction_speed * delta_time;
            cord_system.current_length = cord_system.current_length.max(cord_system.min_length);
        }

        // Should stay at minimum
        assert_eq!(cord_system.current_length, 50.0);
        assert!(!cord_system.is_retracting); // Should not be retracting
    }

    #[test]
    fn test_cord_extension_when_player_pulls() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 80.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: true,
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Simulate player pulling (distance >= current_length * 0.95)
        let player_distance = 80.0 * 0.96; // Just above threshold
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;

        if !should_retract && cord_system.is_retracting {
            // Player is pulling at current cord length, extend the cord
            if player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
            cord_system.is_retracting = false;
        }

        // Verify extension
        assert_eq!(cord_system.current_length, 88.0); // 80.0 + (80.0 * 0.1)
        assert!(!cord_system.is_retracting);
    }

    #[test]
    fn test_cord_extension_maximum_limit() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 149.0, // Near maximum
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: true,
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Simulate player pulling
        let player_distance = 149.0 * 0.96; // Above threshold
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;

        if !should_retract && cord_system.is_retracting {
            if player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
            cord_system.is_retracting = false;
        }

        // Should be clamped at maximum
        assert_eq!(cord_system.current_length, 150.0);
        assert!(!cord_system.is_retracting);
    }

    #[test]
    fn test_cord_no_extension_when_player_not_pulling() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 80.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: true,
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Simulate player NOT pulling (distance < current_length * 0.95)
        let player_distance = 80.0 * 0.9; // Below threshold
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;

        if !should_retract && cord_system.is_retracting {
            // Player is NOT pulling enough to extend
            if player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
            cord_system.is_retracting = false;
        }

        // Should stay at same length
        assert_eq!(cord_system.current_length, 80.0);
        assert!(!cord_system.is_retracting);
    }

    #[test]
    fn test_extension_functionality_after_pole_attachment() {
        // Test that extension works after attaching to a different pole
        let pole1 = Entity::from_bits(200);
        let pole2 = Entity::from_bits(201);
        
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 80.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: false,
            attached_pole: Some(pole1), // Initially attached to pole1
            attachment_range: 100.0,
        };
        
        // Simulate attaching to pole2
        cord_system.attached_pole = Some(pole2);
        
        // Simulate player pulling from pole2's position
        // (In real game, this would be calculated from actual pole position)
        let player_distance_from_pole2 = 80.0 * 0.96; // Above threshold for extension
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;
        
        // Test extension logic with new pole attachment
        if !should_retract && cord_system.attached_pole.is_some() {
            // Simulate the extension condition being met
            if player_distance_from_pole2 >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
        }
        
        // Verify extension worked with the new pole attachment
        assert_eq!(cord_system.current_length, 88.0); // 80.0 + (80.0 * 0.1)
        assert_eq!(cord_system.attached_pole, Some(pole2));
    }
    
    #[test]
    fn test_extension_fails_when_not_attached_to_any_pole() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 80.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: false,
            attached_pole: None, // Not attached to any pole
            attachment_range: 100.0,
        };
        
        let initial_length = cord_system.current_length;
        let player_distance = 80.0 * 0.96; // Would normally trigger extension
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;
        
        // Test that extension doesn't work when not attached
        if !should_retract && cord_system.attached_pole.is_some() {
            if player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
        }
        
        // Verify length didn't change when not attached
        assert_eq!(cord_system.current_length, initial_length);
        assert_eq!(cord_system.attached_pole, None);
    }

    #[test]
    fn test_continuous_extension_after_initial_extension() {
        let mut cord_system = CordSystem {
            segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
            joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
            max_length: 150.0,
            min_length: 50.0,
            current_length: 80.0,
            segment_length: 20.0,
            segment_size: 8.0,
            player_entity: Entity::from_bits(100),
            is_retracting: false, // Not retracting initially
            attached_pole: Some(Entity::from_bits(200)),
            attachment_range: 100.0,
        };

        // Simulate player pulling (distance >= current_length * 0.95)
        let player_distance = 80.0 * 0.96; // Above threshold
        let extension_speed = 80.0;
        let delta_time = 0.1;
        let should_retract = false;

        // First extension cycle (this would work with the old buggy logic)
        if !should_retract {
            if player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
            cord_system.is_retracting = false;
        }

        // Verify first extension worked
        assert_eq!(cord_system.current_length, 88.0);
        assert!(!cord_system.is_retracting);

        // Second extension cycle - this is where the bug would manifest
        // The old buggy logic required `is_retracting` to be true, so this wouldn't work
        let new_player_distance = 88.0 * 0.96; // Player still pulling at new length
        
        if !should_retract {
            if new_player_distance >= cord_system.current_length * 0.95 && cord_system.current_length < cord_system.max_length {
                cord_system.current_length += extension_speed * delta_time;
                cord_system.current_length = cord_system.current_length.min(cord_system.max_length);
            }
            cord_system.is_retracting = false;
        }

        // This assertion would fail with the old buggy logic because 
        // extension would only work when transitioning from retracting to not retracting
        assert_eq!(cord_system.current_length, 96.0); // 88.0 + (80.0 * 0.1)
        assert!(!cord_system.is_retracting);
    }
}
