use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Explode {
    pub origin_col: f32,
    pub origin_row: f32,
    pub force: f32,
    pub chaos: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub active: bool,
}

impl Default for Explode {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            force: 200.0,
            chaos: 0.5,
            elapsed: 0.0,
            duration: 2.5,
            active: true,
        }
    }
}
