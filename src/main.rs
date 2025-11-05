use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_light_2d::prelude::*;
fn handle_reset_button() {
    // TODO: Implement reset button logic
}
// Use types and functions from lib.rs
use balthazar::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TiledPlugin::default(),
            Light2dPlugin,
        ))
        .add_plugins(TiledDebugPluginGroup)
        .insert_resource(SystemToggles {
            player_movement: true,
            cord_systems: true,
            camera_follow: true,
            camera_zoom: true,
            player_rotation: true,
        })
        .insert_resource(DayNightCycle::default()) // Initialize day/night cycle
        .add_systems(Startup, (setup, setup_ui, load_tiled_map))
        .add_systems(Update, (
            move_player,
            grid_movement_system,
            cord_retraction_wrapper,
            cord_attachment_wrapper,
            update_player_sprite_direction,
            update_ui,
            update_battery_display,
            transfer_power,
            solar_charge,
            handle_reset_button,
            update_day_night_cycle,
            apply_day_night_lighting,
            update_sky_color,
        ))
        .add_systems(Update, update_cord_trail.before(render_cord_meshes))
        .add_systems(Update, render_cord_meshes)
        .add_systems(Update, (
            camera_follow_player,
            camera_zoom,
        ))
        .run();
}

