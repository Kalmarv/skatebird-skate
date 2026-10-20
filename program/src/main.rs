//! The SkateBIRD program: each level read with primase's Unity reader, lowered and prepared as [`settings`] states,
//! written as a ReSkate map with primase's writer, then compiled by ReSkate Studio into a mod (a later, visible stage:
//! [`compile`]).
//!
//! ```text
//! skatebird-export build --typetrees <type tree package> --out <folder> [--game-data <SkateBIRD_Data>]
//!                        [--data <folder>] [--studio <reskate_cli.exe> [--game <skate. install>]]
//!                        [--glb] [--web [--ktx2]] [--scene] [--jobs N] [LEVEL ...]
//! skatebird-export map-from-scene <scene folder> <output folder> --name <map name>
//! ```
//!
//! `build` converts the levels named (every level of `data/skatebird-levels.json` when none is), each into
//! `<out>/<level>/`: `map/SkateBIRD_<level>` (the ReSkate map, with the game's spawn and travel points), and with
//! `--studio` its compile `stage/Patch` (the mod folder `SkateBIRD_<level>`, against `--game`, default
//! `$RESKATE_GAME`). `--glb` adds the level as one glTF, `--web` the web tile set (`--ktx2`: KTX2 textures),
//! `--scene` the skate scene folder (interchange v1). Levels run `--jobs` at a time (3), each in its own process with
//! its output in `<out>/<level>/build.log`. A level that fails is reported and the others still run; the
//! exit code says whether all did. `map-from-scene` makes the map of a skate scene folder the same way. Every pass's
//! counts and approximations are printed.

mod ambient;
mod compile;
mod level;
mod package;
mod settings;
mod uv_policy;
mod variants;
mod views;
mod wrap;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use primase_core::World;
use primase_core::pass::{Pass, PassLog, PassManager, PassRegistry, Pipeline, RunReport};
use primase_unity::TypeTreePackage;
use primase_unity::level::read_level;
use primase_unity::player::Player;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "usage: skatebird-export build --typetrees <tpk> --out <folder> [--game-data <SkateBIRD_Data>] [--data <folder>] [--studio <reskate_cli.exe> [--game <skate. install>]] [--glb] [--web [--ktx2]] [--scene] [--jobs N] [--light-scale S] [--ambient-fill M] [--tag T] [LEVEL ...]
       skatebird-export map-from-scene <scene folder> <output folder> --name <map name>
       skatebird-export package --out <build folder> --dest <folder> --version <version> [--data <folder>] [LEVEL ...]
       skatebird-export views make [--data <folder>]            (writes <data>/skatebird-views.json)
       skatebird-export views skate-commands LEVEL [--data <folder>]
       skatebird-export sheets --refs <SkateBIRD shots> --run <run folder> --out <folder>";

/// Where Steam installs the game.
const STEAM_DATA: &str = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data";

/// What `build` writes besides the map, and the experiments it may run.
#[derive(Clone, Default)]
struct Outputs {
    glb: bool,
    web: bool,
    ktx2: bool,
    scene: bool,
    /// `--light-scale S`: lights calibrated by this scale instead of [`settings::LIGHTS`].
    light_scale: Option<f64>,
    /// `--ambient-fill SPACING`: the ambient's fill lights this many metres apart instead of [`settings::AMBIENT_FILL`].
    fill: Option<f64>,
    /// `--tag T`: the level's folder, map and mod named `..._T`, so variants of a level stand side by side.
    tag: Option<String>,
}

