use bevy::prelude::*;

// ── Components ────────────────────────────────────────────────────────────────

/// Added to a shape when it is first spawned.  Drives the scale-in animation.
#[derive(Component)]
pub struct SpawnPop(pub Timer);

/// Added to a shape immediately after a successful merge.
/// The sprite colour fades from HDR-white back to the shape's neon colour.
#[derive(Component)]
pub struct MergeFlash {
    pub timer: Timer,
    /// The HDR colour the sprite should settle at once the flash fades out.
    pub base_color: LinearRgba,
}

/// Added to 3D shapes (levels 26-45) for subtle scale pulsing effect.
#[derive(Component)]
pub struct PulseEffect {
    pub timer: Timer,
    pub base_scale: f32,
}

/// Added to prestige shapes (levels 46-50) for color cycling glow effect.
#[derive(Component)]
pub struct GlowEffect {
    pub timer: Timer,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Scale-in with an elastic overshoot: 0 → ~1.1 → 1.0 over 0.25 s.
pub fn animate_spawn_pop(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut SpawnPop)>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.0.tick(time.delta());
        let t = anim.0.fraction();
        // ease_out_back(0) = 0.0 and rises through ~1.05 before settling at
        // 1.0; the max(0.0) is a safety guard against any floating-point
        // edge cases at t = 0 on the first tick.
        let scale = ease_out_back(t).max(0.0);
        transform.scale = Vec3::splat(scale);
        if anim.0.finished() {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<SpawnPop>();
        }
    }
}

/// Fade the merge-result mesh from bright HDR-white to its natural neon colour
/// over 0.35 s.
pub fn animate_merge_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &mut MergeFlash)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Flash white (HDR) → base colour.  t goes 0 → 1 over the timer duration.
    const FLASH: LinearRgba = LinearRgba {
        red: 3.5,
        green: 3.5,
        blue: 3.5,
        alpha: 1.0,
    };
    for (entity, material_handle, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());
        let t = flash.timer.fraction();
        let base = flash.base_color;
        let r = FLASH.red + (base.red - FLASH.red) * t;
        let g = FLASH.green + (base.green - FLASH.green) * t;
        let b = FLASH.blue + (base.blue - FLASH.blue) * t;
        
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.color = Color::linear_rgb(r, g, b);
        }
        
        if flash.timer.finished() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.color = Color::from(flash.base_color);
            }
            commands.entity(entity).remove::<MergeFlash>();
        }
    }
}

/// Gentle scale pulse for 3D shapes (levels 26-45).
/// Oscillates between 0.95 and 1.05 scale over 1.5 seconds.
pub fn animate_pulse(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PulseEffect)>,
) {
    for (mut transform, mut pulse) in query.iter_mut() {
        pulse.timer.tick(time.delta());
        let t = pulse.timer.fraction();
        
        // Sine wave oscillation: 0.95 to 1.05
        let scale = 1.0 + 0.05 * (t * std::f32::consts::TAU).sin();
        transform.scale = Vec3::splat(scale * pulse.base_scale);
    }
}

/// Color cycling glow for prestige shapes (levels 46-50).
/// Cycles through green spectrum hues over 2 seconds.
pub fn animate_glow(
    time: Res<Time>,
    mut query: Query<(&MeshMaterial2d<ColorMaterial>, &mut GlowEffect)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (material_handle, mut glow) in query.iter_mut() {
        glow.timer.tick(time.delta());
        let t = glow.timer.fraction();
        
        // Cycle through green spectrum: green → cyan → green
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let hue = 120.0 + 60.0 * (t * std::f32::consts::TAU).sin(); // 90-150 degrees (green-cyan range)
            let glow_color = Color::hsl(hue, 0.8, 0.6).to_linear();
            material.color = Color::linear_rgba(
                glow_color.red * 4.0,
                glow_color.green * 4.0,
                glow_color.blue * 4.0,
                glow_color.alpha,
            );
        }
    }
}

// ── Easing ────────────────────────────────────────────────────────────────────

/// Ease-out-back: overshoots slightly before settling at 1.0.
fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}
