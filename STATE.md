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
- 0.3.0 release candidate (Caleb uploads to Thunderstore himself; not published by me): releases/skatebird/0.3.0/
  SkateBIRD_<LevelName>-0.3.0.zip x8, icon.png in each where Caleb gave a screenshot (cache/skatebird/icons/<level>.png:
  1, 2, 5, 7, 12, 13; office and server room still wanted). Program on primase main 95ba2b9a. Changes since 0.2.0:
  level-name maps; mission-only objects, unlocks, the player's board and the camera's arrow left out; GrindableRail
  chains as grind curves (136 in level1); room shells cast no shadow (ambient fill lights gone); office adds cubicle
  surround walls, ceiling lights, window wall; server room its tower; office and server room ceilings not drawn;
  big windows clear, other glass frosted; 5 travel points; level7 spawn dropped to the ground; overhead textured pause
  map over the play space with only walls/ceilings left out; lab plugin fly mode (F6 / L3).
- 0.2.0 / 0.1.0 test zips (superseded): releases/skatebird/0.2.0, 0.1.0.

## Night test 2026-10-20 (Caleb asleep; results for the morning)
- Build: all 8 levels in cache/skatebird/build-0.3.1 (jobs f1af824f + the first three in-session), packaged into
  releases/skatebird/0.3.0 (zips 16-32 MB; icons for 1, 2, 5, 7, 12, 13).
- Game run 64b20afe (kit/run game, ingame.sh copy mode, all 8 in one launch): 32 of 32 shots, no crash dumps; contact
  sheet cache/skatebird/runs/night-031/sheet.jpg. Office start (shot 9) and server room start (13) in daylight with sky
  overhead (ceilings not drawn); flashback spawns on the floor (21; spawn y 1.189 -> 0.0). Still very dark only right
  under big structures (bedroom 2 under the shelving, 19; flashback under the table, 23; playground under the play
  structure, 26): a real shadow plus skate's weak bounce light, the trade of dropping the fill lights. Left for Caleb.
- Offline: server room and office rays up from the start meet nothing (ceiling omitted); pause maps textured with only
  walls/ceilings left out (office floor and server tower show).
- Not switched: primase's generic set-shadow-casting + lower-reskate-shadows (2e03ee20). Check job 53e3bd86: bedroom 2
  gets 149 non-casting materials vs 111, the 38 extras being the scene's own castShadows Off (the Unity reader still
  states core.shadows). Reported to the owner and lanes; main stays on 95ba2b9a; patch in
  scratch/skatebird/skatebird-coord/diag/shadow-switch-2e03ee20.patch.

## Known gaps
1. Fill lights show as bright spots where they sit near ceilings/walls (the writer gives every light specular 1).
2. Teleport views all face Unity yaw 180 (skate has no console turn): several face walls; skate's camera sits behind
   the skater there.
3. Materials: bedroom bowl floor/supports, playground gravel tiling, leaves sparser than SkateBIRD's.

## Caleb's 0.2.0 / 0.3.0 test findings
All in 0.3.0 except 8 (cup lid z-fight, Caleb: "not really that worried about it") and 2 (parked: lighting).
1. Floating pretzels / "tape" on the line: mission collectibles (`SkateCollectible`, "Munch The Leftovers" snacks;
   shown only during their mission) with every `EnableOneAtRandom` candidate stacked (pretzel, donut).
2. Floor colour differs (grey speckle in SkateBIRD, smooth maroon in skate): Caleb, "I think the floor might just be
   the lighting". Parked behind 5. The Voronoi bakes look right (office carpet red velvet bakes dark maroon).
3. Flasks above the magazine ramps: "Elevated Thinking" collectibles (`Science Formula Collectible / Visual States /
   Not Collected / flask_erlenmeyer`, `SkateCollectible`).
4. Big cups above the stapler ramps: "Crucibird / Cups To Stall" (`SkateCollectible`, `TweenToPosOnEnable`).
   Census (scripts/mission_census.py): 117 drawn renderers under `Missions - Bedroom 1` are `SkateCollectible`
   subtrees, ~45 mission-giver birds (`MissionGiver`/`BirbDecorator`); "Stuff That Changes" / "Dirty Stuff That Goes
   Away" (`MissionToggled`) are world state, kept as the scene has them. Fix: the reader states `SkateCollectible`,
   `EnableOneAtRandom`, `MissionGiver`; the program leaves collectible and giver subtrees out.
