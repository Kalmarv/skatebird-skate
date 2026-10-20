//! SkateBIRD's reference shots at exact camera poses, one per view of `data/skatebird-views.json`, taken in the lab
//! copy (`tools/skatebird_lab.sh`) through its plugin (`tools/skatebird-teleport-plugin`, protocol at the top of
//! `Plugin.cs`).
//!
//! One launch at 1024 x 576 windowed shoots every level asked for: the plugin loads the level's scene, then for each
//! view locks Camera.main at (view position - D m along the yaw + H m up), yaw = the view's yaw + Y (Unity: degrees
//! from +Z towards +X), pitch P (Unity Euler x, positive looks down) and vertical FOV F ([`Camera`]), with the HUD,
//! menus, the player's bird and (unless kept) the NPCs hidden and the lens effects off, and saves the rendered frame
//! as `<out>/<view index>_<name>.png`. Each level's `poses.json` holds per shot the requested pose, the plugin's
//! read-back of the frame that was saved and one after it, and the [`checks`]; `checks/player_{shown,hidden}.png` are
//! the control for the bird hiding. With several levels each goes into `<out>/<level>` and the run's BepInEx log into
//! `<out>/plugin.log`.
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

/// The shot size: the client area the game is launched with and the plugin's frames are read back at.
const WIDTH: u64 = 1024;
const HEIGHT: u64 = 576;
/// How far a read-back may be from the request: metres and degrees.
const POS_TOL: f64 = 1e-3;
const ANGLE_TOL: f64 = 0.05;
/// What the plugin turns off unless `keep_post`: they make a frame differ from a pinhole render of the scene.
const POST_OFF: [&str; 5] = [
    "DepthOfField",
    "LensDistortion",
    "ChromaticAberration",
    "Vignette",
    "MotionBlur",
];
/// The bird-hiding control fails when hiding the bird changes fewer pixels than this (by more than [`DIFF_LEVEL`]).
const MIN_CHANGED: usize = 500;
const DIFF_LEVEL: u32 = 24;
/// The lab copy when `SKATEBIRD_LAB` is not set.
const LAB: &str = "C:/Users/kalmarv/modding/cache/skatebird/lab";

/// Where the camera goes relative to a view: `back` metres behind the view point along its yaw, `eye` metres above
/// it, Unity `pitch` (positive looks down), vertical `fov`, `yaw_offset` added to the view's yaw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub back: f64,
    pub eye: f64,
    pub pitch: f64,
    pub fov: f64,
    pub yaw_offset: f64,
}

impl Camera {
    /// At the eye of a standing skater over the view point.
    pub const EYE: Camera = Camera {
        back: 0.0,
        eye: 1.6,
        pitch: 0.0,
        fov: 60.0,
        yaw_offset: 0.0,
    };
    /// skate.'s third-person camera as fitted at level2's spawn (job 0da1c4ce, `cache/skatebird/chase-fit/level2/
    /// fit.json`): two skate launches agree on the position (D -0.25/-0.35, H 1.5; the score is flat over +-0.15 m)
    /// and FOV (55.4/55.2) but not on the orientation (P +1.2, yaw -0.3 in t1; P -6.2, yaw +4.2 in t2); this takes
    /// t1, whose camera looks along the skater's heading.
    pub const CHASE: Camera = Camera {
        back: -0.3,
        eye: 1.5,
        pitch: 1.2,
        fov: 55.3,
        yaw_offset: 0.0,
    };
}

/// How a run shoots.
#[derive(Clone, Debug)]
pub struct Options {
    /// "eye" or "chase": the preset [`Options::camera`] started from, recorded in `poses.json`.
    pub mode: &'static str,
    pub camera: Camera,
    /// Seconds each camera pose is held before its shot.
    pub settle: f64,
    /// Seconds after a level's scene is active.
    pub load_settle: f64,
    /// Keep the game's depth of field, lens distortion, chromatic aberration, vignette and motion blur.
    pub keep_post: bool,
    /// Keep the NPC birds and other skinned meshes (hidden by default: skate. gets no skinned meshes).
    pub keep_npcs: bool,
    /// Leave the game running at the end.
    pub keep_game: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            mode: "eye",
            camera: Camera::EYE,
            settle: 2.0,
            load_settle: 8.0,
            keep_post: false,
            keep_npcs: false,
            keep_game: false,
        }
    }
}

