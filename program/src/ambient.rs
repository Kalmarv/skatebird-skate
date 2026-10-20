//! SkateBIRD's ambient light as fill lights. Unity's built-in pipeline lights every surface with the scene's ambient
//! (RenderSettings: a trilight of sky, equator and ground colours times an intensity), which is most of the light
//! inside its rooms; a ReSkate map has no ambient of its own, only punctual lights and San Van's sky. So the program
//! places a grid of shadowless point lights over the level's play space (the scan's collider bounds), each bright
//! enough that a surface between them sees about the ambient's level, in the same Unity light units as the level's own
//! lights (raise-skate then converts all of them with the one calibration). An approximation, counted: a point light
//! falls off and shines from a place, ambient does neither.
use std::collections::BTreeMap;

use primase_core::pass::PassLog;
use primase_core::{Instance, Light, LightKind, Mesh, Name, Topology, Transform, World};
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
/// (spacing / 2)^2, so a surface half a spacing from a light gets the ambient's level from it. A light closer than a
/// quarter spacing to a drawn surface is left out (counted): a surface 1 m from it would be lit about 50 times the
/// ambient's level, a bright spot.
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
    let clearance = spacing / 4.0;
    let surfaces = Surfaces::of(w, clearance);
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
                // Unity (x, y, z) is the scene's (x, y, -z)
                if surfaces.within([u[0], u[1], -u[2]], clearance) {
                    log.count("ambient fill lights left out (near a drawn surface)", 1);
                    continue;
                }
                let e = w.spawn();
                w.insert(e, Name(format!("ambient fill {ix}.{iy}.{iz}")));
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

/// The world's placed triangles in scene space, bucketed on a grid of `cell` metres.
struct Surfaces {
    cell: f64,
    triangles: Vec<[[f64; 3]; 3]>,
    buckets: BTreeMap<[i64; 3], Vec<usize>>,
}

impl Surfaces {
    fn of(w: &World, cell: f64) -> Surfaces {
        let mut triangles = Vec::new();
        for (e, i) in w.iter::<Instance>() {
            let (Some(t), Some(mesh)) = (w.get::<Transform>(e), w.get::<Mesh>(i.prototype)) else {
                continue;
            };
            let place = |p: [f32; 3]| -> [f64; 3] {
                std::array::from_fn(|r| {
                    (0..3)
                        .map(|c| f64::from(t.cols[c][r]) * f64::from(p[c]))
                        .sum::<f64>()
                        + f64::from(t.cols[3][r])
                })
            };
            for p in mesh
                .primitives()
                .iter()
                .filter(|p| p.topology == Topology::Triangles)
            {
                let Some(pos) = p.positions() else { continue };
                let corners: Vec<usize> = match &p.indices {
                    Some(ix) => ix.iter().map(|&k| k as usize).collect(),
                    None => (0..pos.len()).collect(),
                };
                for c in corners.as_chunks::<3>().0 {
                    if c.iter().all(|&k| k < pos.len()) {
                        triangles.push([place(pos[c[0]]), place(pos[c[1]]), place(pos[c[2]])]);
                    }
                }
            }
        }
        let mut buckets: BTreeMap<[i64; 3], Vec<usize>> = BTreeMap::new();
        let key = |v: f64| (v / cell).floor() as i64;
        for (n, tri) in triangles.iter().enumerate() {
            let lo: [i64; 3] = std::array::from_fn(|a| {
                key(tri.iter().map(|p| p[a]).fold(f64::INFINITY, f64::min))
            });
            let hi: [i64; 3] = std::array::from_fn(|a| {
                key(tri.iter().map(|p| p[a]).fold(f64::NEG_INFINITY, f64::max))
            });
            for x in lo[0]..=hi[0] {
                for y in lo[1]..=hi[1] {
                    for z in lo[2]..=hi[2] {
                        buckets.entry([x, y, z]).or_default().push(n);
                    }
                }
            }
        }
        Surfaces {
            cell,
            triangles,
            buckets,
        }
    }

    /// Whether a placed triangle is within `d` (at most a cell) of `p`.
    fn within(&self, p: [f64; 3], d: f64) -> bool {
        let k: [i64; 3] = std::array::from_fn(|a| (p[a] / self.cell).floor() as i64);
        for x in k[0] - 1..=k[0] + 1 {
            for y in k[1] - 1..=k[1] + 1 {
                for z in k[2] - 1..=k[2] + 1 {
                    for &n in self.buckets.get(&[x, y, z]).into_iter().flatten() {
                        if primase_meshops::point_triangle_distance_squared(p, self.triangles[n])
                            < d * d
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
