use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Glitch {
    pub max_offset: f32,
    pub intensity: f32,
    pub frequency: f32,
    pub active: bool,
}

impl Default for Glitch {
    fn default() -> Self {
        Self {
            max_offset: 30.0,
            intensity: 0.3,
            frequency: 8.0,
            active: true,
        }
    }
}