/// The lab copy's folder: `SKATEBIRD_LAB`, or [`LAB`].
pub fn lab() -> PathBuf {
    std::env::var_os("SKATEBIRD_LAB").map_or_else(|| PathBuf::from(LAB), PathBuf::from)
}

/// The pose asked of the camera for one shot.
#[derive(Clone, Debug)]
pub struct Want {
    pub position: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub fov: f64,
}

impl Want {
    /// `camera` at a view point `p` looking along Unity `yaw`.
    pub fn at(p: [f64; 3], yaw: f64, camera: &Camera) -> Want {
        let a = yaw.to_radians();
        Want {
            position: [
                p[0] - camera.back * a.sin(),
                p[1] + camera.eye,
                p[2] - camera.back * a.cos(),
            ],
            yaw: yaw + camera.yaw_offset,
            pitch: camera.pitch,
            fov: camera.fov,
        }
    }

    fn to_json(&self) -> Value {
        json!({"position": self.position, "yaw": self.yaw, "pitch": self.pitch, "fov": self.fov})
    }
}

/// The game, launched from the lab and driven through its plugin's control folder (`<lab>/control`): every command
/// is `command.json` with the next `seq`, every answer `reply.json` with the same one.
pub struct Game {
    child: Child,
    seq: u64,
    lab: PathBuf,
    control: PathBuf,
}

impl Game {
    /// Launches `<lab>/SkateBIRD.exe` windowed at [`WIDTH`] x [`HEIGHT`] and waits (120 s at most) for the plugin's
    /// `ready.json` naming this process: the plugin clears the control folder when it starts, so nothing may be sent
    /// before.
    pub fn start(lab: &Path) -> Result<Game, String> {
        if running() {
            return Err("SkateBIRD is already running; close it first".into());
        }
        let mut command = Command::new(lab.join("SkateBIRD.exe"));
        command
            .args([
                "-screen-fullscreen",
                "0",
                "-screen-width",
                &WIDTH.to_string(),
                "-screen-height",
                &HEIGHT.to_string(),
            ])
            .current_dir(lab);
        Game::launch(lab, command)
    }

    /// Runs `command` as the game whose plugin answers in `<lab>/control` (the tests' stub is another program).
    fn launch(lab: &Path, mut command: Command) -> Result<Game, String> {
        let control = lab.join("control");
        for f in ["command.json", "reply.json", "ready.json"] {
            let _ = std::fs::remove_file(control.join(f));
        }
        let program = command.get_program().to_string_lossy().into_owned();
        let shown: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        println!("launch {program} {}", shown.join(" "));
        let child = command.spawn().map_err(|e| format!("{program}: {e}"))?;
        let mut game = Game {
            child,
            seq: 0,
            lab: lab.to_owned(),
            control,
        };
        let end = Instant::now() + Duration::from_secs(120);
        while Instant::now() < end {
            game.alive("before the plugin was ready")?;
            let ready = std::fs::read(game.control.join("ready.json")).ok();
            let pid = ready
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .and_then(|v| v["pid"].as_u64());
            if pid == Some(u64::from(game.child.id())) {
                return Ok(game);
            }
            sleep(Duration::from_millis(100));
        }
        Err(format!(
            "the plugin did not write {} in 120 s (is BepInEx loading? see {})",
            game.control.join("ready.json").display(),
            lab.join("BepInEx/LogOutput.log").display()
        ))
    }

    fn alive(&mut self, when: &str) -> Result<(), String> {
        match self.child.try_wait() {
            Ok(Some(status)) => Err(format!("game exited ({status}) {when}")),
            _ => Ok(()),
        }
    }

