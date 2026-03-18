#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Cell data texture: each texel is vec4<u32>
// .r = glyph_index
// .g = modifiers (bit 0 = bold, bit 1 = italic, bit 2 = underlined, bit 3 = dim)
// .b = fg color packed as RGBA8 (R in low byte)
// .a = bg color packed as RGBA8 (R in low byte)
@group(2) @binding(0) var cell_data: texture_2d<u32>;

// Font atlas texture (standard RGBA float)
@group(2) @binding(1) var font_atlas: texture_2d<f32>;
@group(2) @binding(2) var font_sampler: sampler;

// Grid parameters
struct GridParams {
    columns: u32,
    rows: u32,
    cell_width: f32,
    cell_height: f32,
    atlas_columns: u32,
    atlas_glyph_count: u32,
    atlas_stride_w: f32,
    atlas_stride_h: f32,
    atlas_cell_w: f32,
    atlas_cell_h: f32,
    atlas_tex_width: f32,
    atlas_tex_height: f32,
    time: f32,
    _pad: f32,
}
@group(2) @binding(3) var<uniform> grid: GridParams;

// Effect parameters
struct EffectParams {
    include_rects: array<vec4<u32>, 8>,
    exclude_rects: array<vec4<u32>, 8>,
    include_count: u32,
    exclude_count: u32,

    wave_amplitude: f32,
    wave_wavelength: f32,
    wave_speed: f32,
    wave_horizontal: u32,
    wave_active: u32,

    jitter_amplitude: f32,
    jitter_speed: f32,
    jitter_rotate: u32,
    jitter_max_rotation: f32,
    jitter_active: u32,

    glow_speed: f32,
    glow_intensity: f32,
    glow_spread: f32,
    glow_active: u32,

    rainbow_speed: f32,
    rainbow_saturation: f32,
    rainbow_lightness: f32,
    rainbow_spread: f32,
    rainbow_active: u32,

    breathe_min_scale: f32,
    breathe_max_scale: f32,
    breathe_speed: f32,
    breathe_phase_spread: f32,
    breathe_active: u32,

    shiny_speed: f32,
    shiny_width: f32,
    shiny_angle: f32,
    shiny_brightness: f32,
    shiny_active: u32,

    glitch_max_offset: f32,
    glitch_intensity: f32,
    glitch_frequency: f32,
    glitch_active: u32,

    bubbly_speed: f32,
    bubbly_density: f32,
    bubbly_max_scale: f32,
    bubbly_active: u32,

    ripple_origin_col: f32,
    ripple_origin_row: f32,
    ripple_amplitude: f32,
    ripple_wavelength: f32,
    ripple_speed: f32,
    ripple_phase: f32,
    ripple_damping: f32,
    ripple_active: u32,

    slash_elapsed: f32,
    slash_duration: f32,
    slash_amplitude: f32,
    slash_width: f32,
    slash_angle: f32,
    slash_active: u32,

    knock_angle: f32,
    knock_amplitude: f32,
    knock_deviation: f32,
    knock_rotation: f32,
    knock_elapsed: f32,
    knock_duration: f32,
    knock_active: u32,

    explode_origin_col: f32,
    explode_origin_row: f32,
    explode_force: f32,
    explode_chaos: f32,
    explode_elapsed: f32,
    explode_duration: f32,
    explode_active: u32,

    collapse_gravity: f32,
    collapse_elapsed: f32,
    collapse_duration: f32,
    collapse_stagger_per_row: f32,
    collapse_active: u32,

    scatter_origin_col: f32,
    scatter_origin_row: f32,
    scatter_speed: f32,
    scatter_elapsed: f32,
    scatter_duration: f32,
    scatter_spin: f32,
    scatter_active: u32,

    _pad0: u32,
}
@group(2) @binding(4) var<uniform> fx: EffectParams;

// ── Utility functions ──

// sRGB → linear conversion (piecewise transfer function)
fn srgb_channel_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_channel_to_linear(color.r),
        srgb_channel_to_linear(color.g),
        srgb_channel_to_linear(color.b),
    );
}

fn unpack_color(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    // Convert sRGB → linear; alpha stays linear
    let linear = srgb_to_linear(vec3<f32>(r, g, b));
    return vec4<f32>(linear, a);
}

