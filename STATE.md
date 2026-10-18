# SkateBIRD -> skate. (ReSkate) — working state

Owner: current Codex takeover, coordinated through gta. Previous owner ended.

## PAUSED by Caleb through gta — GTA 0.3.0 only (2026-10-18)

All SkateBIRD work stopped. Gta cancelled queued jobs. Do not schedule builds, offline jobs or game
runs until explicitly resumed after GTA 0.3.0.

Resume point: canopy geometry and leaf texture bindings survive compilation; Bird uses blend programs
(1444,3612). Earlier Sledding trees are opaque; its cutout materials use (825,1891,3096). Next inspect
compiled leaf OSK opacity pixels and card facing before selecting a runtime experiment. Evidence:
out/playground-leaf-{audit,depots}.json and out/sledding-reference-{audit,depots}.json.

Open findings: alpha repairs proven on private7b81a68; all8 alpha captures completed but visual parity
still fails. UV variant specialization/base tiling passes numerical proof, not runtime appearance.
Houdini tint fixture passes but is not integrated. Gravel procedural sampling and Shiny Two Tone
view/light-dependent shading are omitted. Lighting trials stopped; proper ambient awaits core/target
contract. Main migration and Bird library commits deferred until after0.3.0. Full evidence remains in
notes/alpha-runtime-2026-10-18.md and docs/VERIFICATION.md. No new jobs started for this handoff.

## Takeover checkpoint (2026-10-18; supersedes historical constraints/status below)

- Latest offline canopy/shader audit: normalized leaf geometry + alpha PNG retained; compiled leaf key
  fd9936984b8d4d4c binds leaf basecolor/OSK and stock blend programs1444,3612. No disappearance boundary
  proven yet: next inspect compiled OSK opacity pixels and card facing. out/playground-leaf-{audit,depots}.json.
  Actual earlier Sledding skate reference audited (v2/out + stage); opaque pine texture alpha255, ordinary
  surface programs; its clip eyes/statues use825,1891,3096. out/sledding-reference-{audit,depots}.json.
  Gravel DXBC proves randomized 3x3 colour/normal sampling; Shiny Two Tone is view/light-dependent with
  reflection sampling. Both fallbacks lose real behavior. Notes alpha-runtime contain exact evidence/jobs.
  Gta/Caleb now defer Bird library landing (v21/alpha/vertex-colour) until AFTER GTA0.3.0; keep private copy.
  No new game checks queued; lighting stopped. Offline scripts completed.

- Latest closure: no-BAM capture0463240c exited1 on the late Park active-marker assertion;
  saved screenshot, exact runtime map and bound skater establish the office was reached. Reviewed
  out/lighting-no-bam/gi_control_level3.png: no useful brightening versus Enlighten control.
  Compiler still reports BAM despite --no-bam-lighting, so effective exclusion remains unproven.
  No further lighting runs queued. Result hashes in lighting-no-bam/level3/result-hashes.json.
  Standard f5f80a88 and ambient data989a5009 checks passed per unity-reader; with lead, NOT confirmed main.

- UV office3adf243a PASSED export/build/compile and numerical desk gate in out/uv-fix-v2/level3/uv-proof.json;
  both prototype_e87/e97 retain required base0.1 transform to floating-point tolerance. Runtime UV still
  blocked by dark office. Tint fixturebed86633 PASSED1 test, not yet integrated into frozen UV program.
  Gta authorized last lighting axis: add --no-bam-lighting to explicit Enlighten control, same map/noon.
  Buildd0c646d5 ->capture0463240c, out/lighting-no-bam, start watch armed. If dark, await agreed core
  Environment.ambient Flat/Gradient linear contract after GTA0.3.0; no more speculative lighting runs.

- Enlighten runtime48b8245d completed: office active and stationary view, but remains almost black, no useful
  visible improvement. Recorded in VERIFICATION and investigation note; parent/unity-reader notified.
  No new lighting run queued. Nonzero generated probes alone do not prove sampled/useful ambient lighting.