    /// Runs one op with its `fields` and returns the plugin's reply; an error when the reply says it failed, the game
    /// exits, or no reply comes within `timeout`.
    pub fn send(&mut self, op: &str, fields: Value, timeout: Duration) -> Result<Value, String> {
        self.seq += 1;
        let mut cmd = Map::new();
        cmd.insert("seq".into(), json!(self.seq));
        cmd.insert("op".into(), json!(op));
        if let Value::Object(f) = fields {
            cmd.extend(f);
        }
        std::fs::create_dir_all(&self.control).map_err(|e| e.to_string())?;
        let tmp = self.control.join("command.tmp");
        std::fs::write(&tmp, Value::Object(cmd).to_string()).map_err(|e| e.to_string())?;
        // The plugin may be reading the old command (no delete sharing on Windows): retry the replace for 2 s.
        let target = self.control.join("command.json");
        let mut tries = 0;
        while let Err(e) = std::fs::rename(&tmp, &target) {
            tries += 1;
            if tries == 200 {
                return Err(format!("{}: {e}", target.display()));
            }
            sleep(Duration::from_millis(10));
        }
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            self.alive(&format!("during {op}"))?;
            let reply = std::fs::read(self.control.join("reply.json"))
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            if let Some(r) = reply.filter(|r| r["seq"].as_u64() == Some(self.seq)) {
                if r["ok"].as_bool() != Some(true) {
                    return Err(format!(
                        "{op} failed: {}",
                        r["error"].as_str().unwrap_or("?")
                    ));
                }
                return Ok(r);
            }
            sleep(Duration::from_millis(50));
        }
        Err(format!(
            "no reply to {op} (seq {}) in {:.0} s",
            self.seq,
            timeout.as_secs_f64()
        ))
    }

    /// Quits through the plugin (20 s), then kills the process if it is still there.
    pub fn stop(mut self) {
        if self.alive("").is_err() {
            return;
        }
        if self
            .send("quit", json!({}), Duration::from_secs(10))
            .is_ok()
        {
            let end = Instant::now() + Duration::from_secs(20);
            while Instant::now() < end && self.alive("").is_ok() {
                sleep(Duration::from_millis(200));
            }
        }
        if self.alive("").is_ok() {
            let pid = self.child.id().to_string();
            let _ = Command::new("taskkill").args(["/PID", &pid, "/F"]).output();
            let end = Instant::now() + Duration::from_secs(30);
            while Instant::now() < end && self.alive("").is_ok() {
                sleep(Duration::from_millis(200));
            }
            if self.alive("").is_ok() {
                println!("warning: SkateBIRD pid {pid} still running after taskkill");
            }
        }
    }
}

/// Whether a SkateBIRD.exe is running (any, not only ours).
fn running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq SkateBIRD.exe"])
        .output()
        .is_ok_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains("skatebird.exe")
        })
}

fn secs(s: f64) -> Duration {
    Duration::from_secs_f64(s)
}

fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

fn floats(v: &Value) -> Vec<f64> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_f64)
        .collect()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| s.as_str().map(str::to_owned))
        .collect()
}

