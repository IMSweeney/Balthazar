use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub fn load_tiled_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TilemapAnchor::Center,
        // For isometric maps, it can be useful to tweak `bevy_ecs_tilemap` render settings.
        // [`TilemapRenderSettings`] provides the `y_sort`` parameter to sort chunks using their y-axis
        // position during rendering.
        // However, it applies to whole chunks, not individual tile, so we have to force the chunk
        // size to be exactly one tile along the y-axis.
        TilemapRenderSettings {
            render_chunk_size: UVec2::new(1, 1),
            y_sort: true,
        },
    ))
    .observe(
            |trigger: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(trigger.event().origin)
                    .insert(RigidBody::Static);
            },
        );
}