impl Outputs {
    /// The level's name with the tag: its folder under `--out`, and after `SkateBIRD_` its map and mod.
    fn named(&self, level: &str) -> String {
        match &self.tag {
            Some(t) => format!("{level}_{t}"),
            None => level.to_owned(),
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("build") => build(args.collect()),
        Some("map-from-scene") => map_from_scene(args.collect()),
        Some("package") => package(args.collect()),
        Some("views") => views_command(args.collect()),
        Some("sheets") => sheets(args.collect()),
        _ => Err(USAGE.into()),
    }
}

fn build(args: Vec<String>) -> Result<()> {
    let mut outputs = Outputs::default();
    let (mut typetrees, mut out, mut studio_cli, mut game) = (None, None, None, None);
    let mut game_data = PathBuf::from(STEAM_DATA);
    let mut data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");
    let mut wanted = Vec::new();
    let mut jobs = 3;
    // the options as given, for the per-level children
    let mut pass_on = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--jobs" {
            let n = args.next().ok_or(USAGE)?;
            jobs = n
                .parse::<usize>()
                .ok()
                .filter(|&n| n > 0)
                .ok_or(format!("--jobs {n}: a count above 0"))?;
            continue;
        }
        if arg.starts_with("--") {
            pass_on.push(arg.clone());
        }
        let mut value = || {
            let v = args.next().ok_or(USAGE)?;
            pass_on.push(v.clone());
            Ok::<_, &str>(PathBuf::from(v))
        };
        match arg.as_str() {
            "--typetrees" => typetrees = Some(value()?),
            "--out" => out = Some(value()?),
            "--game-data" => game_data = value()?,
            "--data" => data = value()?,
            "--studio" => studio_cli = Some(value()?),
            "--game" => game = Some(value()?),
            "--glb" => outputs.glb = true,
            "--web" => outputs.web = true,
            "--ktx2" => outputs.ktx2 = true,
            "--scene" => outputs.scene = true,
            "--light-scale" | "--ambient-fill" | "--tag" => {
                let text = value()?.to_string_lossy().into_owned();
                let number = || {
                    text.parse::<f64>()
                        .ok()
                        .filter(|n| n.is_finite() && *n > 0.0)
                        .ok_or(format!("{arg} {text}: a number above 0"))
                };
                match arg.as_str() {
                    "--light-scale" => outputs.light_scale = Some(number()?),
                    "--ambient-fill" => outputs.fill = Some(number()?),
                    _ => outputs.tag = Some(text.clone()),
                }
            }
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => wanted.push(arg),
        }
    }
    let (Some(typetrees), Some(out)) = (typetrees, out) else {
        return Err(format!("build needs --typetrees and --out; {USAGE}").into());
    };
    if outputs.ktx2 && !outputs.web {
        return Err(format!("--ktx2 needs --web; {USAGE}").into());
    }
    let studio = match studio_cli {
        Some(cli) => Some(compile::Studio {
            cli,
            game: match game.or_else(|| std::env::var_os("RESKATE_GAME").map(PathBuf::from)) {
                Some(g) => g,
                None => return Err("--studio needs --game (or RESKATE_GAME)".into()),
            },
        }),
        None if game.is_some() => return Err("--game needs --studio".into()),
        None => None,
    };
    let levels = level::read_levels(&data.join("skatebird-levels.json"))?;
    let surfaces: BTreeMap<String, String> =
        serde_json::from_slice(&std::fs::read(data.join("skatebird-surfaces.json"))?)?;
    let chosen: Vec<&level::Level> = if wanted.is_empty() {
        levels.values().collect()
    } else {
        wanted
            .iter()
            .map(|id| {
                levels
                    .get(id)
                    .ok_or_else(|| format!("no level {id} in the scan"))
            })
            .collect::<std::result::Result<_, _>>()?
    };
    if jobs > 1 && chosen.len() > 1 {
        return build_in_children(&chosen, &pass_on, jobs, &out, &outputs);
    }
    let player = Player::open(
        &game_data,
        TypeTreePackage::parse(&std::fs::read(&typetrees)?)?,
    )?;
    let mut results = Vec::new();
    for l in chosen {
        println!("== {} ({})", l.id, l.name);
        let started = std::time::Instant::now();
        let named = outputs.named(&l.id);
        let dir = out.join(&named);
        let result = convert(&player, l, &surfaces, &outputs, &dir).and_then(|map| {
            let Some(studio) = &studio else {
                return Ok(format!("map {}", map.display()));
            };
            let c = compile::compile(
                studio,
                &map,
                &dir.join("stage"),
                &settings::map_name(&named),
            )?;
            Ok(format!(
                "map and Studio compile ({:.0} s, {} warnings, log {}): {}",
                c.seconds,
                c.warnings,
                c.log.display(),
                c.patch.display()
            ))
        });
        let line = match &result {
            Ok(what) => format!(
                "{}: ok in {:.0} s, {what}",
                l.id,
                started.elapsed().as_secs_f64()
            ),
            Err(e) => format!(
                "{}: FAILED after {:.0} s: {e}",
                l.id,
                started.elapsed().as_secs_f64()
            ),
        };
        println!("{line}");
        results.push((result.is_ok(), line));
    }
    println!("== summary");
    for (_, line) in &results {
        println!("{line}");
    }
    let failed = results.iter().filter(|(ok, _)| !ok).count();
    if failed > 0 {
        return Err(format!("{failed} of {} levels failed", results.len()).into());
    }
    Ok(())
}

