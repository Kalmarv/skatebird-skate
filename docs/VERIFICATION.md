# SkateBIRD -> skate. verification

## Playground canopy investigation (offline, 2026-10-18)

Leaf geometry and its alpha texture survive normalized export. Compiled MeshSets retain the matching
object; material key `fd9936984b8d4d4c` resolves to leaf base-colour/OSK textures and stock blend programs
`(1444, 3612)`. This rules out simple omission of the named compiled object, not incorrect compiled
opacity or runtime culling. See `out/playground-leaf-audit.json` and `out/playground-leaf-depots.json`.

The earlier shipped Sledding pipeline's trees are opaque/single-sided with an entirely opaque 32x32
texture; they use ordinary surface programs. Its cutout eyes/statues instead use `(825, 1891, 3096)`.
That reference distinguishes the paths but does not prove blended canopy support. No canopy fix or
new runtime pass claimed. Source DXBC also confirms omitted procedural gravel sampling and
view/light-dependent Shiny Two Tone shading; details are in the existing investigation note.

## Final office lighting observation (2026-10-18)

Build `d0c646d5` added only `--no-bam-lighting` to the explicit Enlighten control.
Capture `0463240c` exited 1 because the later Park active-state marker was absent at shutdown.
It did save the office screenshot; the log records the exact requested level, ready marker and
bound skater at (-34.0, 12.9, 78.4). This is a captured target view with a failed runner assertion,
not a successful execution of the original gate.

The [labelled comparison](../out/lighting-no-bam/gi_control_level3.png) shows no useful interior
brightening. The compiler still printed its BAM lighting warning despite the option, so BAM exclusion
is unproven and this does not rule out an override. Log/screenshot/comparison identities are in
`out/lighting-no-bam/level3/result-hashes.json`. Stop further lighting trials pending the ambient
producer/core/target contract. UV correction remains numerically verified, not visually verified.

## Alpha repair continuation (2026-10-18)

Rooftop was rebuilt successfully by job `44e3c11e` from private primase
`7b81a686d46e816e3526a84b2403e2ec12b9d4e3` (adapted unity-reader alpha/base-pass fix `2013740c`).
Gta authorized this private-copy proof before completing the main migration. **Rooftop alpha repair PASS**:
job `73cde79a` reached the requested level's active state and captured the restored solid quarter-pipe face,
brick wall and adjoining ramp. See [corrected rooftop pair](../out/alpha-fix/compare_level2.png) and
[runtime log](../out/alpha-fix/run_level2/ReSkate.log). This verifies the private patched build, not main.
Full visual parity still fails on lighting/material appearance; no scripted rolling, grind or travel test was performed.
All eight alpha-build exports and captures completed; IDs are in `out/alpha-fix/jobs.json` (rooftop above).
The artifact manifest contains scene, compiled layout, runtime log and screenshot hashes for every level;
all eight runtime logs confirm their requested target active. Caleb confirmed he was playing during launches:
camera/player movement is user input, not a harness defect or an automated gameplay verdict. These captures
can assess visible rendering, but are not exact-pose image comparisons.

Rollout results reviewed so far:

| Level | Build / capture jobs | Observed result |
|---|---|---|
| level1 bedroom | c8b134a7 / d70abe84 | Alpha restoration PASS in spawn view: solid floor, wooden supports and ramp faces replace the old see-through surfaces. Wall/floor material appearance and lighting still differ. [Pair](../out/alpha-fix/compare_level1.png). Active level confirmed in saved ReSkate.log; no movement test. |
| level2 rooftop | 44e3c11e / 73cde79a | Alpha restoration PASS, as above. |
| level3 office | 5e2100b3 / 42cec41e | Target active, surfaces visible as silhouettes, but interior almost black. Lighting FAIL; view cannot judge UV/normal appearance. [Pair](../out/alpha-fix/compare_level3.png). |
| level4 server room | 667c7b8d / 436f881f | Target active; interior almost black. Lighting FAIL; view cannot establish full alpha restoration. [Pair](../out/alpha-fix/compare_level4.png). |
| level5 bedroom 2 | 4ecf15df / d21b3d5d | Solid ramp faces visible; bed surface has severe material/lighting mismatch. Full parity FAIL. [Pair](../out/alpha-fix/compare_level5.png). |
| level7 flashback | 3a014cdd / c2d0da8e | Solid floor/wall/ramp surfaces visible. Blue ramp normal detail and strong lighting differ. Full parity FAIL. [Pair](../out/alpha-fix/compare_level7.png). |
| level12 playground | 30c9732c / d0db210a | Ramps, slides, floor and tree trunks visible, but green tree canopies remain largely absent. Gravel tiling differs. Leaf geometry/material/alpha pixels exist in normalized export; disappearance remains unresolved downstream. Full parity FAIL. [Pair](../out/alpha-fix/compare_level12.png). |
| level13 skate heaven | 8b611bae / ec1e2a62 | Solid stone floor/wall and white ramp visible; camera/player moved during capture, so correspondence is limited. Full parity unproven. [Pair](../out/alpha-fix/compare_level13.png). |

The program-side office UV correction and restricted Houdini vertex-tint composition remain unverified
in game. Investigation and exact artifact evidence: [alpha/runtime notes](../notes/alpha-runtime-2026-10-18.md).
Office UV capture e6cb801f cancelled before launch: the alpha control is too dark to judge the intended
effect. Its build remains queued; inspect source lighting and select a valid lit view before the next capture.