/// The failed checks of one read-back `got` against `want`: the pose (position 1 mm, angles and FOV 0.05 deg, roll 0),
/// the screen size, the lock, no game menu, the player hidden (when `player_hidden`), no screen-space canvas, the lens
/// effects off (unless `keep_post`), no collider within 0.2 m of the camera and one below it.
pub fn checks(want: &Want, got: &Value, player_hidden: bool, keep_post: bool) -> Vec<String> {
    let mut bad = Vec::new();
    let pos = floats(&got["cam_position"]);
    if pos.len() != 3 {
        return vec!["no camera".into()];
    }
    let d = pos
        .iter()
        .zip(want.position)
        .map(|(g, w)| (g - w).abs())
        .fold(0.0, f64::max);
    if d > POS_TOL {
        bad.push(format!("position off by {d:.4} m"));
    }
    let euler = floats(&got["cam_euler"]);
    if let [pitch, yaw, roll] = euler[..] {
        for (name, g, w) in [
            ("yaw", yaw, want.yaw),
            ("pitch", pitch, want.pitch),
            ("roll", roll, 0.0),
        ] {
            if angle_diff(g, w).abs() > ANGLE_TOL {
                bad.push(format!("{name} {g:.3} != {w:.3}"));
            }
        }
    } else {
        bad.push("no camera angles".into());
    }
    let (fov, proj) = (
        got["cam_fov"].as_f64().unwrap_or(f64::NAN),
        got["cam_fov_projection"].as_f64().unwrap_or(f64::NAN),
    );
    if !((fov - want.fov).abs() <= ANGLE_TOL && (proj - want.fov).abs() <= ANGLE_TOL) {
        bad.push(format!(
            "fov {fov:.3} (projection {proj:.3}) != {}",
            want.fov
        ));
    }
    let (w, h) = (got["screen_width"].as_u64(), got["screen_height"].as_u64());
    if (w, h) != (Some(WIDTH), Some(HEIGHT)) {
        bad.push(format!("screen {}x{}", w.unwrap_or(0), h.unwrap_or(0)));
    }
    if got["locked"].as_bool() != Some(true) {
        bad.push("camera not locked".into());
    }
    if got["gameplay_obstructed"].as_bool() == Some(true) {
        bad.push("a game menu is open (UIManager.IsGameplayObstructed)".into());
    }
    let roots = strings(&got["player_roots"]);
    let hidden = got["hidden_renderers"].as_u64().unwrap_or(0);
    if player_hidden
        && (roots.is_empty() || roots.iter().any(|r| r.contains("skipped")) || hidden == 0)
    {
        bad.push(format!(
            "player not hidden (roots {roots:?}, {hidden} renderers hidden)"
        ));
    }
    let screen: Vec<_> = strings(&got["kept_canvases"])
        .into_iter()
        .filter(|c| !c.ends_with("WorldSpace"))
        .collect();
    if !screen.is_empty() {
        bad.push(format!("screen canvases still on: {screen:?}"));
    }
    let mut post_on: Vec<_> = strings(&got["post_effects_on"])
        .into_iter()
        .filter(|e| POST_OFF.contains(&e.as_str()))
        .collect();
    post_on.sort();
    if !post_on.is_empty() && !keep_post {
        bad.push(format!("post effects still on: {post_on:?}"));
    }
    let overlaps = strings(&got["cam_overlaps"]);
    if !overlaps.is_empty() {
        bad.push(format!(
            "camera within 0.2 m of colliders {overlaps:?} (in or against geometry)"
        ));
    }
    if got["cam_ground"].as_f64().unwrap_or(0.0) < 0.0 {
        bad.push("no collider below the camera".into());
    }
    bad
}

/// The parts of a reply `poses.json` keeps.
pub fn pose(r: &Value) -> Value {
    const KEYS: [&str; 28] = [
        "frame",
        "time",
        "cam_name",
        "cam_position",
        "cam_euler",
        "cam_forward",
        "cam_bearing",
        "cam_fov",
        "cam_fov_projection",
        "cam_near",
        "cam_far",
        "screen_width",
        "screen_height",
        "player_position",
        "player_yaw",
        "gameplay_obstructed",
        "hidden_canvases",
        "hidden_renderers",
        "player_roots",
        "kept_canvases",
        "other_cameras",
        "post_effects_off",
        "post_effects_on",
        "cam_overlaps",
        "cam_ground",
        "npc_birbs",
        "npc_skinned_other",
        "hidden_npc_renderers",
    ];
    Value::Object(
        KEYS.iter()
            .map(|k| ((*k).to_owned(), r.get(*k).cloned().unwrap_or(Value::Null)))
            .collect(),
    )
}

/// A view's file name: runs of anything but ASCII letters and digits become one `_`, none at the ends.
pub fn file_stem(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_owned()
}

/// Pixels whose difference between two same-size PNGs is above [`DIFF_LEVEL`] (luminance of the channel differences,
/// 0.299/0.587/0.114).
fn changed_pixels(a: &Path, b: &Path) -> Result<usize, String> {
    let read = |p: &Path| -> Result<primase_image::Image, String> {
        let bytes = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(primase_image::read_png(&bytes)
            .map_err(|e| format!("{}: {e}", p.display()))?
            .image)
    };
    let (a, b) = (read(a)?, read(b)?);
    let (primase_image::Samples::U8(pa), primase_image::Samples::U8(pb)) = (&a.samples, &b.samples)
    else {
        return Err("not 8-bit pictures".into());
    };
    if (a.width, a.height) != (b.width, b.height) {
        return Err("pictures of different sizes".into());
    }
    let channels = pa.len() / (a.width as usize * a.height as usize);
    Ok(pa
        .chunks(channels)
        .zip(pb.chunks(channels))
        .filter(|(x, y)| {
            let d = |i: usize| u32::from(x[i].abs_diff(y[i]));
            (299 * d(0) + 587 * d(1) + 114 * d(2)) / 1000 > DIFF_LEVEL
        })
        .count())
}

