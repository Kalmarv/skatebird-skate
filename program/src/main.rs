//! A Unity level end to end: read, lowered, written as one GLB, as web tiles and (with a surface map) as a skate scene
//! and a ReSkate map.
//!
//! Usage: `unity_export <Game>_Data <type tree package> <level> <output folder> [--web] [--ktx2] [--surfaces <map>]
//! [--spawn ...] [--travel <json>] [--name <map name>]`. Writes `<output>/<level>.glb`; with `--web`, the web tile set
//! in `<output>/web` (`--ktx2`: textures as KTX2); with `--surfaces`, the skate scene in `<output>/skate/scene` and the
//! ReSkate map in `<output>/map/<map name>` (`SkateBIRD_<level>` unless `--name`), prepared as [`settings`] states.
//! The surface map is an object of the game's surface names (its physic materials, `default` for none) onto skate
//! surfaces. `--map-from-scene <scene folder> <output folder> --name <map name>` makes the map of a skate scene folder
//! (read and loaded) the same way. Prints every pass's counts and approximations.

mod pack;
mod settings;
mod uv_policy;
mod variants;
mod wrap;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use primase_core::World;
use primase_core::pass::{Pass, PassLog, PassManager, PassRegistry, Pipeline, RunReport};
use primase_unity::TypeTreePackage;
use primase_unity::level::read_level;
use primase_unity::player::Player;

const USAGE: &str = "usage: unity_export <Game>_Data <type tree package> <level> <output folder> [--web] [--ktx2] [--surfaces <map>] [--spawn x,z[,yaw] | x,y,z,yaw] [--travel <json>] [--name <map name>]
       unity_export --map-from-scene <scene folder> <output folder> --name <map name>";

/// Where the spawn marker goes, in scene space: (x, z) with its height found by snapping to the ground, or (x, y, z)
/// exactly (a game's own spawn height: a ceiling above it must not win).
#[derive(Clone, Copy, Debug)]
enum Spawn {
    Ground([f32; 2]),
    Exact([f32; 3]),
}

/// Runs `passes` with the pass manager (each verified after it runs) and prints each one's counts.
fn run(w: &mut World, passes: Vec<Box<dyn Pass>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut report = RunReport::default();
    PassManager::new(PassRegistry::new()).run(w, &Pipeline { passes }, &mut report)?;
    for step in &report.steps {
        show(&step.pass, &step.log);
    }
    Ok(())
}