- Current jobs: explicit Enlighten b26f0559 compile PASSED (19536 probes, 48 input systems; added
  map-specific /enlighten and /lighting/enlighten strings in compiled default CAS), capture48b8245d running.
  Default GI-only e6863abb compiled but no probes; cancelled its capture42aa8f0c. BAM warning is separate
  from map GI, not an enablement verdict. Output out/lighting-enlighten. Caleb asked to leave controls still.
  UV first build a258c1f0 compiled but refused real variant conflict: e395 has 5 material choices.
  program/src/variants.rs now specializes used placements before UV bake (3 prototype copies,34 placements).
  Full-source diagnostic6c8c1d9c PASSED,542 coordinate sets baked/no used transforms left. Full office rebuild
  with numerical desk UV gate3adf243a pending at out/uv-fix-v2. Houdini fixture requeued independently bed86633.

- GI wrapper recovery: 47009645 failed before compile (Get-FileHash unavailable); ec6c2aa5 failed at
  native stderr progress under PowerShell Stop. Their captures1887cb7a/a3c78b78 skipped. Python hash
  helper verified; Continue around compiler with explicit native exit status verified by stderr/exit0
  and stderr/exit7 fixture. Same GI-only test requeued e6863abb ->42aa8f0c, out/lighting-gi2. Start watch
  armed so Caleb can leave controls untouched. No changed lighting parameters or extra runtime trials.

- Unity-reader update: Standard-family fix f5f80a88 and ambient-data989a5009 (unity.render_settings v3:
  mode/sky/equator/ground/intensity) are on its branch with checks queued, not confirmed landed.
  No core ambient lowering exists yet; built-in light conversion is now explicitly counted as an
  approximation. Keep GI-only office experiment unchanged; data preservation alone is no runtime repair.

- Alpha rollout COMPLETE (private7b81a68, not main): all8 exports/captures reached requested active maps,
  all8 scene/layout/log/shot hash sets recorded in out/alpha-fix/artifact-manifest.json. Reviewed pairs and
  updated docs/VERIFICATION.md. Rooftop/bedroom1 solid-surface restoration proven; office/server too dark;
  playground leaf canopies still absent despite leaf geometry and nonzero-alpha texture in final map.
  Other levels retain major texture/lighting mismatch. Caleb is playing during launches: screenshot pose
  differences are expected user input, not harness drift; no automated gameplay certification.

