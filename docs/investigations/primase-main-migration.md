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
