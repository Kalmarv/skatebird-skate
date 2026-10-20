# SkateBIRD screenshot runbook (Windows, 1920x1080)

Target: `C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD` (Steam 971030, build "2.0.0 STEAM",
Unity 2019.4.20f1, Mono). Read-only install; everything below writes only screenshots and the game's own
save/registry data.

Verified on 2026-10-18 by launching the game windowed, opening the in-game console, loading `rooftop` from
the title screen, and capturing 1920x1080 frames with `um win shot`. Raw captures:
`shots/skatebird-runbook/01-title.png` … `07-rooftop-resumed.png`.

Evidence status: window flags + 1920x1080 client, F1 console, `MissionManager.LoadLevel rooftop`, the
first-run chain, and `um win shot` were observed live. The `-devconsole` gate, spawn/reset code, save paths,
and command prefixes are code reads. The raw ffmpeg gfxcapture graph was exercised on a desktop window, not
on SkateBIRD. `tools/skatebird_shot.sh` had its orchestration exercised against stubs (launch args, drive
sequence, capture, kill); it was not run end-to-end after the game was closed.

## Exact camera views (lab copy + plugin)

For reference shots to match against skate. at the same views, use the lab copy and its plugin, not the console route
below: the plugin puts Camera.main at an exact world pose and FOV, so nothing depends on the follow camera.

```bash
# once (and after a game update or a plugin change): lab copy + BepInEx 5.4.23.5 + the plugin, built
tools/skatebird_lab.sh            # --check only reports
# one level's views; opens a game window, so only under the screen lock
C:/Users/kalmarv/modding/kit/run game --job -- "$BASH" tools/skatebird_views.sh level2 C:/Users/kalmarv/modding/cache/skatebird/ref/level2
```

- **Lab**: `C:\Users\kalmarv\modding\cache\skatebird\lab` = copies of `SkateBIRD.exe`, `UnityPlayer.dll`,
  `UnityCrashHandler64.exe`; junctions `SkateBIRD_Data`, `MonoBleedingEdge` into the Steam install (never recurse or
  delete through them; remove one with `rmdir`); BepInEx 5.4.23.5 win x64 (zip pinned by sha256 in
  `cache/skatebird/downloads`); `BepInEx/plugins/SkateBIRDTeleport.dll`. The game runs without Steam
  (`SteamClient.Init` failure is caught), and the save in LocalLow is shared with the Steam copy.
- **Runner**: the program's `skatebird-shots` (`program/src/shots.rs`; `tools/skatebird_views.sh` checks the lab and
  runs it, `SKATEBIRD_EXPORT` picks the build). It replaced `tools/skatebird_views.py` on 2026-10-19: job f7c1b3b5
  (level2 `--chase`) gave the same requested poses and read-backs as the Python run's `ref/level2` to the last digit,
  the same checks and NPC counts, and pixel-identical frames except 37 and 65 pixels of animation in two of them.
  `cargo test` runs the whole protocol against a stub plugin (`shots::tests`).
- **Control**: the plugin reads `<lab>/control/command.json` and answers in `reply.json`; ops `load`, `screen`,
  `teleport`, `camera`, `shot`, `readback`, `release`, `quit` (protocol at the top of `Plugin.cs`). No keyboard input,
  no console, no window capture: the plugin loads the scene with `LoadingManager.LoadScene`, sets
  `Application.runInBackground`, and writes the PNG from the rendered frame (`ScreenCapture.CaptureScreenshotAsTexture`
  at end of frame) together with the camera pose of that frame.
- **Camera**: position = view position + `--eye` m (default 1.6), rotation `Euler(pitch, yaw, 0)` (Unity: yaw from +Z
  towards +X, positive pitch looks down; default pitch 0), vertical FOV `--fov` (default 60), 1024x576 windowed.
  CinemachineBrain is disabled and the pose is re-asserted in Update, LateUpdate and the camera's onPreCull.
- **Hidden**: screen-space canvases (HUD, menus, console), the renderers under the player's root
  (`Skateboard For Skating`, board and bird), and the post effects depth of field, lens distortion, chromatic
  aberration, vignette and motion blur (`--keep-post` keeps them). Colour grading, bloom and the rest stay.
