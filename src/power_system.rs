use bevy::prelude::*;
use crate::components::{Battery, Player, PowerSource, Pole, PoleAttachment, CordSystem};

pub fn transfer_power(
    time: Res<Time>,
    cord_system: Res<CordSystem>,
    mut player_query: Query<&mut Battery, With<Player>>,
    pole_attachment_query: Query<&PoleAttachment>,
    pole_query: Query<&PowerSource, With<Pole>>,
) {
    // Only transfer power if player is attached to a pole
    if let Some(attached_pole_attachment) = cord_system.attached_pole {
        // Get the pole entity from the attachment point
        if let Ok(pole_attachment) = pole_attachment_query.get(attached_pole_attachment) {
            let pole_entity = pole_attachment.pole_entity;
            
            // Check if the pole has a power source
            if let Ok(power_source) = pole_query.get(pole_entity) {
                // Transfer power to player
                if let Ok(mut battery) = player_query.single_mut() {
                    if battery.current_charge < battery.max_charge {
                        let charge_amount = power_source.max_output * time.delta_secs();
                        battery.current_charge = (battery.current_charge + charge_amount).min(battery.max_charge);
                    }
                }
            }
        }
    }
}
