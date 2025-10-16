use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec2::ZERO)) // No gravity for top-down view
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Pole;

#[derive(Component)]
struct RopeSegment;

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn pole in center
    let pole_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.4, 0.2), // Brown color for pole
            custom_size: Some(Vec2::new(20.0, 200.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        RigidBody::Static,
        Collider::rectangle(20.0, 200.0),
        Pole,
    )).id();

    // Create rope segments connecting pole to player
    let rope_length = 150.0;
    let num_segments = 8;
    let segment_length = rope_length / num_segments as f32;
    let segment_size = 8.0;
    
    let mut rope_entities = Vec::new();
    
    // Create rope segments
    for i in 0..num_segments {
        let x = (i + 1) as f32 * segment_length;
        let rope_entity = commands.spawn((
            Sprite {
                color: Color::srgb(0.4, 0.3, 0.1), // Dark brown for rope
                custom_size: Some(Vec2::new(segment_size, segment_size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, 0., 0.)),
            RigidBody::Dynamic,
            Collider::circle(segment_size / 2.0),
            LinearDamping(0.8),
            AngularDamping(0.8),
            RopeSegment,
        )).id();
        
        rope_entities.push(rope_entity);
    }

    // Spawn player at end of rope
    let player_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(rope_length, 0., 0.)),
        RigidBody::Dynamic,
        Collider::rectangle(50.0, 50.0),
        LinearDamping(0.5),
        AngularDamping(0.8),
        Player,
    )).id();

    // Connect pole to first rope segment
    if let Some(&first_rope) = rope_entities.first() {
        commands.spawn(
            DistanceJoint::new(pole_entity, first_rope)
                .with_limits(segment_length, segment_length)
        );
    }

    // Connect rope segments to each other
    for i in 0..rope_entities.len() - 1 {
        commands.spawn(
            DistanceJoint::new(rope_entities[i], rope_entities[i + 1])
                .with_limits(segment_length, segment_length)
        );
    }

    // Connect last rope segment to player
    if let Some(&last_rope) = rope_entities.last() {
        commands.spawn(
            DistanceJoint::new(last_rope, player_entity)
                .with_limits(segment_length, segment_length)
        );
    }
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
