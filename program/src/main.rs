//! The SkateBIRD conversion: each level into a ReSkate map and a Studio-compiled mod ([`recipe`]).
//!
//! ```text
//! skatebird-export --typetrees <type tree package> --out <folder> [--game-data <SkateBIRD_Data>]
//!                  [--studio <reskate_cli.exe> [--game <skate. install>] [--oodle <oo2core_9_win64.dll>]] [--jobs N] [LEVEL ...]
//! ```
//!
//! Converts the levels named (every level of [`levels::LEVELS`] when none is), each into `<out>/<level>/`:
//! `map/SkateBIRD_<Name>`, and with `--studio` its compile `stage/Patch` against `--game` (default `$RESKATE_GAME`) and `--oodle` (default `<game>/oo2core_9_win64.dll`).
//! Levels run `--jobs` at a time (3), each in its own process with its output in `<out>/<level>/build.log`; a level
//! that fails is reported and the others still run, and the exit code says whether all did. Packaging, views and
//! shots are the tooling's (`tools/skatebird-tools`).

mod choices;
mod rails;
mod recipe;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use primase_foundations::compress::oodle::{Oodle, Version};
use primase_reskate::compile::Studio;
use skatebird_export::levels;
use unity_formats::TypeTreePackage;
use unity_formats::player::Player;

use primase_core::convert::Result;

const USAGE: &str = "usage: skatebird-export --typetrees <tpk> --out <folder> [--game-data <SkateBIRD_Data>] [--studio <reskate_cli.exe> [--game <skate. install>] [--oodle <oo2core_9_win64.dll>]] [--jobs N] [LEVEL ...]";

/// Where Steam installs the game.
const STEAM_DATA: &str = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data";

fn main() -> Result<()> {
    let (mut typetrees, mut out, mut cli, mut game, mut oodle) = (None, None, None, None, None);
    let (mut game_data, mut jobs, mut wanted) = (PathBuf::from(STEAM_DATA), 3usize, Vec::new());
    // the options as given, for the per-level children
    let mut pass_on = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(USAGE);
        match arg.as_str() {
            "--jobs" => {
                let n = value()?;
                jobs = n
                    .parse()
                    .ok()
                    .filter(|&n| n > 0)
                    .ok_or(format!("--jobs {n}: a count above 0"))?;
            }
            "--typetrees" | "--out" | "--game-data" | "--studio" | "--game" | "--oodle" => {
                let v = value()?;
                pass_on.extend([arg.clone(), v.clone()]);
                let v = PathBuf::from(v);
                match arg.as_str() {
                    "--typetrees" => typetrees = Some(v),
                    "--out" => out = Some(v),
                    "--game-data" => game_data = v,
                    "--studio" => cli = Some(v),
                    "--oodle" => oodle = Some(v),
                    _ => game = Some(v),
                }
            }
            _ if arg.starts_with("--") => {
                return Err(format!("unknown option {arg}; {USAGE}").into());
            }
            _ => wanted.push(arg),
        }
    }
    let (Some(typetrees), Some(out)) = (typetrees, out) else {
        return Err(USAGE.into());
    };
    let studio = match (
        cli,
        game.or_else(|| std::env::var_os("RESKATE_GAME").map(PathBuf::from)),
    ) {
        (Some(cli), Some(game)) => Some(Studio { cli, game }),
        (Some(_), None) => return Err("--studio needs --game (or RESKATE_GAME)".into()),
        (None, _) => None,
    };
    // the install's Oodle library, which the compile's finishing reads the game's data with
    let oodle = match &studio {
        Some(s) => {
            let dll = oodle.unwrap_or_else(|| s.game.join("oo2core_9_win64.dll"));
            // SAFETY: the caller names the game's own Oodle library.
            #[allow(unsafe_code)]
            Some(unsafe { Oodle::load(dll, Version::V9) }?)
        }
        None => None,
    };
    let chosen = levels::chosen(&wanted)?;
    if jobs > 1 && chosen.len() > 1 {
        return in_children(&chosen, &pass_on, jobs, &out);
    }
    let surfaces: BTreeMap<String, String> = serde_json::from_slice(&std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/skatebird-surfaces.json"),
    )?)?;
    let player = Player::open(
        &game_data,
        TypeTreePackage::parse(&std::fs::read(&typetrees)?)?,
    )?;
    let mut lines = Vec::new();
    for l in chosen {
        println!("== {} ({})", l.id, l.name);
        let started = std::time::Instant::now();
        let secs = || started.elapsed().as_secs_f64();
        let line = match recipe::level(
            &player,
            &l,
            &surfaces,
            &out.join(l.id),
            studio.as_ref().zip(oodle.as_ref()),
        ) {
            Ok((map, None)) => format!("{}: ok in {:.0} s, map {}", l.id, secs(), map.display()),
            Ok((_, Some(c))) => format!(
                "{}: ok in {:.0} s, map and Studio compile ({:.0} s, {} warnings, log {}): {}",
                l.id,
                secs(),
                c.seconds,
                c.warnings,
                c.log.display(),
                c.patch.display()
            ),
            Err(e) => format!("{}: FAILED after {:.0} s: {e}", l.id, secs()),
        };
        println!("{line}");
        lines.push(line);
    }
    summary(lines)
}

/// Each level in its own process (the same options, that one level), `jobs` at a time, each one's output in
/// `<out>/<level>/build.log`; then each level's result line.
fn in_children(
    chosen: &[levels::Level],
    options: &[String],
    jobs: usize,
    out: &Path,
) -> Result<()> {
    let exe = std::env::current_exe()?;
    let started = std::time::Instant::now();
    let mut queue: std::collections::VecDeque<&levels::Level> = chosen.iter().collect();
    let mut running = Vec::new();
    let mut lines = Vec::new();
    while !queue.is_empty() || !running.is_empty() {
        while running.len() < jobs
            && let Some(l) = queue.pop_front()
        {
            let dir = out.join(l.id);
            std::fs::create_dir_all(&dir)?;
            let log = dir.join("build.log");
            let file = std::fs::File::create(&log)?;
            let child = std::process::Command::new(&exe)
                .args(options)
                .args(["--jobs", "1", l.id])
                .stdout(file.try_clone()?)
                .stderr(file)
                .spawn()?;
            println!("{}: started, log {}", l.id, log.display());
            running.push((l, log, child));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut i = 0;
        while i < running.len() {
            let Some(status) = running[i].2.try_wait()? else {
                i += 1;
                continue;
            };
            let (l, log, _) = running.swap_remove(i);
            let prefix = format!("{}: ", l.id);
            let text = std::fs::read_to_string(&log).unwrap_or_default();
            let line = text
                .lines()
                .rfind(|line| line.starts_with(&prefix))
                .map_or_else(
                    || format!("{}: FAILED, exit {status}, see {}", l.id, log.display()),
                    str::to_owned,
                );
            println!("{line}");
            lines.push(line);
        }
    }
    println!(
        "== {:.0} s, {jobs} at a time",
        started.elapsed().as_secs_f64()
    );
    lines.sort();
    summary(lines)
}

/// The result lines, and an error when a level failed.
fn summary(lines: Vec<String>) -> Result<()> {
    println!("== summary");
    for line in &lines {
        println!("{line}");
    }
    let failed = lines.iter().filter(|l| l.contains(": FAILED")).count();
    if failed > 0 {
        return Err(format!("{failed} of {} levels failed", lines.len()).into());
    }
    Ok(())
}