/// `build` of each level in its own process (the same options, that one level), `jobs` at a time, each one's output
/// in `<out>/<level>/build.log`; then each level's result line.
fn build_in_children(
    chosen: &[&level::Level],
    options: &[String],
    jobs: usize,
    out: &Path,
    outputs: &Outputs,
) -> Result<()> {
    let exe = std::env::current_exe()?;
    let started = std::time::Instant::now();
    let mut queue: std::collections::VecDeque<&level::Level> = chosen.iter().copied().collect();
    let mut running: Vec<(&level::Level, PathBuf, std::process::Child)> = Vec::new();
    let mut results = Vec::new();
    while !queue.is_empty() || !running.is_empty() {
        while running.len() < jobs
            && let Some(l) = queue.pop_front()
        {
            let dir = out.join(outputs.named(&l.id));
            std::fs::create_dir_all(&dir)?;
            let log = dir.join("build.log");
            let file = std::fs::File::create(&log)?;
            let child = std::process::Command::new(&exe)
                .arg("build")
                .args(options)
                .args(["--jobs", "1", &l.id])
                .stdout(file.try_clone()?)
                .stderr(file)
                .spawn()?;
            println!("{}: started, log {}", l.id, log.display());
            running.push((l, log, child));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut i = 0;
        while i < running.len() {
            if let Some(status) = running[i].2.try_wait()? {
                let (l, log, _) = running.swap_remove(i);
                let text = std::fs::read_to_string(&log).unwrap_or_default();
                let prefix = format!("{}: ", l.id);
                let line = text
                    .lines()
                    .rfind(|line| line.starts_with(&prefix))
                    .map_or_else(
                        || format!("{}: exit {status}, see {}", l.id, log.display()),
                        str::to_owned,
                    );
                println!("{line}");
                results.push((status.success(), line));
            } else {
                i += 1;
            }
        }
    }
    println!(
        "== summary ({:.0} s, {jobs} at a time)",
        started.elapsed().as_secs_f64()
    );
    results.sort_by(|a, b| a.1.cmp(&b.1));
    for (_, line) in &results {
        println!("{line}");
    }
    let failed = results.iter().filter(|(ok, _)| !ok).count();
    if failed > 0 {
        return Err(format!("{failed} of {} levels failed", results.len()).into());
    }
    Ok(())
}

/// Each built level as an installable mod folder (`<out>/<level>/mod/SkateBIRD_<level>`) and its zip in `--dest`
/// ([`package`]).
fn package(args: Vec<String>) -> Result<()> {
    let (mut out, mut dest, mut version, mut wanted) = (None, None, None, Vec::new());
    let mut data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(USAGE);
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(value()?)),
            "--dest" => dest = Some(PathBuf::from(value()?)),
            "--version" => version = Some(value()?),
            "--data" => data = PathBuf::from(value()?),
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => wanted.push(arg),
        }
    }
    let (Some(out), Some(dest), Some(version)) = (out, dest, version) else {
        return Err(format!("package needs --out, --dest and --version; {USAGE}").into());
    };
    let levels = level::read_levels(&data.join("skatebird-levels.json"))?;
    let chosen: Vec<&level::Level> = if wanted.is_empty() {
        levels.values().collect()
    } else {
        wanted
            .iter()
            .map(|id| {
                levels
                    .get(id)
                    .ok_or_else(|| format!("no level {id} in the scan"))
            })
            .collect::<std::result::Result<_, _>>()?
    };
    std::fs::create_dir_all(&dest)?;
    for l in chosen {
        let name = settings::map_name(&l.id);
        let folder = package::mod_folder(&out.join(&l.id), l, &name, &version)?;
        let zip = dest.join(format!("{name}-{version}.zip"));
        let bytes = package::zip(&folder, &zip)?;
        println!("{}: {} ({:.1} MB)", l.id, zip.display(), bytes as f64 / 1e6);
    }
    Ok(())
}