- **Output**: `<outdir>/<index>_<name>.png` per view, `poses.json` (requested pose, read-back of the saved frame and
  after it, checks), `checks/player_{shown,hidden}.png` (the control for the bird hiding: one pose looking at the bird
  with and without it) and `plugin.log`. Exit 1 if a read-back differs from the request (1 mm, 0.05 deg), the screen
  is not 1024x576, a menu is open, a screen canvas or one of those post effects is still on, or hiding the bird
  changes nothing in the control frame.
- **Verified 2026-10-19** (job ff37a39b, level2, 43 s of run): 4 views, every read-back equal to the request
  (position to float precision, yaw/pitch/roll exact, projection FOV 60.000), 3 screen canvases and 29 player
  renderers hidden, post effects off: DepthOfField, LensDistortion, ChromaticAberration, Vignette (on: AmbientOcclusion,
  ColorGrading, Bloom); the bird control changed 12,615 pixels, and the shots looked at by eye are sharp, without
  HUD or bird. Left over in the hidden control frame: two pixel-sized pink dots where the bird's feet were (something
  outside `Skateboard For Skating`, a decal or particle; not visible in the view shots). The first run (job d8bd98c9,
  before the post-effect fix) was fully blurred by the depth of field.

- **All levels, one launch**: `tools/skatebird_views.sh level1,level2,level3,level4,level5,level7,level12,level13 <root>
  --chase` shoots each level into `<root>/<level>` (job 270b5027, 2026-10-19: 32 shots in 3m17s,
  `cache/skatebird/ref/<level>`). Each shot also records the colliders within 0.2 m of the camera and the distance
  to the collider below it (`cam_overlaps`, `cam_ground`; a camera in or against geometry fails its check). In that
  run the spawns of level3, level4 and level12 failed on `Camera Follow`, the game's camera target 1.5 m above the
  player; the plugin now ignores the player's, the camera's and the virtual cameras' follow/look-at colliders
  (verified in job dad3987b: all 32 shots pass).
- **NPCs hidden** (Caleb: skate. gets no skinned meshes, so no NPC birds): unless `--keep-npcs`, every
  `BirbDecorator` but the player's (with all it renders: body, crest, clothing) and every other
  `SkinnedMeshRenderer` is hidden; `poses.json` has `npcs_hidden` per level and the counts per shot. Job dad3987b
  (2026-10-19, 3m50s) hid 15/15/9/54/15/1/15/6 renderers in level1/2/3/4/5/7/12/13: "Generic Scenery Birb" birds
  everywhere but level7 (a skinned `bird` under `Level`) and level13 (the skinned `Seagull` of `Missions - Skate
  Heaven`). Compared with the shots before (same poses): outside the NPCs the frames are pixel-identical except
  animated things (clock hands, spinning collectibles, leaves, a sparkle); every changed region was looked at.

### skate.'s third-person camera (`--chase`)

`tools/skatebird_views.sh <level> <outdir> --chase` puts the camera where skate.'s third-person camera was at
level2's spawn: D -0.3 (0.3 m *ahead* of the view point along its yaw), H 1.5 m up, pitch +1.2 (down), vertical FOV
55.3, yaw offset 0. Fitted on 2026-10-19 by `tools/skatebird_chase_fit.sh` (job 0da1c4ce, 649 renders in one launch,
13m41s; results `cache/skatebird/chase-fit/level2/fit.json`, side-by-sides `t1_where_1_pair.png`,
`t2_where_1_pair.png`) against skate. shots `cache/skatebird/runs/t{1,2}/level2/where_1.png`:

| target | D | H | L | P | dY | F | score (640 px) |
|---|---|---|---|---|---|---|---|
| t1 | -0.25 | 1.50 | -0.2 | +1.16 | -0.25 | 55.36 | 0.799 |
| t2 | -0.35 | 1.50 | 0.0 | -6.23 | +4.18 | 55.24 | 0.807 |

- Method: only the camera position needs the game. One launch renders a grid of positions (D -1.5..1.5, H 1.2..1.8,
  L -0.6..0.6, then a finer grid around each best) at pitch 0, vertical FOV 100, flat black sky, 1920x1080; pitch,
  yaw offset and FOV are fitted offline per render by warping it (a pinhole rotation/zoom is a pixel re-map) and
  correlating blurred gradient orientations with the target (colour, lighting and sky count little; skate's skater
  area and FPS counter masked). `fit` and `views` re-fit / re-draw from the renders without the game.