/// The control for the bird hiding: one pose 3 m behind and 1.5 m above the player, looking along `yaw` and down at
/// it, shot with the bird shown and hidden. The views put the camera over the player's spawn, where the bird is
/// usually out of frame anyway, so they alone do not show that hiding works.
fn player_check(game: &mut Game, out: &Path, yaw: f64, o: &Options) -> Result<Value, String> {
    let p = floats(&game.send("readback", json!({}), secs(30.0))?["player_position"]);
    let [x, y, z] = p[..] else {
        return Err("no player position".into());
    };
    let a = yaw.to_radians();
    let want = Want {
        position: [x - 3.0 * a.sin(), y + 1.5, z - 3.0 * a.cos()],
        yaw,
        pitch: 1.1f64.atan2(3.0).to_degrees(),
        fov: o.camera.fov,
    };
    let mut result = Map::new();
    result.insert("player_position".into(), json!(p));
    result.insert("requested".into(), want.to_json());
    let mut failed = Vec::new();
    for shown in [true, false] {
        let name = if shown {
            "player_shown"
        } else {
            "player_hidden"
        };
        let path = absolute(&out.join(format!("{name}.png")))?;
        game.send(
            "camera",
            json!({"position": want.position, "yaw": want.yaw, "pitch": want.pitch, "fov": want.fov,
                   "settle_frames": 30, "settle_seconds": 1.0, "show_player": shown, "keep_post": o.keep_post}),
            secs(30.0),
        )?;
        let shot = game.send("shot", json!({"path": path}), secs(30.0))?;
        result.insert(name.into(), pose(&shot));
        failed.extend(
            checks(&want, &shot, !shown, o.keep_post)
                .into_iter()
                .map(|b| format!("{name}: {b}")),
        );
    }
    let changed = changed_pixels(
        &out.join("player_shown.png"),
        &out.join("player_hidden.png"),
    )?;
    if changed < MIN_CHANGED {
        failed.push(format!(
            "hiding the player changed only {changed} pixels (is the bird in the control frame?)"
        ));
    }
    println!(
        "player check: {changed} pixels change when the bird is hidden {}",
        if failed.is_empty() {
            "ok".to_owned()
        } else {
            format!("FAILED {}", failed.join("; "))
        }
    );
    result.insert("changed_pixels".into(), json!(changed));
    result.insert("failed".into(), json!(failed));
    Ok(Value::Object(result))
}

fn absolute(p: &Path) -> Result<PathBuf, String> {
    std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()))
}

