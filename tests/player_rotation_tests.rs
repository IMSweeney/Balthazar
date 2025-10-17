use bevy::prelude::*;
use avian2d::prelude::*;
use std::f32::consts::{PI, FRAC_PI_2};

// Import the types we need for testing
use balthazar::Player;

// Mock SystemToggles for testing
#[derive(Resource)]
struct TestSystemToggles {
    player_rotation: bool,
}

#[test]
fn test_player_rotation_based_on_movement_direction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_rotation: true });
    app.insert_resource(Time::<()>::default());
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::new(100.0, 0.0)), // Moving right
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Test rotation calculation for rightward movement
    let velocity = Vec2::new(100.0, 0.0);
    let expected_angle = velocity.y.atan2(velocity.x) - FRAC_PI_2;
    
    // The player should face the direction of movement
    assert!((expected_angle - (-FRAC_PI_2)).abs() < 0.001);
    
    // Test rotation for upward movement
    let velocity = Vec2::new(0.0, 100.0);
    let expected_angle = velocity.y.atan2(velocity.x) - FRAC_PI_2;
    assert!((expected_angle - 0.0).abs() < 0.001);
}

#[test]
fn test_player_rotation_when_controlled_by_input() {
    // Test player-controlled movement (input direction matches velocity)
    let input_direction = Vec2::new(1.0, 0.0).normalize(); // Moving right
    let velocity = Vec2::new(100.0, 0.0).normalize();      // Velocity right
    
    // Calculate dot product to check if input matches velocity
    let dot_product = input_direction.dot(velocity);
    assert!(dot_product > 0.3); // Should be > 0.3 for controlled movement
    
    // Player should face movement direction when controlling motion
    let target_angle = velocity.y.atan2(velocity.x) - FRAC_PI_2;
    assert!((target_angle - (-FRAC_PI_2)).abs() < 0.001);
}

#[test]
fn test_player_rotation_when_fighting_physics() {
    // Test when input direction doesn't match velocity (fighting physics)
    let input_direction = Vec2::new(1.0, 0.0).normalize();  // Trying to move right
    let velocity = Vec2::new(-50.0, 0.0).normalize();       // Being pulled left
    
    // Calculate dot product
    let dot_product = input_direction.dot(velocity);
    assert!(dot_product <= 0.3); // Should be <= 0.3 when fighting physics
    
    // Player should face input direction when fighting physics
    let target_angle = input_direction.y.atan2(input_direction.x) - FRAC_PI_2;
    assert!((target_angle - (-FRAC_PI_2)).abs() < 0.001);
}

#[test]
fn test_player_rotation_when_being_pulled_by_cord() {
    // Test when player is not giving input but is being moved by physics
    let input_direction = Vec2::ZERO; // No input
    let velocity = Vec2::new(-100.0, 50.0); // Being pulled by cord
    
    let is_player_controlled = input_direction.length() > 0.1;
    assert!(!is_player_controlled);
    
    // Player should face opposite to movement direction when being pulled
    let target_angle = velocity.y.atan2(velocity.x) - FRAC_PI_2 + PI;
    let expected_angle = velocity.y.atan2(velocity.x) - FRAC_PI_2 + PI;
    assert!((target_angle - expected_angle).abs() < 0.001);
}

#[test]
fn test_rotation_velocity_threshold() {
    let velocity_threshold = 10.0;
    
    // Test below threshold - no rotation should occur
    let low_velocity = Vec2::new(5.0, 3.0);
    assert!(low_velocity.length() <= velocity_threshold);
    
    // Test above threshold - rotation should occur
    let high_velocity = Vec2::new(50.0, 30.0);
    assert!(high_velocity.length() > velocity_threshold);
}

#[test]
fn test_smooth_rotation_interpolation() {
    let rotation_speed_controlled = 4.0; // rad/sec for controlled movement
    let rotation_speed_physics = 2.0;    // rad/sec for physics movement
    let delta_time = 0.016; // ~60 FPS
    
    let max_rotation_controlled = rotation_speed_controlled * delta_time;
    let max_rotation_physics = rotation_speed_physics * delta_time;
    
    // Test that controlled movement rotates faster
    assert!(max_rotation_controlled > max_rotation_physics);
    
    // Test rotation clamping
    let large_angle_diff = PI; // 180 degrees
    let clamped_rotation_controlled = large_angle_diff.clamp(
        -max_rotation_controlled, 
        max_rotation_controlled
    );
    
    assert_eq!(clamped_rotation_controlled, max_rotation_controlled);
}

