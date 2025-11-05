use bevy::prelude::*;
use crate::components::{Battery, Player, PowerSource, Pole, CordSystem, SolarPanel};
use crate::day_night_cycle::DayNightCycle;

pub fn transfer_power(
    time: Res<Time<Virtual>>,
    cord_system: Option<Res<CordSystem>>,
    mut player_query: Query<&mut Battery, With<Player>>,
    pole_query: Query<&PowerSource, With<Pole>>,
) {
    let Some(cord_system) = cord_system else { return; };
    
    // Only transfer power if player is attached to a pole
    if let Some(attached_pole) = cord_system.attached_pole {
        // Check if the pole has a power source
        if let Ok(power_source) = pole_query.get(attached_pole) {
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

pub fn solar_charge(
    time: Res<Time<Virtual>>,
    cycle: Res<DayNightCycle>,
    mut query: Query<(&SolarPanel, &mut Battery), With<Player>>,
) {
    // Charge amount scales with brightness (0.2 at night, 1.0 at noon)
    let brightness = cycle.get_brightness();
    
    for (solar_panel, mut battery) in query.iter_mut() {
        if battery.current_charge < battery.max_charge {
            let charge_amount = solar_panel.max_output * brightness * time.delta_secs();
            battery.current_charge = (battery.current_charge + charge_amount).min(battery.max_charge);
        }
    }
}
