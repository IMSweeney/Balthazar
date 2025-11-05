use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use crate::components::{Battery, Player, CordSystem, CordMaterial, SolarPanel, PlayerSprite, PlayerDirection, GridMovement, IsometricGrid};

// Z-layer constants for rendering depth
// Tilemap is typically at Z = 0-100 range
pub const Z_CORD_LAYER: f32 = 150.0;  // Above tilemap, below player
pub const Z_GAME_LAYER: f32 = 200.0;  // Player and game entities

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
    
    // Initialize isometric grid (64 pixel tiles to match sprite size)
    commands.insert_resource(IsometricGrid {
        tile_size: 64.0,
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

    // Convert isometric grid coordinates to world positions
    let tile_size = 64.0;
    let grid_to_world = |grid_x: f32, grid_y: f32| -> Vec2 {
        let x = (grid_x - grid_y) * tile_size / 2.0;
        let y = (grid_x + grid_y) * tile_size / 4.0;
        Vec2::new(x, y)
    };
    
    // Spawn player at grid (0, 0)
    let player_pos = grid_to_world(0.0, 0.0);
    
    // Load robot spritesheet
    let texture_handle = asset_server.load("robo.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 1, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    
    // Spawn player
    let player_entity = commands.spawn((
        Sprite::from_atlas_image(
            texture_handle,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 3, // Start with up direction (swapped mapping)
            },
        ),
        Transform::from_translation(player_pos.extend(Z_GAME_LAYER)),
        Player,
        PlayerSprite {
            current_direction: PlayerDirection::Up,
        },
        GridMovement {
            target_position: None,
            move_speed: 400.0,
            is_moving: false,
        },
        Battery {
            max_charge: 100.0,
            current_charge: 100.0,
        },
        SolarPanel {
            max_output: 5.0,
        },
        PointLight2d {
            radius: 200.0,
            intensity: 0.8,
            color: Color::srgb(1.0, 0.9, 0.7),
            cast_shadows: true,
            ..default()
        },
    )).id();
    
    // Initialize cord system with no attachment
    commands.insert_resource(CordSystem {
        visual_meshes: Vec::new(),
        player_entity,
        attached_pole: None,
        attachment_range: 100.0,
        trail_path: Vec::new(),
        min_trail_segment_distance: 10.0,
    });
}