- Latest: dark-wood normal scale bug confirmed; Standard must use base0.1 for bump, not saved bump1.
  Unity-reader verified exact2019.4.20f1 shader source and owns main fix. Independent DXT5nm channel/pixel
  trace finds no green inversion evidence (out/bedroom-normal-trace.json). Do not flip normals blindly.
  Office/server alpha views are almost black; UV game e6cb801f CANCELLED before launch (can't judge UVs).
  UV build a258c1f0 still queued; run offline --expect-fixed check after it. Source ambient trilight omitted,
  no source baked lightmaps, target compile --no-gi. Gta approved same-map GI-only office experiment:
  build47009645 -> capture1887cb7a, out/lighting-fix, behind GTA. Full notes/evidence in investigation record.

- Houdini vertex-colour loss traced through actual ForwardBase DXBC and saved office GLBs. Source/scene
  preserve COLOR_0; target builder ignores it unless lower-tint-reskate runs. Office has one affected Houdini
  primitive (two colours, zero mixed-corner triangles), but nine Standard + two Legacy primitives also carry
  nonwhite colours, so global tint lowering is wrong. Unity-reader notified of core semantics gap.
  Staged restricted program composition and fixture in tools/next-program; test job b5466286 follows UV
  build a258c1f0. Queued program/ remains unchanged. Evidence: out/office-color-semantics.json and
  notes/alpha-runtime-2026-10-18.md. No new runtime tint claim.

- User requested digging into missing objects/wrong textures after viewing old comparisons. Proven additional
  office UV loss: Desk surface material sharedassets1.assets:117 has base scale0.1/normal1; exported e87/e97 UVs
  remain unchanged after blanket transform clear. `out/office-uv-loss-proof.json` and investigation note detail it.
  gta authorized program-side base-first bake with explicitly counted secondary-slot losses, no blanket library
  clear. Implemented external `program/` composition + `program/src/uv_policy.rs`; cargo fmt passes, build/runtime
  pending jobs a258c1f0 / e6cb801f behind GTA and current alpha jobs. Output root out/uv-fix. Do not edit queued
  scripts or rebuild target/release/examples/unity_export.exe until frozen alpha rollout finishes.
  Playground trees are present in pre-bake GLB, skate-scene and normalized leaf mesh/material data; await corrected
  capture before attributing remaining disappearance. Custom gravel shader is explicitly base-only fallback.

- Successor continuation: gta explicitly authorized proving alpha using the private copy first, while main migration
  continues separately (supersedes main-first order for this experiment). Cherry-picked unity-reader 2013740c,
  resolving lower/drawn overlaps while retaining the existing Standard formulas and UI selection, as private
  commit 7b81a686d46e816e3526a84b2403e2ec12b9d4e3. Release exporter build passed.
  Rooftop export job 44e3c11e PASSED; one-look run 73cde79a PASSED the alpha restoration check:
  quarter-pipe face, brick wall and adjoining ramp now solid. Active target and screenshot retained at
  out/alpha-fix/run_level2; pair out/alpha-fix/compare_level2.png. Full parity/playability remain unverified.
  Remaining 7 exports/captures queued behind GTA (current IDs in out/alpha-fix/jobs.json).
  Export jobs use noinstall; only screen-locked captures stage the patch. A first queued batch was cancelled
  before starting to enforce that separation. Earlier f0a82025 reservation was cancelled before starting,
  replaced by heavy=20,screen 73cde79a so the visual proof could coexist with library gates.
  Outputs go to out/alpha-fix, preserving old artifacts. Scale, skyline, spawn and compiler flags unchanged.
  Earlier jobs 6829ba9c / 663318e0 were setup failures (CRLF shell / system Bash environment); fixed with LF and
  explicit Git Bash. 99623a63 was skipped; no game observations from those jobs.
- v21 9435a217 pushed to work remote gate/unity-v21. Lead's current landing stack includes it and alpha 2013740c;
  neither landed at last report. Main migration also needs private target-shaping/program APIs; do not mistake
  this private-copy runtime experiment for verification of main. No new library work in the private copy beyond
  the coordinator-authorized alpha cherry-pick.

- Machine reservation (latest Caleb/gta instruction): gta-city owns the machine until the city ZIP exists.
  Do not queue any heavy build/test/game jobs until gta explicitly confirms it is built. Offline code/reading
  may continue. Bird clippy follow-up 286ff626 cancelled before starting; requeue only after release notice.
- Recovery check: VERIFICATION.md and this checkpoint read back intact after the earlier rejected multi-operation
  patch (successful separate delete/add calls had already recovered it). Offline export runner fix:
  `tools/export_level.sh` now uses pipefail, terminating PowerShell invocation errors and explicit native exit
  propagation. Bash syntax check passes; a controlled native exit 7 through PowerShell/tee/tail aborts before
  the next command. No actual export/build/game was run for this check.
- Priority (Caleb via gta): GTA 0.3.0 precedes Bird in the job queue and shared owners' time
  (unity-reader, primase-lead, skate-blend). Keep Bird queued behind GTA; never promote it ahead.
  Continue independent offline work without requesting expedited shared-owner work.
- Latest verification rule (Caleb via gta): make it work, then correct, then fast. One focused check showing
  each fix works; land on fmt/clippy/unit/architecture. No additional censuses, reference runs or provenance
  chases unless results are ambiguous. Earlier census-coverage follow-up is not a prerequisite; owner informed.
  Existing evidence remains useful, but do not expand its collection as a separate workstream.
- Handoff: `C:/Users/kalmarv/modding/work/skatebird/BRIEF-codex-bird.md`. Library changes now go to
  primase main through primase-lead's work-remote gate branches, one logical commit per gate. No further
  library development in `ds-modding/primase`; retain it as the existing artifact reference until migrated.
  Program work stays here. Nothing else pushed/published. Never write Caleb's game install.
- Job 32be0cad finished (exit 0, 13m48s). Saved its log as `out/capture-job-32be0cad.log` and built/inspected
  all eight `out/compare_levelN.png` sheets. All levels reached active state, but **all eight fail visual parity**:
  missing/transparent ramps, walls and some floors. Idle screenshots do not establish rolling or collision.
  `docs/VERIFICATION.md` now gives honest per-level findings. Flashback source reference is present.
- Refreshed `docs/QA-report.md` with `python -X utf8 tools/qa_report.py`: all current marker coordinates match
  source at report precision. Earlier ceiling-height findings are obsolete. Initial plain-python command wrote
  the report but failed printing Unicode to cp1252; UTF-8 rerun completed successfully.
- Confirmed producer defect: private `lower.rs::blends` classifies opaque base + additive lighting as blended.
  UnityPy 1.25.4 audit `out/alpha-audit-level4.json` preserves source properties/pass states and input hashes.
  `CardboardBox_mat` (sharedassets1.assets:18) and `Black rubber` (sharedassets0.assets:341) resolve base One/Zero,
  additive One/One; export incorrectly blends them. Runtime repair remains unverified.
- unity-reader accepted alpha/base-pass correction on main; provided source paths and audit. No concurrent lower.rs edits.
  Follow-up: I own the SkateBIRD real-data alpha check after v21 lands. Raised a review concern with the proposed
  queue<=2500 override: opaque sorting / ZWrite does not itself disable blend factors. Requested a fixture retaining
  Blend for SrcAlpha/OneMinusSrcAlpha at queue2000 with ZWrite on; target approximation must not become source semantics.
  Unity owner revised this to a counted glTF limitation (only blending + depth-write + opaque queue becomes Opaque).
  Escalated the remaining shared-lowering policy to gta: recommend Blend by default with explicit target policy;
  depth writes do not remove transmitted background color, and existing opaque output is only a baseline.
  Base-pass selection and legacy keyword reading remain independently supported.
  FINAL gta decision (supersedes the initial no-override decision): land base-pass/keyword/pass-kind fixes while
  retaining main's Opaque behavior for the depth-writing blended opaque-queue edge case, explicitly counted as debt.
  Future switch to Blend must land together with a core depth-write fact and viewer/skate consumers. Unity owner
  sends core proposal. Owner reports zero affected SkateBIRD materials and 14 Sledding materials (13 SnowIce + water);
  Saved owner census inspected at `modding/work/primase/unity-reader/census/{skatebird,sledding}/opaque_range_blend.json`:
  0/1166 SkateBIRD, 14/448 Sledding material objects. This is a source-state census using the lowering's selection
  rule, not an independent semantic oracle. Resources/ excluded; inactive/unreferenced materials included.
  Recount corrects the usage summary: 7 affected materials referenced in Sledding level1, 9 in any file,
  256 level1 material-slot references (not deduplicated or activity-filtered renderers). Sent correction to owner.
  Requested skipped/unresolved shader/pass counts to qualify zero-case coverage; script currently omits them.
- Preparing v21 parser/header validation and registered independent census in main worktree
  `library-worktrees/unity-v21`, branch `gate/unity-v21`, rebased on a3ed317e, candidate commit **9435a217**.
  Job **1142f72e** passed all 3 header fixtures and independent census (31 files, 503045 objects), then failed
  clippy's manual_is_multiple_of lint in the fixture. Replaced the expression with is_multiple_of; formatting
  passes. Follow-up **286ff626** was cancelled while queued for the city machine reservation. Resume clippy only
  via `tools/check_unity_v21_clippy.ps1`, heavy=20, after gta explicitly releases the machine.
  Prior job eca340ac was cancelled before starting for the rebase. Source/script frozen for the follow-up.
  No push/landing yet.
  Lead has completion notification. After pass: rebase if main moved, send work-remote gate branch for full gates.
- Skyline: retain pending Caleb's decision, program exclusion must remain one option. Scale: gta clarified
  Caleb wants natural feature dimensions, not automatically 1:1. Measure rails/ledges/ramps/stairs/gaps, propose
  a uniform factor, then validate rooftop rider fit. Hold scale fixed during the alpha causality experiment.
- `tools/measure_features.py level2` recorded 339 named collider candidates in
  `out/feature-dimensions-level2.json`. They are AABB extents, not yet validated ledge/rail heights above ground.
  Rooftop ramp groups include 3.5 m vertical spans, with some 7/8.5 m spans; repeated names can combine multiple
  instances. No factor selected from this incomplete sample. Need per-instance feature/ground identification.
- `out/verification-audit-manifest.json` hashes 42 current log/reference/scene/map/screenshot inputs. It explicitly
  does not claim historical installed-package hashes. Keep the failed comparison baseline for the corrected run.
- Next: land v21 prerequisite; validate alpha correction in one rebuilt level, then all levels; measure scale;
  continue remaining material/raw-data, surface, grind and combined-map work. No release-ready claim.

The following is historical context; obsolete ownership, write/push restrictions, in-flight jobs and camera
parity claims below are superseded by this checkpoint, the current handoff, and `docs/VERIFICATION.md`.

Durable, verified facts (formats, tool gotchas, commands) live in **`KNOWLEDGE.md`**; the level-by-level proof
sheet is **`docs/VERIFICATION.md`**; the combined-map design is **`docs/COMBINED_MAP.md`**.

## Goal (Caleb, updated 2026-10-18)
**Phase 1: all SkateBIRD levels as ReSkate map(s).** Verified by matching in-game screenshots
(SkateBIRD vs ReSkate). No writes under `modding`, no pushes; primase additions allowed as local
commits. Work goes through primase readers and writers only.
**Phase 2 (after SkateBIRD is fully done): Riders Republic** - install through Steam if not installed,
then port its world(s) the same way through primase. Keep this file and `KNOWLEDGE.md` current; disk
cleanup is allowed **only inside `ds-modding`**.

## Phase 2 notes (Riders Republic)
- Steam appid **2290180**, **not installed** as of 2026-10-18; install it via `steam://install/2290180` when Phase 1
  is complete. It is a Ubisoft (AnvilNext) title, so expect a very different reader path than Unity.

## Hard constraints from Caleb (2026-10-18)
- **NEVER write, edit, create, delete, move or `git commit` anything under `C:/Users/kalmarv/modding`.**
  Read-only reference. Includes `modding/.orca/worktrees`.
- All work in `C:/Users/kalmarv/ds-modding` (own repo/workspace). Installs go there.
- Local commits allowed; **no `git push`** anywhere.
- primase may be extended here (own clone `ds-modding/primase`, local commits only).
- Keep `C:/Users/kalmarv/ds-modding/STATE.md` (this file) current: decisions, evidence, next step.
- Watch C: free space (118 GB free at start; Steam install + Rust targets cost GBs).
- Screen: Aerial (screensaver) covers the desktop; kill its window pid (`um win ps`, Aerial with a title)
  before screenshots. Use `um` (universal-modder) for launch/shot/drive.
- Helpers: use `orca spawn` with omp + coralbricks/deepseek-v4.1-flash-fast.

## Decisions
- D1. SkateBIRD is our target because it ships **loose Unity serialized files, Mono, Unity 2019.4.20f1**
  (header version 21, data offset 0x1000), no bundles/Addressables: the easiest possible Unity input.
- D2. Use primase's Unity reader (not the older UnityPy/Blender `reskate-ports/ports/unity` pipeline).
  That port stays a reference for surface tables, spawn/travel points and mod packaging only.
- D3. Target output = normalized ReSkate map folder (primase `worlds/reskate`) -> `reskate_cli compile-map`.
- D4 (Caleb, 2026-10-18): scale "whatever feels natural, easy to adjust"; combined map if the levels are
  small enough, otherwise split; **rooftop (level2) first**. Start at 1:1 for screenshot parity, expose a
  scale knob in the export composition, decide gameplay scale after the first in-game look.
- D5 (Caleb, 2026-10-18): goal = **all levels** as ReSkate map(s), verified by matching in-game
  screenshots SkateBIRD vs ReSkate.

## Verified so far (2026-10-18)
- Steam appid 971030 installed at `C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD` (3.2 GB).
- `SkateBIRD_Data`: `Managed/` (Mono), `globalgamemanagers` + `globalgamemanagers.assets`,
  `level0`..`level13` (+ `.resS`), `sharedassets0.assets` (43 MB) + `.resS` (744 MB), `resources.assets`,
  `StreamingAssets/`, `Plugins/`. No AssetBundles.
- Unity version string `2019.4.20f1` in globalgamemanagers/level0 headers; serialized format 21.
- primase Unity reader already does: SerializedFile + TPK type trees + .resS, hierarchy/world transforms,
  static batches, meshes (incl. compressed), DXT/BC textures, materials for **URP Lit/Unlit + SpeedTree7**,
  terrain, box/sphere/capsule/mesh/convex colliders -> core.collision, lights, RenderSettings sun.
- primase has a Unity -> skate path: `examples/unity_export <_Data> <tpk> <level> <out> --surfaces <map>`
  writes `out/skate/scene` (skate-scene v1) + assets; then `examples/build_map <scene> --out <dir>`
  (primase-reskate) writes the normalized map folder; then `reskate_cli compile-map <Skate folder> <map> <stage>`.
- `primase convert` CLI currently registers only the GTA reader; Unity is example-only (board: "register in
  primase convert once the convert contract is in core").
- ReSkate writer existing gates: native `<4800` style checks not yet read; Sledding port used
  `compile-map ... --stream --no-gi --time-of-day noon` + `install_mod.py` (copies `stage/Patch`, writes
  `.reskate-studio-patch` with skate.exe sha256, `manifest.json`, `reskate-levels.json`, `trainer.json`).

## Progress log
- 2026-10-18: primase clone at `ds-modding/primase` (main 7da1c6a), TPK copied to `ds-modding/data/typetrees.tpk`
  (covers 2019.4.20f1 exactly).
- primase patched locally (commit 3bfa13e):
  - `worlds/unity/src/serialized.rs`: SerializedFile **v21** (legacy header, metadata at 20, u32 object offsets).
    Gated against UnityPy: all 31 files, 503,045 objects, byte offsets/sizes exact
    (`worlds/unity/tests/skatebird_census.rs`, env `PRIMASE_UNITY_SKATEBIRD` + `_CENSUS`).
  - `worlds/unity/src/mesh.rs`: UV0/UV1 3-4 components -> xy.
  - `worlds/unity/src/level.rs`: non-finite material numbers -> 0, counted.
  - `formats/skate-scene/src/raise.rs`: textures named by their unique key (game names repeat).
  - `toolkit/meshops`: `clear-texture-transforms` (drops unbakeable tilings, counted).
- `unity_export level2 --surfaces`: 1816 renderers, 80 materials, 87 textures, 485 colliders, 17 lights;
  146,359 triangles; scene + assets written.
- `build_map`: 3 tiles, 421 placements, 63 materials, 59 textures, 0.09 M collision triangles.
- `reskate_cli compile-map` (Studio 20261012, `--no-gi --time-of-day noon`): **compiled** in 19 s.
  Warning: the map spans 11,322 m because of `Level Elements/Bounds` (city skyline + clouds backdrop at
  +-5.7 km). Studio still fits it in 726 cells; decide later whether to cut or re-scale the backdrop.
- Mod installed to the isolated install (`ds-modding/game`, Data junction, own Mods + LOCALAPPDATA) and skate.
  launched with `RESKATE_STARTUP_COMMANDS="load <asset>"`. **The rooftop level loads in skate. and the skater
  stands on the roof** (screenshot `shots/skate_now.png`; log: "Level destination ... @ DefaultStartPoint",
  "Starting level load"). Caleb's own install was restored (mod removed, mods.json backup restored) - its 55
  enabled mods make a black HUB screen, so tests use the isolated install.

- **Materials and lights (2026-10-18):** primase gained built-in Standard/Legacy lowering, the built-in sun rule,
  `unify-texture-wrap` and `pack-orm` (local commit c5d56b2). Rooftop re-exported with the game's spawn,
  16 travel markers and packed masks; compiled and loaded: skater spawns at the game's own (-18.7, 1.9, -84.8),
  sun and textures visible, the "Middle Roof" fast-travel point appears (screenshot `out/run_rooftop_2/shot_2.png`).

## State (2026-10-18, later)
- All 8 levels exported, compiled and installed into the isolated game: level1,2,3,4,5,7,12,13
  (`out/<level>/stage/Patch`, mods `SkateBIRD_<level>`). Level5 and level7 needed two fixes: lenient vertex
  decode for uninitialised static-batch channels, and dropping submeshes no material/variant draws (commit 669c6eb).
- SkateBIRD teleport works: BepInEx plugin in `skatebird-lab/` + `tools/skatebird_teleport.py` sets the skater's
  position and yaw exactly (verified read-back within 1 cm on rooftop).
- **Camera parity (blocker for pixel-level matching)**: the spawn yaw sets the *pose* (verified in both games) but not
  the view: skate's third-person camera starts at an arbitrary orbit and SkateBIRD's camera keeps its own bearing after
  a teleport (log: pose 172.93, camera 0 then 18.79 after a nudge). The helper adds `cam_yaw` to the SkateBIRD plugin
  so its reference view is the pose's heading; the skate camera can be turned with `scanmode on` + `rel` (only the
  first turn lands; the raw hook must stay on).
- **All 8 levels re-exported/compiled** with the exact spawn and overlay drop (levels 1-4 at 01:11-01:18, levels
  5/7/12/13 at ~01:35-01:46). Final in-game capture pass over the 8 maps is running (`tools/run_levels.sh 45 3`).
- Camera parity solved: SkateBIRD references use the plugin's `cam_yaw` = pose yaw (measured bearing == pose yaw on
  all 8 levels); the skate camera converges to the spawn marker's yaw. Confirmed yaw mapping **scene = 180 - unity**
  (bedroom comparison at 234.82 shows the same wooden platform/wall on the same sides).