Explicit Enlighten experiment: build `b26f0559` generated 19,536 probes and map-specific Enlighten assets;
capture `48b8245d` reached the office active state with a stationary view. **Lighting still FAIL**: interior
remains almost black, with no useful visible improvement over the alpha/no-GI control. See
[source comparison](../out/lighting-enlighten/compare_level3.png). The file
`out/lighting-enlighten/gi_control_level3.png` compares two skate captures (no-GI left, Enlighten right);
its generic montage labels must not be interpreted as source-game labels. No unchanged rerun queued.

Corrected artifacts are under `out/alpha-fix`; the old artifacts and comparisons below are retained as controls.
`python tools/alpha_result_manifest.py` counts skate-scene material entries and records artifact hashes.
Rooftop changes from 55 blend / 7 opaque / 3 mask to 26 blend / 36 opaque / 3 mask. Black rubber,
Bricks red smooth Mat and asphalt now export opaque. Scale, spawn, skyline and compile settings are unchanged.
The patch also adds legacy keyword reading and changes additive draw filtering, so this tests the complete patch.

## Prior failed baseline

Audit of capture job **32be0cad**, 2026-10-18. All eight requested levels reached active state in its
saved log (`out/capture-job-32be0cad.log`). **None passes visual parity; playability is unverified.**
The run only launched and captured idle views; it did not exercise rolling, grinds or travel transitions.
No new game launches were needed for this audit.

Each linked sheet shows SkateBIRD on the left and skate. on the right. Source paths and pose/camera
readbacks are in `out/skatebird_spawn_refs.json`; target inputs are `out/run_levelN/shot_2.png`.

| level | scene | comparison | visual verdict and observed defects |
|---|---|---|---|
| level1 | bedroom 1 | [pair](../out/compare_level1.png) | FAIL: ramps and wall-facing surfaces transparent/absent; floor and ramp textures differ strongly. Wooden platform recognizable. |
| level2 | rooftop | [pair](../out/compare_level2.png) | FAIL: surrounding ramp/wall surfaces missing or transparent. Roof floor visible; sky/lighting differ. Camera correspondence not exact. |
| level3 | office | [pair](../out/compare_level3.png) | FAIL: opaque quarter pipes and enclosing room surfaces absent/transparent; isolated trim remains. Floor visible; distant surfaces bright magenta. |
| level4 | server room | [pair](../out/compare_level4.png) | FAIL: source floor, walls and ramp/cabinet faces absent from target view; skater appears over void beneath floating geometry. Still image does not prove collision loss. |
| level5 | bedroom 2 | [pair](../out/compare_level5.png) | FAIL: bed/floor and surrounding ramps transparent/absent; exterior and lower geometry visible through them. Poster and pictures recognizable. |
| level7 | flashback | [pair](../out/compare_level7.png) | FAIL: room walls and major ramps absent; floor and portions of props remain. Source flashback reference exists (old table incorrectly marked missing). |
| level12 | playground | [pair](../out/compare_level12.png) | FAIL: solid blue play structure and slide surfaces largely absent/transparent; source trees not visible in target view; ground remains. |
| level13 | skate heaven | [pair](../out/compare_level13.png) | FAIL: central stone floor, walls and ramps absent from target view; skater appears over void, isolated geometry remains. Collision behavior untested. |

## Validity and limits

- Captures came from `tools/run_levels.sh 40 2`, not the previously documented 45-second run.
  Job duration was 13m48s. Zero exit is execution evidence only; the runner does not enforce a visual verdict.
- Source camera was locked to recorded source yaw. Target yaw was configured in the spawn marker,
  but no target camera position/FOV/bearing readback was saved. These are qualitative comparisons,
  not pixel-matched proofs. They do identify the large visible defects above.
- `python -X utf8 tools/qa_report.py` reread current map artifacts: all eight exported spawn coordinates
  match source data to the report's 0.001 m precision. This supersedes the old ceiling-height findings.
  Runtime skater positions include engine offsets; do not equate them to marker coordinates.
- Original runner overwrote the game log between launches. Orca job log preserves per-run excerpts
  including each requested target's active-state line. Current artifact hashes cannot retroactively
  establish the installed package's original hash.

## First causal finding

`python tools/audit_material_alpha.py level4` reads installed source with **UnityPy 1.25.4**, saving
material properties/subshader states plus export label matches to `out/alpha-audit-level4.json`.
Source scope: 267 unique material references from all level4 MeshRenderers, including inactive ones.
Current skate-scene table: 85 entries, 71 blend, 2 mask, 12 opaque (not normalized-map counts).

`sharedassets1.assets`, material path ID 18, `CardboardBox_mat` uses Standard with `_Mode=0`,
`_SrcBlend=1`, `_DstBlend=0`, `_ZWrite=1`. Stored FORWARDBASE resolves One/Zero; FORWARDADD is additive.
Export incorrectly calls this material `class=alpha, alpha=blend`. The private fork's
`worlds/unity/src/lower.rs::blends` treats any pass as evidence for surface blending.
This proves misclassification; it does not yet prove correcting it restores every missing surface.
The Unity owner is preparing a main-ready base-pass fix. No fix was made in the private library copy.

Next decisive experiment: after the producer fix lands, rebuild one affected level retaining geometry,
scale, spawn and lighting; verify source-opaque materials export opaque, then capture the same view.
Recovered ramps/floor would support the alpha explanation; persistent missing surfaces require separate
geometry/culling/collision investigation. Keep skyline and scale fixed for this experiment.

## Remaining release gates

All levels need corrected visual captures plus rolling/collision and travel checks. Material formulas/tiling,
authored grind curves, gameplay-scale measurements and combined map remain open. Gameplay scale is not assumed
1:1: measure feature dimensions and propose a uniform factor before the rooftop rider-fit check. Retain skyline
until Caleb decides; expose exclusion as a program option.
