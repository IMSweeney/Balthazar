use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use crate::components::{Battery, Player, CordSegment, CordSystem, CordMaterial, SolarPanel, PlayerSprite, PlayerDirection};

// Z-layer constants for rendering depth
pub const Z_GAME_LAYER: f32 = 1.0;
pub const Z_GAME_LAYER_DETAIL: f32 = 2.0;  // For elements like eyes, backpacks
pub const Z_GAME_LAYER_DETAIL_2: f32 = 3.0; // For elements that need to be above other details


pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 0.25;
    commands.spawn((
        Camera2d,
        Projection::Orthographic(projection),
        Light2d {
            ambient_light: AmbientLight2d {
                brightness: 0.1,
                ..default()
            },
        },
    ));

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

    // Load robot spritesheet (64x256 = 4 sprites of 64x64 stacked vertically)
    let texture_handle = asset_server.load("robo.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 1, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    
    // Spawn player with sprite from robo.png
    let player_entity = commands.spawn((
        Sprite::from_atlas_image(
            texture_handle,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, // Start with up direction
            },
        ),
        Transform::from_translation(Vec3::new(0., 0., Z_GAME_LAYER)), // Player at origin
        RigidBody::Dynamic,
        Collider::circle(20.0), // Circular collider sized for 64x64 sprite
        LinearDamping(1.5), // Higher damping for stability
        AngularDamping(2.0), // Higher angular damping
        Player,
        PlayerSprite {
            current_direction: PlayerDirection::Up,
        },
        Battery {
            max_charge: 100.0,
            current_charge: 100.0,
        },
        SolarPanel {
            max_output: 5.0, // 5 charge per second at full brightness (slower than pole charging)
        },
        PointLight2d {
            radius: 200.0,
            intensity: 0.8,
            color: Color::srgb(1.0, 0.9, 0.7), // Warm white light
            cast_shadows: true,
            ..default()
        },
    )).id();

    // Connect cord segments to each other with very stiff joints
    for i in 0..cord_entities.len() - 1 {
        let joint = commands.spawn(
            DistanceJoint::new(cord_entities[i], cord_entities[i + 1])
                .with_limits(segment_length * 0.99, segment_length * 1.01) // Very tight limits for stability
        ).id();
        joint_entities.push(joint);
    }

    // Connect last cord segment to player center (sprite handles visual offset)
    if let Some(&last_chord) = cord_entities.last() {
        let joint = commands.spawn(
            FixedJoint::new(last_chord, player_entity)
                .with_local_anchor2(Vec2::new(0.0, 0.0)) // Attachment point at player center
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