/// `views make`: the views file from the scan; `views skate-commands LEVEL`: one run's console commands ([`views`]).
fn views_command(args: Vec<String>) -> Result<()> {
    let mut data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");
    let mut words = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data" => data = PathBuf::from(args.next().ok_or(USAGE)?),
            _ => words.push(arg),
        }
    }
    let file = data.join("skatebird-views.json");
    match &words[..] {
        [w] if w == "make" => {
            let levels = level::read_levels(&data.join("skatebird-levels.json"))?;
            let v = views::make(&levels);
            std::fs::write(
                &file,
                serde_json::to_string_pretty(&v)?
                    + "
",
            )?;
            println!(
                "{}: {} levels",
                file.display(),
                v.as_object().map_or(0, |o| o.len())
            );
        }
        [w, level] if w == "skate-commands" => {
            let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&file)?)?;
            println!("{}", views::skate_commands(&v, level)?);
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

/// `sheets`: side-by-side pictures of a run against SkateBIRD's reference shots ([`views::sheets`]).
fn sheets(args: Vec<String>) -> Result<()> {
    let (mut refs, mut run, mut out) = (None, None, None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let v = PathBuf::from(args.next().ok_or(USAGE)?);
        match arg.as_str() {
            "--refs" => refs = Some(v),
            "--run" => run = Some(v),
            "--out" => out = Some(v),
            _ => return Err(format!("unknown option {arg}; {USAGE}").into()),
        }
    }
    let (Some(refs), Some(run), Some(out)) = (refs, run, out) else {
        return Err(USAGE.into());
    };
    for p in views::sheets(&refs, &run, &out)? {
        println!("{p}");
    }
    Ok(())
}

fn map_from_scene(args: Vec<String>) -> Result<()> {
    let (mut paths, mut name) = (Vec::new(), None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--name" => name = Some(args.next().ok_or(USAGE)?),
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => paths.push(arg),
        }
    }
    let ([scene, out], Some(name)) = (&paths[..], name) else {
        return Err(USAGE.into());
    };
    // a skate scene folder (this program's, or another's) -> the map, prepared as this program prepares it
    let scene = Path::new(scene);
    let mut w = World::new();
    primase_reskate::read::interchange::read(scene, &mut w)?;
    primase_reskate::read::interchange::load(scene, &mut w)?;
    write_the_map(&mut w, Path::new(out), &name).map(|_| ())
}

/// One level into `out`: read, lowered, the optional outputs, then raised for skate and written as its map; returns
/// the map folder.
fn convert(
    player: &Player,
    l: &level::Level,
    surfaces: &BTreeMap<String, String>,
    outputs: &Outputs,
    out: &Path,
) -> Result<PathBuf> {
    let mut w = World::new();
    let started = std::time::Instant::now();
    let report = read_level(player, &l.id, &mut w)?;
    println!(
        "read {}: {} renderers, {} tree renderers, {} prototypes, {} primitives, {} materials, {} textures, {} \
         terrains, {} lights (sun named: {}), {} colliders ({} by trees); axes {:?}, conditioned {:?}, LODGroups {:?}, \
         {} LOD forms",
        l.id,
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
        "not decoded {:?}; not reproduced {:?}; read otherwise {:?}; colliders not colliding {:?}",
        report.textures_not_decoded,
        report.not_reproduced,
        report.notices,
        report.colliders_not_colliding
    );
    // the lowering, as a pipeline: each pass whose input the level has, verified after each
    for p in settings::lowering() {
        // a pass's input may come from an earlier pass (terrain textures for normalize-normals)
        if p.requires().iter().all(|r| w.has_component_named(r)) {
            run(&mut w, vec![p])?;
        } else {
            println!("{}: nothing to lower", p.name());
        }
    }
    println!("lowered in {:.1} s", started.elapsed().as_secs_f64());
    std::fs::create_dir_all(out)?;
    if outputs.glb {
        write_glb(&w, &l.id, out)?;
    }
    if outputs.web {
        write_web(&w, outputs.ktx2, out)?;
    }
    let spacing = outputs.fill.unwrap_or(settings::AMBIENT_FILL);
    let bounds = l
        .bounds
        .ok_or("the ambient fill needs the level's bounds in the scan")?;
    show("ambient-fill", &ambient::add_fill(&mut w, bounds, spacing));
    let lights = match outputs.light_scale {
        Some(s) => primase_reskate::passes::LightCalibration::Scale(s),
        None => settings::LIGHTS,
    };
    // skate does not draw blended world geometry: blended surfaces (not decals) as cutouts
    run(&mut w, vec![Box::new(settings::blended_surfaces()?)])?;
    one_uv_set(&mut w)?;
    // a part with no material (an empty Unity material slot) has no surface skate can draw
    run(
        &mut w,
        vec![Box::new(primase_meshops::OmitMateriallessPrimitives)],
    )?;
    // skate's materials carry one wrap for all their textures: take the base colour's; and one mask image:
    // occlusion, roughness and metallic pack into it
    run(
        &mut w,
        vec![
            Box::new(wrap::UnifyTextureWrap),
            Box::new(primase_image::PackOrm::new(
                primase_image::OrmSize::MetallicRoughness,
                primase_image::MissingLevels::Generate,
            )),
        ],
    )?;
    // a normal map's scale is a material value the target has no slot for; it goes into the pixels
    show(
        "bake-normal-scale",
        &primase_image::bake_normal_scale(&mut w)?,
    );
    show(
        "raise-skate",
        &primase_reskate::prepare::raise_skate(
            &mut w,
            &settings::raise(&l.id, surfaces.clone(), lights)?,
        )?,
    );
    let travel = level::add_markers(&mut w, l);
    println!(
        "markers: spawn at {:?} (Unity), yaw {}; {travel} travel",
        l.spawn.position, l.spawn.yaw_deg
    );
    if outputs.scene {
        let r = primase_reskate::write::interchange::write(&w, &out.join("skate").join("scene"))?;
        println!("skate scene: {} tables written", r.tables.len());
    }
    write_the_map(&mut w, out, &settings::map_name(&outputs.named(&l.id)))
}

