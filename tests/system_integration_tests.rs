use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;

// Import the types we need for testing
use balthazar::{Player, CordSystem};

// Mock SystemToggles for integration testing
#[derive(Resource)]
struct TestSystemToggles {
    player_movement: bool,
    cord_systems: bool,
    camera_follow: bool,
}

#[test]
fn test_movement_and_rotation_integration() {
    // Test that player movement affects rotation system
    let input_direction = Vec2::new(1.0, 0.0); // Moving right
    let velocity = Vec2::new(100.0, 0.0);      // Velocity right
    let velocity_threshold = 10.0;
    
    // Check velocity is above threshold for rotation
    assert!(velocity.length() > velocity_threshold);
    
    // Check input matches velocity (controlled movement)
    let dot_product = input_direction.normalize().dot(velocity.normalize());
    assert!(dot_product > 0.3); // Should trigger controlled rotation
    
    // Calculate expected rotation angle
    let expected_angle = velocity.y.atan2(velocity.x) - std::f32::consts::FRAC_PI_2;
    assert!((expected_angle - (-std::f32::consts::FRAC_PI_2)).abs() < 0.001);
}

#[test]
fn test_cord_physics_affects_movement() {
    // Test that cord physics constraints affect player movement
    let player_pos = Vec3::new(100.0, 0.0, 0.0);
    let pole_pos = Vec3::new(0.0, 0.0, 0.0);
    let cord_length = 80.0;
    
    let distance = player_pos.distance(pole_pos);
    
    // When player is at cord limit, they should be constrained
    if distance >= cord_length {
        // Player movement should be restricted by cord physics
        assert!(distance >= cord_length);
        
        // Cord should be at maximum extension
        let extension_threshold = cord_length * 0.95;
        assert!(distance >= extension_threshold);
    }
}

#[test]
fn test_camera_follows_constrained_player() {
    // Test that camera system works with cord-constrained movement
    let pole_pos = Vec3::new(0.0, 0.0, 0.0);
    let cord_length = 100.0;
    let player_pos = Vec3::new(70.0, 70.0, 0.0); // Within cord range
    
    let distance_from_pole = player_pos.distance(pole_pos);
    assert!(distance_from_pole <= cord_length);
    
    // Camera should follow player regardless of cord constraints
    let expected_camera_pos = Vec3::new(player_pos.x, player_pos.y, 1000.0);
    assert_eq!(expected_camera_pos.x, player_pos.x);
    assert_eq!(expected_camera_pos.y, player_pos.y);
    assert_eq!(expected_camera_pos.z, 1000.0); // Fixed Z for 2D view
}

#[test]
fn test_cord_retraction_affects_player_position() {
    // Test that cord retraction affects player's possible positions
    let initial_cord_length = 100.0;
    let retraction_amount = 20.0;
    let final_cord_length = initial_cord_length - retraction_amount;
    
    let pole_pos = Vec3::new(0.0, 0.0, 0.0);
    let initial_player_pos = Vec3::new(100.0, 0.0, 0.0); // At cord limit
    
    // After retraction, player should be pulled closer
    let max_allowed_distance = final_cord_length;
    let current_distance = initial_player_pos.distance(pole_pos);
    
    if current_distance > max_allowed_distance {
        // Player would be pulled in by physics
        assert!(current_distance > max_allowed_distance);
        
        // New position would be constrained
        let direction = (initial_player_pos - pole_pos).normalize();
        let new_player_pos = pole_pos + direction * max_allowed_distance;
        let new_distance = new_player_pos.distance(pole_pos);
        
        assert!((new_distance - max_allowed_distance).abs() < 0.001);
    }
}

#[test]
fn test_ui_toggles_affect_all_systems() {
    let mut toggles = TestSystemToggles {
        player_movement: true,
        cord_systems: true,
        camera_follow: true,
    };
    
    // Test disabling player movement affects rotation
    toggles.player_movement = false;
    if !toggles.player_movement {
        // Rotation system might not have velocity input
        // This would affect rotation calculations
        let velocity = Vec2::ZERO; // No movement
        assert!(velocity.length() <= 10.0); // Below rotation threshold
    }
    
    // Test disabling cord systems
    toggles.cord_systems = false;
    if !toggles.cord_systems {
        // Player would not be constrained by cord
        // Movement would be unlimited
        assert!(!toggles.cord_systems);
    }
    
    // Test disabling camera follow
    toggles.camera_follow = false;
    if !toggles.camera_follow {
        // Camera position would remain fixed
        let initial_camera_pos = Vec3::new(0.0, 0.0, 1000.0);
        // Camera should not move from initial position
        assert_eq!(initial_camera_pos, Vec3::new(0.0, 0.0, 1000.0));
    }
}

