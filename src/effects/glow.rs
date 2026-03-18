use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Glow {
    pub speed: f32,
    pub intensity: f32,
    pub spread: f32,
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            speed: 2.0,
            intensity: 0.5,
            spread: 0.4,
        }
    }
}