/// Loads one level and shoots its `views` (`data/skatebird-views.json`'s entry) into `out`; writes its `poses.json`
/// and returns how many shots and checks failed (a failed op counts one).
fn shoot_level(
    game: &mut Game,
    key: &str,
    level: &Value,
    out: &Path,
    o: &Options,
) -> Result<usize, String> {
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let c = &o.camera;
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut record = json!({
        "level": key, "scene": level["name"], "mode": o.mode, "back": c.back, "eye": c.eye, "fov": c.fov,
        "pitch": c.pitch, "yaw_offset": c.yaw_offset, "post_effects_kept": o.keep_post, "npcs_kept": o.keep_npcs,
        "settle_s": o.settle, "screen": [WIDTH, HEIGHT], "lab": game.lab.display().to_string(),
        "started_unix": started, "shots": [],
    });
    let mut failed = 0;
    let outcome = (|| -> Result<(), String> {
        let scene = level["name"]
            .as_str()
            .ok_or(format!("{key}: no scene name"))?;
        let r = game.send(
            "load",
            json!({"scene": scene, "settle_frames": 30, "settle_seconds": o.load_settle}),
            secs(150.0),
        )?;
        println!(
            "{key}: loaded {}; player at {}",
            r["scene"].as_str().unwrap_or("?"),
            r["player_position"]
        );
        let views = level["views"]
            .as_array()
            .ok_or(format!("{key}: no views"))?;
        for (i, view) in views.iter().enumerate() {
            let p = floats(&view["position"]);
            let [x, y, z] = p[..] else {
                return Err(format!("{key}: view {i} has no position"));
            };
            let yaw = view["yaw_deg"]
                .as_f64()
                .ok_or(format!("{key}: view {i} has no yaw"))?;
            let want = Want::at([x, y, z], yaw, c);
            let name = view["name"].as_str().unwrap_or("view");
            let file = format!("{i}_{}.png", file_stem(name));
            let path = absolute(&out.join(&file))?;
            game.send(
                "camera",
                json!({"position": want.position, "yaw": want.yaw, "pitch": want.pitch, "fov": want.fov,
                       "settle_frames": 30, "settle_seconds": o.settle, "keep_post": o.keep_post,
                       "hide_npcs": !o.keep_npcs}),
                secs(30.0),
            )?;
            let shot = game.send("shot", json!({"path": path}), secs(30.0))?;
            let after = game.send("readback", json!({}), secs(30.0))?;
            let mut bad = checks(&want, &shot, true, o.keep_post);
            bad.extend(
                checks(&want, &after, true, o.keep_post)
                    .into_iter()
                    .map(|b| format!("after the shot: {b}")),
            );
            failed += usize::from(!bad.is_empty());
            let cam: Vec<String> = floats(&shot["cam_position"])
                .iter()
                .map(|v| format!("{v:.3}"))
                .collect();
            println!(
                "{key} {file}: cam [{}] ground {:.2} m, NPC renderers hidden {} {}",
                cam.join(", "),
                shot["cam_ground"].as_f64().unwrap_or(-1.0),
                shot["hidden_npc_renderers"],
                if bad.is_empty() {
                    "ok".to_owned()
                } else {
                    format!("FAILED {}", bad.join("; "))
                }
            );
            record["shots"].as_array_mut().expect("shots").push(json!({
                "index": i, "name": name, "file": file,
                "view": {"position": view["position"], "yaw_deg": view["yaw_deg"]},
                "requested": want.to_json(), "shot": pose(&shot), "after": pose(&after), "failed": bad,
            }));
        }
        if let Some(last) = record["shots"]
            .as_array()
            .and_then(|s| s.last())
            .map(|s| s["shot"].clone())
        {
            record["npcs_hidden"] = json!({"birbs": last["npc_birbs"], "other_skinned": last["npc_skinned_other"],
                                          "renderers": last["hidden_npc_renderers"]});
        }
        let yaw = views
            .first()
            .and_then(|v| v["yaw_deg"].as_f64())
            .unwrap_or(0.0);
        let check = player_check(game, &out.join("checks"), yaw, o)?;
        failed += usize::from(check["failed"].as_array().is_some_and(|f| !f.is_empty()));
        record["player_check"] = check;
        game.send("release", json!({}), secs(30.0))?;
        Ok(())
    })();
    if let Err(e) = outcome {
        println!("{key}: ERROR {e}");
        record["error"] = json!(e);
        failed += 1;
    }
    let path = out.join("poses.json");
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())? + "\n";
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    let shots = record["shots"].as_array().map_or(0, Vec::len);
    println!(
        "{key}: {shots} shots, {failed} failed -> {}",
        path.display()
    );
    Ok(failed)
}

/// Shoots `levels` (keys of `views`, `data/skatebird-views.json`) in one launch of the lab copy into `out` (one level)
/// or `out/<level>` (several). Returns each level's failed count, in order; a level the game did not get to counts
/// one.
pub fn run(
    views: &Value,
    levels: &[String],
    out: &Path,
    o: &Options,
) -> Result<Vec<(String, usize)>, String> {
    let lab = lab();
    if !lab.join("BepInEx/plugins/SkateBIRDTeleport.dll").exists() {
        return Err(format!(
            "no lab plugin under {}; run tools/skatebird_lab.sh",
            lab.display()
        ));
    }
    shoot(views, levels, out, o, &lab, Game::start)
}

