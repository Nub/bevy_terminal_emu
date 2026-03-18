use bevy::prelude::*;

use crate::effects::breathe::Breathe;
use crate::effects::bubbly::Bubbly;
use crate::effects::collapse::Collapse;
use crate::effects::explode::Explode;
use crate::effects::glitch::Glitch;
use crate::effects::glow::Glow;
use crate::effects::jitter::Jitter;
use crate::effects::knock::Knock;
use crate::effects::rainbow::Rainbow;
use crate::effects::ripple::Ripple;
use crate::effects::scatter::Scatter;
use crate::effects::shiny::Shiny;
use crate::effects::slash::Slash;
use crate::effects::wave::Wave;
use crate::effects::{EffectRegion, TargetTerminal};
use crate::render::{TerminalEffectUniforms, TerminalMaterial, TerminalQuadEntity};

/// Maximum number of include/exclude rectangles encoded in shader uniforms.
const MAX_RECTS: usize = 8;

/// System that aggregates all effect components targeting terminal `T` into
/// the `TerminalEffectUniforms` on the material, and ticks elapsed timers.
pub fn aggregate_effects<T: 'static + Send + Sync>(
    quad: Res<TerminalQuadEntity<T>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    time: Res<Time>,
    mut effects: Query<
        (
            Option<&Wave>,
            Option<&Jitter>,
            Option<&Glow>,
            Option<&Rainbow>,
            Option<&Breathe>,
            Option<&Shiny>,
            Option<&Glitch>,
            Option<&Bubbly>,
            Option<&Ripple>,
            Option<&mut Slash>,
            Option<&mut Knock>,
            Option<&mut Explode>,
            Option<&mut Collapse>,
            Option<&mut Scatter>,
            Option<&EffectRegion>,
        ),
        With<TargetTerminal<T>>,
    >,
) {
    let Some(mat) = materials.get_mut(&quad.material_handle) else {
        return;
    };

    let dt = time.delta_secs();
    let mut u = TerminalEffectUniforms::default();

    for (
        wave,
        jitter,
        glow,
        rainbow,
        breathe,
        shiny,
        glitch,
        bubbly,
        ripple,
        mut slash,
        mut knock,
        mut explode,
        mut collapse,
        mut scatter,
        region,
    ) in effects.iter_mut()
    {
        // ── Continuous effects ──

        if let Some(w) = wave {
            if u.wave_active == 0 {
                u.wave_amplitude = w.amplitude;
                u.wave_wavelength = w.wavelength;
                u.wave_speed = w.speed;
                u.wave_horizontal = if w.horizontal { 1 } else { 0 };
                u.wave_active = 1;
                encode_region(region.as_deref(), &mut u);
            }
        }
        if let Some(j) = jitter {
            if u.jitter_active == 0 {
                u.jitter_amplitude = j.amplitude;
                u.jitter_speed = j.speed;
                u.jitter_rotate = if j.rotate { 1 } else { 0 };
                u.jitter_max_rotation = j.max_rotation;
                u.jitter_active = 1;
            }
        }
        if let Some(g) = glow {
            if u.glow_active == 0 {
                u.glow_speed = g.speed;
                u.glow_intensity = g.intensity;
                u.glow_spread = g.spread;
                u.glow_active = 1;
            }
        }
        if let Some(r) = rainbow {
            if u.rainbow_active == 0 {
                u.rainbow_speed = r.speed;
                u.rainbow_saturation = r.saturation;
                u.rainbow_lightness = r.lightness;
                u.rainbow_spread = r.spread;
                u.rainbow_active = 1;
            }
        }
        if let Some(b) = breathe {
            if u.breathe_active == 0 {
                u.breathe_min_scale = b.min_scale;
                u.breathe_max_scale = b.max_scale;
                u.breathe_speed = b.speed;
                u.breathe_phase_spread = b.phase_spread;
                u.breathe_active = 1;
            }
        }
        if let Some(s) = shiny {
            if u.shiny_active == 0 {
                u.shiny_speed = s.speed;
                u.shiny_width = s.width;
                u.shiny_angle = s.angle;
                u.shiny_brightness = s.brightness;
                u.shiny_active = 1;
            }
        }
        if let Some(g) = glitch {
            if u.glitch_active == 0 && g.active {
                u.glitch_max_offset = g.max_offset;
                u.glitch_intensity = g.intensity;
                u.glitch_frequency = g.frequency;
                u.glitch_active = 1;
            }
        }
        if let Some(b) = bubbly {
            if u.bubbly_active == 0 {
                u.bubbly_speed = b.speed;
                u.bubbly_density = b.density;
                u.bubbly_max_scale = b.max_scale;
                u.bubbly_active = 1;
            }
        }
        if let Some(r) = ripple {
            if u.ripple_active == 0 {
                u.ripple_origin_col = r.origin_col;
                u.ripple_origin_row = r.origin_row;
                u.ripple_amplitude = r.amplitude;
                u.ripple_wavelength = r.wavelength;
                u.ripple_speed = r.speed;
                u.ripple_phase = r.phase;
                u.ripple_damping = r.damping;
                u.ripple_active = 1;
            }
        }

        // ── Timed effects (tick elapsed + aggregate) ──

        if let Some(ref mut s) = slash {
            if s.active {
                s.elapsed += dt;
                if s.elapsed >= s.duration {
                    s.active = false;
                }
                if u.slash_active == 0 {
                    u.slash_elapsed = s.elapsed;
                    u.slash_duration = s.duration;
                    u.slash_amplitude = s.amplitude;
                    u.slash_width = s.width;
                    u.slash_angle = s.angle;
                    u.slash_active = 1;
                }
            }
        }
        if let Some(ref mut k) = knock {
            if k.active {
                k.elapsed += dt;
                if k.elapsed >= k.duration {
                    k.active = false;
                }
                if u.knock_active == 0 {
                    u.knock_angle = k.angle;
                    u.knock_amplitude = k.amplitude;
                    u.knock_deviation = k.deviation;
                    u.knock_rotation = k.rotation;
                    u.knock_elapsed = k.elapsed;
                    u.knock_duration = k.duration;
                    u.knock_active = 1;
                }
            }
        }
        if let Some(ref mut e) = explode {
            if e.active {
                e.elapsed += dt;
                if e.elapsed >= e.duration {
                    e.active = false;
                }
                if u.explode_active == 0 {
                    u.explode_origin_col = e.origin_col;
                    u.explode_origin_row = e.origin_row;
                    u.explode_force = e.force;
                    u.explode_chaos = e.chaos;
                    u.explode_elapsed = e.elapsed;
                    u.explode_duration = e.duration;
                    u.explode_active = 1;
                }
            }
        }
        if let Some(ref mut c) = collapse {
            if c.active {
                c.elapsed += dt;
                if c.elapsed >= c.duration {
                    c.active = false;
                }
                if u.collapse_active == 0 {
                    u.collapse_gravity = c.gravity;
                    u.collapse_elapsed = c.elapsed;
                    u.collapse_duration = c.duration;
                    u.collapse_stagger_per_row = c.stagger_per_row;
                    u.collapse_active = 1;
                }
            }
        }
        if let Some(ref mut s) = scatter {
            if s.active {
                s.elapsed += dt;
                if s.elapsed >= s.duration {
                    s.active = false;
                }
                if u.scatter_active == 0 {
                    u.scatter_origin_col = s.origin_col;
                    u.scatter_origin_row = s.origin_row;
                    u.scatter_speed = s.speed;
                    u.scatter_elapsed = s.elapsed;
                    u.scatter_duration = s.duration;
                    u.scatter_spin = s.spin;
                    u.scatter_active = 1;
                }
            }
        }
    }

    mat.effect_params = u;
}

/// Encode an EffectRegion into the uniform's include/exclude rect arrays.
fn encode_region(region: Option<&EffectRegion>, u: &mut TerminalEffectUniforms) {
    let Some(region) = region else { return };

    // Already encoded?
    if u.include_count > 0 || u.exclude_count > 0 {
        return;
    }

    for (i, rect) in region.include.iter().take(MAX_RECTS).enumerate() {
        u.include_rects[i] = UVec4::new(
            rect.col as u32,
            rect.row as u32,
            rect.width as u32,
            rect.height as u32,
        );
        u.include_count = (i + 1) as u32;
    }

    for (i, rect) in region.exclude.iter().take(MAX_RECTS).enumerate() {
        u.exclude_rects[i] = UVec4::new(
            rect.col as u32,
            rect.row as u32,
            rect.width as u32,
            rect.height as u32,
        );
        u.exclude_count = (i + 1) as u32;
    }
}
