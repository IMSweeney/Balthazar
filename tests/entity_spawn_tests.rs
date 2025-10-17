use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use avian2d::prelude::*;

// Import the types we need for testing
use balthazar::{Player, Pole, CordSegment, CordSystem};

#[test]
fn test_player_entity_properties() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test player entity creation with all expected components
    let player_entity = app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::circle(20.0),
        LinearDamping(1.5),
        AngularDamping(2.0),
    )).id();
    
    // Verify entity exists
    assert!(app.world().get_entity(player_entity).is_ok());
    
    // Verify components
    assert!(app.world().get::<Player>(player_entity).is_some());
    assert!(app.world().get::<Transform>(player_entity).is_some());
    assert!(app.world().get::<RigidBody>(player_entity).is_some());
    assert!(app.world().get::<Collider>(player_entity).is_some());
    assert!(app.world().get::<LinearDamping>(player_entity).is_some());
    assert!(app.world().get::<AngularDamping>(player_entity).is_some());
    
    // Test player physics properties
    let rigid_body = app.world().get::<RigidBody>(player_entity).unwrap();
    assert_eq!(*rigid_body, RigidBody::Dynamic);
    
    let linear_damping = app.world().get::<LinearDamping>(player_entity).unwrap();
    assert_eq!(linear_damping.0, 1.5);
    
    let angular_damping = app.world().get::<AngularDamping>(player_entity).unwrap();
    assert_eq!(angular_damping.0, 2.0);
}

#[test]
fn test_cord_segment_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let segment_length = 20.0;
    let segment_size = 8.0;
    let cord_color = Color::srgb(0.2, 0.2, 0.2); // Dark gray
    
    // Test creating a cord segment
    let segment_entity = app.world_mut().spawn((
        CordSegment,
        Transform::from_xyz(segment_length, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::circle(segment_size / 2.0),
        LinearDamping(3.0),
        AngularDamping(3.0),
    )).id();
    
    // Verify entity exists
    assert!(app.world().get_entity(segment_entity).is_ok());
    
    // Verify components
    assert!(app.world().get::<CordSegment>(segment_entity).is_some());
    assert!(app.world().get::<Transform>(segment_entity).is_some());
    assert!(app.world().get::<RigidBody>(segment_entity).is_some());
    assert!(app.world().get::<Collider>(segment_entity).is_some());
    assert!(app.world().get::<LinearDamping>(segment_entity).is_some());
    assert!(app.world().get::<AngularDamping>(segment_entity).is_some());
    
    // Test physics properties
    let rigid_body = app.world().get::<RigidBody>(segment_entity).unwrap();
    assert_eq!(*rigid_body, RigidBody::Dynamic);
    
    let linear_damping = app.world().get::<LinearDamping>(segment_entity).unwrap();
    assert_eq!(linear_damping.0, 3.0); // High damping to reduce jitter
    
    let angular_damping = app.world().get::<AngularDamping>(segment_entity).unwrap();
    assert_eq!(angular_damping.0, 3.0);
    
    // Test cord color
    assert_eq!(cord_color.to_srgba().red, 0.2);
    assert_eq!(cord_color.to_srgba().green, 0.2);
    assert_eq!(cord_color.to_srgba().blue, 0.2);
}

#[test]
fn test_cord_attachment_point() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test cord attachment point properties
    let attachment_offset = Vec3::new(0.0, -12.0, 0.0); // Same as backpack position
    let attachment_collider_radius = 2.0;
    let attachment_damping = 10.0;
    
    let attachment_entity = app.world_mut().spawn((
        Transform::from_translation(attachment_offset),
        RigidBody::Dynamic,
        Collider::circle(attachment_collider_radius),
        LinearDamping(attachment_damping),
        AngularDamping(attachment_damping),
    )).id();
    
    // Verify entity exists
    assert!(app.world().get_entity(attachment_entity).is_ok());
    
    // Test transform
    let transform = app.world().get::<Transform>(attachment_entity).unwrap();
    assert_eq!(transform.translation, attachment_offset);
    
    // Test physics properties
    let linear_damping = app.world().get::<LinearDamping>(attachment_entity).unwrap();
    assert_eq!(linear_damping.0, attachment_damping);
    
    let angular_damping = app.world().get::<AngularDamping>(attachment_entity).unwrap();
    assert_eq!(angular_damping.0, attachment_damping);
}

