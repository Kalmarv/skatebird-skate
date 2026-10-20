# SkateBIRD: state (skatebird-coord, 2026-10-19 evening; rewrite, never append)

## Goal
Caleb, 2026-10-19: "get all the Skate Bird maps into Reskate through primase with visual matching and polish."
Done = each level matches SkateBIRD at fixed views in side-by-sides Caleb accepts, plays (roll, grind, collision,
travel), on primase main.

## What works (verified)
- The program (`program/`, `skatebird-export build --jobs 3`) reads, lowers, writes and Studio-compiles all 8 levels
  from the scan's spawn/travel data in ~100 s on primase main 6feaa93f (job 4658f24a). Choices in
  `program/src/settings.rs`: compile resident, no LODs, no GI, noon; lights x10000 the standard calibration plus
  SkateBIRD's ambient as 15 m fill lights; blended surfaces (not decals) as 0.5 cutouts.
- In game, all 8 in one launch (run t5, job bd52e06d): 32/32 view shots, no crash dumps. Side-by-sides
  `cache/skatebird/compare/t5/<level>.jpg` shown to Caleb: geometry matches on every level; interiors lit (office,
  server room, bedroom 2); playground canopies drawn. Rooftop matched earlier (Caleb: "that looks matched").
- Matching method: fixed views (`data/skatebird-views.json`); SkateBIRD camera fitted to skate's chase camera;
  SkateBIRD references `cache/skatebird/ref/<level>/` (no UI, player or NPC birds).
## Shipped
- 0.2.0 test zips for Caleb (not published): releases/skatebird/0.2.0/SkateBIRD_<level>-0.2.0.zip x8 (16-30 MB), copy in
  X:/modding-archive/skatebird/releases/0.2.0. primase main 062d6943 (everything on primase: markers, ambient fill,
  shader waves 1-2 incl. the Voronoi bake of 10 materials on 6 levels, compile stage), build job 4f7def3f (21 min,
  the Voronoi bake most of it), package 88fa3edb. Not loaded in game by me: Caleb tests it himself.
- 0.1.0 test zips (superseded): releases/skatebird/0.1.0.

## In flight
Nothing running.

## Known gaps
1. Fill lights show as bright spots where they sit near ceilings/walls (the writer gives every light specular 1).
2. Teleport views all face Unity yaw 180 (skate has no console turn): several face walls; skate's camera sits behind
   the skater there.
3. Materials: bedroom bowl floor/supports, playground gravel tiling, leaves sparser than SkateBIRD's.

## Caleb's 0.2.0 test findings (open)
All bedroom 1 (level1) so far. Caleb decided (2026-10-19): "any mission only objects should be removed".
1. Floating pretzels / "tape" on the line: mission collectibles (`SkateCollectible`, "Munch The Leftovers" snacks;
   shown only during their mission) with every `EnableOneAtRandom` candidate stacked (pretzel, donut).
2. Floor colour differs (grey speckle in SkateBIRD, smooth maroon in skate): open. The Voronoi bakes look right
   (office carpet red velvet bakes dark maroon, as SkateBIRD draws the office carpet); which floor this is still to
   find.
3. Flasks above the magazine ramps: "Elevated Thinking" collectibles (`Science Formula Collectible / Visual States /
   Not Collected / flask_erlenmeyer`, `SkateCollectible`).
4. Big cups above the stapler ramps: "Crucibird / Cups To Stall" (`SkateCollectible`, `TweenToPosOnEnable`).
   Census (scripts/mission_census.py): 117 drawn renderers under `Missions - Bedroom 1` are `SkateCollectible`
   subtrees, ~45 mission-giver birds (`MissionGiver`/`BirbDecorator`); "Stuff That Changes" / "Dirty Stuff That Goes
   Away" (`MissionToggled`) are world state, kept as the scene has them. Fix: the reader states `SkateCollectible`,
   `EnableOneAtRandom`, `MissionGiver`; the program leaves collectible and giver subtrees out.
5. Lighting poor overall, very dark under the bed. Caleb's idea: let the room shell pass light and light by the sky.
   Lever found: skate materials take `cast_shadows` (writer: `material.rs`, `map.rs` ~2339; no pass sets it yet).
   Proposed test: shell materials (`Level Elements / Bounds`) cast no shadows, so noon sun and sky reach inside.
