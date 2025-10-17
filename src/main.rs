use avian2d::prelude::*;
use bevy::prelude::*;

// Use types and functions from lib.rs
use balthazar::{Player, Pole, CordSegment, CordSystem, handle_cord_retraction, handle_cord_attachment};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec2::ZERO)) // No gravity for top-down view
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, handle_cord_retraction, handle_cord_attachment))
        .run();
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


