//! The stage after compiling: each level as an installable ReSkate mod. `<build>/<level>/mod/SkateBIRD_<level>/`
//! holds Studio's Patch (with its `.reskate-studio-patch` marker, which the launcher requires), the map's
//! `trainer.json` (the trainer's spots: the level's travel points), a `manifest.json` and the level's name as the
//! game shows it; then `SkateBIRD_<level>-<version>.zip` of that folder in the releases folder. Nothing in it adapts
//! the map.
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::level::Level;

/// The mod folder for `level` built in `dir` (`<build>/<level>`), named `name`, at `version`; returns its path.
pub fn mod_folder(dir: &Path, level: &Level, name: &str, version: &str) -> Result<PathBuf, String> {
    let patch = dir.join("stage").join("Patch");
    if !patch.join(".reskate-studio-patch").is_file() {
        return Err(format!(
            "{}: no Studio Patch (compile with --studio first)",
            patch.display()
        ));
    }
    let out = dir.join("mod").join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    copy_tree(&patch, &out)?;
    let trainer = dir.join("map").join(name).join("trainer.json");
    std::fs::copy(&trainer, out.join("trainer.json"))
        .map_err(|e| format!("{}: {e}", trainer.display()))?;
    // the level's name in the game's level list
    let levels = out.join("reskate-levels.json");
    let mut doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&levels).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}: {e}", levels.display()))?;
    for row in doc["levels"].as_array_mut().into_iter().flatten() {
        row["displayName"] = format!("SkateBIRD: {}", level.name).into();
    }
    write(
        &levels,
        &serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?,
    )?;
    let manifest = serde_json::json!({
        "name": name,
        "author": "Kalmarv",
        "version_number": version,
        "dependencies": [],
        "description": format!("SkateBIRD's {} as a skate. level, converted from the game's own files with primase.", level.name),
    });
    write(
        &out.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )?;
    Ok(out)
}

/// `folder` zipped as `zip`, its files under the folder's name (no ZIP64 records: the launcher's reader rejects them
/// below 4 GiB).
pub fn zip(folder: &Path, zip: &Path) -> Result<u64, String> {
    let file = std::fs::File::create(zip).map_err(|e| format!("{}: {e}", zip.display()))?;
    let mut w = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .large_file(false);
    let root = folder
        .file_name()
        .ok_or("a folder without a name")?
        .to_string_lossy()
        .into_owned();
    let mut files = Vec::new();
    list(folder, &mut files)?;
    files.sort();
    for f in &files {
        let rel = f.strip_prefix(folder).map_err(|e| e.to_string())?;
        let name = format!("{root}/{}", rel.to_string_lossy().replace('\\', "/"));
        w.start_file(name, options).map_err(|e| e.to_string())?;
        w.write_all(&std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?)
            .map_err(|e| e.to_string())?;
    }
    w.finish().map_err(|e| e.to_string())?;
    std::fs::metadata(zip)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn list(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            list(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let target = to.join(path.file_name().ok_or("an entry without a name")?);
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}
