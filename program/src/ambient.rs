//! SkateBIRD's ambient light as fill lights. Unity's built-in pipeline lights every surface with the scene's ambient
//! (RenderSettings: a trilight of sky, equator and ground colours times an intensity), which is most of the light
//! inside its rooms; a ReSkate map has no ambient of its own, only punctual lights and San Van's sky. So the program
//! places a grid of shadowless point lights over the level's play space (the scan's collider bounds), each bright
//! enough that a surface between them sees about the ambient's level, in the same Unity light units as the level's own
//! lights (raise-skate then converts all of them with the one calibration). An approximation, counted: a point light
//! falls off and shines from a place, ambient does neither.
use primase_core::pass::PassLog;
use primase_core::{Light, LightKind, Name, Transform, World};
use primase_unity::components::UnityRenderSettings;

/// One stored gamma-encoded colour channel as linear light.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Fill lights `spacing` metres apart over `bounds` (Unity min and max corners); none when the scene states no
/// ambient. Each light's colour is the ambient's mean colour, and its intensity the ambient's level times
/// (spacing / 2)^2, so a surface half a spacing from a light gets the ambient's level from it.
pub fn add_fill(w: &mut World, bounds: ([f64; 3], [f64; 3]), spacing: f64) -> PassLog {
    let mut log = PassLog::default();
    let Some(a) = w
        .resource::<UnityRenderSettings>()
        .and_then(|s| s.ambient.clone())
    else {
        log.count("levels without an ambient: no fill", 1);
        return log;
    };
    // trilight: the mean of the three colours; flat (3): the sky colour alone
    let colours: Vec<[f32; 4]> = if a.mode == 3 {
        vec![a.sky]
    } else {
        vec![a.sky, a.equator, a.ground]
    };
    let mut mean = [0.0f32; 3];
    for c in &colours {
        for (m, v) in mean.iter_mut().zip(c) {
            *m += linear(*v) * a.intensity / colours.len() as f32;
        }
    }
    let level = mean.iter().copied().fold(0.0f32, f32::max);
    if level <= 0.0 {
        log.count("levels with a black ambient: no fill", 1);
        return log;
    }
    let colour = mean.map(|c| c / level);
    let reach = spacing / 2.0;
    let intensity = level * (reach * reach) as f32;
    let (lo, hi) = bounds;
    let steps = |i: usize| (((hi[i] - lo[i]) / spacing).ceil() as usize).max(1);
    for ix in 0..steps(0) {
        for iy in 0..steps(1) {
            for iz in 0..steps(2) {
                let u = [
                    lo[0] + (ix as f64 + 0.5) * spacing,
                    lo[1] + (iy as f64 + 0.5) * spacing,
                    lo[2] + (iz as f64 + 0.5) * spacing,
                ];
                let e = w.spawn();
                w.insert(e, Name(format!("ambient fill {ix}.{iy}.{iz}")));
                // Unity (x, y, z) is the scene's (x, y, -z)
                w.insert(
                    e,
                    Transform {
                        cols: [
                            [1.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 0.0, 1.0],
                            [u[0] as f32, u[1] as f32, -u[2] as f32],
                        ],
                    },
                );
                w.insert(
                    e,
                    Light {
                        kind: LightKind::Point,
                        color: colour,
                        intensity,
                        range: Some((spacing * 1.5) as f32),
                        cast_shadows: Some(false),
                    },
                );
                log.count("ambient fill lights", 1);
            }
        }
    }
    log.approximate("ambient light as shadowless point lights on a grid", 1);
    log
}