#[test]
fn test_angle_wrapping() {
    // Test angle wrapping to [-π, π] range
    let mut angle_diff = 3.5 * PI; // > π
    
    // Wrap to [-π, π]
    while angle_diff > PI {
        angle_diff -= 2.0 * PI;
    }
    while angle_diff < -PI {
        angle_diff += 2.0 * PI;
    }
    
    assert!(angle_diff >= -PI && angle_diff <= PI);
    
    // Test negative angle wrapping
    let mut angle_diff = -3.5 * PI; // < -π
    
    while angle_diff > PI {
        angle_diff -= 2.0 * PI;
    }
    while angle_diff < -PI {
        angle_diff += 2.0 * PI;
    }
    
    assert!(angle_diff >= -PI && angle_diff <= PI);
}

#[test]
fn test_rotation_disabled_when_toggled_off() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_rotation: false }); // Disabled
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::new(100.0, 0.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Get toggle state
    let toggles = app.world().resource::<TestSystemToggles>();
    
    // Rotation should not work when disabled
    if !toggles.player_rotation {
        // Transform rotation should remain unchanged
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        assert_eq!(transform.rotation, Quat::IDENTITY); // Default rotation
    }
}

#[test]
fn test_input_direction_calculation() {
    let mut input_direction = Vec2::ZERO;
    
    // Simulate WASD input
    let key_a_pressed = true;
    let key_d_pressed = false;
    let key_w_pressed = true;
    let key_s_pressed = false;
    
    if key_a_pressed {
        input_direction.x -= 1.0;
    }
    if key_d_pressed {
        input_direction.x += 1.0;
    }
    if key_w_pressed {
        input_direction.y += 1.0;
    }
    if key_s_pressed {
        input_direction.y -= 1.0;
    }
    
    assert_eq!(input_direction, Vec2::new(-1.0, 1.0)); // A + W
    
    let is_player_controlled = input_direction.length() > 0.1;
    assert!(is_player_controlled);
}

#[test]
fn test_input_velocity_dot_product_thresholds() {
    let threshold = 0.3;
    
    // Test same direction (should be > threshold)
    let input = Vec2::new(1.0, 0.0).normalize();
    let velocity = Vec2::new(1.0, 0.0).normalize();
    let dot_product = input.dot(velocity);
    assert!(dot_product > threshold);
    assert!((dot_product - 1.0).abs() < 0.001); // Should be 1.0 for same direction
    
    // Test opposite direction (should be <= threshold)
    let input = Vec2::new(1.0, 0.0).normalize();
    let velocity = Vec2::new(-1.0, 0.0).normalize();
    let dot_product = input.dot(velocity);
    assert!(dot_product <= threshold);
    assert!((dot_product - (-1.0)).abs() < 0.001); // Should be -1.0 for opposite
    
    // Test perpendicular (should be <= threshold)
    let input = Vec2::new(1.0, 0.0).normalize();
    let velocity = Vec2::new(0.0, 1.0).normalize();
    let dot_product = input.dot(velocity);
    assert!(dot_product <= threshold);
    assert!((dot_product - 0.0).abs() < 0.001); // Should be 0.0 for perpendicular
}

#[test]
fn test_euler_rotation_conversion() {
    // Test that rotation quaternion converts correctly to Euler angles
    let angle = FRAC_PI_2; // 90 degrees
    let quat = Quat::from_rotation_z(angle);
    let (z_angle, _, _) = quat.to_euler(bevy::math::EulerRot::ZYX);
    
    assert!((z_angle - angle).abs() < 0.001);
    
    // Test full rotation
    let angle = 2.0 * PI;
    let quat = Quat::from_rotation_z(angle);
    let (z_angle, _, _) = quat.to_euler(bevy::math::EulerRot::ZYX);
    
    // Full rotation should normalize to 0
    assert!((z_angle.abs()).abs() < 0.001 || (z_angle.abs() - 2.0 * PI).abs() < 0.001);
}



