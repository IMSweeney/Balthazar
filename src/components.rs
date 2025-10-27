use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Pole;

#[derive(Component)]
pub struct CordSegment;

#[derive(Component)]
pub struct PoleAttachment {
    pub pole_entity: Entity,
}

#[derive(Component)]
pub struct CordAttachment;

#[derive(Component)]
pub struct ToggleButton(pub String);

#[derive(Resource)]
pub struct CordSystem {
    pub segments: Vec<Entity>,
    pub joints: Vec<Entity>,
    pub visual_meshes: Vec<Entity>, // Visual mesh entities for rendering cord segments
    pub max_length: f32,
    pub min_length: f32,
    pub current_length: f32,
    pub segment_length: f32,
    pub segment_size: f32,
    pub player_entity: Entity,
    pub is_retracting: bool,
    pub attached_pole: Option<Entity>,
    pub attachment_range: f32,
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