6. Clotheslines not grindable: SkateBIRD's grinds are `GrindableRail` point chains (level1: 540 nodes, 155 chains;
   raw layout: prev PPtr, next PPtr, two more PPtrs, flags), no colliders; the ropes (`CurvedLine3D` meshes) have no
   collision, so nothing grinds. The map has 0 grind curves (`prepare-reskate-grinds`: "made prisms": 0). Fix: the
   reader states the chains, lowered to `skate.grind_curve` (primase has curves and Studio's prisms already).
7. Glass tabletop invisible though it collides: `Level Elements / Thin Table / OfficeTable / Table Glass`, "Clean Shiny
   Glass" alpha 0x06, drawn as a cutout at 0.5 by `prepare-reskate-blended-surfaces` (its documented "glass mostly
   vanishes"). ClearGlass (windows, 0x7d) vanishes too. Caleb: make it slightly cloudy. Fix: the pass draws a
   uniform-alpha (glass) material under the cutoff as opaque frosted glass instead of dropping it.
8. Cup top z-fighting (a non-mission cup, `Level Elements / ... / Cup` + `Lid` "Glass_MAT" alpha 0x90, kept opaque by
   the cutout): the lid now draws solid over the cup's top; geometry gap to check.

## Next
0. In flight on primase (2026-10-19), pin each rev as it lands, rebuild only on Caleb's go:
   - primase-owner -> reskate lane: `set-reskate-shadow-casting{materials=...|placements=<prefix>, shared=copy|refuse}`
     (program: room shell placements, shared=copy); `prepare-reskate-blended-surfaces` gains required
     `uniform=cutout|frost{opacity=F}` (program: `frost{opacity=0.35}`).
   - primase-gta-lane: reader states SkateCollectible / EnableOneAtRandom / MissionGiver + a pass dropping collectible
     and giver subtrees; GrindableRail chains read and lowered to `skate.grind_curve`.
1. 0.2.0 zips (primase 062d6943, everything on primase) to Caleb; he tests in game himself.
2. Later, only when Caleb says go (via the modding coordinator): the conversion as one readable recipe, ~100-150
   lines (read_level + read_markers + read_voronoi_programs, one Pipeline of typed passes from Unity lowering to ReSkate
   preparation, write_map, compile_map); the tooling (shots, views, sheets, packaging) in its own folder or small binary crate, nothing fancy, so program/ is only the conversion; no
   hand-printed reports (notices); raise-skate and bake-normal-scale as passes; markers from primase_skatebird's core
   markers instead of level.rs.

## Decided (Caleb's words; never dropped, only replaced by a newer decision)
- 2026-10-19 · "Wait until all additions are in": switch code as each primase promotion lands, but no rebuild, game run or new zips until all have.
- 2026-10-19 · Shader list approved (via the modding coordinator): primase wave 1 Legacy Bumped family, Autodesk Interactive, Houdini specular, Self-Illumin/Unlit as emission; wave 2 Voronoi as a bake; Two Tone stays base colour with a notice.
- 2026-10-19 · Interior light: ambient as "Fill lights" (shadowless point grid from SkateBIRD's ambient); brightness "x10000" the standard calibration (run light1).
- 2026-10-19 · Language/shape: "this should be 90% rust when it's "done". Using primase, with maybe a handful of scripts"; "This is my goal for all game readers I think. The mod is fine" (the C# BepInEx plugin stays); order: after the side-by-sides (repo cleanup, then the primase promotion requests; plan in this session: ~1,100 of the program's 1,821 Rust lines to primase).
- 2026-10-19 · Pinning: "you can use a branch idc, the fixes must land on primase main before it's "done" though"; screenshots: "you can also do all the ss in one run". Repo: "just use main".
- 2026-10-19 · NPC birds: "I don't think we need the npc birds in skate".
- 2026-10-19 · Sky: "sky is okay, every custom skate level has the skate sky. we can leave it. you can add it to the reader, but for reskate writer we're not gonna use it".
- 2026-10-19 · Library requests: "lets wait a bit then we can submit this stuff" (promotion requests held; bug fixes already with the owner).
- 2026-10-19 · Per-level maps first ("per level"); gameplay scale "1:1 for now"; skylines: "keep".
- 2026-10-19 · Sub-agents: "I like opus, sonnet is fine for simple shit."
- 2026-10-19 · Delete C:\Users\kalmarv\ds-modding: "yes" (after checking out/ for evidence worth keeping).
- 2026-10-19 · Goal restated: "get all the Skate Bird maps into Reskate through primase with visual matching and polish."
- 2026-10-18 · "All the Skate Bird levels as ReSkate Map(s). Verified by matching in game screenshots in Skate Bird to ReSkate."
- 2026-10-18 · "SkateBIRD: keep all its progress (incl. the 6 ds-modding-only primase commits); resumes after GTA."
- 2026-10-18 · Caleb, relayed by gta: "make it work, then correct, then fast. One focused check showing each fix works; land on fmt/clippy/unit/architecture. No additional censuses, reference runs or provenance chases unless ambiguous."
- 2026-10-18 · "Drop all skate bird shit for now, focus only on gta" (`orca log gta --at 431480`); Caleb started this
  coordinator on 2026-10-19 (coordinator task: "Caleb chose to start you now").

## Open questions for Caleb
None now.

## Where things are
- Repo: github Kalmarv/skatebird-skate + gitea skatebird-skate, branch main. Helper skatebird-ref (Opus) for the
  SkateBIRD side (`tools/skatebird_views.sh`, lab copy `cache/skatebird/lab`, runbook `docs/screenshot-runbook.md`).
- Build output `cache/skatebird/build/<level>/{map,stage}`; runs `cache/skatebird/runs/<tag>`; target
  `.targets/skatebird-coord`. Type trees `cache/primase/pm-core/w9/typetrees.tpk`; Studio `tools/ReSkateStudio-20261012`.
- `C:\Users\kalmarv\ds-modding` was deleted 2026-10-18 (only `shots/` archived at
  `X:\modding-archive\skatebird\ds-modding-shots`); `docs/VERIFICATION.md` links into its `out/` are dead.
- Disk budget 120 GB; using ~3 GB (cache/skatebird) plus the target dir.