// Deterministic hash matching the Rust simple_hash
fn simple_hash(a: u32, b: u32) -> u32 {
    var h = a * 2654435761u + b * 2246822519u;
    h ^= h >> 16u;
    h = h * 2246822519u;
    h ^= h >> 13u;
    h = h * 3266489917u;
    h ^= h >> 16u;
    return h;
}

fn hash_to_float(h: u32) -> f32 {
    return f32(h & 0xFFFFu) / 65535.0;
}

fn smoothstep_f(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// HSL to RGB conversion for rainbow effect
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let hp = h * 6.0;
    let x = c * (1.0 - abs(hp % 2.0 - 1.0));
    var rgb = vec3<f32>(0.0);
    if hp < 1.0 {
        rgb = vec3<f32>(c, x, 0.0);
    } else if hp < 2.0 {
        rgb = vec3<f32>(x, c, 0.0);
    } else if hp < 3.0 {
        rgb = vec3<f32>(0.0, c, x);
    } else if hp < 4.0 {
        rgb = vec3<f32>(0.0, x, c);
    } else if hp < 5.0 {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    let m = l - c * 0.5;
    return rgb + vec3<f32>(m);
}

// Check if a cell is within the effect region
fn in_effect_region(col: u32, row: u32) -> bool {
    // Check excludes first
    for (var i = 0u; i < fx.exclude_count; i++) {
        let r = fx.exclude_rects[i];
        if col >= r.x && col < r.x + r.z && row >= r.y && row < r.y + r.w {
            return false;
        }
    }
    // If no includes, everything is included
    if fx.include_count == 0u {
        return true;
    }
    // Check includes
    for (var i = 0u; i < fx.include_count; i++) {
        let r = fx.include_rects[i];
        if col >= r.x && col < r.x + r.z && row >= r.y && row < r.y + r.w {
            return true;
        }
    }
    return false;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Map UV to grid column and row (before displacement effects)
    var col_f = uv.x * f32(grid.columns);
    var row_f = uv.y * f32(grid.rows);

    // ── Displacement effects (modify col_f/row_f before cell lookup) ──

    // Glitch: horizontal row offset
    if fx.glitch_active != 0u {
        let row_u = u32(floor(row_f));
        let time_slot = u32(floor(grid.time * fx.glitch_frequency));
        let h = simple_hash(row_u, time_slot);
        let hf = hash_to_float(h);
        if hf < fx.glitch_intensity {
            let offset_h = simple_hash(row_u + 1000u, time_slot);
            let offset = (hash_to_float(offset_h) - 0.5) * 2.0 * fx.glitch_max_offset;
            col_f += offset;
        }
    }

    let col = u32(floor(col_f));
    let row = u32(floor(row_f));

    // Clamp to valid range
    if col >= grid.columns || row >= grid.rows {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let in_region = in_effect_region(col, row);

    // Load cell data via textureLoad (integer texture, no sampler)
    let cell = textureLoad(cell_data, vec2<i32>(i32(col), i32(row)), 0);
    let glyph_index = cell.r;
    let modifiers = cell.g;
    let fg_packed = cell.b;
    let bg_packed = cell.a;

    var fg = unpack_color(fg_packed);
    var bg = unpack_color(bg_packed);

    // Sub-cell UV: position within this cell (0..1)
    var sub_u = fract(col_f);
    var sub_v = fract(row_f);

    // ── Sub-cell displacement effects ──

    if in_region {
        // Wave: sinusoidal displacement
        if fx.wave_active != 0u {
            let phase = grid.time * fx.wave_speed;
            if fx.wave_horizontal != 0u {
                let wave_val = sin(f32(row) / fx.wave_wavelength + phase) * fx.wave_amplitude;
                sub_u += wave_val;
            } else {
                let wave_val = sin(f32(col) / fx.wave_wavelength + phase) * fx.wave_amplitude;
                sub_v += wave_val;
            }
        }

        // Jitter: per-cell random offset
        if fx.jitter_active != 0u {
            let time_slot = u32(floor(grid.time * fx.jitter_speed));
            let h1 = simple_hash(col * 1000u + row, time_slot);
            let h2 = simple_hash(col + row * 1000u, time_slot + 7u);
            let ox = (hash_to_float(h1) - 0.5) * 2.0 * fx.jitter_amplitude;
            let oy = (hash_to_float(h2) - 0.5) * 2.0 * fx.jitter_amplitude;
            sub_u += ox;
            sub_v += oy;
        }

        // Breathe: per-cell scale pulsing
        if fx.breathe_active != 0u {
            let cell_phase = f32(simple_hash(col, row) & 0xFFFFu) / 65535.0 * fx.breathe_phase_spread;
            let t = sin(grid.time * fx.breathe_speed + cell_phase) * 0.5 + 0.5;
            let scale = mix(fx.breathe_min_scale, fx.breathe_max_scale, t);
            let center = 0.5;
            sub_u = center + (sub_u - center) / scale;
            sub_v = center + (sub_v - center) / scale;
        }

        // Bubbly: random per-cell scale pop
        if fx.bubbly_active != 0u {
            let time_base = grid.time * fx.bubbly_speed;
            let cell_hash = simple_hash(col, row);
            let phase_offset = hash_to_float(cell_hash) * 6.2832;
            let density_hash = simple_hash(col + 500u, row + 500u);
            if hash_to_float(density_hash) < fx.bubbly_density {
                let pop = abs(sin(time_base + phase_offset));
                let scale = 1.0 + pop * (fx.bubbly_max_scale - 1.0);
                let center = 0.5;
                sub_u = center + (sub_u - center) / scale;
                sub_v = center + (sub_v - center) / scale;
            }
        }

        // Ripple: concentric wave displacement from origin
        if fx.ripple_active != 0u {
            let dx = f32(col) - fx.ripple_origin_col;
            let dy = f32(row) - fx.ripple_origin_row;
            let dist = sqrt(dx * dx + dy * dy);
            let wave_phase = dist / fx.ripple_wavelength - grid.time * fx.ripple_speed + fx.ripple_phase;
            let damping = exp(-dist * fx.ripple_damping);
            let displacement = sin(wave_phase * 6.2832) * fx.ripple_amplitude * damping;
            if dist > 0.001 {
                sub_u += displacement * dx / dist;
                sub_v += displacement * dy / dist;
            }
        }

        // Slash: 2-phase blade displacement
        if fx.slash_active != 0u {
            let progress = fx.slash_elapsed / fx.slash_duration;
            let cos_a = cos(fx.slash_angle);
            let sin_a = sin(fx.slash_angle);
            // Distance along the cut line
            let cx = f32(col) - f32(grid.columns) * 0.5;
            let cy = f32(row) - f32(grid.rows) * 0.5;
            let along = cx * cos_a + cy * sin_a;
            let perp = -cx * sin_a + cy * cos_a;
            // Blade position sweeps across
            let blade_pos = mix(-f32(max(grid.columns, grid.rows)) * 0.7, f32(max(grid.columns, grid.rows)) * 0.7, progress);
            let dist_to_blade = abs(along - blade_pos);
            if dist_to_blade < fx.slash_width {
                let intensity = 1.0 - dist_to_blade / fx.slash_width;
                let push = intensity * fx.slash_amplitude * sign(perp);
                sub_u += push * (-sin_a);
                sub_v += push * cos_a;
            }
        }

        // Knock: damped impulse displacement
        if fx.knock_active != 0u {
            let progress = fx.knock_elapsed / fx.knock_duration;
            let decay = exp(-progress * 5.0) * (1.0 - progress);
            let cos_a = cos(fx.knock_angle);
            let sin_a = sin(fx.knock_angle);
            let cell_hash = simple_hash(col, row);
            let deviation = (hash_to_float(cell_hash) - 0.5) * 2.0 * fx.knock_deviation;
            let push = fx.knock_amplitude * decay;
            sub_u += (cos_a + deviation) * push;
            sub_v += (sin_a + deviation) * push;
        }

        // Explode: radial scatter with spin and shrink
        if fx.explode_active != 0u {
            let progress = fx.explode_elapsed / fx.explode_duration;
            let dx = f32(col) - fx.explode_origin_col;
            let dy = f32(row) - fx.explode_origin_row;
            let dist = sqrt(dx * dx + dy * dy) + 0.001;
            let cell_hash = simple_hash(col, row);
            let chaos_offset = (hash_to_float(cell_hash) - 0.5) * fx.explode_chaos;
            let push = fx.explode_force * progress * progress;
            sub_u += (dx / dist + chaos_offset) * push;
            sub_v += (dy / dist + chaos_offset) * push;
            // Shrink cells as they fly out
            let shrink = max(0.0, 1.0 - progress);
            let center = 0.5;
            sub_u = center + (sub_u - center) * shrink;
            sub_v = center + (sub_v - center) * shrink;
        }

        // Collapse: row-staggered gravity fall
        if fx.collapse_active != 0u {
            let row_delay = f32(grid.rows - 1u - row) * fx.collapse_stagger_per_row;
            let t = max(0.0, fx.collapse_elapsed - row_delay);
            let fall = 0.5 * fx.collapse_gravity * t * t;
            sub_v += fall;
        }

        // Scatter: radial outward displacement with spin
        if fx.scatter_active != 0u {
            let progress = fx.scatter_elapsed / fx.scatter_duration;
            let dx = f32(col) - fx.scatter_origin_col;
            let dy = f32(row) - fx.scatter_origin_row;
            let dist = sqrt(dx * dx + dy * dy) + 0.001;
            let push = fx.scatter_speed * progress;
            let spin_angle = fx.scatter_spin * progress;
            let cos_s = cos(spin_angle);
            let sin_s = sin(spin_angle);
            let dir_x = dx / dist;
            let dir_y = dy / dist;
            let rot_x = dir_x * cos_s - dir_y * sin_s;
            let rot_y = dir_x * sin_s + dir_y * cos_s;
            sub_u += rot_x * push;
            sub_v += rot_y * push;
        }
    }

    // ── Glyph sampling ──

    // Compute atlas UV for this glyph
    let atlas_col = glyph_index % grid.atlas_columns;
    let atlas_row_idx = glyph_index / grid.atlas_columns;

    // Clamp sub-cell UV to avoid sampling neighboring glyphs
    let su = clamp(sub_u, 0.0, 0.999);
    let sv = clamp(sub_v, 0.0, 0.999);

    let atlas_u = (f32(atlas_col) * grid.atlas_stride_w + su * grid.atlas_cell_w) / grid.atlas_tex_width;
    let atlas_v = (f32(atlas_row_idx) * grid.atlas_stride_h + sv * grid.atlas_cell_h) / grid.atlas_tex_height;

    // Sample the font atlas
    let glyph_sample = textureSample(font_atlas, font_sampler, vec2<f32>(atlas_u, atlas_v));
    let glyph_alpha = glyph_sample.a;

    // ── Color effects ──

    // Apply dim modifier (bit 3)
    if (modifiers & 8u) != 0u {
        fg.a *= 0.5;
    }

    if in_region {
        // Glow: alpha pulse on fg color
        if fx.glow_active != 0u {
            let cell_phase = f32(simple_hash(col, row) & 0xFFFFu) / 65535.0 * fx.glow_spread;
            let pulse = sin(grid.time * fx.glow_speed + cell_phase) * 0.5 + 0.5;
            fg = vec4<f32>(fg.rgb * (1.0 + pulse * fx.glow_intensity), fg.a);
        }

        // Rainbow: hue cycling on fg color
        if fx.rainbow_active != 0u {
            let cell_offset = (f32(col) + f32(row)) * fx.rainbow_spread;
            let hue = fract(grid.time * fx.rainbow_speed + cell_offset * 0.01);
            let rainbow_rgb = hsl_to_rgb(hue, fx.rainbow_saturation, fx.rainbow_lightness);
            fg = vec4<f32>(rainbow_rgb, fg.a);
        }

        // Shiny: sweeping highlight band
        if fx.shiny_active != 0u {
            let cos_a = cos(fx.shiny_angle);
            let sin_a = sin(fx.shiny_angle);
            let pos = f32(col) * cos_a + f32(row) * sin_a;
            let sweep_range = f32(grid.columns + grid.rows);
            let sweep_pos = fract(grid.time * fx.shiny_speed) * sweep_range * 2.0 - sweep_range * 0.5;
            let dist_to_sweep = abs(pos - sweep_pos);
            let highlight = smoothstep_f(fx.shiny_width, 0.0, dist_to_sweep) * fx.shiny_brightness;
            fg = vec4<f32>(fg.rgb + vec3<f32>(highlight), fg.a);
        }
    }

    // ── Final compositing ──

    let color = mix(bg.rgb, fg.rgb, glyph_alpha * fg.a);
    let alpha = bg.a + glyph_alpha * fg.a * (1.0 - bg.a);

    // For displacement effects that push sub_u/sub_v out of [0,1], make the cell transparent
    if sub_u < -0.5 || sub_u > 1.5 || sub_v < -0.5 || sub_v > 1.5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return vec4<f32>(color, alpha);
}
