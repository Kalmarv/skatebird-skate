# Durable knowledge (verified, reusable)

Linked from `STATE.md`. Add only facts verified against files or runs, with the artifact that proves them.

## SkateBIRD (Steam 971030)

- Install: `C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD`, 3.2 GB, no anti-cheat, no bundles,
  no Addressables. `MonoBleedingEdge` => Mono, `Assembly-CSharp.dll` readable.
- Unity **2019.4.20f1**, serialized container **version 21** in every file.
- Layout: `SkateBIRD_Data/{level0..level13, sharedassets0..13.assets, globalgamemanagers,
  globalgamemanagers.assets, resources.assets}`.
- Scene order (BuildSettings): 0 title, 1 bedroom 1, 2 rooftop, 3 office, 4 server room, 5 bedroom 2,
  6 credits, 7 flashback, 8 loading, 9 MasterAudio, 10 Win10 Platform, 11 Steam Platform, 12 playground,
  13 skate heaven.
- Playable extents ~200-400 m; ~150k triangles per level. `Level Elements/Bounds` holds a city skyline
  and clouds at up to 5.7 km (make a compile span 11 km; candidate for a filter).
- Meshes are ProBuilder (`pb_Mesh*`) and Unity static-batch `Combined Mesh (root: scene)*`; **no LODs, no
  terrain, no lightmaps**, 1-23 reflection probes per scene. UV0 is often 4 components.
- Shaders are built-in ones shipped as real Shader objects: Standard, Standard (Specular setup),
  Legacy Shaders/Bumped Diffuse|Specular|Transparent[/Cutout]/Bumped Specular, Houdini/SpecularVertexColor,
  Shiny/Unlit Two Tone, Minimap/Mission Marker, Particles/Standard Unlit.
- PhysicMaterials (in `sharedassets0`): `SkateBase Frictionless`, `SkateBase InAir`, `SkateBase Tumbling`,
  `SkateWheels`, `Bouncy Birb`; most colliders have none. Surface map used:
  `data/skatebird-surfaces.json` (default -> concrete, wheels/bouncy -> rubber, skatebase -> default).
- Materials may carry non-finite shader parameters (`_CameraFadeParams` g = inf), which the world cannot
  represent.

## Unity SerializedFile v21 (vs primase's v22-only reader)

- Header: `u32 metadata_size, u32 file_size, u32 version, u32 data_offset` (big-endian), then
  `u8 endian` + 3 reserved. **Metadata starts at offset 20**; its tables end `metadata_size` later;
  object data starts at `data_offset`.
- Object table entry: `i64 path_id` (4-aligned), **`u32 byte_start`** (v22 widened it to i64),
  `u32 byte_size`, `u32 type_index`.
- `SerializedType` for v21: class id, stripped bool (v16+), script type index i16 (v17+), script hash
  only for MonoBehaviour, old type hash. Same as v22.
- Reference for the whole layout: UnityPy `files/SerializedFile.py` (`__init__`, `ObjectReader.from_reader`).
- Gate: `worlds/unity/tests/skatebird_census.rs` compared primase against a UnityPy census of all 31
  files: 503,045 objects, exact path ids, class ids, byte offsets and sizes.

## SkateBIRD level data (`data/skatebird-levels.json`, from `tools/skatebird_scan.py`)

