# Moving the program onto primase main (in progress, handed over 2026-10-19)

Branch `primase-reskate-lane/new-api` (on `settings/explicit` 69053f7). Started by the primase reskate lane; handed to
the GTA lane (after its step 13) by the primase owner. Nothing is merged.

## What the branch does (builds; not yet proven against the baseline)
- `program/Cargo.toml`, `tools/next-program/Cargo.toml`: primase by git rev a8ddf9f7 (github main; was a path
  dependency on the ds-modding fork, primase-work `archive/ds-skatebird` 7b81a686).
- `primase-skate-scene` -> `primase-reskate` (dialect, interchange, raise-skate, map writer).
- `unify-texture-wrap` and `pack-orm`: copied unchanged from the fork into `program/src/{wrap,pack}.rs`, run by the
  pass manager. Promotion candidates for primase (main has neither; main's raise-skate refuses mixed wraps and wants
  one mask image).
- The spawn: a `skate.marker` the program adds after raise-skate (main's raise-skate has no spawn option), field for
  field what the fork's raise made (exact height, the yaw as given, no snap; a 3-number spawn snaps to the ground).
- `--drop` / DropByName removed (fork-only; `tools/export_level.sh` never passed it).
- The GLB: main's glTF writer invents no material and no texture coordinates, so the program runs
  `prepare-gltf-materials` (the fork's fallback: 0.8 grey, metallic 0, roughness 0.85) and `fill-missing-texcoords`
  on a copy of the world for the GLB only; the skate path sees the world as lowered.
- The map: the program now writes it itself (`<out>/map/SkateBIRD_<level>`, `--name` to change), after the scene
  folder, with every choice the fork's `build_map` made inside `BuildOptions`/the writer stated in
  `program/src/settings.rs` with its fork source line: texture cap 256 (box), overlays 4 mm, blend grid 200, blend
  objects 1000 (the fork's fixed count, not the cull-ID pool), water over the content + 400 m, ocean as stated, pause
  map from geometry 2160 px over the content, no light cap, grinds as Studio, markers snapped for the encoding,
  encoding 200 m tiles, 3/150 instancing, limit 4780.
- `--map-from-scene <scene folder> <out> --name N`: the same map preparation and writer on a scene folder (for the
  comparison below).
- `tools/next-program/tint_policy.rs`: `LowerTintReskate::default()` -> `uncapped()` (default is gone on main).

## Not done: the proof
Per level (8: level1-5, 7, 12, 13; spawns in `tools/export_level.sh`), split so each difference has an owner:
(a) old scene folder vs new scene folder (main's reworked Unity stack vs the fork);
(b) `--map-from-scene` on the OLD scene vs the fork's map (writer changes, primase steps 1-10);
(c) `--map-from-scene` on the NEW scene vs the program's direct map (composition; the interchange round trip).
Byte-identical, or each difference explained by file/field.

Baseline (the fork): built and run for level2 only (exit 0; 73 s export + 10 s map; 445 MB). The baseline worktree is
`.wt/primase-reskate-lane-base` (69053f7 with primase 7b81a686 checked out as its `primase/`); scripts and the
baseline binaries in the reskate lane's scratch (`scratch/primase/primase-reskate-lane/bird_*.sh`,
`unity_export-old.exe`, `build_map-old.exe`; scratch is cleared after 3 days, so rebuild them with
`bird_old_build.sh <target dir>` if gone). Inputs: SkateBIRD_Data (Steam), `cache/primase/pm-core/w9/typetrees.tpk`,
`data/skatebird-surfaces.json`, `data/skatebird-levels.json`.

Builds sharing a target dir across checkouts must touch every tracked file first (kb note 90ac8495): cargo reuses
artifacts of a different checkout of the same package otherwise.

## Expected differences
Fork-only reader behaviour not on main (interface-overlay drop, built-in sun, some static-batch/line handling) and the
Unity reworks that landed differently (alpha from the drawn pass, Standard, double-sidedness from culling, vertex
colour statement, ambient as data); the declared writer changes of steps 1-10; the scene folder's layout (main's
interchange writes the assets beside the scene, no separate assets folder); stats.json's recorded source path.

## Handover from primase-gta-lane (2026-10-19): where it stands

The primase-gta-lane took this branch over from the reskate lane, then handed it on (the owner's decision: the
SkateBIRD coordinator takes it). Nothing below is merged.

### Branch state
- `primase-gta-lane/main-migration` = 0879c2a, the program and `tools/next-program` pinned to primase main 458e1c79;
  `cargo test` 8 passed, `clippy --all-targets -D warnings` clean. The commits since the reskate lane's 8dddef8:
  - e1a902f onto 0871f934: `settings::web` is `Options::new(TileSize)` with KTX2, 512 MiB encoding memory and the byte
    budget; `settings::web_textures` is the fork's KTX2 caps (512 colour and normal, 256 mask) as
    `prepare-web-textures`, run on a copy of the world before the KTX2 tiles only (the fork left PNG tiles at their
    size; the skate scene is made from the world as lowered); material variants set through `PrimitiveMut`; the
    copied pack.rs/uv_policy.rs made clippy clean.
  - fc3708c onto cb2245cd (main's Unity reader reads UV channels of 3 or 4 components as u and v, counted).
  - 0879c2a onto 458e1c79 (main's Unity reader: quads and line strips, non-finite values, material sentinels kept);
    `uv_policy`'s refusal names each use's primitive topology.
- Still to do: repin to c0d6539b (the reskate batch) and add its recipe lines (the reskate commit messages have them
  exactly): `prepare-reskate-uv-repeats{decals=omit,blended=cutout}` before the overlays,
  `prepare-reskate-collision{surfaces=coalesce,weld=mm}` after the blend order, raise-skate's light calibration stated
  (`RaiseOptions::new(name, LightCalibration::Scale(..))`), `place-reskate-spawn` with `at=` if used.
  `summarize-reskate-map` is only needed before a pool budget; SkateBIRD's is `objects=1000`, so not needed. c0d6539b
  also carries the interchange name fix that proof (c) needs.

### Proofs (reports: scratch/gta/primase-gta-lane/bird/reports/<level>/)
- (b) `--map-from-scene` on the fork's scene folder vs the fork's map (job ff568715, program e1a902f): **identical in
  content on all 7 levels that build** (level1-5, 7, 13). `compare_map_meaning.py` compares order-independently:
  triangles per material (all of a material's objects together), collision, textures by content, lights, travel
  points, spawn. Only two things differ: the GLB generator string (`gta-skate` -> `primase`, +112 to +536 mesh bytes a
  level) and where a material's long merged section is split across objects. `compare_city.py` reports these as 28-30
  index-order groups per level; do not read its group count as failure.
- (a) old vs new scene folder and (c) `--map-from-scene` on the new scene vs the program's direct map: job b4a3e524
  (program 0879c2a on 458e1c79). The new program builds levels 2, 3, 4 and 13.
  - (a), levels 3/4/13: about 98 % of the files identical by content; scene.json differs in materials (21-31 of 81-119
    changed, 1 added), textures (37-63 changed, 10-23 fewer images), prototypes (11-30 changed, 1 added) and
    placements. Not classified yet (the expected differences listed above are the first candidates).
  - (c) failed on every level: main's interchange could not read its own scene folder back ("texture 'diffuse_2':
    name 'diffuse' is used twice"); c0d6539b's interchange fix should cure it. Rerun after the repin.

### Open failures on 458e1c79
- level1: uv_policy refuses "vinyl plastic gross - orange: base tiling still unbaked ... not baked: a primitive lacks
  the coordinate set"; the use is `e1006/1: Triangles, placed=true, uv0=absent`. **Correction to an earlier lead:**
  it is not a line submesh (it is Triangles). Main now reads quads as triangles where the fork skipped every
  non-triangle submesh, so it may be a quad submesh without UV0; not confirmed. Printing `LevelReport::notices` in
  the program (it does not yet) shows the quad and line conversions per level.
- level5: `prepare-reskate-overlays`: "prototype 'prototype_e631' part 2: no material". Probably a submesh the fork
  skipped (quads or lines) or one past the renderer's materials (Unity does not draw it); not investigated.
- level7: `Invalid("Combined Mesh (root: scene) 6 Tangent: invalid vertex input: non-finite value")`. It comes from
  `toolkit/vertex` `Format::decode` (lib.rs:101 refuses non-finite values) at `UnityMeshData::read`, before the Unity
  reader's own finite rules can apply (on main: a non-finite normal or tangent drops that stream from the primitive,
  a non-finite UV or colour reads 0, a non-finite position leaves the primitive out; all counted in
  `LevelReport::notices`). Proposed fix, not started: a non-refusing `decode_raw` / `decode_stream_raw` in
  toolkit/vertex (the existing functions unchanged), used by the Unity reader.
- level12: fails in the fork baseline itself: "Duck Tape 1: base tiling still unbaked; refusing to erase it. Bake
  counts: coordinate sets baked 626, not baked: materials sample a set with different transforms 4, not baked: no
  primitive uses the material 3, slot transforms cleared 18". The owner wants a program pass for it (each mixed
  material its own UV set, or bake per material, reported as an approximation) once the migration is proven.

### Where things are
- Fork baseline outputs: scratch/gta/primase-gta-lane/bird/old/<level>/ (`unity/` with the scene folder and GLB,
  `map/SkateBIRD_<level>`), 7 levels plus level12's failed export; job 4afc9125.
- Fork binaries: scratch/gta/primase-gta-lane/bird/unity_export-old.exe (1f70ffc3...) and build_map-old.exe
  (4190dec2...), copied from the reskate lane (`bird_old_build.sh`: program 69053f7 on fork 7b81a686), with
  bird_spawns.sh.
- Scripts: scratch/gta/primase-gta-lane/scripts/job_bird_old.sh (baseline), job_bird_b.sh (proof b),
  job_bird_new.sh (the new program, proofs a and c), compare_scene.py, compare_map_meaning.py, compare_stats.py,
  compare_city.py.
- Shared-target rule (kb note d31027b3): a second checkout of primase or of this program built into the same
  CARGO_TARGET_DIR needs its own cargo profile (`--config 'profile.X.inherits="release"' --profile X`) and every
  tracked file touched; outputs of a shared build are discarded.