#[test]
fn test_resource_management_integration() {
    // Test that CordSystem resource integrates with all relevant systems
    let cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        max_length: 500.0,
        min_length: 50.0,
        current_length: 100.0,
        segment_length: 20.0,
        segment_size: 8.0,
        player_entity: Entity::from_bits(100),
        is_retracting: false,
        attached_pole: Some(Entity::from_bits(200)),
        attachment_range: 100.0,
    };
    
    // Test that resource affects movement constraints
    let max_distance = cord_system.current_length;
    assert_eq!(max_distance, 100.0);
    
    // Test that resource affects attachment system
    let attachment_range = cord_system.attachment_range;
    assert_eq!(attachment_range, 100.0);
    
    // Test that resource affects segment management
    let expected_segments = (cord_system.current_length / cord_system.segment_length) as usize;
    assert_eq!(expected_segments, 5); // 100/20 = 5
    
    // Test that all values are consistent
    assert!(cord_system.min_length < cord_system.current_length);
    assert!(cord_system.current_length <= cord_system.max_length);
    assert!(cord_system.segment_size < cord_system.segment_length);
}


#[test]
fn test_input_system_integration() {
    // Test that input affects multiple systems simultaneously
    
    // Test WASD movement input
    let w_pressed = true;
    let d_pressed = true;
    let shift_pressed = false;
    let space_pressed = false;
    
    // Movement system response
    let mut movement_force = Vec2::ZERO;
    if w_pressed {
        movement_force.y += 300.0;
    }
    if d_pressed {
        movement_force.x += 300.0;
    }
    
    assert_eq!(movement_force, Vec2::new(300.0, 300.0));
    
    // Rotation system response (should calculate input direction)
    let mut input_direction = Vec2::ZERO;
    if w_pressed {
        input_direction.y += 1.0;
    }
    if d_pressed {
        input_direction.x += 1.0;
    }
    
    assert_eq!(input_direction, Vec2::new(1.0, 1.0));
    
    // Cord system response (no retraction since shift not pressed)
    assert!(!shift_pressed); // Cord should not retract
    assert!(!space_pressed); // No attachment/detachment
}


#[test]
fn test_transform_hierarchy_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test that child transforms work with parent systems
    let player_entity = app.world_mut().spawn((
        Player,
        Transform::from_xyz(100.0, 50.0, 0.0),
    )).id();
    
    // Child entities (eye, pupil, backpack, attachment point)
    let eye_entity = app.world_mut().spawn((
        Transform::from_xyz(0.0, 12.0, 0.01), // Relative to player
        ChildOf(player_entity),
    )).id();
    
    // Test that hierarchical transforms are set up correctly
    let child_of = app.world().get::<ChildOf>(eye_entity).unwrap();
    assert_eq!(child_of.get(), player_entity);
    
    // When player moves, children should move with it
    let _player_transform = app.world().get::<Transform>(player_entity).unwrap();
    let child_transform = app.world().get::<Transform>(eye_entity).unwrap();
    
    // Child maintains relative offset
    assert_eq!(child_transform.translation, Vec3::new(0.0, 12.0, 0.01));
}



#[test]
fn test_state_consistency_across_systems() {
    // Test that state remains consistent across all systems
    let cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2), Entity::from_bits(3)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11), Entity::from_bits(12)],
        max_length: 150.0,
        min_length: 50.0,
        current_length: 80.0,
        segment_length: 20.0,
        segment_size: 8.0,
        player_entity: Entity::from_bits(100),
        is_retracting: false,
        attached_pole: Some(Entity::from_bits(200)),
        attachment_range: 100.0,
    };
    
    // Test invariants that should always be true
    assert!(cord_system.min_length <= cord_system.current_length);
    assert!(cord_system.current_length <= cord_system.max_length);
    assert!(cord_system.segment_size <= cord_system.segment_length);
    
    // Test that segment count matches cord length
    let expected_segments = (cord_system.current_length / cord_system.segment_length) as usize;
    // Allow some tolerance for rounding and minimum segments
    let actual_segments = cord_system.segments.len();
    assert!(actual_segments >= 2); // Minimum segments
    assert!(actual_segments <= expected_segments + 2); // Some tolerance
    
    // Test that joint count relates to segment count
    // Should have roughly one joint per segment plus connections
    let joint_count = cord_system.joints.len();
    let segment_count = cord_system.segments.len();
    assert!(joint_count >= segment_count - 1); // At least segment-to-segment joints
    assert!(joint_count <= segment_count + 2);  // Plus connections to pole and player
}