- Spawn = the `ResetPosition` with `startingPosition=true` (level7 has none: `Player Birb` at (0,1.189,0)).
- Bounds = enabled non-trigger colliders, mesh AABBs transformed, excluding anything over 100 m.
- Travel points = the game's own `ResetPosition` anchors + `MissionGiver`s, 11-16 per level.
- **Coordinate mapping**: Unity (x, y, z) -> primase scene (x, y, -z) (`SourceFrame::new([PosX, PosY, NegZ])`).
  Unity yaw 0 faces +Z, which becomes -Z after the mirror, so **scene yaw = (180 - unity yaw) mod 360**
  (the writer's forward is `(sin yaw, cos yaw)`; travel markers use the same rule).
  Verified: the rooftop spawn (-18.7, 1.189, 84.8) maps to scene (-18.7, -84.8), where the map has geometry.

## The route (program/, primase pinned by git rev)

```
cd program && cargo build --release        # CARGO_TARGET_DIR=<modding>/.targets/<agent>
skatebird-export build --typetrees <modding>/cache/primase/pm-core/w9/typetrees.tpk --out <modding>/cache/skatebird/build     --studio <modding>/tools/ReSkateStudio-20261012/reskate_cli.exe [level2 ...]
```

- Per level: `<out>/<level>/map/SkateBIRD_<level>` (the normalized map) and `<out>/<level>/stage/Patch` (the mod
  folder `SkateBIRD_<level>`). Spawn and travel points come from `data/skatebird-levels.json`; every choice is in
  `program/src/settings.rs`, the Studio compile's too (`--no-stream --no-lods --no-gi --time-of-day noon`).
- `--no-stream` keeps these small levels resident: no WorldPartition cells, so none of Studio's own vista proxies,
  and the shared runner's vista guard (`check_no_studio_vistas.py`) passes (level2, 2026-10-19).
- Type tree package: AssetRipper TPK covering 2019.4.20f1 exactly.
- Game: `$RESKATE_GAME`; mods under `Mods/`, enabled in `Mods/mods.json`.

## Running skate. for tests

- Through the shared runner only: reskate-tools `ports/gta/engine/ingame.sh OUT copy <stage>/Patch SkateBIRD_<level>
  "<commands>" <seconds>` under `kit/run game --job` (it copies the mod in, disables every other mod, restores the
  player's mods/profile/DLL after, and refuses a level with Studio vistas). `NO_DLL=1` runs without the probe DLL.
- Load a level: `load levels/game/dingolevel_reskate_skatebird_<level>/dingolevel_reskate_skatebird_<level>`; the
  runner takes a screenshot at every `trainer where`. `tools/views.py skate-commands <level>` makes a run's commands.
- Caleb's own install has 55 mods enabled; loading it produces a black HUB screen, so never judge a map there.

## Scripts in `tools/`

- `views.py` - the fixed views per level (`data/skatebird-views.json`) and the skate commands that shoot them.
- `install_mod.py <stage> <game> <name> --display "..."` - copy `stage/Patch` into `Mods/`, write manifest,
  set the display name, enable it in `mods.json` (with a backup).
- `compare_shots.py out.png left.png right.png`, `contact_sheet.py` - side-by-side verification sheets.
- `skatebird_shot.sh`, `skatebird_reference_shots.py`, `skatebird_teleport.py`, `skatebird-teleport-plugin/` - the
  SkateBIRD side (lab copy with BepInEx); runbook in `docs/screenshot-runbook.md`.
- `skatebird_scan.py` - makes `data/skatebird-levels.json`; `qa_report.py` - spawn/marker checks of written maps;
  `unitypy_census.py` - the independent census used to gate the v21 container reader.
- **Shell trap**: a child `bash` started from an agent's own shell is *WSL* bash (C:\WINDOWS\system32\bash.exe is
  first on PATH). `orca job run -- bash ...` starts **Git Bash**; call `"$BASH"` inside scripts.
- Write scripts with **LF line endings** (CRLF breaks `<<'EOF'` heredocs).

## Unity overlay meshes
- SkateBIRD draws mission markers with `Minimap/Mission Marker` on MeshRenderers (big yellow spheres in a naive
  export; invisible in game). The ds-modding fork's `unity-drawn` left out interface shader families by name; main's
  does not (the review dropped that filter), so check the maps for these spheres.
- ReSkate's live time of day overrides the compile's `--time-of-day` (log: `TOD_3_Afternoon`), so a map's sky is
  set by BAM, not by the compile flag.

## Screenshot parity: what works (2026-10-18, verified)
- SkateBIRD's view is only reproducible when the plugin **sets Camera.main** to `Euler(cam_pitch, cam_yaw, 0)` and
  re-asserts it (Cinemachine fights it otherwise). With `cam_yaw = pose yaw`, the measured camera bearing equals the
  pose yaw to 0.00 deg on all 8 levels (`out/skatebird_spawn_refs.json`).
- The skate camera converges to the spawn marker's yaw after it settles, so a shot ~30-45 s after the level loads
  looks along the marker yaw.
- Confirmed mapping: **scene yaw = 180 - unity yaw** (bedroom reference and skate shot at 234.82 show the same wooden
  platform/wall on the same sides). The camera must look along that scene yaw on both sides.
- Remaining differences are camera distance/height/FoV (skate's third-person is higher and farther than the bird's).

## Screenshot parity: what does NOT work (kept for the record)
- **The spawn yaw does not set the view.** Two runs at the same spawn marker (bedroom, yaw 234.82) gave different camera
  headings; skate's third-person camera starts at a direction unrelated to the marker (the offline profile has no
  camera-yaw setting). SkateBIRD's camera likewise keeps its own orientation after a teleport.
- Mouse `rel` through WinDrive does **not** turn the skate camera (4x 800 counts: identical view); retesting with
  `scanmode on` (raw input). Without camera control, pixel-matching views is not possible; match the player pose and
  landmarks instead, or find a camera command in both games.
- ReSkate has a free camera (`status` shows `freecam=`) but no console verb to set its pose.

## Screenshot parity method (same spot in both games)

- **skate.** has a console (`~`, launcher `console_key` 192) and ReSkate's trainer: `trainer tp <x> <y> <z>` moves the
  skater, `trainer where` prints `(x, y, z) heading N`, `trainer option hud 0` hides the HUD
  (`modding/refs/ReSkate/Extension/Trainer/trainer.cpp`, read-only reference). Commands can also be queued at launch
  with `RESKATE_STARTUP_COMMANDS`. Heading follows the mouse (`rel dx`), so turn until `where` reports the target.
- **SkateBIRD** has no teleport command in its ReflexCLI (`MissionManager.LoadLevel` only). A BepInEx 5 plugin in a lab
  copy (`ds-modding/skatebird-lab`) reads `data/skatebird-teleport.json` and sets the skater's position/rotation.
- Coordinates: Unity (x, y, z) -> skate scene (x, y, -z), yaw 180 - unity yaw, identical on both sides.

## Machine/tool gotchas

- This bash tool refuses to execute `.exe` files under `C:/Users/kalmarv/ds-modding` ("command not found");
  run them through `powershell -NoProfile -Command "& '<path>' args..."` or `cmd /c`.
- The Aerial screensaver (a fullscreen topmost window) covers the desktop and makes screen captures show
  the wallpaper: `um win ps`, then `um win kill <pid>` on the Aerial entry that has a title.
- `um win shot` (gfxcapture) times out on CEF/Steam windows; use GDI CopyFromScreen of the desktop instead.
- Steam install: `Start-Process 'steam://install/<appid>'`, then click the Install button by client coords
  (`um win drive --proc steamwebhelper "rect"` gives the client rect; screenshot at 1:1 to map pixels).
- ReSkate launcher PLAY button: client 1440x840, button at about (1184, 658).
- Windows Security / firewall prompts (`PickerHost`, UAC) block synthetic input to other windows (UIPI). Do not
  close them (Caleb, 2026-10-18); work around them by targeting the game window/process after it has focus.