5. Lighting poor overall, very dark under the bed. Caleb's idea: let the room shell pass light and light by the sky.
   Wired (primase 949dcb01 `SetReskateShadowCasting`, after raise-skate): `settings::room_shell` per indoor level
   (level1 `Level Elements/Bounds/`, level3 `Level Bounds/`, level4 `Room Bounds/`, level5 `Bounds/`, level7
   `Brick Bedroom/Bounds/`; the reader's bare-`/` paths; outdoor 2/12/13 none), shared=copy. The ambient fill lights
   are gone (Caleb: "with the lighting change let's remove the ambient lights cause we won't need it").
6. Clotheslines not grindable: SkateBIRD's grinds are `GrindableRail` point chains (level1: 540 nodes, 155 chains;
   raw layout: prev PPtr, next PPtr, two more PPtrs, flags), no colliders; the ropes (`CurvedLine3D` meshes) have no
   collision, so nothing grinds. The map has 0 grind curves (`prepare-reskate-grinds`: "made prisms": 0). Fix: the
   reader states the chains, lowered to `skate.grind_curve` (primase has curves and Studio's prisms already).
7. Glass tabletop invisible though it collides: `Level Elements / Thin Table / OfficeTable / Table Glass`, "Clean Shiny
   Glass" alpha 0x06, drawn as a cutout at 0.5 by `prepare-reskate-blended-surfaces` (its documented "glass mostly
   vanishes"). ClearGlass (windows, 0x7d) vanishes too. Caleb: make it slightly cloudy. Wired: primase 949dcb01
   `UniformAlpha::Frost{opacity: 0.35, grey: 0.8}` (opaque, a*base + (1-a)*grey, a = max(alpha, 0.35)).
8. Cup top z-fighting (a non-mission cup, `Level Elements / ... / Cup` + `Lid` "Glass_MAT" alpha 0x90, kept opaque by
   the cutout): the lid now draws solid over the cup's top; geometry gap to check.
9. "Way too many mission markers" (rooftop, up to 20 travel points): done in the program: travel points are reset
   positions only (givers go with the missions), the start first, spread to `settings::TRAVEL_POINTS` = 5.
10. A floating skateboard: the player's own board, `Skateboard For Skating` (`SkateboardGateway`), placed in every
   scene; also `Unlocks / ... Unlocking Collectible` (`PermanentUnlockingCollectible`: copper decks, wheels, clothes)
   and `Main Camera / Mission Arrow` (levels 7, 12). Added to primase-gta-lane's mission-only drop.

## Next
1. Read night-031's shots; report to Caleb in the morning. Office and server room icons from him.
2. Later, only when Caleb says go (via the modding coordinator): the conversion as one readable recipe, ~100-150
   lines; tooling in its own folder or small crate; markers from primase_skatebird's core markers instead of level.rs.

## Decided (Caleb's words; never dropped, only replaced by a newer decision)
- 2026-10-20 · Night testing: "I'm going to sleep now so you can launch skate yourself to finish testing. I'll take ss tomorrow".
- 2026-10-19 · Release: "I'm gonna release this version on thunder store, it's plenty good" (he uploads). Windows: "fully clear windows, opaque other glass. just the big windows clear". Server room: "just make the floors or whatever above not shadow cast". TOD: "default tod is noon, noon is fine".
- 2026-10-19 · Release gate: "lights, mission, grinds, waypoint cleanup and they're almost ready to release"; mod images: "I'll take SS of the maps".
- 2026-10-19 · Missions: "any mission only objects should be removed". Markers: "way too many mission markers" (now 5).
- 2026-10-19 · Lighting: "make the room material allow light to pass through and just use skybox lighting"; "with the lighting change let's remove the ambient lights cause we won't need it" (replaces the "Fill lights" decision below).
- 2026-10-19 · Glass: "I get it's glass but lets make it slightly more cloudy". Grinds: "you can't grind on the clothesline ... I want to".
- 2026-10-19 · Lab plugin fly mode: "push a button and I just float in the air"; controller: "left stick press to activate then just thumb sticks to move and look, lt down, rt up".
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
