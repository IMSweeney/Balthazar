use bevy::prelude::*;

/// Resource that tracks the day/night cycle state
#[derive(Resource)]
pub struct DayNightCycle {
    /// Time of day in seconds (0.0 = midnight, 43200.0 = noon)
    pub time_of_day: f32,
    /// Duration of a full day in seconds (default: 120 seconds for a 2-minute day)
    pub day_duration: f32,
    /// Speed multiplier for the cycle (1.0 = normal, 2.0 = 2x speed)
    pub speed: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 60.0, // Start at noon (half of day_duration)
            day_duration: 120.0,  // 2-minute day/night cycle
            speed: 1.0,
        }
    }
}

impl DayNightCycle {
    /// Get the current time as a normalized value (0.0-1.0)
    pub fn normalized_time(&self) -> f32 {
        self.time_of_day / self.day_duration
    }
    
    /// Check if it's currently day time
    pub fn is_day(&self) -> bool {
        let normalized = self.normalized_time();
        normalized > 0.25 && normalized < 0.75 // Day between 6am and 6pm
    }
    
    /// Get the sun/moon brightness (0.0 = darkest night, 1.0 = brightest day)
    pub fn get_brightness(&self) -> f32 {
        let normalized = self.normalized_time();
        // Create a smooth sine wave for brightness that peaks at noon and dips at midnight
        let brightness = ((normalized * std::f32::consts::TAU).cos() + 1.0) / 2.0;
        // Clamp minimum brightness to 0.2 so it's never completely dark
        brightness.max(0.2)
    }
    
    /// Get the ambient color based on time of day
    pub fn get_ambient_color(&self) -> Color {
        let normalized = self.normalized_time();
        
        // Define colors for different times of day
        let midnight = Color::srgb(0.1, 0.1, 0.2); // Dark blue
        let dawn = Color::srgb(1.0, 0.6, 0.4);     // Orange/pink
        let noon = Color::srgb(1.0, 1.0, 0.95);    // Bright white-yellow
        let dusk = Color::srgb(1.0, 0.5, 0.3);     // Orange/red
        
        // Interpolate between colors based on time
        if normalized < 0.25 {
            // Midnight to dawn (0.0 - 0.25)
            let t = normalized / 0.25;
            Color::srgb(
                midnight.to_srgba().red.lerp(dawn.to_srgba().red, t),
                midnight.to_srgba().green.lerp(dawn.to_srgba().green, t),
                midnight.to_srgba().blue.lerp(dawn.to_srgba().blue, t),
            )
        } else if normalized < 0.5 {
            // Dawn to noon (0.25 - 0.5)
            let t = (normalized - 0.25) / 0.25;
            Color::srgb(
                dawn.to_srgba().red.lerp(noon.to_srgba().red, t),
                dawn.to_srgba().green.lerp(noon.to_srgba().green, t),
                dawn.to_srgba().blue.lerp(noon.to_srgba().blue, t),
            )
        } else if normalized < 0.75 {
            // Noon to dusk (0.5 - 0.75)
            let t = (normalized - 0.5) / 0.25;
            Color::srgb(
                noon.to_srgba().red.lerp(dusk.to_srgba().red, t),
                noon.to_srgba().green.lerp(dusk.to_srgba().green, t),
                noon.to_srgba().blue.lerp(dusk.to_srgba().blue, t),
            )
        } else {
            // Dusk to midnight (0.75 - 1.0)
            let t = (normalized - 0.75) / 0.25;
            Color::srgb(
                dusk.to_srgba().red.lerp(midnight.to_srgba().red, t),
                dusk.to_srgba().green.lerp(midnight.to_srgba().green, t),
                dusk.to_srgba().blue.lerp(midnight.to_srgba().blue, t),
            )
        }
    }
}

/// Component marker for entities that should be affected by day/night lighting
#[derive(Component)]
pub struct AffectedByDayNight {
    /// Original color to multiply with ambient lighting
    pub base_color: Color,
}

/// System that updates the day/night cycle time
pub fn update_day_night_cycle(
    mut cycle: ResMut<DayNightCycle>,
    time: Res<Time>,
) {
    cycle.time_of_day += time.delta_secs() * cycle.speed;
    
    // Wrap around after a full day
    if cycle.time_of_day >= cycle.day_duration {
        cycle.time_of_day -= cycle.day_duration;
    }
}

/// System that applies day/night lighting to affected entities
pub fn apply_day_night_lighting(
    cycle: Res<DayNightCycle>,
    mut query: Query<(&AffectedByDayNight, &mut Sprite)>,
) {
    let ambient = cycle.get_ambient_color();
    let brightness = cycle.get_brightness();
    
    for (affected, mut sprite) in query.iter_mut() {
        // Multiply base color with ambient color and brightness
        let base = affected.base_color.to_srgba();
        sprite.color = Color::srgb(
            base.red * ambient.to_srgba().red * brightness,
            base.green * ambient.to_srgba().green * brightness,
            base.blue * ambient.to_srgba().blue * brightness,
        );
    }
}

/// System that updates the clear color (sky/background) based on time of day
pub fn update_sky_color(
    cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
) {
    let normalized = cycle.normalized_time();
    
    // Define sky colors for different times
    let midnight_sky = Color::srgb(0.05, 0.05, 0.15); // Dark blue-black
    let dawn_sky = Color::srgb(0.4, 0.3, 0.5);        // Purple
    let noon_sky = Color::srgb(0.53, 0.81, 0.92);     // Light blue
    let dusk_sky = Color::srgb(0.8, 0.4, 0.3);        // Orange-red
    
    // Interpolate sky color
    let sky_color = if normalized < 0.25 {
        let t = normalized / 0.25;
        Color::srgb(
            midnight_sky.to_srgba().red.lerp(dawn_sky.to_srgba().red, t),
            midnight_sky.to_srgba().green.lerp(dawn_sky.to_srgba().green, t),
            midnight_sky.to_srgba().blue.lerp(dawn_sky.to_srgba().blue, t),
        )
    } else if normalized < 0.5 {
        let t = (normalized - 0.25) / 0.25;
        Color::srgb(
            dawn_sky.to_srgba().red.lerp(noon_sky.to_srgba().red, t),
            dawn_sky.to_srgba().green.lerp(noon_sky.to_srgba().green, t),
            dawn_sky.to_srgba().blue.lerp(noon_sky.to_srgba().blue, t),
        )
    } else if normalized < 0.75 {
        let t = (normalized - 0.5) / 0.25;
        Color::srgb(
            noon_sky.to_srgba().red.lerp(dusk_sky.to_srgba().red, t),
            noon_sky.to_srgba().green.lerp(dusk_sky.to_srgba().green, t),
            noon_sky.to_srgba().blue.lerp(dusk_sky.to_srgba().blue, t),
        )
    } else {
        let t = (normalized - 0.75) / 0.25;
        Color::srgb(
            dusk_sky.to_srgba().red.lerp(midnight_sky.to_srgba().red, t),
            dusk_sky.to_srgba().green.lerp(midnight_sky.to_srgba().green, t),
            dusk_sky.to_srgba().blue.lerp(midnight_sky.to_srgba().blue, t),
        )
    };
    
    clear_color.0 = sky_color;
}
