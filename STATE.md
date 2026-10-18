# SkateBIRD: state (staged 2026-10-18 by stage-games; the game agent rewrites this file, never appends)

## Goal
Caleb, 2026-10-18: "All the Skate Bird levels as ReSkate Map(s). Verified by matching in game screenshots in Skate Bird to
ReSkate." Then one combined map (agent plan, `docs/COMBINED_MAP.md`); gameplay scale not decided.

## What works (verified)
- All 8 levels read, lowered, compiled (Studio 20261012, `--no-gi --time-of-day noon`) and load to "active" in an
  isolated skate. copy, with the game's exact spawn height/yaw (to 0.001 m) and 11-16 travel markers each (capture job
  32be0cad, 2026-10-18; `docs/QA-report.md`).
- v21 SerializedFile reader against an independent UnityPy census: 31 files, 503,045 objects exact (job 1142f72e on
  `gate/unity-v21` 9435a217).
- Alpha fix (draw pass, not any additive pass): rooftop and bedroom 1 PASS for solid surfaces in game (job 73cde79a,
  `out/alpha-fix/compare_level2.png` in ds-modding).
- Camera parity method: the BepInEx plugin's bearing equals the pose yaw to 0.00 deg on all 8 levels.

## Shipped
Nothing. No zip, release or tag.

## In flight
Nothing running. Paused 2026-10-18 ("Drop all skate bird shit for now, focus only on gta"). Resume point
(`docs/history/STATE-ds-modding-2026-10-18.md`, top): canopy geometry and leaf texture bindings survive compilation; next,
inspect the compiled leaf OSK opacity pixels and card facing before choosing a runtime experiment. Then UV (office),
ambient lighting (needs a core field), Houdini tint, gravel, Shiny Two Tone. Details: `docs/investigations/alpha-runtime.md`,
`docs/VERIFICATION.md`.
- Library, all unlanded: primase-work `archive/ds-skatebird` 7b81a686 (the six ds-modding fork commits on 7da1c6a;
  review TAKE/DROP table `archive\primase-review-2026-10\archive\work-2026-10\review\ds-modding-primase.md`, kept for Caleb's primase review), `gate/unity-v21` 9435a217, `gate/unity-alpha` 989a5009,
  `archive/unity/material-usage` b0158258 (cull), `archive/wt/unity-reader/tree3` 770e236e (vertex colour producer),
  `gate/vertex-color` 4053fbe8 (GitHub primase), `gate/depth-write` ec25950b. Order: `work\primase\unity-reader\RESUME.md`.

## Blockers
1. The library work above is unlanded; `program/Cargo.toml` still path-depends on `../primase` (the ds-modding fork).
2. Lighting: office and server room are nearly black; needs a core `Environment.ambient` (primase owner, after 0.3.0).
3. Playground canopies vanish skate-side (cause unknown; next check above).
4. Combined map: Studio's 25 travel points (119 exist), 5.7 km skylines (8-11 km compile spans), 17.5 km at 2,500 m spacing.
5. Scale and skyline are Caleb's calls.

## Next (smallest first; all proposed unless marked)
1. Resume only when Caleb says (Decided: after GTA).
2. Pin primase by git rev instead of the `../primase` path (first `archive/ds-skatebird`, then main once the Unity stack lands).
3. Land the Unity stack through the primase owner (v21, alpha, cull, vertex colour, depth write); port the fork's TAKE
   items; move fork-only logic the review marked PROGRAM into `program/`.
4. Canopy check, then UV, then ambient. Then the gameplay checks nobody ran: roll, grind, collision, travel points.

## Decided (Caleb's words; never dropped, only replaced by a newer decision)
- 2026-10-18 · "All the Skate Bird levels as ReSkate Map(s). Verified by matching in game screenshots in Skate Bird to ReSkate."
- 2026-10-18 · "Drop all skate bird shit for now, focus only on gta" (`orca log gta --at 431480`).
- 2026-10-18 · "SkateBIRD: keep all its progress (incl. the 6 ds-modding-only primase commits); resumes after GTA."
- 2026-10-18 · Caleb, relayed by gta (ds-modding STATE): "make it work, then correct, then fast. One focused check showing each fix works; land on fmt/clippy/unit/architecture. No additional censuses, reference runs or provenance chases unless ambiguous."

## Open questions for Caleb (recommendation first)
1. Gameplay scale: 1:1 or 2-4x for a human skater? Recommended: decide from one level played at 1:1.
2. Skylines (5.7 km backdrops): keep or cut with `--drop 'Level Elements/Bounds'`? Recommended: cut for the combined map.
3. Eight per-level mods first, or the combined map first? Recommended: per-level first.
4. Delete `C:\Users\kalmarv\ds-modding` (40 GB, mostly regenerable outputs) now that the source is here and the fork
   commits are on primase-work? Recommended: yes after a last check of `out/` evidence to keep.
5. Models for sub-agents (BRIEF: ask Caleb). (The port so far was DeepSeek, then Codex.)

## Disk budget: 120 GB
Today: `C:\Users\kalmarv\ds-modding` 40.4 GB (`out` 22.5, `target` 7.3, `game` 6.1 with junctions into the ReSkate
install, `library-targets` 2.5, `primase/target` 1.6); source and evidence < 0.4 GB.

## Where things are
- Code: this repo (from `ds-modding`, phase 0 rescue); library on primase-work as above; review of the fork
  `archive\primase-review-2026-10\archive\work-2026-10\review\ds-modding-primase.md`.
- Old workspace (not a repo; do not write there): `C:\Users\kalmarv\ds-modding` (outputs `out/`, game copy `game/` with
  junctions `Data`, `__Installer` into the ReSkate install, lab copy `skatebird-lab/` with junctions into Steam).
- Library worktree `ds-modding\library-worktrees\unity-v21` belongs to `modding\primase` (prune there after removing).
- Game: `C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD` (read-only). Skate: `$env:RESKATE_GAME`.
- Type trees: `work\primase\pm-core\w9\typetrees.tpk`; Unity built-in shader source `work\primase\unity-reader\ext`.
- kb `projects/primase` ("SkateBIRD port: alpha/blend parity failure", "SkateBIRD vertex-colour mismatch").
