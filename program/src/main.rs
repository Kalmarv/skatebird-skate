//! A Unity level end to end: read, lowered, written as one GLB, as web tiles and (with a surface map) as a skate scene.
//!
//! Usage: `unity_export <Game>_Data <type tree package> <level> <output folder> [--web] [--ktx2] [--surfaces <map>]`.
//! Writes `<output>/<level>.glb`; with `--web`, the web tile set in `<output>/web` (`--ktx2`: textures as KTX2); with
//! `--surfaces`, `<output>/skate/assets` and `<output>/skate/scene`. `--drop <prefix>` despawns every entity whose
//! name (or an ancestor's, in its path) starts with the prefix after lowering, before the writers. The surface map is
//! an object of the game's surface names (its physic materials, `default` for none) onto skate surfaces. Prints every
//! pass's counts and approximations.

mod settings;
mod uv_policy;
mod variants;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use primase_core::World;
use primase_core::pass::{Pass, PassLog, PassManager, PassRegistry, Pipeline, RunReport};
use primase_unity::TypeTreePackage;
use primase_unity::level::read_level;
use primase_unity::player::Player;

const USAGE: &str = "usage: unity_export <Game>_Data <type tree package> <level> <output folder> [--web] [--ktx2] [--surfaces <map>] [--spawn x,z[,yaw] | x,y,z,yaw] [--travel <json>] [--drop <prefix>]";

fn show(pass: &str, log: &PassLog) {
    println!("{pass}: {:?}", log.counts);
    if !log.approximations.is_empty() {
        println!("{pass} approximates: {:?}", log.approximations);
    }
}

/// A travel marker from the game's scan: its name, Unity position (metres) and Unity yaw in degrees.
struct TravelPoint {
    name: String,
    position: [f64; 3],
    yaw_deg: f64,
}

/// The travel points for `level` from a scan file: the whole `skatebird-levels.json` (`levels.<level>.travel_points`),
/// an object with `travel_points`, or a bare array of points. Each point needs `name`, `position` `[x, y, z]` and may
/// carry `yaw_deg`.
fn read_travel_points(
    path: &Path,
    level: &str,
) -> Result<Vec<TravelPoint>, Box<dyn std::error::Error>> {
    let doc: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let points = match &doc {
        serde_json::Value::Array(points) => points,
        serde_json::Value::Object(o) => match o.get("levels") {
            Some(levels) => levels
                .get(level)
                .and_then(|l| l.get("travel_points"))
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{}: no levels.{level}.travel_points", path.display()))?,
            None => o
                .get("travel_points")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{}: no travel_points", path.display()))?,
        },
        _ => return Err(format!("{}: not a travel point list", path.display()).into()),
    };
    let mut out = Vec::with_capacity(points.len());
    for (i, p) in points.iter().enumerate() {
        let what = format!("{}: travel_points[{i}]", path.display());
        let position = p
            .get("position")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("{what}: no position"))?;
        if position.len() < 3 {
            return Err(format!("{what}: position needs 3 numbers").into());
        }
        let (x, y, z) = (&position[0], &position[1], &position[2]);
        let coord = |v: &serde_json::Value| {
            v.as_f64()
                .ok_or_else(|| format!("{what}: position is not a number"))
        };
        let name = p
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Travel {i}"));
        out.push(TravelPoint {
            name,
            position: [coord(x)?, coord(y)?, coord(z)?],
            yaw_deg: p
                .get("yaw_deg")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
        });
    }
    Ok(out)
}

