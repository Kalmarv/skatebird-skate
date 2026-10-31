//! The SkateBIRD tooling around the conversion (`program/`, the recipe): each built level packaged as a release
//! (mod folder, icon, zip), the skate. console commands of the levels' views, the SkateBIRD reference shots in one
//! launch of the lab copy, and side-by-side sheets of a run against them.
//!
//! ```text
//! skatebird-tools package --out <build folder> --dest <folder> --version <version> [--icons <folder of LEVEL.png>] [LEVEL ...]
//! skatebird-tools views skate-commands LEVEL [--data <folder>]
//! skatebird-tools views map-name LEVEL
//! skatebird-tools sheets --refs <SkateBIRD shots> --run <run folder> --out <folder>
//! skatebird-tools skatebird-shots <level>[,<level>...] <out> [--chase] [--back D] [--eye H] [--fov F] [--pitch P]
//!                 [--yaw-offset Y] [--settle S] [--load-settle S] [--keep-post] [--keep-npcs] [--keep] [--data <folder>]
//! ```

mod package;
mod shots;
mod views;

use std::path::{Path, PathBuf};

use skatebird_export::levels;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "usage: skatebird-tools package --out <build folder> --dest <folder> --version <version> [--icons <folder of LEVEL.png>] [LEVEL ...]
       skatebird-tools views skate-commands LEVEL [--data <folder>]
       skatebird-tools views map-name LEVEL
       skatebird-tools sheets --refs <SkateBIRD shots> --run <run folder> --out <folder>
       skatebird-tools skatebird-shots <level>[,<level>...] <out> [--chase] [--back D] [--eye H] [--fov F] [--pitch P] [--yaw-offset Y] [--settle S] [--load-settle S] [--keep-post] [--keep-npcs] [--keep] [--data <folder>]";

/// The repository's data folder (views, surfaces), next to this crate.
fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("package") => package(args.collect()),
        Some("views") => views_command(args.collect()),
        Some("sheets") => sheets(args.collect()),
        Some("skatebird-shots") => skatebird_shots(args.collect()),
        _ => Err(USAGE.into()),
    }
}

/// Each built level as an installable mod folder (`<out>/<level>/mod/SkateBIRD_<level>`) and its zip in `--dest`
/// ([`package`]).
fn package(args: Vec<String>) -> Result<()> {
    let (mut out, mut dest, mut version, mut icons, mut wanted) =
        (None, None, None, None, Vec::new());
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(USAGE);
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(value()?)),
            "--dest" => dest = Some(PathBuf::from(value()?)),
            "--version" => version = Some(value()?),
            "--icons" => icons = Some(PathBuf::from(value()?)),
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => wanted.push(arg),
        }
    }
    let (Some(out), Some(dest), Some(version)) = (out, dest, version) else {
        return Err(format!("package needs --out, --dest and --version; {USAGE}").into());
    };
    let chosen = levels::chosen(&wanted)?;
    std::fs::create_dir_all(&dest)?;
    for l in chosen {
        let name = levels::map_name(l.id);
        // the mod's picture: `<icons>/<level>.png`, a screenshot of the level in skate
        let shot = icons.as_ref().map(|d| d.join(format!("{}.png", l.id)));
        let png = match &shot {
            Some(p) if p.is_file() => Some(std::fs::read(p)?),
            _ => None,
        };
        if png.is_none() {
            println!(
                "{}: no icon ({})",
                l.id,
                shot.map_or("no --icons".into(), |p| format!("{} missing", p.display()))
            );
        }
        let folder = package::mod_folder(&out.join(l.id), &l, &name, &version, png.as_deref())?;
        let zip = dest.join(format!("{name}-{version}.zip"));
        let bytes = package::zip(&folder, &zip)?;
        println!("{}: {} ({:.1} MB)", l.id, zip.display(), bytes as f64 / 1e6);
    }
    Ok(())
}

/// `views skate-commands LEVEL`: one run's console commands for the views of `data/skatebird-views.json` ([`views`]).
fn views_command(args: Vec<String>) -> Result<()> {
    let mut data = data_dir();
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
        [w, level] if w == "map-name" => println!("{}", levels::map_name(level)),
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

/// `skatebird-shots`: SkateBIRD's reference shots of the levels' views in one launch of the lab copy ([`shots`]):
/// the eye preset, or with `--chase` skate.'s third-person camera, each value overridable; exit status 1 when a shot
/// or check failed.
fn skatebird_shots(args: Vec<String>) -> Result<()> {
    let mut data = data_dir();
    let mut o = shots::Options::default();
    let (mut chase, mut over) = (false, Vec::new());
    let mut words = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let number = |args: &mut std::vec::IntoIter<String>| -> Result<f64> {
            let v = args.next().ok_or(USAGE)?;
            Ok(v.parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .ok_or(format!("{arg} {v}: a number"))?)
        };
        match arg.as_str() {
            "--chase" => chase = true,
            "--back" | "--eye" | "--fov" | "--pitch" | "--yaw-offset" => {
                let n = number(&mut args)?;
                over.push((arg.clone(), n));
            }
            "--settle" => o.settle = number(&mut args)?,
            "--load-settle" => o.load_settle = number(&mut args)?,
            "--keep-post" => o.keep_post = true,
            "--keep-npcs" => o.keep_npcs = true,
            "--keep" => o.keep_game = true,
            "--data" => data = PathBuf::from(args.next().ok_or(USAGE)?),
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => words.push(arg),
        }
    }
    let [levels, out] = &words[..] else {
        return Err(USAGE.into());
    };
    if chase {
        o.mode = "chase";
        o.camera = shots::Camera::CHASE;
    }
    for (name, n) in over {
        let c = &mut o.camera;
        *match name.as_str() {
            "--back" => &mut c.back,
            "--eye" => &mut c.eye,
            "--fov" => &mut c.fov,
            "--pitch" => &mut c.pitch,
            _ => &mut c.yaw_offset,
        } = n;
    }
    if !(o.camera.fov > 1.0 && o.camera.fov < 179.0) {
        return Err(format!("--fov {}: between 1 and 179 degrees", o.camera.fov).into());
    }
    let views: serde_json::Value =
        serde_json::from_slice(&std::fs::read(data.join("skatebird-views.json"))?)?;
    let levels: Vec<String> = levels
        .split(',')
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    let failed = shots::run(&views, &levels, Path::new(out), &o)?;
    let summary: Vec<String> = failed
        .iter()
        .map(|(l, n)| format!("{l} {n} failed"))
        .collect();
    println!("summary: {}", summary.join(", "));
    if failed.iter().any(|(_, n)| *n > 0) {
        std::process::exit(1);
    }
    Ok(())
}
