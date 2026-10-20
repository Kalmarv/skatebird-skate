//! The fixed views each level is compared at, the same in SkateBIRD and in skate., and the side-by-side sheets. The
//! views are data (`data/skatebird-views.json`: per level the spawn and 3 of its travel anchors spread over the list,
//! made once from the game's scan on 2026-10-19; every non-spawn view looks along Unity yaw 180).
//!
//! A view is a place on the ground in Unity coordinates (metres) and the Unity yaw the camera looks along. skate's own
//! coordinates are the scene's: Unity (x, y, z) is skate (x, y, -z) (`trainer where` at level2's spawn, Unity
//! (-18.7, 1.189, 84.8), reads (-18.70, 1.95, -84.80)). skate gives no console way to turn the skater: the spawn view
//! is shot where the level starts (facing the spawn's yaw), and `trainer tp` leaves the skater at heading 0 (skate +z,
//! Unity -z: Unity yaw 180), so every other view looks along Unity yaw 180, with skate's third-person chase camera.
//! SkateBIRD's camera is put where that camera is (tools/skatebird_views.sh --chase).
use std::path::Path;

use serde_json::Value;

/// Metres above the anchor a teleport lands (skate drops the skater onto the ground).
const LIFT: f64 = 0.3;
/// Seconds a view is held after its shot is asked for: under load the runner's capture starts 15-20 s late.
const HOLD: u32 = 25;

/// RESKATE_STARTUP_COMMANDS for one level's run (a variant `level3_T` has `level3`'s views): load it, settle, a shot
/// at the spawn, then a teleport and a shot per other view (each `trainer where` is a shot).
pub fn skate_commands(views: &Value, level: &str) -> Result<String, String> {
    let base = level.split('_').next().unwrap_or(level);
    let list = views[base]["views"]
        .as_array()
        .ok_or(format!("no views for {base}"))?;
    let asset = {
        let l = format!(
            "dingolevel_reskate_{}",
            crate::settings::map_name(level).to_lowercase()
        );
        format!("levels/game/{l}/{l}")
    };
    let mut c = vec![
        "wait 100".to_owned(),
        format!("load {asset}"),
        "wait 45".into(),
        "trainer option hud 0".into(),
        "hideui 1".into(),
        "trainer where".into(),
        format!("wait {HOLD}"),
    ];
    for v in list.iter().skip(1) {
        let p: Vec<f64> = v["position"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_f64)
            .collect();
        let [x, y, z] = p[..] else {
            return Err(format!("{base}: a view without a position"));
        };
        c.push(format!("trainer tp {x} {:.3} {}", y + LIFT, -z));
        c.extend([
            "wait 12".into(),
            "trainer where".into(),
            format!("wait {HOLD}"),
        ]);
    }
    Ok(c.join(";"))
}

/// Side-by-side sheets, one per level of the run (`<run>/levels.txt`, in shot order, four shots each): SkateBIRD's
/// reference views (`<refs>/<level>/<i>_<name>.png`) on the left, the run's `where_<n>.png` on the right, a row per
/// view, 960 x 540 each; a missing shot stays black. Writes `<out>/<level>.png`.
pub fn sheets(refs: &Path, run: &Path, out: &Path) -> Result<Vec<String>, String> {
    let order = std::fs::read_to_string(run.join("levels.txt"))
        .map_err(|e| format!("{}: {e}", run.display()))?;
    let shots = run.join("run");
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let (w, h) = (960usize, 540usize);
    let mut written = Vec::new();
    for (li, level) in order.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let base = level.split('_').next().unwrap_or(level);
        let mut ref_shots: Vec<_> = std::fs::read_dir(refs.join(base))
            .map_err(|e| format!("{}: {e}", refs.join(base).display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension().is_some_and(|x| x == "png")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.as_bytes()[0].is_ascii_digit())
            })
            .collect();
        ref_shots.sort();
        let mut sheet = vec![0u8; 2 * w * h * ref_shots.len() * 3];
        for (vi, r) in ref_shots.iter().enumerate() {
            let skate = shots.join(format!("where_{}.png", li * 4 + vi + 1));
            for (col, path) in [(0, r.as_path()), (1, skate.as_path())] {
                if let Some(img) = load(path)? {
                    paste(&mut sheet, 2 * w, &img, col * w, vi * h, w, h);
                }
            }
        }
        let png =
            primase_image::write_png_rgb((2 * w) as u32, (h * ref_shots.len()) as u32, &sheet)
                .map_err(|e| e.to_string())?;
        let path = out.join(format!("{level}.png"));
        std::fs::write(&path, png).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path.display().to_string());
    }
    Ok(written)
}

/// An RGBA8 picture, or none when the file is not there.
fn load(path: &Path) -> Result<Option<primase_image::Image>, String> {
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(None);
    };
    let png = primase_image::read_png(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Some(png.image))
}

/// `img` scaled nearest to `w` x `h` into the RGB `sheet` (`stride` pixels wide) at (`x0`, `y0`).
fn paste(
    sheet: &mut [u8],
    stride: usize,
    img: &primase_image::Image,
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
) {
    let (iw, ih) = (img.width as usize, img.height as usize);
    let primase_image::Samples::U8(px) = &img.samples else {
        return;
    };
    for y in 0..h {
        for x in 0..w {
            let s = ((y * ih / h) * iw + x * iw / w) * 4;
            let d = ((y0 + y) * stride + x0 + x) * 3;
            sheet[d..d + 3].copy_from_slice(&px[s..s + 3]);
        }
    }
}