/// [`run`] with the game started by `start` from `lab`.
fn shoot(
    views: &Value,
    levels: &[String],
    out: &Path,
    o: &Options,
    lab: &Path,
    start: impl FnOnce(&Path) -> Result<Game, String>,
) -> Result<Vec<(String, usize)>, String> {
    let unknown: Vec<_> = levels
        .iter()
        .filter(|l| views.get(l.as_str()).is_none())
        .collect();
    if levels.is_empty() || !unknown.is_empty() {
        return Err(format!("not in skatebird-views.json: {unknown:?}"));
    }
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut game = start(lab)?;
    let mut failed = Vec::new();
    let result = (|| -> Result<(), String> {
        game.send(
            "screen",
            json!({"width": WIDTH, "height": HEIGHT}),
            secs(120.0),
        )?;
        for key in levels {
            if game.alive("").is_err() {
                failed.push((key.clone(), 1));
                continue;
            }
            let dir = if levels.len() > 1 {
                out.join(key)
            } else {
                out.to_owned()
            };
            failed.push((
                key.clone(),
                shoot_level(&mut game, key, &views[key.as_str()], &dir, o)?,
            ));
        }
        Ok(())
    })();
    if !o.keep_game {
        game.stop();
    }
    let log = lab.join("BepInEx/LogOutput.log");
    if log.exists() {
        let _ = std::fs::copy(&log, out.join("plugin.log"));
    }
    result?;
    Ok(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The plugin's side of the protocol for [`a_stub_game_runs_the_whole_protocol`], run as its own process (this test
    /// binary again, with `STUB_LAB` set): answers every op with a read-back of exactly what was asked, writes a small
    /// PNG for `shot` (white with the bird shown, black otherwise) and stops after `quit`.
    #[test]
    fn stub_plugin() {
        let Some(lab) = std::env::var_os("STUB_LAB").map(PathBuf::from) else {
            return;
        };
        let control = lab.join("control");
        std::fs::create_dir_all(&control).unwrap();
        std::fs::write(
            control.join("ready.json"),
            json!({"pid": std::process::id()}).to_string(),
        )
        .unwrap();
        let (mut last, mut cam, mut shown) = (0, json!({}), false);
        loop {
            let cmd = std::fs::read(control.join("command.json"))
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            let Some(cmd) = cmd.filter(|c| c["seq"].as_u64() != Some(last)) else {
                sleep(Duration::from_millis(10));
                continue;
            };
            last = cmd["seq"].as_u64().unwrap();
            let op = cmd["op"].as_str().unwrap_or("").to_owned();
            if op == "camera" {
                cam = cmd.clone();
                shown = cmd["show_player"].as_bool() == Some(true);
            }
            if op == "shot" {
                let v = if shown { 255 } else { 0 };
                let png = primase_image::write_png_rgb(64, 36, &vec![v; 64 * 36 * 3]).unwrap();
                // the plugin makes the shot's folder (Plugin.cs DoShot)
                let path = PathBuf::from(cmd["path"].as_str().unwrap());
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, png).unwrap();
            }
            let reply = json!({
                "seq": last, "op": op, "ok": true, "error": "", "scene": "rooftop",
                "locked": cam.get("position").is_some(), "cam_position": cam["position"],
                "cam_euler": [cam["pitch"], cam["yaw"], 0.0], "cam_fov": cam["fov"], "cam_fov_projection": cam["fov"],
                "screen_width": WIDTH, "screen_height": HEIGHT, "player_position": [-18.6, 1.16, 84.1],
                "gameplay_obstructed": false,
                "player_roots": if shown { json!([]) } else { json!(["Skateboard For Skating"]) },
                "hidden_renderers": if shown { 0 } else { 29 }, "kept_canvases": [], "post_effects_on": ["Bloom"],
                "cam_overlaps": [], "cam_ground": 1.5, "npc_birbs": ["Generic Scenery Birb"],
                "npc_skinned_other": [], "hidden_npc_renderers": 15,
            });
            std::fs::write(control.join("reply.tmp"), reply.to_string()).unwrap();
            std::fs::rename(control.join("reply.tmp"), control.join("reply.json")).unwrap();
            if op == "quit" {
                return;
            }
        }
    }

    /// The runner against [`stub_plugin`] for two levels: the ready handshake, every op, the checks, the bird control,
    /// a folder and a `poses.json` per level.
    #[test]
    fn a_stub_game_runs_the_whole_protocol() {
        if std::env::var_os("STUB_LAB").is_some() {
            return;
        }
        let root =
            std::env::temp_dir().join(format!("skatebird-shots-test-{}", std::process::id()));
        let (lab, out) = (root.join("lab"), root.join("out"));
        std::fs::create_dir_all(&lab).unwrap();
        let views = json!({
            "level2": {"name": "rooftop", "views": [
                {"name": "spawn", "position": [-18.7, 1.189, 84.8], "yaw_deg": 172.9341},
                {"name": "Middle Roof (2)", "position": [-0.05, 1.189, 63.72], "yaw_deg": 180.0}]},
            "level7": {"name": "flashback", "views": [
                {"name": "spawn", "position": [0.0, 1.189, 0.0], "yaw_deg": 172.9341}]},
        });
        let o = Options {
            mode: "chase",
            camera: Camera::CHASE,
            settle: 0.0,
            load_settle: 0.0,
            ..Options::default()
        };
        let levels = ["level2".to_owned(), "level7".to_owned()];
        let failed = shoot(&views, &levels, &out, &o, &lab, |lab| {
            let mut c = Command::new(std::env::current_exe().unwrap());
            c.args(["--exact", "shots::tests::stub_plugin", "--nocapture"])
                .env("STUB_LAB", lab);
            Game::launch(lab, c)
        })
        .unwrap();
        assert_eq!(
            failed,
            vec![("level2".to_owned(), 0), ("level7".to_owned(), 0)]
        );
        for (level, n) in [("level2", 2), ("level7", 1)] {
            let poses: Value =
                serde_json::from_slice(&std::fs::read(out.join(level).join("poses.json")).unwrap())
                    .unwrap();
            assert_eq!(poses["shots"].as_array().unwrap().len(), n);
            assert_eq!(poses["player_check"]["changed_pixels"], json!(64 * 36));
            assert_eq!(poses["npcs_hidden"]["renderers"], json!(15));
        }
        assert!(out.join("level2/1_Middle_Roof_2.png").exists());
        assert!(out.join("level7/checks/player_hidden.png").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn file_names_keep_letters_and_digits() {
        assert_eq!(file_stem("Middle Roof (2)"), "Middle_Roof_2");
        assert_eq!(
            file_stem("Top of cubicle start (1)"),
            "Top_of_cubicle_start_1"
        );
        assert_eq!(file_stem("spawn"), "spawn");
    }

    #[test]
    fn the_chase_camera_sits_ahead_of_the_view_point() {
        // level2's spawn as the Python runner placed it (job 270b5027: cam -18.663, 2.689, 84.502)
        let w = Want::at([-18.7, 1.189, 84.8], 172.9341, &Camera::CHASE);
        assert!((w.position[0] + 18.663).abs() < 1e-3 && (w.position[1] - 2.689).abs() < 1e-9);
        assert!((w.position[2] - 84.502).abs() < 1e-3);
        assert_eq!((w.pitch, w.fov), (1.2, 55.3));
    }

    #[test]
    fn checks_pass_a_matching_read_back_and_name_what_differs() {
        let want = Want::at([0.0, 0.0, 0.0], 180.0, &Camera::EYE);
        let mut got = json!({
            "cam_position": want.position, "cam_euler": [360.0, -180.0, 0.0], "cam_fov": 60.0,
            "cam_fov_projection": 60.000004, "screen_width": 1024, "screen_height": 576, "locked": true,
            "gameplay_obstructed": false, "player_roots": ["Skateboard For Skating"], "hidden_renderers": 29,
            "kept_canvases": ["Sign WorldSpace"], "post_effects_on": ["Bloom"], "cam_overlaps": [], "cam_ground": 1.6,
        });
        assert!(checks(&want, &got, true, false).is_empty());
        got["cam_overlaps"] = json!(["Wall (Level)"]);
        got["post_effects_on"] = json!(["Vignette", "Bloom"]);
        let bad = checks(&want, &got, true, false);
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert!(checks(&want, &got, true, true).len() == 1);
    }
}
