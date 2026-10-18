# Combined SkateBIRD map (design note)

All eight skateable scenes are 100-300 m across (collider bounds), so one combined level is cheap geometry-wise.
Two deliverables are reasonable:
- **Per-level mods** (current pipeline): one ReSkate mod per scene, each verified in game. This is what exists today.
- **One combined map**: every scene on a grid, with travel points between them.

## Why not simply compile the eight scenes into one folder

`reskate_cli compile-map` takes one normalized map. Merging after compile is the stitching problem (quadrant
bundles, level root, depots, MeshVariationDatabase) that the GTA pipeline solved with tile stitching; not worth it
for 8 small scenes.

## Path: merge in primase, before the writer

The one-pipeline way is to build one `World` holding all eight levels:

1. Read level A with the Unity reader into a fresh `World`.
2. Apply a **translate pass** to every spatial component (instances/transforms, collision triangles, lights,
   markers, curves) by the level's grid offset.
3. Read level B into the same `World` and translate it by its offset, and so on (the reader spawns entities; it
   does not assume an empty world, but the scene resources like `Surfaces`/`SkateScene` need one owner).
4. Run the target passes once, write one scene, build one normalized map, compile once.

Missing piece: a `translate` pass in core (or meshops) that offsets `core.instance` transforms, `core.collision`
triangles, `core.light` positions, `skate.marker` positions and `core.curve` points by `(dx, dy, dz)`, counting
anything it cannot move. That is a small, generic pass.

## Offsets and backdrops

Each scene ships its own `Level Elements/Bounds` skyline and clouds reaching 5.7 km; two levels closer than ~12 km
would overlap backdrops. Options: place levels 12 km apart (outside skate's 9.6 km root - not possible for all 8),
cut the backdrop geometry (lose the skyline look), or keep per-level maps. **Recommendation: ship per-level maps
and treat a combined map as a follow-up that cuts the backdrops.**

## Travel points

ReSkate writes at most 25 travel points per level (`worlds/reskate/src/build.rs`); the game data has 11-16 per
level (100+ total). A combined map must pick 25 (e.g. the spawn anchor plus 3 per level).

## Built once (2026-10-18)

`tools/merge_all.sh` merged the eight final scenes at 2,500 m spacing into `out/all-levels/scene` (15,801 table
files, 5,821 copied files, 1.79 GB) and compiled `out/all-levels/stage` (29 tiles, 959 streamed cells, 1,866 bundles).
Two problems:
- The grid spans 17.5 km, past skate's 9.6 km root; use ~800-1,000 m spacing, or drop `Level Elements/Bounds` first.
- `build_map` reports only 22 drawn prototypes and 668k merged triangles although the scene has 4,851 prototypes with
  valid `s<i>/prototypes/...` GLBs (per-level sum is ~2.8M). Not yet explained; per-level maps are unaffected.
- 119 travel markers exist; ReSkate builds at most 25 (warning in the compile log).
