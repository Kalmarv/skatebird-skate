//! The SkateBIRD levels the program converts, and the skate markers made of the game's own spawn and travel points.
//!
//! primase's SkateBIRD reader (`primase_skatebird::read_markers`) reads every `ResetPosition` and `MissionGiver` of a
//! level into the world, each with its GameObject's world matrix in the core frame (Unity (x, y, z) is (x, y, -z)).
//! The program's policy over them, after the level's state is resolved (what is stored inactive is gone): the spawn
//! is the `ResetPosition` marked `startingPosition` ([`settings::fallback_spawn`] where a level has none: level7), and
//! the travel points are a few reset positions spread over the level ([`curate`]). Mission givers are not travel
//! points: the maps carry no missions. A marker's (east, north) is the core frame's (x, z) and its height y; a core
//! forward (x, z) faces the scene yaw `atan2(x, z)`.
use primase_core::{Name, Transform, World};
use primase_reskate::dialect::{MarkerKind, SkateMarker};
use primase_skatebird::components::SkatebirdResetPosition;

use crate::settings;

/// A level the program converts: its scene file's name (`level2`) and the game's name for it (`rooftop`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub id: &'static str,
    pub name: &'static str,
}

/// A spawn or travel point in the core frame: metres, and the scene yaw in degrees (0 faces +z, clockwise from
/// above, as the map writer reads it).
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub name: String,
    pub position: [f64; 3],
    pub yaw_deg: f64,
}

/// A level's spawn and travel points.
#[derive(Clone, Debug)]
pub struct Markers {
    pub spawn: Point,
    /// Whether the spawn is the game's own starting reset position (else the program's fallback).
    pub starting: bool,
    pub travel: Vec<Point>,
}

/// The levels of [`settings::LEVELS`] named in `wanted` (all when it is empty).
pub fn chosen(wanted: &[String]) -> Result<Vec<Level>, String> {
    if wanted.is_empty() {
        return Ok(settings::LEVELS.to_vec());
    }
    wanted
        .iter()
        .map(|id| {
            settings::LEVELS
                .iter()
                .find(|l| l.id == id)
                .copied()
                .ok_or_else(|| format!("no level {id} (the program converts {:?})", ids()))
        })
        .collect()
}

fn ids() -> Vec<&'static str> {
    settings::LEVELS.iter().map(|l| l.id).collect()
}

/// A Unity yaw (0 faces Unity +Z) as the scene's: `180 - a`, i.e. `atan2(sin a, -cos a)` (the mirror turns +Z into -Z).
pub fn scene_yaw(unity_yaw_deg: f64) -> f64 {
    let a = unity_yaw_deg.to_radians();
    a.sin().atan2(-a.cos()).to_degrees()
}

/// The point of an entity's core transform: its translation, and the yaw its GameObject's forward faces. The core
/// frame mirrors z, so Unity's local +Z is the transform's -z column (checked against the game's own yaws: level2's
/// start, Unity 172.9341 -> scene 7.0659; level1's, Unity -54.8157 -> scene 234.8157).
fn point(name: String, t: &Transform) -> Point {
    let [x, _, z] = t.cols[2];
    Point {
        name,
        position: t.cols[3].map(f64::from),
        yaw_deg: (-f64::from(x)).atan2(-f64::from(z)).to_degrees(),
    }
}

/// The level's spawn and travel points from the markers primase read into `w` (see the module documentation), the
/// travel points chosen within `space` (the play space, core frame) by [`curate`].
pub fn markers(w: &World, level: &str, space: [[f64; 3]; 2]) -> Result<Markers, String> {
    let name = |e| {
        w.get::<Name>(e)
            .map_or_else(|| format!("{e:?}"), |n| n.0.clone())
    };
    let mut spawn = None;
    let mut resets = Vec::new();
    for (e, r) in w.iter::<SkatebirdResetPosition>() {
        let Some(t) = w.get::<Transform>(e) else {
            continue;
        };
        let p = point(name(e), t);
        if r.starting_position {
            if spawn.is_some() {
                return Err(format!("{level}: more than one starting reset position"));
            }
            spawn = Some(p.clone());
        }
        resets.push((r.starting_position, p));
    }
    let travel = curate(resets, space, settings::TRAVEL_POINTS);
    let (spawn, starting) = match spawn {
        Some(s) => (s, true),
        None => (
            settings::fallback_spawn(level).ok_or(format!(
                "{level}: no starting reset position and no fallback"
            ))?,
            false,
        ),
    };
    Ok(Markers {
        spawn,
        starting,
        travel,
    })
}

/// The travel points: the reset positions inside the play space (10 m margin), the starting one first, then spread
/// farthest-first to `n` in all.
fn curate(resets: Vec<(bool, Point)>, space: [[f64; 3]; 2], n: usize) -> Vec<Point> {
    let inside = |p: &Point| {
        (0..3).all(|i| space[0][i] - 10.0 <= p.position[i] && p.position[i] <= space[1][i] + 10.0)
    };
    let (starters, rest): (Vec<_>, Vec<_>) = resets
        .into_iter()
        .filter(|(_, p)| inside(p))
        .partition(|(s, _)| *s);
    let mut chosen: Vec<Point> = starters.into_iter().take(1).map(|(_, p)| p).collect();
    let rest: Vec<Point> = rest.into_iter().map(|(_, p)| p).collect();
    let left = n.saturating_sub(chosen.len());
    chosen.extend(spread(rest, left));
    chosen
}