## Fixes after the first QA pass (commit e45e60f)
- Spawn height: level3/4/7 started on the ceiling because the marker snapped to the topmost surface. `Spawn::AtWithHeight`
  now carries the game's y exactly; `--spawn` takes `x,y,z,yaw`.
- Interface overlays: mission markers (`Minimap/Mission Marker`) drew as opaque yellow spheres; `unity-drawn` drops
  placements whose materials are all on overlay shader families.
- Verified after the fix: level2 map spawn is exactly (-18.7, 1.189, -84.8) (no ceiling snap), 16 travel points.
- Backdrop spans 55-70x the collider bounds (skyline kept for screenshot fidelity). `--drop 'Level Elements/Bounds'`
  cuts it (level2: 11,322 m -> 248 m, 739 cells -> 0, 310k -> 284k compiled triangles) if perf ever matters.

## In flight (2026-10-18)
- Export loop for all 8 levels (`orca job` 6b4f9050, `tools/export_level.sh`), then `tools/run_levels.sh`
  screenshots each in the isolated install; SkateBIRD references in `out/skatebird_<level>.png`.
- Helpers: `sb-merge` (translate pass + `merge_scenes` example for a combined map), `sb-qa`
  (`docs/QA-report.md` over all exported levels), `sb-census-gate` (UnityPy decode-level oracle gate).