/// Adds the points as `travel` markers: Unity (x, y, z) -> scene (x, y, -z), and a reflected frame turns a Unity yaw
/// into `atan2(sin, -cos)` (the same conversion the Sekiro markers use).
fn add_travel_markers(w: &mut World, points: &[TravelPoint]) -> usize {
    for p in points {
        let a = p.yaw_deg.to_radians();
        let yaw = a.sin().atan2(-a.cos()).to_degrees();
        let e = w.spawn();
        w.insert(e, primase_core::Name(p.name.clone()));
        w.insert(
            e,
            primase_skate_scene::SkateMarker {
                kind: primase_skate_scene::MarkerKind::Travel,
                position: Some((
                    [p.position[0] as f32, -(p.position[2] as f32)],
                    Some(p.position[1] as f32),
                )),
                yaw: Some(yaw),
                m: None,
                blueprint: None,
                snap: None,
                lift: None,
                shelter: None,
                trainer: None,
                other: Default::default(),
            },
        );
    }
    points.len()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut paths, mut web, mut ktx2, mut surfaces) = (Vec::new(), false, false, None);
    let mut spawn: Option<(primase_skate_scene::Spawn, f64)> = None;
    let mut travel: Option<PathBuf> = None;
    let mut drops: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--web" => web = true,
            "--ktx2" => ktx2 = true,
            "--surfaces" => surfaces = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            "--travel" => travel = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            "--drop" => drops.push(args.next().ok_or(USAGE)?),
            "--spawn" => {
                // x,z[,yaw] (height snapped to the ground) or x,y,z,yaw (a game's own spawn height, exact)
                let text = args.next().ok_or(USAGE)?;
                let parts: Vec<&str> = text.split(',').map(str::trim).collect();
                let num = |i: usize, what: &str| -> Result<f32, Box<dyn std::error::Error>> {
                    let v = parts
                        .get(i)
                        .ok_or_else(|| format!("--spawn needs {what}"))?;
                    v.parse::<f32>()
                        .map_err(|_| format!("--spawn {what} {v:?} is not a number").into())
                };
                match parts.len() {
                    3 => {
                        let (x, z) = (num(0, "x")?, num(1, "z")?);
                        let yaw = f64::from(num(2, "yaw")?);
                        spawn = Some((primase_skate_scene::Spawn::At([x, z]), yaw));
                    }
                    4 => {
                        let (x, y, z) = (num(0, "x")?, num(1, "y")?, num(2, "z")?);
                        let yaw = f64::from(num(3, "yaw")?);
                        spawn = Some((primase_skate_scene::Spawn::AtWithHeight([x, y, z]), yaw));
                    }
                    _ => {
                        return Err(format!("--spawn needs x,z[,yaw] or x,y,z,yaw; {USAGE}").into());
                    }
                }
            }
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => paths.push(arg),
        }
    }
    let [data, tpk, level, out] = &paths[..] else {
        return Err(USAGE.into());
    };
    if ktx2 && !web {
        return Err(format!("--ktx2 needs --web; {USAGE}").into());
    }
    let out = Path::new(out);
    let player = Player::open(
        Path::new(data),
        TypeTreePackage::parse(&std::fs::read(tpk)?)?,
    )?;
    let mut w = World::new();
    let started = std::time::Instant::now();
    let report = read_level(&player, level, &mut w)?;
    println!(
        "read {level}: {} renderers, {} tree renderers, {} prototypes, {} primitives, {} materials, {} \
         textures, {} terrains, {} lights (sun named: {}), {} colliders ({} by trees); axes {:?}, conditioned {:?}, \n         LODGroups {:?}, {} LOD forms",
        report.renderers,
        report.tree_renderers,
        report.prototypes,
        report.primitives,
        report.materials,
        report.textures,
        report.terrains,
        report.lights,
        report.sun_named,
        report.colliders,
        report.tree_colliders,
        report.axes,
        report.conditioned,
        report.lod_groups,
        report.lod_forms
    );
    println!(
        "not decoded {:?}; not reproduced {:?}",
        report.textures_not_decoded, report.not_reproduced
    );
    // the lowering, as a pipeline: each pass whose input the level has, verified after each
    let passes = settings::lowering();
    let mut lowering = Vec::new();
    for p in passes {
        // a pass's input may come from an earlier pass (terrain textures for normalize-normals)
        let present =
            |w: &World, p: &dyn Pass| p.requires().iter().all(|r| w.has_component_named(r));
        if present(&w, p.as_ref()) {
            let mut report = RunReport::default();
            PassManager::new(PassRegistry::new()).run(
                &mut w,
                &Pipeline { passes: vec![p] },
                &mut report,
            )?;
            lowering.extend(report.steps);
        } else {
            println!("{}: nothing to lower", p.name());
        }
    }
    for step in &lowering {
        show(&step.pass, &step.log);
    }
    println!("lowered in {:.1} s", started.elapsed().as_secs_f64());
    // named scenery groups leave after lowering: a level's bounds are not part of its playable map
    for prefix in &drops {
        let mut report = RunReport::default();
        PassManager::new(PassRegistry::new()).run(
            &mut w,
            &Pipeline {
                passes: vec![Box::new(primase_meshops::DropByName {
                    prefix: Some(prefix.clone()),
                    equals: None,
                })],
            },
            &mut report,
        )?;
        for step in &report.steps {
            show(&step.pass, &step.log);
        }
    }
    std::fs::create_dir_all(out)?;
    if std::env::var_os("BIRD_UV_DIAGNOSTIC").is_some() {
        show("specialize-variants", &variants::specialize(&mut w)?);
        show(
            "preserve-base-tiling",
            &uv_policy::preserve_base_tiling(&mut w)?,
        );
        return Ok(());
    }
    let glb = out.join(format!("{level}.glb"));
    let r = primase_gltf::write_file(&w, &settings::gltf(level), &glb)?;
    println!("glTF {}: {r:?}", glb.display());
    if web {
        let set = primase_web::write(&w, &settings::web(ktx2))?;
        let folder = out.join("web");
        set.write_to(&folder)?;
        for o in &set.over_budget {
            println!(
                "over the budget: {} ({} bytes): {}",
                o.file, o.bytes, o.reason
            );
        }
        let tiles = &set.index.tiles;
        println!(
            "web {} (format {}): tiles={} triangles={} placements={} instanced={} lights={} collision records={} \
             images={} sun={}",
            folder.display(),
            set.index.format_version,
            tiles.len(),
            tiles.iter().map(|t| t.triangles).sum::<u64>(),
            tiles.iter().map(|t| t.placements).sum::<u64>(),
            tiles.iter().map(|t| t.instanced_placements).sum::<u64>(),
            tiles.iter().map(|t| t.lights).sum::<u64>(),
            tiles
                .iter()
                .filter_map(|t| t.collision.as_ref())
                .map(|c| c.records)
                .sum::<usize>(),
            set.index.images.len(),
            set.index
                .environment
                .as_ref()
                .map_or("none", |e| e.sun.name.as_str())
        );
    }
    let Some(map) = surfaces else { return Ok(()) };
    let surfaces: BTreeMap<String, String> = serde_json::from_slice(&std::fs::read(map)?)?;
    show("specialize-variants", &variants::specialize(&mut w)?);
    show(
        "preserve-base-tiling",
        &uv_policy::preserve_base_tiling(&mut w)?,
    );
    // skate's materials carry one wrap for all their textures: take the base colour's
    show(
        "unify-texture-wrap",
        &primase_skate_scene::unify_texture_wrap(&mut w),
    );
    // skate's material has one mask image: occlusion, roughness and metallic pack into it
    show("pack-orm", &primase_skate_scene::pack_orm(&mut w)?);
    // a normal map's scale is a material value the target has no slot for; it goes into the pixels
    show(
        "bake-normal-scale",
        &primase_image::bake_normal_scale(&mut w)?,
    );
    show(
        "raise-skate",
        &primase_skate_scene::raise_skate(
            &mut w,
            &settings::raise(
                level.replace(
                    |c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                    "_",
                ),
                surfaces,
                spawn,
            ),
        )?,
    );
    if let Some(path) = travel {
        let points = read_travel_points(&path, level)?;
        let n = add_travel_markers(&mut w, &points);
        println!("travel markers: {n} from {}", path.display());
    }
    primase_skate_scene::prepare_scene_assets(&mut w, &out.join("skate").join("assets"))?;
    let r = primase_skate_scene::write(&w, &out.join("skate").join("scene"))?;
    println!("skate scene: {r:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(dir: &Path, name: &str, text: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn travel_points_from_a_level_scan_become_markers() {
        let dir = std::env::temp_dir().join(format!("unity-export-travel-{}", std::process::id()));
        let path = scan(
            &dir,
            "skatebird-levels.json",
            r#"{"game":"SkateBIRD","levels":{"level2":{"travel_points":[
                {"name":"Middle Roof","position":[-18.7,1.189,84.8],"yaw_deg":172.9341},
                {"name":"Lower Roof","position":[-109.6,-3.0,89.0],"yaw_deg":0.0}
            ]}}}"#,
        );
        let points = read_travel_points(&path, "level2").unwrap();
        assert_eq!(points.len(), 2);

        let mut w = World::new();
        assert_eq!(add_travel_markers(&mut w, &points), 2);
        let mut found: Vec<(String, primase_skate_scene::SkateMarker)> = w
            .iter::<primase_skate_scene::SkateMarker>()
            .map(|(e, m)| (w.get::<primase_core::Name>(e).unwrap().0.clone(), m.clone()))
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));

        let (name, lower) = &found[0];
        assert_eq!(name, "Lower Roof");
        assert_eq!(lower.position, Some(([-109.6, -89.0], Some(-3.0))));
        assert!((lower.yaw.unwrap() - 180.0).abs() < 1e-9);

        let (name, middle) = &found[1];
        assert_eq!(name, "Middle Roof");
        assert_eq!(middle.position, Some(([-18.7, -84.8], Some(1.189))));
        let expected = {
            let a = 172.9341f64.to_radians();
            a.sin().atan2(-a.cos()).to_degrees()
        };
        assert!((middle.yaw.unwrap() - expected).abs() < 1e-9);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn travel_points_also_read_a_bare_array() {
        let dir =
            std::env::temp_dir().join(format!("unity-export-travel-bare-{}", std::process::id()));
        let path = scan(
            &dir,
            "points.json",
            r#"[{"name":"Start","position":[45.4,16.796,55.77],"yaw_deg":-54.8157}]"#,
        );
        let points = read_travel_points(&path, "anything").unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].name, "Start");
        assert_eq!(points[0].position, [45.4, 16.796, 55.77]);

        let bad = scan(&dir, "bad.json", r#"[{"name":"No position"}]"#);
        assert!(read_travel_points(&bad, "anything").is_err());

        std::fs::remove_dir_all(&dir).ok();
    }
}
