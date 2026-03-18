use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Bubbly {
    pub speed: f32,
    pub density: f32,
    pub max_scale: f32,
}

impl Default for Bubbly {
    fn default() -> Self {
        Self {
            speed: 0.8,
            density: 0.15,
            max_scale: 1.4,
        }
    }
}
