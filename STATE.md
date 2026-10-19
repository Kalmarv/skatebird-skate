# SkateBIRD: state (skatebird-coord, 2026-10-19 evening; rewrite, never append)

## Goal
Caleb, 2026-10-19: "get all the Skate Bird maps into Reskate through primase with visual matching and polish."
Done = each level matches SkateBIRD at fixed views in side-by-sides Caleb accepts, plays (roll, grind, collision,
travel), on primase main.

## What works (verified)
- The program (`program/`, `skatebird-export build --jobs 3`) reads, lowers, writes and Studio-compiles every level
  from the scan's spawn/travel data: all 8 in 2 m 34 s on primase f374e4e7 (job 96742069). Choices in
  `program/src/settings.rs`; compile resident, no LODs, no GI, noon (so no Studio vistas).
- In game (shared runner): level1, level2 load, no dumps (runs t1/t2). Rooftop geometry matches SkateBIRD at the
  spawn view (Caleb: "that looks matched").
- Matching method: fixed views (`data/skatebird-views.json`, spawn + 3 travel anchors); SkateBIRD camera fitted to
  skate's chase camera (F 55.3 deg vertical, 1.5 m up; skate's framing varies ~7 deg pitch / 4 deg yaw per launch);
  SkateBIRD reference shots of all 8 levels, no UI/player/NPCs: `cache/skatebird/ref/<level>/` (job dad3987b).
- One-launch runs: reskate-tools ingame.sh takes several mods (492b302); `tools/ingame_levels.sh`.

## Shipped
Nothing.

## In flight
- All 8 levels in one skate launch (job 9a27d3ba, run t3), then side-by-sides for Caleb.

## Blockers / known gaps
1. primase fixes are on primase-work only (f374e4e7: lowerings declare their effects; aa7daf03 level7 decode;
   72172370 screen-camera rule): they must reach main before done; the owner is landing them.
2. Teleport views all face Unity yaw 180 (skate has no console turn): several face walls; pick better view points.
3. From the old fork, unchecked on main: dark office/server interiors (ambient), playground canopies, office UV.

## Next
1. Side-by-sides of all 8 for Caleb; per-level differences and their causes.
2. Polish in order: canopies, office UV, ambient lighting, tint, gravel, two-tone. Then gameplay checks.
3. Held for Caleb's go: promotion requests (level scan into primase's SkateBIRD crate; variants, unmaterialed,
   unify-texture-wrap, pack-orm passes).

## Decided (Caleb's words; never dropped, only replaced by a newer decision)
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