- Precision: H is sharp (1.45 and 1.55 score clearly lower); D and L are flat within about +-0.15 m; F agrees to
  0.1 deg between the launches. The orientation does not: the two skate launches differ by 7 deg in pitch and 4.4 deg
  in yaw (skate's camera had not settled the same way), so a SkateBIRD shot matches one launch's framing, not both.
  The preset takes t1 (camera along the heading).
- What differs between the games at that pose (from the edge overlays): skate has a yellow half-dome at the ramp's
  right end, a row of small posts and a green bag on the far ledge, a bump at the foot of the ramp and strong
  sun shadows; SkateBIRD has a small NPC bird at the ramp foot. Everything structural (ramp lip, walls, pipes, arches,
  wires, buildings) lines up.

## TL;DR (one level, one shot)

```bash
# 1. Display/lock check (game windows here are coordinated through the screen lock).
orca lock status screen | head -1

# 2. Launch windowed at 1920x1080 with the dev console enabled. This call blocks until the game exits.
um win launch "C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD.exe" -- \
  -screen-fullscreen 0 -screen-width 1920 -screen-height 1080 -devconsole

# 3. From another shell: open the console, load the level.
um win drive --proc SkateBIRD "focus" "sleep 3000" \
  "key 0x70" "sleep 800" \
  "type MissionManager.LoadLevel rooftop" "key 0x0D"
sleep 12

# 4. Capture the client area (1920x1080, no cursor, works while covered).
um win shot --exe SkateBIRD.exe out/rooftop-spawn.png

# 5. Close it by PID.
um win ps | grep SkateBIRD        # take the pid
um win kill <pid>
```

Or run `tools/skatebird_shot.sh rooftop out/rooftop-spawn.png` (see "Scripts" below).

Fresh save only: after the load, the game chains three first-run screens. Send `Esc`, `Esc`, `Enter`
(2 s apart). The first-options popup is gated on `OptionsManager.HasShownFirstOptions`
(`UIManager.cs:762-765`), which the first run writes as `__options_has_shown_first_options = true`; the
repeat-run reload itself was not visually confirmed (the game was stopped before that shot).

## 1. How levels load

`Assembly-CSharp` decompiled at `out/decomp/skatebird/` (ilspycmd 9.1.0).

- `Skatebirb.Util.ImportantStuff.GameLevels` (`ImportantStuff.cs:10-23`):
  `title, bedroom_1, rooftop, office, server_room, bedroom_2, credits, playground, skate_heaven, NumLevels, Invalid`.
- Scene names are a hard-coded array (`ImportantStuff.cs:39`):
  `title screen, bedroom 1, rooftop, office, server room, bedroom 2, credits, playground, skate heaven`.
  Index == enum value. `GetLevelScene()` (`:117`) maps enum -> name; `GetCurLevel()` (`:130`) maps the
  active scene name -> enum.
- `Skatebirb.Managers.LoadingManager.LoadScene(string)` (`LoadingManager.cs:133`) starts a coroutine that
  fades out, loads the additive `loading` scene, then `SceneManager.LoadSceneAsync(targetScene)`
  (`:152-190`).
- The only in-game caller exposed to us is the ReflexCLI command `MissionManager.LoadLevel`
  (`MissionManager.cs:2395`): parses a `GameLevels` name with `Enum.Parse`, then calls
  `LoadingManager.LoadScene(ImportantStuff.GetLevelScene(level))`.
- There is **no command-line level flag**. The only `Environment.GetCommandLineArgs()` scan in the game
  assembly is `OptionsManager.DevConsole_Enabled` looking for a substring `devconsole`
  (`OptionsManager.cs:832-845`). Unity's own `-screen-*` flags are handled by the player.
- Scenes outside the enum (`flashback` = level7, `loading`, `MasterAudio`, platform scenes) have no
  `LoadLevel` path. `LoadLevel` also accepts `title`, `credits`, `bedroom_1`, etc.
- `SceneManager.LoadScene("rooftop")` is the actual Unity scene name, so a hypothetical in-engine loader
  must use that string, not the display name "Rooftop".

Level/spawn table (from `out/skatebird-scan.json`, generated by `tools/skatebird_scan.py`; Unity world
space, metres, quaternion as serialized):

| LoadLevel arg | Scene name | Spawn marker | Position (x,y,z) | Yaw | Quaternion (x,y,z,w) |
|---|---|---|---|---|---|
| `title` | `title screen` | – | – | – | – |
| `bedroom_1` | `bedroom 1` | `Reset Positions/Level Start` | 45.4, 16.796, 55.77 | -54.8157° | 0, -0.460322, 0, 0.887752 |
| `rooftop` | `rooftop` | `Start Positions/Middle Roof` | -18.7, 1.189, 84.8 | 172.9341° | 0, -0.9981, 0, -0.061623 |
| `office` | `office` | `Start Positions/Keyboard start` | -34.026, 11.98, -78.359 | 0° | 0, 0, 0, 1 |
| `server_room` | `server room` | `Main Tower/First Level/Reset Points/Level Start` | 28.36, -1.0, -16.86 | -58.6617° | 0, -0.489852, 0, 0.871806 |
| `bedroom_2` | `bedroom 2` | `Reset Positions/Bed` | -26.68, 15.2, 10.05 | -54.9471° | 0, 0.461339, 0, -0.887224 |
| `credits` | `credits` | – | – | – | – |
| `playground` | `playground` | `Reset Positions/Start` | -59.09, 1.1, 62.76 | 106.1471° | 0, 0.799407, 0, 0.600789 |
| `skate_heaven` | `skate heaven` | `Reset Points/Level Start` | 30.7, 0.2, -23.6 | -14.0914° | 0, -0.122661, 0, 0.992449 |

Yaw is the world-space forward `(0,0,1)` rotated by the spawn quaternion, computed as `atan2(x, z)`.
`rooftop` spawn faces roughly `(0.123, 0, -0.992)` (mostly -Z).

## 2. Windowed at a fixed size

Verified: launched with

```
-screen-fullscreen 0 -screen-width 1920 -screen-height 1080
```

and `um win drive --proc SkateBIRD "rect"` returned a **1920x1080 client area** (window at 320,191 on the
2560x1440 display). Unity command-line flags apply to that run; use them every time instead of relying on
saved state.

What the game itself writes (not the install):

- Registry `HKCU\Software\Glass Bottom Games\SkateBIRD` (Unity Screenmanager + PlayerPrefs):
  - `Screenmanager Resolution Width` = 1920, `Resolution Height` = 1080 (DWORDs, hashed value names)
  - `Screenmanager Fullscreen mode` = 3 (`Windowed`; 0 exclusive, 1 fullscreen-window, 2 maximized)
  - `Screenmanager Resolution Use Native` = 0
- Save data `%USERPROFILE%\AppData\LocalLow\Glass Bottom Games\SkateBIRD\`:
  `saved_bools.sav`, `saved_ints.sav`, `saved_floats.sav`, `saved_strings.sav`, `Player.log`
  (`SaveToDiskManager.cs:216` builds `persistentDataPath/saved_<type>.sav`).
- The options menu's resolution slider calls `Screen.SetResolution(w, h, !OptionsManager.Windowed, rate)`
  (`SliderOptionMenuItem.cs:684`); `OptionsManager.Windowed` is just `!Screen.fullScreen`
  (`OptionsManager.cs:224-240`). There is no separate game-defined windowed PlayerPref.

`um win shot` captures only the client area, so the captured image is exactly 1920x1080 regardless of the
window's screen position. Keep the window unmaximized and at a fixed size (`um win drive ... "size 1920 1080"`
can reassert the client size if a resize happens).

## 3. Skipping the title screen

There is no "start level" command-line switch. The verified route is:

1. Launch with `-devconsole`.
2. At the title screen press `F1` (console), type `MissionManager.LoadLevel rooftop`, Enter.
3. `LoadingManager` closes the console itself and loads `loading` -> `rooftop`.

This was observed with the title screen visible and worked. [INFERENCE, code-only: the console canvas exists
in every scene and the component that clears the key (`ControlBoard`) is only in gameplay scenes, so it
may also open on the title screen without `-devconsole` — we did not test that.] In gameplay
`ControlBoard.Start()` clears the console key unless `OptionsManager.DevConsole_Enabled` is true
(`ControlBoard.cs:212-218`), so always pass `-devconsole`.

First-run chain (fresh save, observed in this order):

1. `These can all be changed at any time...` accessibility/options popup -> `Esc`.
2. `CREATE-A-BIRB` -> `Esc` (Go Back).
3. Pause book (`SCARED OF HEISTS`) -> `Enter` on `RESUME SKATEBOARDING`.

Then gameplay appears at the spawn (screenshot `07-rooftop-resumed.png`).

## 4. Spawn and camera (what a comparable screenshot means)

- `Skatebirb.Gameplay.ControlBoard.Update()` calls `ResetPosition.ResetToStart(targetBoard)` once per
  scene load (`ControlBoard.cs:374-379`, flag `hasResetToStart`). That teleports the board to the single
  active `ResetPosition` whose serialized `startingPosition` is true (`ResetPosition.cs:25-42`).
- Therefore the gameplay spawn is the marker in the table above, **not** the `Player Birb` object's
  authored transform. Example rooftop: spawn `(-18.7, 1.189, 84.8)`, while `Player Birb` is placed at
  `(-10.4, 1.189, 82.3)` in the scene.
- Respawn/`Reset` (R) uses `TeleportToResetPosition`, which is the player's saved reset point, not the
  level start. Do not use R to return to spawn.
- The camera is a Cinemachine rig (`ImportantStuff.MainStateCamera`, `CinemachineStateDrivenCamera`);
  `SkateCamStateBrain` switches between Unified/Panning/zoom states and follows the skater
  (`Skatebirb.Camera/SkateCamStateBrain.cs`). Its initial world transform is runtime state, not an authored
  scene value, so an exact pixel match against another engine is not a valid contract.
- Reproducible contract: same level, no input after load, fixed settle delay, fixed 1920x1080 client.
  Compare the spawn position/orientation and landmark geometry. For a skate. capture, place the camera at
  the spawn point + yaw, match FOV empirically, and compare geometry rather than identical framing.
- Determinism limits: idle bird animation, ambient particles/confetti, sky/cloud motion, camera smoothing.
  A 3-4 s settle after the menu chain removes camera motion; it does not freeze animation. A fixed delay is
  the reproducible part. `Options.ToggleUI` hides the HUD (verified in code, not yet re-shot).
- HUD-free shots: open console and run `Options.ToggleUI` (class prefix is `Options`, see below). It
  persists as `__options_disable_ui` in the save, so toggle it back after capture. `Options.ToggleButtonGFX`
  hides button prompts.

## 5. Console / debug build flag

- Build flag: `-devconsole` (substring match, `OptionsManager.cs:144,832-845`). It does not add commands;
  it only stops `ControlBoard` from disabling the ReflexCLI open key in gameplay.
- ReflexCLI is compiled in. `ReflexCLI.UI.BootStrap.CreateUI()` instantiates `ReflexConsoleCanvas` and
  `ReflexConsoleEventSystem` from Resources after every scene load.
- Actual keys come from `resources.assets` object 208, `ReflexConsoleSettings` (MonoScript
  `ReflexCLI.Settings`, `globalgamemanagers.assets` id 744). Serialized fields:
  `_EnableConsole = true`, `_OpenConsoleKey = 282`, `_CloseConsoleKey = 282`, `_DisplayUnityLog = true`,
  `_DefaultNamingConvention = 1` (`ClassPrefix`).
  `KeyCode` 282 is **F1**. The class default (Tab) is overridden by the asset: **Tab does nothing**, F1
  opens and closes (observed).
- `ClassPrefix` (`CommandRegistry.cs:104-125`) means command keys are `Class.Method` for classes without a
  `ConsoleCommandClassCustomizer`. Observed working: `MissionManager.LoadLevel rooftop`.
- Useful commands (prefix, method, args):
  - `MissionManager.LoadLevel <GameLevels name>`
  - `MissionManager.SkipAllMissions`
  - `Options.ToggleUI`, `Options.ToggleButtonGFX`, `Options.SetFakePlatform Desktop`
  - `UI.DisplayFPS`
  - `System.Quit` (clean exit; `um win kill` is fine for screenshots)
  - `Physics.SetTimescale <float>`
  - `Assist.CyclingTeleport` (toggles cycling of reset points, not spawn selection)
  - `Persistence.ClearAllSavedData` (destructive)
  - `PrintCommands` prints every registered key into the console overlay.
- The console input line is at the bottom-left of the client area (~x 20-1000, y ~1000). After `F1`, just
  `type` text; `Enter` executes. `Esc` closes the console.

## Capture

`um win shot` (verified on SkateBIRD; writes 1920x1080 PNG, no cursor):

```bash
um win shot --exe SkateBIRD.exe out/rooftop.png
```

Raw ffmpeg equivalent (use the **um-bundled** ffmpeg; the winget one on PATH has no gfxcapture). Input
graph verified against a live desktop window; this is the same graph `um win shot` uses:

```bash
FF="C:/Users/kalmarv/AppData/Local/universal-modder/ffmpeg/bin/ffmpeg.exe"
"$FF" -hide_banner -loglevel error -f lavfi \
  -i "gfxcapture=window_exe=SkateBIRD.exe:capture_cursor=0:max_framerate=60,hwdownload,format=bgra" \
  -frames:v 1 -y out/rooftop.png
```

The capture is of the window, not the desktop, so it works while covered and does not capture the desktop
cursor. `-frames:v 1` can take a moment; do not kill the game during capture (we lost one shot that way).

## Menu-only fallback (click coordinates at 1920x1080)

All coordinates are **client** coordinates (the captured PNG is exactly the client area). Measured from
`shots/skatebird-runbook/06-rooftop-gameplay.png` (pause book), `04-rooftop.png` (first-options popup),
`05-rooftop-after-popup.png` (Create-a-Birb).

Pause book (`SKATE ANOTHER PARK` is the level-select entry). Items are left-aligned at x ~= 1069; click
x = 1150 on the text row:

| Item | y |
|---|---|
| RESUME SKATEBOARDING | 479 |
| SKATE ANOTHER PARK | 548 |
| GAME OPTIONS | 617 |
| DRESS UP BIRD | 686 |
| YOUR MIXTAPE | 756 |
| HOW TO SKATE | 824 |
| PET THE BIRD | 896 |
| QUIT FOR THE DAY | 965 |

Other captured screens:

| Screen | Action | Coordinate |
|---|---|---|
| First-options popup (fresh save) | RESUME SKATEBOARDING | 958, 828 (or `Esc`) |
| Create-a-Birb (fresh save) | Go Back / `Esc` | top-right, y ~120 (use `Esc`) |
| Console input | click to focus input | 400, 1002 |
| Title screen | "press any button" | keyboard/mouse; screen not re-captured after advancing |

**Not verified:** the title-screen main menu and the Level Select button grid were not captured before the
game was stopped. Do not invent coordinates for them; use `MissionManager.LoadLevel` (supported), or
keyboard navigation (`W`/`S` + `Enter`, `Esc` per the on-screen legend) and capture a reference first.

## Scripts

`tools/skatebird_shot.sh`:

```bash
# one level, fresh save (dismisses the 3 first-run screens), HUD hidden, then closes the game
tools/skatebird_shot.sh rooftop out/rooftop-spawn.png --first-run --no-ui

# repeat run with an existing save, HUD left on
tools/skatebird_shot.sh office out/office-spawn.png

# capture an already-running window, um path or raw ffmpeg path
tools/skatebird_shot.sh capture out/now.png
tools/skatebird_shot.sh capture out/now.png --ffmpeg
```

The script checks that no SkateBIRD is running, prints `orca lock status screen` (warns if held/paused),
launches with the fixed flags, waits for the window, sends `F1` + `MissionManager.LoadLevel <level>` +
Enter, waits, optionally dismisses first-run screens / toggles the HUD, captures, restores the UI toggle,
and kills the game by PID. `--settle N` changes the post-load wait (default 12 s), `--keep` leaves it open.

Run it with Git Bash (`C:\Program Files\Git\bin\bash.exe`), not WSL bash: the commands are Windows
(`um`, `powershell`) and the script uses `C:/` paths.

## Gotchas

- Do not run two instances. `um win ps` must show no `SkateBIRD`.
- The game is a Steam build but launches directly by exe; keep Steam running (steam_api64 is loaded).
- `-devconsole` is required for the console in gameplay; `F1` is the key, not `Tab`.
- `MissionManager.LoadLevel` accepts enum names, not display names (`bedroom_1`, not `bedroom 1`;
  `skate_heaven`, not `skate heaven`).
- Loading from the title screen or from gameplay works; the console closes automatically when loading
  starts.
- Level7 `flashback` and the loading/audio scenes cannot be reached with `LoadLevel`.
- The first-run chain only happens on a fresh save. `--first-run` on an existing save sends Esc/Esc/Enter
  into normal gameplay and can open/close menus; omit it for repeat runs.
- HUD toggle is persisted in `saved_bools.sav`; the script toggles it back, but a killed run can leave
  `__options_disable_ui = true`.
- The desktop is shared: check `orca lock status screen`; do not drive the `Skate` window, and release the
  display by killing SkateBIRD when done.
- Nothing here writes to the game install. Changes are limited to screenshots, the LocalLow save folder,
  and Unity's `HKCU\Software\Glass Bottom Games\SkateBIRD` registry key.