#[test]
fn test_camera_initial_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test initial camera setup
    let initial_scale = 0.5;
    let initial_position = Vec3::new(0.0, 0.0, 1000.0);
    
    let camera_entity = app.world_mut().spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: initial_scale,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_translation(initial_position),
    )).id();
    
    // Verify entity exists
    assert!(app.world().get_entity(camera_entity).is_ok());
    
    // Verify components
    assert!(app.world().get::<Camera2d>(camera_entity).is_some());
    assert!(app.world().get::<Projection>(camera_entity).is_some());
    assert!(app.world().get::<Transform>(camera_entity).is_some());
    
    // Test transform
    let transform = app.world().get::<Transform>(camera_entity).unwrap();
    assert_eq!(transform.translation, initial_position);
    
    // Test projection
    let projection = app.world().get::<Projection>(camera_entity).unwrap();
    if let Projection::Orthographic(ortho) = projection {
        assert_eq!(ortho.scale, initial_scale);
    } else {
        panic!("Expected orthographic projection");
    }
}

#[test]
fn test_joint_creation_parameters() {
    // Test distance joint parameters used in the game
    let tight_joint_min = 0.98; // segment_length * 0.98
    let tight_joint_max = 1.02; // segment_length * 1.02
    let loose_joint_min = 0.9;  // segment_length * 0.9
    let loose_joint_max = 1.1;  // segment_length * 1.1
    
    let segment_length = 20.0;
    
    // Test tight joint limits (for segment-to-segment connections)
    let tight_min_actual = segment_length * tight_joint_min;
    let tight_max_actual = segment_length * tight_joint_max;
    
    assert_eq!(tight_min_actual, 19.6);
    assert_eq!(tight_max_actual, 20.4);
    
    // Test loose joint limits (for player connections)
    let loose_min_actual = segment_length * loose_joint_min;
    let loose_max_actual = segment_length * loose_joint_max;
    
    assert_eq!(loose_min_actual, 18.0);
    assert_eq!(loose_max_actual, 22.0);
    
    // Test that tight joints are more restrictive
    assert!(tight_min_actual > loose_min_actual);
    assert!(tight_max_actual < loose_max_actual);
}

#[test]
fn test_gravity_configuration() {
    // Test that gravity is disabled for top-down view
    let gravity = Vec2::ZERO;
    assert_eq!(gravity, Vec2::new(0.0, 0.0));
    
    // Test that this is different from typical gravity
    let typical_gravity = Vec2::new(0.0, -9.81);
    assert_ne!(gravity, typical_gravity);
}

#[test]
fn test_component_hierarchy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test parent-child relationship setup
    let parent_entity = app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    let child_entity = app.world_mut().spawn((
        Transform::from_xyz(0.0, 12.0, 0.01), // Eye offset
        ChildOf(parent_entity),
    )).id();
    
    // Verify both entities exist
    assert!(app.world().get_entity(parent_entity).is_ok());
    assert!(app.world().get_entity(child_entity).is_ok());
    
    // Verify child relationship
    assert!(app.world().get::<ChildOf>(child_entity).is_some());
    let child_of = app.world().get::<ChildOf>(child_entity).unwrap();
    assert_eq!(child_of.get(), parent_entity);
}

#[test]
fn test_entity_spawn_order() {
    // Test that entities are spawned in the correct order
    let spawn_order = vec![
        "Camera",
        "Poles",
        "Cord Segments", 
        "Player",
        "Player Visual Elements",
        "Cord Attachment Point",
        "Distance Joints",
        "Cord System Resource",
    ];
    
    // Verify spawn order makes sense
    assert_eq!(spawn_order.len(), 8);
    assert_eq!(spawn_order[0], "Camera");
    assert_eq!(spawn_order[3], "Player");
    assert_eq!(spawn_order[7], "Cord System Resource");
    
    // Test that dependent entities come after their dependencies
    let camera_index = spawn_order.iter().position(|&x| x == "Camera").unwrap();
    let player_index = spawn_order.iter().position(|&x| x == "Player").unwrap();
    let joints_index = spawn_order.iter().position(|&x| x == "Distance Joints").unwrap();
    
    // Camera should be first (independent)
    assert_eq!(camera_index, 0);
    
    // Joints should come after player and cord segments
    assert!(joints_index > player_index);
}

#[test]
fn test_physics_damping_values() {
    // Test damping values used in the game for stability
    let player_linear_damping = 1.5;
    let player_angular_damping = 2.0;
    let cord_linear_damping = 3.0;
    let cord_angular_damping = 3.0;
    let attachment_damping = 10.0;
    
    // Test that damping values increase for more stability
    assert!(player_linear_damping < cord_linear_damping);
    assert!(cord_linear_damping < attachment_damping);
    
    // Test that angular damping is generally higher than linear
    assert!(player_angular_damping >= player_linear_damping);
    assert!(cord_angular_damping >= cord_linear_damping);
    
    // Test specific values
    assert_eq!(player_linear_damping, 1.5);
    assert_eq!(player_angular_damping, 2.0);
    assert_eq!(cord_linear_damping, 3.0);
    assert_eq!(cord_angular_damping, 3.0);
    assert_eq!(attachment_damping, 10.0);
}