use avian2d::prelude::*;
use bevy::prelude::*;
use crate::components::{Battery, Player, CordSegment, CordSystem, CordMaterial, SolarPanel};

// Z-layer constants for rendering depth
pub const Z_GAME_LAYER: f32 = 1.0;
pub const Z_GAME_LAYER_DETAIL: f32 = 2.0;  // For elements like eyes, backpacks
pub const Z_GAME_LAYER_DETAIL_2: f32 = 3.0; // For elements that need to be above other details


pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    
    // Create cord texture: black with thin purple line running along length
    let texture_width = 32;  // Width is along the cord length
    let texture_height = 8;  // Height is across the cord width
    let mut texture_data = vec![0u8; (texture_width * texture_height * 4) as usize];
    
    for y in 0..texture_height {
        for x in 0..texture_width {
            let idx = ((y * texture_width + x) * 4) as usize;
            
            // Check if this is the center line (thin purple line running horizontally in texture)
            if y == texture_height / 2 || y == texture_height / 2 - 1 {
                // Purple line
                texture_data[idx] = 128;     // R
                texture_data[idx + 1] = 0;   // G
                texture_data[idx + 2] = 128; // B
                texture_data[idx + 3] = 255; // A
            } else {
                // Black background
                texture_data[idx] = 0;       // R
                texture_data[idx + 1] = 0;   // G
                texture_data[idx + 2] = 0;   // B
                texture_data[idx + 3] = 255; // A
            }
        }
    }
    
    let cord_texture = images.add(Image::new(
        bevy::render::render_resource::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        texture_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    ));
    
    let cord_material = materials.add(ColorMaterial::from(cord_texture));
    
    commands.insert_resource(CordMaterial {
        material: cord_material,
    });
    
    // Spawn 2D camera with orthographic projection for isometric view
    commands.spawn(Camera2d);

    // Create cord segments connecting pole to player
    let max_cord_length = 500.0; // Much longer cord
    let min_cord_length = 50.0;
    let segment_length = 10.0; // Fixed length per segment
    let segment_size = 4.0; // Visual size of each segment
    let initial_num_segments = 2;
    
    let mut cord_entities = Vec::new();
    let mut joint_entities = Vec::new();
    
    // Create cord segments
    for i in 0..initial_num_segments {
        let x = (i + 1) as f32 * segment_length;
        let cord_entity = commands.spawn((
            // Make cord segments invisible - we'll render lines instead
            Sprite {
                color: Color::NONE, // Invisible
                custom_size: Some(Vec2::new(segment_size, segment_size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, 0., Z_GAME_LAYER)), // At ground level
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
        Transform::from_translation(Vec3::new(0., 0., Z_GAME_LAYER)), // Player at origin
        RigidBody::Dynamic,
        Collider::circle(20.0), // Circular collider to match the circular mesh
        LinearDamping(1.5), // Higher damping for stability
        AngularDamping(2.0), // Higher angular damping
        Player,
        Battery {
            max_charge: 100.0,
            current_charge: 100.0,
        },
        SolarPanel {
            max_output: 5.0, // 5 charge per second at full brightness (slower than pole charging)
        },
    )).id();
    
    // Add an eye to the player (positioned towards the "top" initially)
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0), // White eye
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 12.0, Z_GAME_LAYER_DETAIL)), // Relative position: 12 pixels above center
        ChildOf(player_entity), // Make the eye a child of the player so it rotates with it
    ));
    
    // Add a smaller black pupil inside the eye
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0), // Black pupil
            custom_size: Some(Vec2::new(4.0, 4.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 12.0, Z_GAME_LAYER_DETAIL_2)), // Same relative position as eye, but higher Z
        ChildOf(player_entity),
    ));
    
    // Add visible backpack sprite on the back of the player (purely visual)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.2, 0.1), // Dark brown backpack
            custom_size: Some(Vec2::new(8.0, 6.0)), // Small rectangular box
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -12.0, Z_GAME_LAYER_DETAIL)), // Relative position: 12 pixels behind center
        ChildOf(player_entity), // Make it a child of the player so it rotates with them
    ));

    // Connect cord segments to each other with very stiff joints
    for i in 0..cord_entities.len() - 1 {
        let joint = commands.spawn(
            DistanceJoint::new(cord_entities[i], cord_entities[i + 1])
                .with_limits(segment_length * 0.99, segment_length * 1.01) // Very tight limits for stability
        ).id();
        joint_entities.push(joint);
    }

    // Connect last cord segment to player's attachment point (at backpack)
    if let Some(&last_chord) = cord_entities.last() {
        let joint = commands.spawn(
            FixedJoint::new(last_chord, player_entity)
                .with_local_anchor2(Vec2::new(0.0, -12.0)) // Attachment point at backpack, rotates with player body
        ).id();
        joint_entities.push(joint);
    }

    // Initialize cord system resource
    let initial_length = initial_num_segments as f32 * segment_length;
    commands.insert_resource(CordSystem {
        segments: cord_entities,
        joints: joint_entities,
        visual_meshes: Vec::new(), // Will be populated by render system
        max_length: max_cord_length,
        min_length: min_cord_length,
        current_length: initial_length, // Start with actual length based on initial segments
        segment_length,
        segment_size,
        player_entity,
        is_retracting: false,
        attached_pole: None,
        attachment_range: 100.0, // Can attach to poles within 100 units
    });
}
