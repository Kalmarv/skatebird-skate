//! The stage after compiling: each level as an installable ReSkate mod. `<build>/<level>/mod/SkateBIRD_<level>/`
//! holds Studio's Patch (with its `.reskate-studio-patch` marker, which the launcher requires), the map's
//! `trainer.json` (the trainer's spots: the level's travel points), a `manifest.json` and the level's name as the
//! game shows it; then `SkateBIRD_<level>-<version>.zip` of that folder in the releases folder. Nothing in it adapts
//! the map.
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::level::Level;

/// The mod's picture: the launcher shows a mod folder's `icon.png`, 256 x 256 (as the GTA and Sledding releases ship
/// theirs).
pub const ICON_SIZE: u32 = 256;

/// The mod folder for `level` built in `dir` (`<build>/<level>`), named `name`, at `version`, with `icon` (a PNG
/// screenshot, made [`icon`]) as its picture where given; returns its path.
pub fn mod_folder(
    dir: &Path,
    level: &Level,
    name: &str,
    version: &str,
    icon_png: Option<&[u8]>,
) -> Result<PathBuf, String> {
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
    if let Some(png) = icon_png {
        write(&out.join("icon.png"), &icon(png)?)?;
    }
    Ok(out)
}

/// A screenshot (PNG) as the mod's picture: its centre square, averaged down to [`ICON_SIZE`] (each icon pixel the
/// mean of the source pixels it covers, weighted by coverage), opaque, as a PNG.
pub fn icon(png: &[u8]) -> Result<Vec<u8>, String> {
    let image = primase_image::read_png(png).map_err(|e| format!("icon: {e}"))?.image;
    let (w, h) = (image.width as usize, image.height as usize);
    let rgba: Vec<f64> = match &image.samples {
        primase_image::Samples::U8(s) => s.iter().map(|&v| f64::from(v)).collect(),
        primase_image::Samples::U16(s) => s.iter().map(|&v| f64::from(v) / 257.0).collect(),
    };
    let side = w.min(h);
    if side < ICON_SIZE as usize {
        return Err(format!("icon: a {w} x {h} screenshot is smaller than {ICON_SIZE} px"));
    }
    let (x0, y0) = ((w - side) / 2, (h - side) / 2);
    let n = ICON_SIZE as usize;
    let step = side as f64 / n as f64;
    // the source span [a, b) of output cell i, as (pixel, weight) pairs
    let span = |i: usize| -> Vec<(usize, f64)> {
        let (a, b) = (i as f64 * step, (i + 1) as f64 * step);
        (a.floor() as usize..(b.ceil() as usize).min(side))
            .map(|p| (p, (b.min(p as f64 + 1.0) - a.max(p as f64)).max(0.0)))
            .collect()
    };
    let spans: Vec<Vec<(usize, f64)>> = (0..n).map(span).collect();
    let mut out = Vec::with_capacity(n * n * 4);
    for sy in &spans {
        for sx in &spans {
            let mut acc = [0.0f64; 3];
            let mut total = 0.0;
            for &(py, wy) in sy {
                for &(px, wx) in sx {
                    let i = ((y0 + py) * w + x0 + px) * 4;
                    for (c, a) in acc.iter_mut().enumerate() {
                        *a += rgba[i + c] * wx * wy;
                    }
                    total += wx * wy;
                }
            }
            out.extend(acc.map(|a| (a / total).round().clamp(0.0, 255.0) as u8));
            out.push(255);
        }
    }
    primase_image::write_png(&primase_image::Image {
        width: ICON_SIZE,
        height: ICON_SIZE,
        samples: primase_image::Samples::U8(out),
    })
    .map_err(|e| format!("icon: {e}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A 600 x 400 screenshot: its centre 400 x 400 square (red left half, blue right) at 256 x 256, opaque; the
    /// 100 px green margins cropped away.
    #[test]
    fn the_icon_is_the_centre_square_at_256() {
        let (w, h) = (600u32, 400u32);
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for _y in 0..h {
            for x in 0..w {
                px.extend(match x {
                    0..100 | 500.. => [0, 255, 0, 128],
                    100..300 => [255, 0, 0, 128],
                    _ => [0, 0, 255, 128],
                });
            }
        }
        let png = primase_image::write_png(&primase_image::Image {
            width: w,
            height: h,
            samples: primase_image::Samples::U8(px),
        })
        .unwrap();
        let out = primase_image::read_png(&icon(&png).unwrap()).unwrap().image;
        assert_eq!((out.width, out.height), (ICON_SIZE, ICON_SIZE));
        let primase_image::Samples::U8(s) = out.samples else { panic!("8-bit") };
        let at = |x: usize, y: usize| &s[(y * 256 + x) * 4..][..4];
        assert_eq!(at(0, 0), [255, 0, 0, 255], "left edge: red, no green margin, opaque");
        assert_eq!(at(255, 255), [0, 0, 255, 255], "right edge: blue");
        assert_eq!(at(127, 10), [255, 0, 0, 255]);
        assert_eq!(at(128, 10), [0, 0, 255, 255]);
    }

    #[test]
    fn a_screenshot_smaller_than_the_icon_is_refused() {
        let png = primase_image::write_png(&primase_image::Image {
            width: 300,
            height: 200,
            samples: primase_image::Samples::U8(vec![0; 300 * 200 * 4]),
        })
        .unwrap();
        assert!(icon(&png).is_err());
    }
}
