use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Breathe {
    pub min_scale: f32,
    pub max_scale: f32,
    pub speed: f32,
    pub phase_spread: f32,
}

impl Default for Breathe {
    fn default() -> Self {
        Self {
            min_scale: 0.92,
            max_scale: 1.08,
            speed: 1.0,
            phase_spread: 0.0,
        }
    }
}
