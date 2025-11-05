use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerSprite {
    pub current_direction: PlayerDirection,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayerDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct GridMovement {
    pub target_position: Option<Vec2>,
    pub move_speed: f32,
    pub is_moving: bool,
}

#[derive(Resource)]
pub struct IsometricGrid {
    pub tile_size: f32, // Size of each grid tile
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Battery {
    pub max_charge: f32,
    pub current_charge: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Pole;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PowerSource {
    pub max_output: f32, // Power per second (infinite source for now)
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SolarPanel {
    pub max_output: f32, // Maximum power per second at full brightness
}

#[derive(Component)]
pub struct CordSegment;

#[derive(Component)]
pub struct CordAttachment;

#[derive(Component)]
pub struct AttachmentPoint {
    pub parent_pole: Entity,
}

#[derive(Component)]
pub struct ToggleButton(pub String);

#[derive(Resource)]
pub struct CordSystem {
    pub visual_meshes: Vec<Entity>, // Visual mesh entities for rendering cord
    pub player_entity: Entity,
    pub attached_pole: Option<Entity>,
    pub attachment_range: f32,
    pub trail_path: Vec<Vec2>, // Path of cord trail laid on ground
    pub min_trail_segment_distance: f32, // Minimum distance before adding new trail point
}

#[derive(Resource)]
pub struct SystemToggles {
    pub player_movement: bool,
    pub cord_systems: bool,
    pub camera_follow: bool,
    pub camera_zoom: bool,
    pub player_rotation: bool,
}

#[derive(Resource)]
pub struct CordMaterial {
    pub material: Handle<ColorMaterial>,
}
