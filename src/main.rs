use avian2d::prelude::*;
use bevy::prelude::*;

// Use types and functions from lib.rs
use balthazar::{Player, Pole, CordSegment, CordSystem, PoleAttachment, handle_cord_retraction, handle_cord_attachment};

// Marker component for the cord attachment point
#[derive(Component)]
struct CordAttachment;

// Resource to track which systems are enabled
#[derive(Resource)]
struct SystemToggles {
    player_movement: bool,
    cord_systems: bool,
    camera_follow: bool,
    camera_zoom: bool,
    player_rotation: bool,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec2::ZERO)) // No gravity for top-down view
        .insert_resource(SystemToggles {
            player_movement: true,
            cord_systems: true,
            camera_follow: true,
            camera_zoom: true,
            player_rotation: true,
        })
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (
            move_player,
            cord_retraction_wrapper,
            cord_attachment_wrapper,
            camera_follow_player,
            camera_zoom,
            rotate_player,
            update_ui,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera with orthographic projection for isometric view
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5, // Zoom out to see more of the scene
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));

    // Spawn multiple poles for testing
    let pole_positions = vec![
        Vec3::new(0., 0., 0.), // Center pole
        Vec3::new(300., 200., 0.),
        Vec3::new(-300., 150., 0.),
        Vec3::new(200., -250., 0.),
        Vec3::new(-150., -200., 0.),
    ];
    
    let mut pole_entities = Vec::new();
    let mut pole_attachment_entities = Vec::new();
    for (i, pos) in pole_positions.iter().enumerate() {
        // Main pole shaft
        let pole_entity = commands.spawn((
            Sprite {
                color: if i == 0 { 
                    Color::srgb(0.4, 0.3, 0.2) // Dark brown for center pole
                } else { 
                    Color::srgb(0.5, 0.4, 0.3) // Lighter brown for other poles
                },
                custom_size: Some(Vec2::new(15.0, 80.0)), // Tall and narrow like a power pole
                ..default()
            },
            Transform::from_translation(*pos),
            RigidBody::Static,
            Collider::rectangle(15.0, 80.0), // Match the sprite size
            Pole,
        )).id();
        pole_entities.push(pole_entity);
        
        // Create an invisible attachment point at the base of each pole
        let attachment_pos = Vec3::new(pos.x, pos.y - 40.0, pos.z); // 40 units below pole center (base)
        let attachment_entity = commands.spawn((
            Transform::from_translation(attachment_pos),
            RigidBody::Static,
            Collider::circle(1.0), // Small invisible collider for physics attachment
            PoleAttachment { pole_entity },
        )).id();
        pole_attachment_entities.push(attachment_entity);
        
        // Add crossbar at the top to make it look more like a power pole
        commands.spawn((
            Sprite {
                color: if i == 0 { 
                    Color::srgb(0.3, 0.2, 0.1) // Darker crossbar for center pole
                } else { 
                    Color::srgb(0.4, 0.3, 0.2) // Matching crossbar for other poles
                },
                custom_size: Some(Vec2::new(60.0, 8.0)), // Wide horizontal crossbar
                ..default()
            },
            Transform::from_translation(Vec3::new(pos.x, pos.y + 30.0, pos.z)), // Position above pole
        ));
    }
    
    let center_pole_attachment = pole_attachment_entities[0]; // First pole attachment is the center one

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
            Transform::from_translation(Vec3::new(x, 0., 0.)), // At ground level
            RigidBody::Dynamic,
            Collider::circle(segment_size / 2.0),
            LinearDamping(3.0), // Much higher damping to reduce jitter
            AngularDamping(3.0),
            CordSegment,
        )).id();
        
        cord_entities.push(cord_entity);
    }

    // Spawn player at end of rope as a true circle using mesh
    let player_entity = commands.spawn((
        Mesh2d(meshes.add(Circle::new(20.0))), // True circular mesh with 20px radius
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.2, 0.4, 0.8)))), // Blue material
        Transform::from_translation(Vec3::new(max_cord_length, 0., 0.)), // Player at same level
        RigidBody::Dynamic,
        Collider::circle(20.0), // Circular collider to match the circular mesh
        LinearDamping(1.5), // Higher damping for stability
        AngularDamping(2.0), // Higher angular damping
        Player,
    )).id();
    
    // Add an eye to the player (positioned towards the "top" initially)
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0), // White eye
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 12.0, 0.01)), // Relative position: 12 pixels above center
        ChildOf(player_entity), // Make the eye a child of the player so it rotates with it
    ));
    
    // Add a smaller black pupil inside the eye
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0), // Black pupil
            custom_size: Some(Vec2::new(4.0, 4.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 12.0, 0.02)), // Same relative position as eye, but higher Z
        ChildOf(player_entity),
    ));
    
    // Add visible backpack sprite on the back of the player (purely visual)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.2, 0.1), // Dark brown backpack
            custom_size: Some(Vec2::new(8.0, 6.0)), // Small rectangular box
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -12.0, 0.01)), // Relative position: 12 pixels behind center
        ChildOf(player_entity), // Make it a child of the player so it rotates with them
    ));
    
    // Create a separate invisible physics attachment point for the cord
    let cord_attachment = commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -12.0, 0.0)), // Same relative position as backpack
        RigidBody::Dynamic,
        Collider::circle(2.0), // Small invisible collider for physics
        LinearDamping(10.0), // High damping to keep it stable
        AngularDamping(10.0),
        CordAttachment, // Marker component
        ChildOf(player_entity), // Also a child of the player
    )).id();

    // Connect pole attachment point to first cord segment
    if let Some(&first_cord) = cord_entities.first() {
        let joint = commands.spawn(
            DistanceJoint::new(center_pole_attachment, first_cord)
                .with_limits(segment_length * 0.98, segment_length * 1.02) // Very tight limits for stability
        ).id();
        joint_entities.push(joint);
    }

    // Connect cord segments to each other with very stiff joints
    for i in 0..cord_entities.len() - 1 {
        let joint = commands.spawn(
            DistanceJoint::new(cord_entities[i], cord_entities[i + 1])
                .with_limits(segment_length * 0.99, segment_length * 1.01) // Very tight limits for stability
        ).id();
        joint_entities.push(joint);
    }

    // Connect last cord segment to player's attachment point
    if let Some(&last_cord) = cord_entities.last() {
        let joint = commands.spawn(
            DistanceJoint::new(last_cord, cord_attachment)
                .with_limits(segment_length * 0.98, segment_length * 1.02) // Tight limits for stability
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
        attached_pole: Some(center_pole_attachment), // Initially attached to center pole attachment
        attachment_range: 100.0, // Can attach to poles within 100 units
    });
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_movement { return; }
    if let Ok(mut velocity) = player_query.single_mut() {
        let mut force = Vec2::ZERO;
        let move_speed = 300.0;

        // Map keys to isometric movement
        // A/D moves along X axis
        // W/S moves along Z axis (depth)
        if keyboard_input.pressed(KeyCode::KeyA) {
            force.x -= move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            force.x += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            // Move "forward" in isometric view (negative Z)
            // Since we're using 2D physics, we map this to Y
            force.y += move_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            // Move "backward" in isometric view (positive Z)
            force.y -= move_speed;
        }

        // Apply force as velocity (since we have no gravity, this works well)
        velocity.0 = force;
    }
}

fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.camera_follow { return; }
    if let (Ok(player_transform), Ok(mut camera_transform)) = (player_query.single(), camera_query.single_mut()) {
        // Smoothly follow the player with some offset for better view
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
        // Keep camera Z position fixed for 2D view
    }
}

fn camera_zoom(
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.camera_zoom { return; }
    for event in scroll_events.read() {
        if let Ok(mut projection) = camera_query.single_mut() {
            if let Projection::Orthographic(ref mut ortho) = projection.as_mut() {
                // Zoom in/out based on scroll direction
                let zoom_factor = 1.1;
                match event.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => {
                        if event.y > 0.0 {
                            // Scroll up - zoom in
                            ortho.scale /= zoom_factor;
                        } else {
                            // Scroll down - zoom out
                            ortho.scale *= zoom_factor;
                        }
                    }
                    bevy::input::mouse::MouseScrollUnit::Pixel => {
                        let zoom_change = -event.y * 0.001;
                        ortho.scale *= (1.0 + zoom_change).max(0.1);
                    }
                }
                
                // Clamp zoom to reasonable limits
                ortho.scale = ortho.scale.clamp(0.1, 10.0);
            }
        }
    }
}

fn rotate_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &LinearVelocity), With<Player>>,
    toggles: Res<SystemToggles>,
) {
    if !toggles.player_rotation { return; }
    for (mut transform, velocity) in player_query.iter_mut() {
        let vel = velocity.0;
        
        // Only rotate if the player is moving with significant velocity
        if vel.length() > 10.0 {
            // Calculate player input direction
            let mut input_direction = Vec2::ZERO;
            if keyboard_input.pressed(KeyCode::KeyA) {
                input_direction.x -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                input_direction.x += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyW) {
                input_direction.y += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                input_direction.y -= 1.0;
            }
            
            let is_player_controlled = input_direction.length() > 0.1;
            
            let target_angle = if is_player_controlled {
                // Check if input direction roughly matches velocity direction
                let input_normalized = input_direction.normalize();
                let velocity_normalized = vel.normalize();
                let dot_product = input_normalized.dot(velocity_normalized);
                
                if dot_product > 0.3 { // Player is moving in the direction they're trying to go
                    // Face the direction of movement when successfully controlling motion
                    vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2
                } else {
                    // Player is fighting against physics (like being pulled by cord while trying to move away)
                    // Face the direction they're trying to go (input direction)
                    input_direction.y.atan2(input_direction.x) - std::f32::consts::FRAC_PI_2
                }
            } else {
                // Face opposite to movement direction when being pulled by cord/physics
                // Add π to reverse the direction
                vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2 + std::f32::consts::PI
            };
            
            // Get current rotation angle
            let current_rotation = transform.rotation;
            let current_angle = current_rotation.to_euler(bevy::math::EulerRot::ZYX).0;
            
            // Calculate the shortest angular distance
            let mut angle_diff = target_angle - current_angle;
            // Wrap to [-π, π] range for shortest rotation
            while angle_diff > std::f32::consts::PI {
                angle_diff -= 2.0 * std::f32::consts::PI;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += 2.0 * std::f32::consts::PI;
            }
            
            // Rotation speed (radians per second)
            let rotation_speed = if is_player_controlled { 4.0 } else { 2.0 }; // Slower when transitioning to physics motion
            let max_rotation_this_frame = rotation_speed * time.delta_secs();
            
            // Clamp the rotation change to the maximum allowed this frame
            let rotation_change = angle_diff.clamp(-max_rotation_this_frame, max_rotation_this_frame);
            let new_angle = current_angle + rotation_change;
            
            // Apply the smoothed rotation
            transform.rotation = Quat::from_rotation_z(new_angle);
        }
    }
}

