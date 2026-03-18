use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Collapse {
    pub gravity: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub stagger_per_row: f32,
    pub active: bool,
}

impl Default for Collapse {
    fn default() -> Self {
        Self {
            gravity: 800.0,
            elapsed: 0.0,
            duration: 3.0,
            stagger_per_row: 0.05,
            active: true,
        }
    }
}