## Open questions / next steps
1. **Skyline backdrop**: `Level Elements/Bounds` (city skyline, clouds) reaches 5.7 km, so a compile spans 11.3 km
   (1430 WorldPartition bundles for one rooftop). Works, but consider cutting or re-rooting it.
2. **Reference screenshots**: capture SkateBIRD at each level's spawn and pair with the skate. run
   (`tools/skatebird_shot.sh`, `tools/compare_shots.py`). Blocked once by a Windows firewall prompt stealing
   foreground; work around it by targeting the game window, do not close system dialogs.
3. **Combined map**: all 8 levels are 100-300 m across; one map on a grid (spacing >= 1 km to cover the skyline
   radius) with the 100+ travel points needs a scene-offset step (no primase pass yet).
4. **Unmodelled shaders**: `Houdini/SpecularVertexColor`, `Shiny/Unlit Two Tone`, `Minimap/Mission Marker`,
   `Particles/Standard Unlit` still base colour only (the yellow goal spheres/domes in the rooftop shot).
5. **Grind/rails**: SkateBIRD's grind nodes are meshes; ReSkate's compile auto-joins collision edges, but no
   authored grind curves are exported.
6. **Scale**: 1:1 for screenshot parity; a gameplay-scale knob (`2-4x`) is a writer/scene option if wanted.
