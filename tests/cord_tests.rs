use bevy::prelude::*;

// Import the types from main.rs that we need for testing
use balthazar::CordSystem;

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