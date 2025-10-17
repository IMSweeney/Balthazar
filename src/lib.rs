use bevy::prelude::*;

pub mod cord_system;

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

#[derive(Resource)]
pub struct CordSystem {
    pub segments: Vec<Entity>,
    pub joints: Vec<Entity>,
    pub max_length: f32,
    pub min_length: f32,
    pub current_length: f32,
    pub segment_length: f32,
    pub segment_size: f32,
    pub player_entity: Entity,
    pub is_retracting: bool,
    pub attached_pole: Option<Entity>, // Now refers to the attachment point entity
    pub attachment_range: f32,
}

// Re-export cord system functions for convenience
pub use cord_system::*;
