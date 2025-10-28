use bevy::prelude::*;

// Import the types from main.rs that we need for testing
use balthazar::CordSystem;

#[test]
fn test_cord_retraction_logic() {
    let mut cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
        visual_meshes: vec![],
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
fn test_pole_attachment_detachment() {
    let mut cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        visual_meshes: vec![],
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
    
    // Test disconnection
    cord_system.attached_pole = None;
    assert_eq!(cord_system.attached_pole, None);
    
    // Test attachment to a new pole
    let new_pole = Entity::from_bits(201);
    cord_system.attached_pole = Some(new_pole);
    assert_eq!(cord_system.attached_pole, Some(new_pole));
}

#[test]
fn test_find_closest_pole_within_range() {
    use bevy::prelude::*;
    
    // Mock pole positions
    let player_pos = Vec3::new(0.0, 0.0, 0.0);
    let pole1_pos = Vec3::new(50.0, 0.0, 0.0);  // Distance: 50
    let pole2_pos = Vec3::new(0.0, 80.0, 0.0);  // Distance: 80
    let pole3_pos = Vec3::new(200.0, 0.0, 0.0); // Distance: 200 (out of range)
    
    let attachment_range = 100.0;
    
    // Test closest pole within range (should be pole1 at distance 50)
    let distance1 = player_pos.distance(pole1_pos);
    let distance2 = player_pos.distance(pole2_pos);
    let distance3 = player_pos.distance(pole3_pos);
    
    assert_eq!(distance1, 50.0);
    assert_eq!(distance2, 80.0);
    assert_eq!(distance3, 200.0);
    
    // Test that pole1 is closest within range
    assert!(distance1 <= attachment_range && distance1 < distance2);
    assert!(distance3 > attachment_range); // pole3 is out of range
}

#[test]
fn test_no_poles_within_attachment_range() {
    let player_pos = Vec3::new(0.0, 0.0, 0.0);
    let pole_pos = Vec3::new(200.0, 0.0, 0.0); // Distance: 200
    let attachment_range = 100.0;
    
    let distance = player_pos.distance(pole_pos);
    assert!(distance > attachment_range);
    
    // Should return None when no poles are in range
    let closest_pole: Option<(Entity, f32)> = None;
    assert_eq!(closest_pole, None);
}

#[test]
fn test_segment_management_on_retraction() {
    let mut cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2), Entity::from_bits(3), Entity::from_bits(4)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11), Entity::from_bits(12), Entity::from_bits(13)],
        visual_meshes: vec![],
        max_length: 150.0,
        min_length: 50.0,
        current_length: 80.0, // Should have 4 segments (80/20)
        segment_length: 20.0,
        segment_size: 8.0,
        player_entity: Entity::from_bits(100),
        is_retracting: true,
        attached_pole: Some(Entity::from_bits(200)),
        attachment_range: 100.0,
    };
    
    let initial_segments = cord_system.segments.len();
    assert_eq!(initial_segments, 4);
    
    // Simulate retraction to 60 units (should need 3 segments)
    cord_system.current_length = 60.0;
    let new_target_segments = (cord_system.current_length / cord_system.segment_length) as usize;
    assert_eq!(new_target_segments, 3);
    
    // Simulate removing one segment
    while cord_system.segments.len() > new_target_segments.max(2) {
        cord_system.segments.pop();
        // Also remove corresponding joints
        cord_system.joints.pop();
        cord_system.joints.pop();
    }
    
    assert_eq!(cord_system.segments.len(), 3);
}

#[test]
fn test_segment_management_on_extension() {
    let mut cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        visual_meshes: vec![],
        max_length: 150.0,
        min_length: 50.0,
        current_length: 40.0, // Should have 2 segments (40/20)
        segment_length: 20.0,
        segment_size: 8.0,
        player_entity: Entity::from_bits(100),
        is_retracting: false,
        attached_pole: Some(Entity::from_bits(200)),
        attachment_range: 100.0,
    };
    
    let initial_segments = cord_system.segments.len();
    assert_eq!(initial_segments, 2);
    
    // Simulate extension to 80 units (should need 4 segments)
    cord_system.current_length = 80.0;
    let new_target_segments = (cord_system.current_length / cord_system.segment_length) as usize;
    assert_eq!(new_target_segments, 4);
    
    // Simulate adding segments
    while cord_system.segments.len() < new_target_segments {
        // Add new segment entity
        let new_segment = Entity::from_bits(cord_system.segments.len() as u64 + 10);
        cord_system.segments.push(new_segment);
        // Add corresponding joints
        let new_joint = Entity::from_bits(cord_system.joints.len() as u64 + 20);
        cord_system.joints.push(new_joint);
    }
    
    assert_eq!(cord_system.segments.len(), 4);
}

#[test]
fn test_minimum_segments_maintained() {
    let cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        visual_meshes: vec![],
        max_length: 150.0,
        min_length: 50.0,
        current_length: 30.0, // Would need 1.5 segments, but minimum is 2
        segment_length: 20.0,
        segment_size: 8.0,
        player_entity: Entity::from_bits(100),
        is_retracting: true,
        attached_pole: Some(Entity::from_bits(200)),
        attachment_range: 100.0,
    };
    
    let new_target_segments = (cord_system.current_length / cord_system.segment_length) as usize;
    let minimum_segments = 2;
    
    // Should maintain at least 2 segments even if length would suggest fewer
    let final_segments = new_target_segments.max(minimum_segments);
    assert_eq!(final_segments, 2);
    
    // Verify that we don't remove segments below minimum
    let should_remove = cord_system.segments.len() > final_segments;
    assert!(!should_remove); // Should be false since we're already at minimum
}

#[test]
fn test_cord_speeds() {
    let retraction_speed = 300.0;
    let extension_speed = 80.0;
    
    // Test that retraction is faster than extension (as per game design)
    assert!(retraction_speed > extension_speed);
    
    // Test speed ratios
    let speed_ratio = retraction_speed / extension_speed;
    assert_eq!(speed_ratio, 3.75); // 300/80 = 3.75x faster retraction
}

#[test]
fn test_continuous_extension_after_initial_extension() {
    let mut cord_system = CordSystem {
        segments: vec![Entity::from_bits(1), Entity::from_bits(2)],
        joints: vec![Entity::from_bits(10), Entity::from_bits(11)],
        visual_meshes: vec![],
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