/// At most `n` of `pool`, the first and then each the one farthest from those taken.
fn spread(pool: Vec<Point>, n: usize) -> Vec<Point> {
    if pool.len() <= n {
        return pool;
    }
    let d2 = |a: &Point, b: &Point| {
        (0..3)
            .map(|i| (a.position[i] - b.position[i]).powi(2))
            .sum::<f64>()
    };
    let nearest = |i: usize, taken: &[usize]| {
        taken
            .iter()
            .map(|&t| d2(&pool[i], &pool[t]))
            .fold(f64::INFINITY, f64::min)
    };
    let mut taken = vec![0usize];
    while taken.len() < n {
        let next = (0..pool.len())
            .filter(|i| !taken.contains(i))
            .max_by(|&a, &b| nearest(a, &taken).total_cmp(&nearest(b, &taken)))
            .expect("fewer taken than the pool");
        taken.push(next);
    }
    taken.into_iter().map(|i| pool[i].clone()).collect()
}

fn marker(kind: MarkerKind, p: &Point) -> SkateMarker {
    SkateMarker {
        kind,
        position: Some((
            [p.position[0] as f32, p.position[2] as f32],
            Some(p.position[1] as f32),
        )),
        yaw: Some(p.yaw_deg),
        m: None,
        blueprint: None,
        snap: None,
        lift: None,
        shelter: None,
        trainer: None,
        other: Default::default(),
    }
}

/// The spawn marker (named `spawn`) and one travel marker per travel point, at the game's own heights (a ceiling above
/// must not win); returns the travel count.
pub fn add_markers(w: &mut World, m: &Markers) -> usize {
    let e = w.spawn();
    w.insert(e, Name("spawn".into()));
    w.insert(e, marker(MarkerKind::Spawn, &m.spawn));
    for p in &m.travel {
        let e = w.spawn();
        w.insert(e, Name(p.name.clone()));
        w.insert(e, marker(MarkerKind::Travel, p));
    }
    m.travel.len()
}

/// The play space in the core frame (min and max corners): the box of every collision soup placed in the world, each
/// soup over [`settings::PLAY_SPACE_MAX`] on an axis left out (skyline backdrops and the buildings' 360 m sides).
pub fn play_space(w: &World) -> Option<[[f64; 3]; 2]> {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    let placed = |c: &primase_core::Collision, t: Option<&Transform>| {
        let mut a = [f64::INFINITY; 3];
        let mut b = [f64::NEG_INFINITY; 3];
        for p in &c.positions {
            let q: [f64; 3] = match t {
                Some(t) => std::array::from_fn(|r| {
                    (0..3)
                        .map(|k| f64::from(t.cols[k][r]) * f64::from(p[k]))
                        .sum::<f64>()
                        + f64::from(t.cols[3][r])
                }),
                None => p.map(f64::from),
            };
            for i in 0..3 {
                a[i] = a[i].min(q[i]);
                b[i] = b[i].max(q[i]);
            }
        }
        (a, b)
    };
    for (e, c) in w.iter::<primase_core::Collision>() {
        let (a, b) = placed(c, w.get::<Transform>(e));
        // a soup too large on an axis (or with no finite extent) is not play space
        if (0..3).any(|i| (b[i] - a[i]).is_nan() || b[i] - a[i] > settings::PLAY_SPACE_MAX) {
            continue;
        }
        for i in 0..3 {
            lo[i] = lo[i].min(a[i]);
            hi[i] = hi[i].max(b[i]);
        }
    }
    (lo[0] <= hi[0]).then_some([lo, hi])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_face_their_forward_and_sit_at_their_height() {
        // level2's starting reset position: Unity (-18.7, 1.189, 84.8), yaw 172.9341 -> core (-18.7, 1.189, -84.8),
        // facing core (sin a, 0, -cos a), the transform's -z column (as primase's reader gives it)
        let a = 172.9341f64.to_radians();
        let t = Transform {
            cols: [
                [(-a.cos()) as f32, 0.0, (-a.sin()) as f32],
                [0.0, 1.0, 0.0],
                [(-a.sin()) as f32, 0.0, a.cos() as f32],
                [-18.7, 1.189, -84.8],
            ],
        };
        let p = point("Middle Roof".into(), &t);
        assert!(
            (p.yaw_deg - scene_yaw(172.9341)).abs() < 1e-4,
            "{}",
            p.yaw_deg
        );
        assert!((p.yaw_deg - 7.0659).abs() < 1e-3);
        let mut w = World::new();
        let m = Markers {
            spawn: p,
            starting: true,
            travel: vec![],
        };
        assert_eq!(add_markers(&mut w, &m), 0);
        let (_, s) = w.iter::<SkateMarker>().next().unwrap();
        assert_eq!(s.position, Some(([-18.7, -84.8], Some(1.189))));
        assert_eq!(s.snap, None);
    }

    #[test]
    fn the_program_converts_eight_levels() {
        assert_eq!(chosen(&[]).unwrap().len(), 8);
        assert!(chosen(&["level6".into()]).is_err());
    }
}

#[cfg(test)]
mod curate_tests {
    use super::*;

    fn at(name: &str, x: f64) -> Point {
        Point {
            name: name.into(),
            position: [x, 0.0, 0.0],
            yaw_deg: 0.0,
        }
    }

    #[test]
    fn travel_points_are_capped_spread_and_inside() {
        let space = [[0.0, -1.0, -1.0], [100.0, 1.0, 1.0]];
        let mut resets: Vec<(bool, Point)> = (0..20)
            .map(|i| (i == 5, at(&format!("r{i}"), f64::from(i) * 5.0)))
            .collect();
        resets.push((false, at("far", 500.0)));
        let t = curate(resets, space, 5);
        assert_eq!(t.len(), 5);
        assert_eq!(t[0].name, "r5", "the starting reset position first");
        assert!(t.iter().all(|p| p.name != "far"), "outside the play space");
        assert!(t.iter().any(|p| p.name == "r0") && t.iter().any(|p| p.name == "r19"), "spread to both ends");
    }
}