fn cord_retraction_wrapper(
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

fn cord_attachment_wrapper(
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

fn setup_ui(mut commands: Commands) {
    // Create UI root node
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(200.0),
            height: Val::Px(300.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // Semi-transparent background
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("System Controls"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // Toggle buttons
        let buttons = [
            ("Player Movement", "player_movement"),
            ("Cord Systems", "cord_systems"),
            ("Camera Follow", "camera_follow"),
            ("Camera Zoom", "camera_zoom"),
            ("Player Rotation", "player_rotation"),
        ];
        
        for (label, system_name) in buttons {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                // Label
                row.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                
                // Toggle button
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(50.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)), // Green for enabled
                    ToggleButton(system_name.to_string()),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("ON"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        }
    });
}


#[derive(Component)]
struct ToggleButton(String);

fn update_ui(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ToggleButton, &Children),
        (Changed<Interaction>, With<Button>)
    >,
    mut text_query: Query<&mut Text>,
    mut toggles: ResMut<SystemToggles>,
) {
    for (interaction, mut color, toggle_button, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Get current state
            let current_state = match toggle_button.0.as_str() {
                "player_movement" => toggles.player_movement,
                "cord_systems" => toggles.cord_systems,
                "camera_follow" => toggles.camera_follow,
                "camera_zoom" => toggles.camera_zoom,
                "player_rotation" => toggles.player_rotation,
                _ => true,
            };
            
            // Toggle the state
            let new_state = !current_state;
            match toggle_button.0.as_str() {
                "player_movement" => toggles.player_movement = new_state,
                "cord_systems" => toggles.cord_systems = new_state,
                "camera_follow" => toggles.camera_follow = new_state,
                "camera_zoom" => toggles.camera_zoom = new_state,
                "player_rotation" => toggles.player_rotation = new_state,
                _ => {},
            }
            
            // Update button appearance
            if new_state {
                *color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2)); // Green for enabled
                if let Ok(mut text) = text_query.get_mut(children[0]) {
                    **text = "ON".to_string();
                }
            } else {
                *color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2)); // Red for disabled
                if let Ok(mut text) = text_query.get_mut(children[0]) {
                    **text = "OFF".to_string();
                }
            }
        }
    }
}