/// The spawn marker, as the program states it: named `spawn`, at `spawn` facing `yaw` degrees (0 faces +Z,
/// clockwise seen from above), snapped to the ground only when it has no height.
fn add_spawn_marker(w: &mut World, spawn: Spawn, yaw: f64) {
    let (position, snap) = match spawn {
        Spawn::Ground(p) => (
            (p, None),
            Some(primase_reskate::dialect::Snap::Mode(
                primase_reskate::dialect::SnapMode::Ground,
            )),
        ),
        Spawn::Exact([x, y, z]) => (([x, z], Some(y)), None),
    };
    let e = w.spawn();
    w.insert(e, primase_core::Name("spawn".into()));
    w.insert(
        e,
        primase_reskate::dialect::SkateMarker {
            kind: primase_reskate::dialect::MarkerKind::Spawn,
            position: Some(position),
            yaw: Some(yaw),
            m: None,
            blueprint: None,
            snap,
            lift: None,
            shelter: None,
            trainer: None,
            other: Default::default(),
        },
    );
}

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
            primase_reskate::dialect::SkateMarker {
                kind: primase_reskate::dialect::MarkerKind::Travel,
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
    let mut spawn: Option<(Spawn, f64)> = None;
    let mut travel: Option<PathBuf> = None;
    let mut map_name: Option<String> = None;
    let mut map_from_scene = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--web" => web = true,
            "--ktx2" => ktx2 = true,
            "--surfaces" => surfaces = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            "--travel" => travel = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            "--name" => map_name = Some(args.next().ok_or(USAGE)?),
            "--map-from-scene" => map_from_scene = true,
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
                        spawn = Some((Spawn::Ground([x, z]), yaw));
                    }
                    4 => {
                        let (x, y, z) = (num(0, "x")?, num(1, "y")?, num(2, "z")?);
                        let yaw = f64::from(num(3, "yaw")?);
                        spawn = Some((Spawn::Exact([x, y, z]), yaw));
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
    if map_from_scene {
        // a skate scene folder (this program's, or another's) -> the map, prepared as this program prepares it
        let [scene, out] = &paths[..] else {
            return Err(USAGE.into());
        };
        let scene = Path::new(scene);
        let mut w = World::new();
        primase_reskate::interchange::read(scene, &mut w)?;
        primase_reskate::interchange::load(scene, &mut w)?;
        let name = map_name.ok_or("--map-from-scene needs --name")?;
        return write_the_map(&mut w, Path::new(out), &name);
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
    // the glTF's own needs, on a copy: the world the skate scene is made from stays as lowered
    let mut for_gltf = w.clone();
    run(&mut for_gltf, settings::gltf_preparation()?)?;
    let r = primase_gltf::write_file(&for_gltf, &settings::gltf(level), &glb)?;
    drop(for_gltf);
    println!("glTF {}: {r:?}", glb.display());
    if web {
        // the web's texture sizes on a copy, as the glTF's needs: the skate scene is made from the world as lowered
        let mut for_web = w.clone();
        if ktx2 {
            run(&mut for_web, vec![Box::new(settings::web_textures()?)])?;
        }
        let set = primase_web::write(&for_web, &settings::web(ktx2)?)?;
        drop(for_web);
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
    // skate's materials carry one wrap for all their textures: take the base colour's; and one mask image:
    // occlusion, roughness and metallic pack into it
    run(
        &mut w,
        vec![Box::new(wrap::UnifyTextureWrap), Box::new(pack::PackOrm)],
    )?;
    // a normal map's scale is a material value the target has no slot for; it goes into the pixels
    show(
        "bake-normal-scale",
        &primase_image::bake_normal_scale(&mut w)?,
    );
    let scene_name = level.replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "_",
    );
    show(
        "raise-skate",
        &primase_reskate::prepare::raise_skate(&mut w, &settings::raise(&scene_name, surfaces)?)?,
    );
    if let Some((at, yaw)) = spawn {
        add_spawn_marker(&mut w, at, yaw);
        println!("spawn marker: {at:?}, yaw {yaw}");
    }
    if let Some(path) = travel {
        let points = read_travel_points(&path, level)?;
        let n = add_travel_markers(&mut w, &points);
        println!("travel markers: {n} from {}", path.display());
    }
    let r = primase_reskate::interchange::write(&w, &out.join("skate").join("scene"))?;
    println!("skate scene: {r:?}");
    let name = map_name.unwrap_or_else(|| format!("SkateBIRD_{level}"));
    write_the_map(&mut w, out, &name)
}

/// The ReSkate map of a raised world into `<out>/map/<name>`: prepared as the program chooses ([`settings`]), then
/// the writer.
fn write_the_map(w: &mut World, out: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encoding = settings::encoding(out.join("map"), name);
    run(w, settings::map_preparation(&encoding)?)?;
    let r = primase_reskate::write_map(w, Path::new(""), &encoding)?;
    println!(
        "map '{}': {} tiles, {} merged placements / {:.2} M tris, {} instances of {} prototypes, {} object records, {} materials, {} textures, collision {:.2} M tris",
        r.out_dir.display(),
        r.tiles,
        r.merged_entities,
        r.merged_tris as f64 / 1e6,
        r.instances,
        r.prototypes,
        r.object_records,
        r.materials,
        r.textures,
        r.collision_tris as f64 / 1e6,
    );
    for (what, n) in &r.unsupported {
        println!("map: not written: {what} x {n}");
    }
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
        let mut found: Vec<(String, primase_reskate::dialect::SkateMarker)> = w
            .iter::<primase_reskate::dialect::SkateMarker>()
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
