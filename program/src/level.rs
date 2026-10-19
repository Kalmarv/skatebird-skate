//! What the program knows of each SkateBIRD level, from the game's own scan (`data/skatebird-levels.json`, made by
//! `tools/skatebird_scan.py`): its name, its spawn (the `ResetPosition` marked `startingPosition`, else where the
//! scene places `Player Birb`) and its travel points (the game's `ResetPosition` anchors and mission givers); and the
//! skate markers made of them.
//!
//! Unity (x, y, z) is scene (x, y, -z) (the reader's frame), and a Unity yaw `a` (0 faces +Z, clockwise from above)
//! is the scene yaw `180 - a` (the mirror turns +Z into -Z): `atan2(sin a, -cos a)`.
use std::collections::BTreeMap;
use std::path::Path;

use primase_core::World;
use primase_reskate::dialect::{MarkerKind, SkateMarker};

/// A level's spawn or travel point, in Unity's coordinates as the game stores it.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub name: String,
    /// Metres.
    pub position: [f64; 3],
    /// Degrees, 0 faces +Z, clockwise seen from above.
    pub yaw_deg: f64,
}

/// One level as the program converts it.
#[derive(Clone, Debug)]
pub struct Level {
    /// The scene's file name (`level2`).
    pub id: String,
    /// The game's name for it (`rooftop`).
    pub name: String,
    pub spawn: Point,
    pub travel: Vec<Point>,
    /// The play space: the box of the level's enabled, non-trigger colliders (Unity min and max corners), those over
    /// 100 m (skyline backdrops) left out; `None` when the scan has none.
    pub bounds: Option<([f64; 3], [f64; 3])>,
}

/// Every level of the scan, by id, in the scan's order of ids.
pub fn read_levels(path: &Path) -> Result<BTreeMap<String, Level>, String> {
    let text = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: serde_json::Value =
        serde_json::from_slice(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let levels = doc
        .get("levels")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("{}: no levels", path.display()))?;
    let mut out = BTreeMap::new();
    for (id, l) in levels {
        let what = format!("{}: levels.{id}", path.display());
        let spawn = l
            .get("spawn")
            .filter(|s| !s.is_null())
            .or_else(|| l.get("player_birb_instance"))
            .ok_or_else(|| format!("{what}: no spawn and no player_birb_instance"))?;
        let travel = l
            .get("travel_points")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("{what}: no travel_points"))?;
        out.insert(
            id.clone(),
            Level {
                id: id.clone(),
                name: l
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(id)
                    .to_owned(),
                spawn: point(spawn, "spawn", &format!("{what}.spawn"))?,
                travel: travel
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        point(
                            p,
                            &format!("Travel {i}"),
                            &format!("{what}.travel_points[{i}]"),
                        )
                    })
                    .collect::<Result<_, _>>()?,
                bounds: bounds(l),
            },
        );
    }
    Ok(out)
}

fn bounds(l: &serde_json::Value) -> Option<([f64; 3], [f64; 3])> {
    let corner = |k: &str| -> Option<[f64; 3]> {
        let a = l.get("bounds")?.get(k)?.as_array()?;
        Some([
            a.first()?.as_f64()?,
            a.get(1)?.as_f64()?,
            a.get(2)?.as_f64()?,
        ])
    };
    Some((corner("min")?, corner("max")?))
}

fn point(v: &serde_json::Value, default_name: &str, what: &str) -> Result<Point, String> {
    let position = v
        .get("position")
        .and_then(serde_json::Value::as_array)
        .filter(|p| p.len() == 3)
        .ok_or_else(|| format!("{what}: position needs 3 numbers"))?;
    let mut xyz = [0.0; 3];
    for (out, c) in xyz.iter_mut().zip(position) {
        *out = c
            .as_f64()
            .ok_or_else(|| format!("{what}: position is not a number"))?;
    }
    Ok(Point {
        name: v
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_name)
            .to_owned(),
        position: xyz,
        yaw_deg: v
            .get("yaw_deg")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
    })
}

/// A Unity yaw as the scene's (see the module documentation).
pub fn scene_yaw(unity_yaw_deg: f64) -> f64 {
    let a = unity_yaw_deg.to_radians();
    a.sin().atan2(-a.cos()).to_degrees()
}

/// The marker of `p` in the scene: at the game's own height (a ceiling above it must not win), facing its yaw.
fn marker(kind: MarkerKind, p: &Point) -> SkateMarker {
    SkateMarker {
        kind,
        position: Some((
            [p.position[0] as f32, -(p.position[2] as f32)],
            Some(p.position[1] as f32),
        )),
        yaw: Some(scene_yaw(p.yaw_deg)),
        m: None,
        blueprint: None,
        snap: None,
        lift: None,
        shelter: None,
        trainer: None,
        other: Default::default(),
    }
}

/// The level's spawn marker (named `spawn`) and one travel marker per travel point; returns the travel count.
pub fn add_markers(w: &mut World, level: &Level) -> usize {
    let e = w.spawn();
    w.insert(e, primase_core::Name("spawn".into()));
    w.insert(e, marker(MarkerKind::Spawn, &level.spawn));
    for p in &level.travel {
        let e = w.spawn();
        w.insert(e, primase_core::Name(p.name.clone()));
        w.insert(e, marker(MarkerKind::Travel, p));
    }
    level.travel.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_from_the_scan_become_markers() {
        let dir = std::env::temp_dir().join(format!("skatebird-levels-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("skatebird-levels.json");
        std::fs::write(
            &path,
            r#"{"game":"SkateBIRD","levels":{
                "level2":{"name":"rooftop",
                  "spawn":{"position":[-18.7,1.189,84.8],"yaw_deg":172.9341},
                  "travel_points":[{"name":"Lower Roof","position":[-109.6,-3.0,89.0],"yaw_deg":0.0}]},
                "level7":{"name":"flashback","spawn":null,
                  "player_birb_instance":{"position":[0.0,1.189,0.0],"yaw_deg":172.9341},"travel_points":[]}}}"#,
        )
        .unwrap();
        let levels = read_levels(&path).unwrap();
        assert_eq!(levels["level7"].spawn.position, [0.0, 1.189, 0.0]);

        let mut w = World::new();
        assert_eq!(add_markers(&mut w, &levels["level2"]), 1);
        let mut found: Vec<(String, SkateMarker)> = w
            .iter::<SkateMarker>()
            .map(|(e, m)| (w.get::<primase_core::Name>(e).unwrap().0.clone(), m.clone()))
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        let (name, lower) = &found[0];
        assert_eq!(name, "Lower Roof");
        assert_eq!(lower.kind, MarkerKind::Travel);
        assert_eq!(lower.position, Some(([-109.6, -89.0], Some(-3.0))));
        assert!((lower.yaw.unwrap() - 180.0).abs() < 1e-9);
        let (name, spawn) = &found[1];
        assert_eq!(name, "spawn");
        assert_eq!(spawn.kind, MarkerKind::Spawn);
        assert_eq!(spawn.position, Some(([-18.7, -84.8], Some(1.189))));
        // the frozen table the program replaces: level2's scene yaw 7.0659
        assert!((spawn.yaw.unwrap() - 7.0659).abs() < 1e-3);
        assert_eq!(spawn.snap, None);
        std::fs::remove_dir_all(&dir).ok();
    }
}