/// Runs `passes` with the pass manager (each verified after it runs) and prints each one's counts.
fn run(w: &mut World, passes: Vec<Box<dyn Pass>>) -> Result<()> {
    let mut report = RunReport::default();
    PassManager::new(PassRegistry::new()).run(w, &Pipeline { passes }, &mut report)?;
    for step in &report.steps {
        show(&step.pass, &step.log);
    }
    Ok(())
}

fn show(pass: &str, log: &PassLog) {
    println!("{pass}: {:?}", log.counts);
    if !log.approximations.is_empty() {
        println!("{pass} approximates: {:?}", log.approximations);
    }
}

/// The level as one glTF, from a copy given the glTF's own needs (the world the map is made from stays as lowered).
fn write_glb(w: &World, level: &str, out: &Path) -> Result<()> {
    let glb = out.join(format!("{level}.glb"));
    let mut for_gltf = w.clone();
    run(&mut for_gltf, settings::gltf_preparation()?)?;
    let r = primase_gltf::write_file(&for_gltf, &settings::gltf(level), &glb)?;
    println!("glTF {}: {r:?}", glb.display());
    Ok(())
}

/// The web tile set, from a copy given the web's texture sizes.
fn write_web(w: &World, ktx2: bool, out: &Path) -> Result<()> {
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
    Ok(())
}

/// skate's one texture-coordinate set: every material's tiling baked into its primitives' coordinates. A primitive
/// stored without the set its texture samples reads (0, 0) in Unity, so it gets the set as zeros first; placements
/// with other materials get their own geometry; then the base tiling is kept ([`uv_policy`]).
fn one_uv_set(w: &mut World) -> Result<()> {
    run(
        w,
        vec![Box::new(primase_gltf::prepare::FillMissingTexcoords)],
    )?;
    show("specialize-variants", &variants::specialize(w)?);
    show("preserve-base-tiling", &uv_policy::preserve_base_tiling(w)?);
    Ok(())
}

/// The ReSkate map of a raised world into `<out>/map/<name>`: prepared as the program chooses ([`settings`]), then
/// the writer; returns the map folder.
fn write_the_map(w: &mut World, out: &Path, name: &str) -> Result<PathBuf> {
    let encoding = settings::encoding(out.join("map"), name);
    run(w, settings::map_preparation(&encoding)?)?;
    let r = primase_reskate::write_map(w, Path::new(""), &encoding)?;
    println!(
        "map '{}': {} tiles, {} merged placements / {:.2} M tris, {} instances of {} prototypes, {} object records, \
         {} materials, {} textures, collision {:.2} M tris",
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
    for (what, n) in &r.omitted {
        println!("map: not written: {what} x {n}");
    }
    for (what, n) in &r.approximated {
        println!("map: approximated: {what} x {n}");
    }
    for (what, n) in &r.unsupported_names {
        println!("map: unsupported: {what} x {n}");
    }
    for warning in &r.warnings {
        println!("map warning: {warning}");
    }
    Ok(r.out_dir)
}
