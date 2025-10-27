use bevy::prelude::*;
use crate::components::{ToggleButton, SystemToggles};

pub fn setup_ui(mut commands: Commands) {
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

pub fn update_ui(
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
