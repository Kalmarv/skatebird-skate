# SkateBIRD: state (rewritten by skatebird-coord, 2026-10-19; rewrite, never append)

## Goal
Caleb, 2026-10-19: "get all the Skate Bird maps into Reskate through primase with visual matching and polish."
Verified by matching in-game screenshots, SkateBIRD vs ReSkate, at fixed camera positions per level.

## What works (verified)
- primase main (c0d6539b) carries the whole Unity stack: v21 containers (a3812258), alpha from the drawn pass
  (337dc297), Standard (904c2d89), ambient as data (b60f768e), cull/double-sided (3d8a257a), vertex colour
  (1f0f2612, 56bf50ee), depth write (37cab7fe), quads/lines and non-finite values (458e1c79). The fork commits are
  kept on primase-work `archive/ds-skatebird` 7b81a686.
- Program on main 458e1c79 (branch `primase-gta-lane/main-migration` 0879c2a): builds, 8 unit tests, clippy clean;
  levels 2, 3, 4, 13 export (job b4a3e524). Writer proof (b) identical in content to the fork on 7 levels (job ff568715).
- On the old fork (2026-10-18): all 8 levels compile and load "active" with the game's spawn (0.001 m) and 11-16
  travel markers (job 32be0cad); alpha fix PASS in game for rooftop and bedroom 1 solid surfaces (job 73cde79a).
- Camera parity method: the BepInEx plugin's bearing equals the pose yaw to 0.00 deg on all 8 levels.

## Shipped
Nothing. No zip, release or tag.

## In flight
- Migration, now mine (skatebird-coord): branch `coord/main-migration` (from e8aa21b) in `.wt/skatebird-coord-migration`,
  repinning to c0d6539b with batch 38's passes (uv-repeats, collision plan, stated light calibration). Plan and
  handover: that branch's `docs/investigations/primase-main-migration.md`.

## Blockers (on main 458e1c79, from the GTA lane's runs)
1. level1: uv_policy refuses `vinyl plastic gross - orange` (a Triangles use without UV0, e1006/1; cause unconfirmed).
2. level5: prepare-reskate-overlays "prototype_e631 part 2: no material" (a submesh past the renderer's materials?).
3. level7: toolkit/vertex decode refuses a non-finite tangent before the Unity reader's rules (library request).
4. level12: uv_policy "Duck Tape 1": mixed transforms on one UV set (fails on the fork too); program fix needed.
5. Lighting: office/server room nearly black on the fork; ambient is now data on main, no ReSkate consumer yet.

## Next
1. Finish the migration (build on c0d6539b, all 8 levels export), merge into main.
2. One in-game load per level of the new build; matching camera shots in SkateBIRD and ReSkate; `orca show` Caleb.
3. Polish in order: canopies, office UV, ambient lighting (core field via the owner), tint, gravel, two-tone.
4. Gameplay checks nobody ran: roll, grind, collision, travel points.
5. Delete `C:\Users\kalmarv\ds-modding` after moving `out/` evidence worth keeping (Decided).

## Decided (Caleb's words; never dropped, only replaced by a newer decision)
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

## Disk budget: 120 GB
`C:\Users\kalmarv\ds-modding` 40.4 GB (to delete); my target `.targets/skatebird-coord` (< 40 GB).

## Where things are
- Program: `program/` (repo `skatebird-skate`); per-level truth `docs/VERIFICATION.md`; evidence links there point
  into ds-modding `out/` until moved. Fork baseline worktree `.wt/primase-reskate-lane-base` (69053f7 + fork 7b81a686).
- Inputs: Steam `SkateBIRD_Data`; type trees `cache/primase/pm-core/w9/typetrees.tpk`; `data/skatebird-*.json`.
- Game: `$env:RESKATE_GAME`; runs only `kit/run game --job -- ...`. Studio compiles of these maps take no lock.
- kb `projects/primase` (SkateBIRD sections), `topics/windows-shell-traps`.
