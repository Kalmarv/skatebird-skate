# Alpha repair: one rooftop proof, then the remaining levels

Decision: do opaque ramp and wall faces return in skate. after Unity base-pass selection is repaired?
Coordinator explicitly authorized the private-copy experiment before main migration, to avoid blocking runtime proof.

Candidate: private primase `7b81a686d46e816e3526a84b2403e2ec12b9d4e3`, adapted cherry-pick of
unity-reader `2013740c`. Existing Standard/Legacy formulas and UI selection were retained during conflict resolution.
The patch also reads legacy keywords and pass kinds and shares base-pass selection with additive draw filtering;
this experiment tests that complete patch, not blend selection in isolation.

Control: old `out/level2` artifacts and `out/compare_level2.png`. Same installed source, scale, spawn, skyline,
surface table, Studio binary, noon/no-GI options and isolated game. New outputs: `out/alpha-fix/level2`.
Prediction: solid ramp faces replace the obvious gaps in the old view. Persistent gaps falsify the patch as a
sufficient repair; stop the all-level rollout and inspect that remaining failure. A launch without the requested
level active and a valid screenshot is a setup failure, not visual evidence.

Build job `44e3c11e` passed (4m40s). Game job `73cde79a` queued behind GTA with heavy=20,screen;
one 60-second settled screenshot, no movement or collision claim. Earlier setup-only jobs and cancelled game
reservations are documented in STATE.md; none counts as a game trial.

`python tools/alpha_result_manifest.py` records the source revision, scene/layout/log/screenshot SHA256s and counts
from the final scene files. Rooftop changed from 55 blend / 7 opaque / 3 mask entries to 26 / 36 / 3.
Black rubber, Bricks red smooth Mat and asphalt changed from blend to opaque. These are material-table entries,
not texture or renderer counts. Runtime verdict: PASS for restored rooftop opaque surfaces. Job `73cde79a`
finished in 1m22s, after 7m20s queued. Active target line at 08:42:38 local, screenshot contains the solid
quarter-pipe face, brick wall and adjacent ramp. Pair: `out/alpha-fix/compare_level2.png`.
Remaining seven exports/captures queued; `out/alpha-fix/jobs.json` is the current job list. The first set was
cancelled before starting to separate export staging from screen-owned installation (export now uses noinstall).

## Remaining mismatches: targeted source/artifact trace

- **Proven UV loss, independent of alpha.** Office material `sharedassets1.assets:117`,
  `hardwood vinyl sheet gross - pine`, used by `Desk surface`: source base texture scale `(0.1,0.1)`,
  normal `(1,1)`. Pre-bake GLB retains KHR_texture_transform scale `(0.1,0.1)`, offset `(0,0.9)`;
  exported prototype_e87 (24 UVs) and prototype_e97 (78 UVs) retain identical untransformed UV arrays.
  Target material has no per-slot transform. Thus the base repeats ten times as often along each axis.
  `python tools/prove_uv_loss.py` -> `out/office-uv-loss-proof.json`.
  Real producer: private `bake_texture_transforms` refuses mixed slot transforms, then program composition calls
  `clear_texture_transforms`, which erases them. Existing logs: bedroom 13 base + 6 normal transforms dropped;
  office 10 base + 4 normal; playground 6 base + 5 normal. Counts are bindings, not unique textures.
- **Trees are present in the exported data.** Playground pre-bake GLB contains 33 named tree placements;
  skate-scene prototypes include leaf `branch_tex` and trunk `Default` parts, and the normalized map retains
  leaf material `alpha_branch_tex_7761539246` and its mesh object record. Its PNG visibly contains green leaves.
  Both leaf and trunk were alpha-blended in the old scene; only leaf blending is supported by its source Fade
  material. The corrected playground capture is needed to separate the known alpha defect from remaining
  leaf rendering/culling issues. No claim that all trees were missing from extraction.
- **Custom shader effects omitted.** Playground `Kid Safe Gravel Asphalt` and `... Gross` use
  `Bumped Specular Voronoi`; the source has StackedStone3 base and StackedStone5 normal bindings. The lowering
  explicitly falls back to base colour only. This supports missing shader behavior, not a swapped-image claim.
  Houdini/SpecularVertexColor and Shiny/Unlit Two Tone are also reported unmodelled; their visual effects still
  need a targeted source/shader trace before implementing them.
- **Dynamic renderers outside current reader scope.** `scene.rs:499` selects MeshRenderer only. Stored scenes
  also have SkinnedMeshRenderer/ParticleSystemRenderer objects. Counts include inactive templates and cannot
  be used as a count of visibly missing objects. This is not the explanation for the static playground trees.

Raw source traces: `python tools/trace_visual_mismatches.py level3` / `level12` ->
`out/visual-source-level{3,12}.json` (UnityPy, stored refs including inactive objects).
No additional source game runs or new RE were needed for these artifact findings.

### Next fix, authorized by gta

External Cargo program `program/` now owns exporter composition (still referencing frozen private library
7b81a68 until main prerequisites land). `program/src/uv_policy.rs` preserves the base slot's UV transform,
aligns incompatible secondary slots with material/slot-specific approximation counts, invokes the baker,
and refuses remaining used-material transforms instead of erasing them. Target raise_materials.rs:74 requires
UV0 with no remaining transform; separate per-slot UVs are unavailable on this path.
Formatting passes; compile/export/runtime remain pending. Office build job `a258c1f0`, one dependent
office check `e6cb801f`, behind GTA and the frozen alpha rollout. The game job first checks the corrected
desk UVs against the saved source transform (max error <1e-5), then captures one view. No claim of a verified
UV repair until those jobs complete. Existing alpha exporter executable and output roots are unchanged.

### Vertex colour: second proven composition loss

Actual game shader `sharedassets0.assets:1207`, Houdini/SpecularVertexColor, ForwardBase/DIRECTIONAL:
VS passes COLOR0 to the pixel shader, which multiplies sampled base * material colour * vertex RGB.
Private disassembly and selection metadata: `research/shaders/Houdini_SpecularVertexColor/manifest.json`.
Selected VS program0 SHA256 `337da3887b79679dc50ede08d245283b0a01ce6a0499a9ae5b37cb118124e1e8`;
PS program16 SHA256 `5d577ec7a928b405965d9e003229a430fd5221a935b1b6896faf9a42ca073489`.
Earlier depth-only program34/36 files are not evidence for this claim. The actual ForwardBase tags select 0/16.
SideFX's current Houdini22.0 Legacy shader is a corroborating, differently versioned reference:
https://raw.githubusercontent.com/sideeffects/HoudiniEngineForUnity/Houdini22.0/Plugins/HoudiniEngineUnity/Shaders/Legacy/HoudiniSpecularVertexColorLegacy.shader

`python tools/audit_color_semantics.py` reads the saved office GLB and writes
`out/office-color-semantics.json`. Unit: mesh primitives, not placements. One Houdini primitive,
prototype_e1477 / generic snack mat, has 1829 vertices and two RGB values (brown and white), zero
triangles with different corner colours. Source and skate-scene GLBs preserve COLOR_0; final map mesh
builder omits it and the program did not run lower-tint-reskate. The target colour split can preserve
this pattern apart from byte tint quantization. No runtime improvement claimed yet.

The same audit finds nonwhite COLOR_0 on nine Standard and two Legacy Bumped Diffuse primitives.
Unity-reader confirms source colours are already preserved and the core glTF convention multiplies them;
representing shaders that ignore COLOR_0 is a separate library gap. Reported this concrete conflict to
unity-reader and gta. Do not enable target tint lowering globally in Bird.

Staged `tools/next-program/tint_policy.rs` selects verified Houdini materials for the existing target pass,
temporarily excludes other SkateMaterial components and restores them even on pass refusal; rejects
variants with mixed shader semantics. SkateParts carries no material references and follows the pass's
primitive remap. Place after raise-skate, before prepare_scene_assets. Frozen `program/` unchanged.
Fixture checks Houdini split, Standard preservation and SkateParts remap; compile/test job `b5466286`
depends on UV build a258c1f0. Formatting passed; compilation and runtime still pending. Cancelled
4a99a9cf before execution because its test-name filter would have selected no tests.

### Bedroom dark-wood normals and interior lighting

Caleb flagged possibly inverted normals and mismatched texture/normal scale. Main-table material
`hardwood gross - hawaiian koa` has base transform 0.1/offset(0,0.9), but the inherited Standard lowering
assigns the normal its saved identity transform. Unity-reader verified against exact Unity 2019.4.20f1
UnityStandardInput.cginc: main normal/metallic/occlusion/emission/parallax all use _MainTex coordinates;
detail maps use _DetailAlbedoMap coordinates. The own_tiling(_BumpMap) lowering is wrong. Owner is
implementing Standard on main with the koa regression case; no private library patch here.

`python tools/trace_bedroom_normal.py` -> `out/bedroom-normal-trace.json`: texture sharedassets0.assets:509
is DXT5nm usage3. Independent UnityPy decode versus exported normalized PNG: alpha->red mean byte error
0.00636, green->green 0.36305. Source GLB and scene RGB pixels identical. Final map downsamples1024->256;
against repeated box-filter reference, green error0.415 versus8.488 if inverted. This supports preserved
normal sign, not inversion. Native compiler tangent basis remains untested. Do not blindly invert green.

Office/server alpha captures 42cec41e/436f881f reached active targets but show nearly black interiors.
Cancelled UV office capture e6cb801f before execution because it cannot observe the UV change. UV build
a258c1f0 remains queued; run tools/prove_uv_loss.py with --expect-fixed when it finishes.

Offline lighting inventory `tools/trace_interior_lighting.py level3|level4` saves source-lighting JSON:
both AmbientMode1 (trilight), intensity1, sky(.5647,.5972,.6795), equator(.7835,.7835,.7835),
ground(.5457,.5457,.5457), nonzero ambient SH. No scene lightmaps; baked/realtime lightmaps disabled;
referenced LightProbes have zero baked coefficients. Office has two realtime spots6.58/range20 and sun;
bright-window emission2.425 is also absent in target. Server stored light counts include inactive parents,
so use exported17 punctual lights for target count, not all35 stored components. Ambient data is dropped
by UnityRenderSettings (sun/pipeline only), punctual intensity uses a URP physical-unit approximation
despite BuiltIn source, target emission omitted, and compile explicitly disables GI. Reported ambient
and intensity ownership gaps to unity-reader at gta's request.

Gta approved a single-variable lighting experiment: compile SAME alpha office normalized map with
--no-gi removed, all other options unchanged. Prediction: default Studio GI adds enough fill to inspect
surfaces; falsification: interior remains unreadably dark. Either outcome is not proof of faithful Unity
ambient conversion. Frozen scripts build_lit_office.ps1/run_lit_office.ps1, build47009645/capture1887cb7a,
out/lighting-fix. Same spawn, noon, compiler and map; expected compile+one60s observation, queue behind GTA.
Input hashes recorded; runtime must reach target active state and retain screenshot. No additional run
until this one is judged. Correct ambient data transport and BuiltIn-light lowering remain library work.

Owner update: Standard fix f5f80a88 and ambient-data989a5009 are queued for checks. Ambient resource v3
preserves mode, three colours and intensity; there is no core ambient lowering yet. BuiltIn punctual
conversion now reports its URP-model approximation explicitly. Neither commit is claimed landed or
runtime-proven. The GI-only experiment remains unchanged.

GI runner setup failures: 47009645 missing Get-FileHash before compiler; ec6c2aa5 terminated on compiler
stderr progress because Windows PowerShell ErrorActionPreference=Stop elevates redirected native stderr.
No game trials; dependent1887cb7a/a3c78b78 skipped. Hashing moved to tools/hash_inputs.py; compiler uses
Continue only during native invocation, then captures LASTEXITCODE and restores Stop. Native-process
fixture check_native_stderr.ps1 proved stderr/exit0 succeeds and stderr/exit7 remains failure (the initial
inline fixture had a Windows quote-stripping syntax error; fixed by using a .py fixture file).
Fresh experiment paths out/lighting-gi2; build e6863abb, capture42aa8f0c. Same source map/compiler/noon,
only --no-gi removed; no illumination parameters changed. Original failed output dirs retained.

e6863abb succeeded but default compile produced0 GI cells/probes and no Enlighten build messages.
Cancelled42aa8f0c before launch. Gta clarified BAM stripping warning is a separate axis from map GI;
explicit --enlighten approved next. b26f0559 compiled same alpha map/noon with --enlighten:
Building Enlighten probes/input systems;19536 probes in9 volumes,48 input systems,75908 clusters,
2656 inside geometry (compiler counts). GI-grid counters remain0: separate paths. Artifact gate:
ASCII strings in map defaultinstallpackage/cas_01.cas compared with alpha control add
levels/game/dingolevel_reskate_skatebird_level3/enlighten and .../lighting/enlighten.
Capture48b8245d now tests this output, out/lighting-enlighten. Only one game trial after nonzero gate.

UV build a258c1f0 compiled the external program but refused Metal pattern07 remaining transform.
Diagnostic72c2cbef showed a real variant group, not absent geometry: e395/0 is placed and has f32x2 UV0,
defaulte394 with variants1:e930,2:e1528,3:e1858,4:e1926. Program variants.rs resolves each used material
choice onto its own prototype before baking, reusing the original for one choice so no orphan collision
prototype remains; preserves placements and their chosen materials. It refuses unsupported LOD graphs.
Full-source diagnostic6c8c1d9c passed:3 prototype copies,34 placements resolved,542 coordinate sets baked,
44 slot transforms cleared; only2 slots unused by any primitive remain. Full office build3adf243a at
out/uv-fix-v2 includes the numerical desk UV gate before map compile. Houdini fixture jobbed86633
is now independent (original b5466286 skipped after UV failure). No runtime UV repair claimed yet.

Runtime48b8245d completed and saved active office log/screenshot. Stationary matched view versus alpha
control shows no useful visible brightening: floor/ramp interior remains nearly black. Prediction falsified
for explicit Enlighten under this configuration, not proof that every GI path fails. Figure gi_control_level3
contains two skate captures (left noGI/right Enlighten); existing montage utility labels are generic and
must not be read as source/target here. Next missing observation: actual probe illumination/coverage and
native sampling/overrides (BAM is a separate axis), or a proper source ambient-to-target contract. No
unchanged game retry scheduled; result sent to gta/unity-reader.

UV office3adf243a PASS: export, numerical UV comparison, map build and native compile. Actual desk proof
out/uv-fix-v2/level3/uv-proof.json compares both e87/e97 against pre-bake source transform; max error below
1e-5 gate. Runtime appearance remains unverified. Tint fixturebed86633 PASS (one test): selected Houdini
split, Standard unchanged, SkateParts remapped. Candidate remains staged separately.

Gta authorized one final lighting axis: same map/compiler/noon/Enlighten as b26f0559, adding only
--no-bam-lighting. Buildd0c646d5 ->capture0463240c, out/lighting-no-bam. Prediction: if inherited BAM
lighting suppresses this map's bake, interior becomes readable. If no visible change, stop lighting trials
and await ambient contract work after GTA0.3.0. Unity-reader/primase-core agreed Environment.ambient as
Flat or Gradient{sky,equator,ground}, linear; Unity producer and target consumer ownership established.

No-BAM compile d0c646d5 passed, capture0463240c started. Validity caveat: compiler still prints the exact
BAM Lighting_Global/live-ToD/without-Enlighten warning despite literal --no-bam-lighting in the command.
SHA256 comparison against b26f0559 patch:4 files identical,6 changed; reskate-build.json only changes
timestamp. Differences alone do not establish BAM exclusion. A dark result cannot rule out the override
hypothesis unless the target graph is checked. Notified gta; no additional experiment queued.

Capture0463240c finished exit1 at the late Park active-marker assertion. Saved log has requested
runtime map, ready marker and bound skater; saved shot shows the office. Reviewed labelled control
montage out/lighting-no-bam/gi_control_level3.png: still nearly black, no useful brightening.
Treat the assertion failure separately from this visual observation. Effective no-BAM remains
unproven because compile warning persists; no further lighting run scheduled. Recorded log/shot/
montage SHA256 via tools/hash_inputs.py in lighting-no-bam/level3/result-hashes.json.
Regenerated both GI control montages with compare_shots.py --labels to identify actual controls.
Unity-reader reports f5f80a88 and989a5009 passed checks and are with lead, not yet confirmed main.

## Offline canopy and custom shader follow-up

Lighting trials stopped by gta. Read-only compiled section census5e51f5f4 uses existing
skate-vcolor/py/seccensus.py and reskate-ports fbpatch; output out/playground-sections.json.
tools/audit_playground.py joins the normalized leaf object (objects[246], t_0_0 / section39,
meshes/t_0_0.glb primitive38) to matching compiled MeshSet names. Input has7428 vertices,
3714 triangles and UV0 bounds0..1. Leaf PNG256x256 has17732 nonzero-alpha pixels and12205
at alpha>=128 out of65536. The compiled census contains295 section/LOD rows under matching
MeshSet names (NOT295 trees or leaf sections: compiled meshes also contain other material keys).
Keys are fd9936984b8d4d4c,63042519b4d6aae3,6001dbf81a98ccf6. Depot/program linkage pending.

Sledding reference correction from unity-reader: its own Sledding work reached web tiles only,
not a skate mod. Its opaque low-poly SpeedTree7 Evergreens are not a leaf-card control. It cannot
confirm Caleb's installed Sledding mod's lineage. Main's planned alpha path agrees with Bird:
Standard Fade is Blend, fixed Cull Back is single-sided, Standard has no foliage usage rule.
Therefore no evidence yet to force Bird leaves to cutout or double-sided. Fixed Cull Off handling
for other shaders is a separate owner gap, not this Standard material's cause.

Gravel shader extraction43b304a4 passed: sharedassets0.assets:1196, Bumped Specular Voronoi,
FORWARD pass. Exact DXBC hashes and disassemblies in research/shaders/Bumped Specular Voronoi.
VS program0 emits two transformed UV pairs. PS program8 uses nested -1..1 loops, sine/fract
cell hash, exp2(-7.213475*distance_squared) weights, derivative sampling and normalization,
separately for colour and packed normal; then reconstructs normal Z. It is not ordinary base sampling.
Source Gravel main tiling50,10 vs bump10,2; Gross main10,3 vs bump10,2. These differing slots are
real shader behavior, unlike Standard's ignored bump ST. Existing lower.rs Family::Other intentionally
keeps base only, dropping procedural sampling and normal. Source grass uses this shader too.
No shader approximation or runtime fix claimed. JSON/text census in out/playground-leaf-audit.*.

Depot trace d7cbcdfc completed with tools/audit_leaf_depots.py (existing fbpatch/fbebx/sbdfmt/blk
readers, no new CAS parser). Leaf key fd9936984b8d4d4c binds branch_tex basecolor4 and osk5 plus
flat normal; its registered blocks are byte-identical across the sampled patch. Program lookup
is (1444,3612), the stock decal/blend family identified in research-blendshader/REPORT.md.
Adjacent key63042519b4d6aae3 binds brown flat817354 and ordinary surface programs682,819,1474,2756.
Key6001dbf81a98ccf6 has a program row but no depot here; it also occurs in the working Sledding
meshes, so its absent block alone is not a canopy diagnosis. Compiled OSK pixels and runtime
front/back exposure have not been inspected. Do not equate binding presence with correct opacity.

Caleb identified the actual earlier Sledding skate pipeline under work/sledding/lab. Audited its
v2/out/manifest.json and v2/stage/Patch, not unity-reader's unrelated main/web export.
tools/audit_sledding_reference.py -> out/sledding-reference-audit.json: Tree_Mat_4f81a4 is opaque,
single-sided SpeedTree7; tree texture32x32 has1024/1024 pixels alpha255. Manifest has130 opaque,
42 clip and8 blend material records. Release Kalmarv-Sledding_Game_Mountain-0.2.1.zip layout.toc
and sledding level TOC SHA256 match stage (only those two files checked, not whole-package identity).
Compiled section census249257a9 and depot trace658c9d6e:
- Tree key eb6d7639ba8a93a4 binds tree basecolor/MSK/normal, programs682,819,1474,2756.
- Glowing Eyes clip key3f16830dd1881429 and Glowing Statue clip key68b7652171d077a1 bind their
  basecolor/OSK maps, programs825,1891,3096. Thus the reference has a compiled cutout path distinct
  from Bird's blend path; those are not evidence that blended leaf cards render.
Earlier build_blend.py render_material links texture Alpha directly to Principled Alpha, sets
alpha_source base_color_texture, mask for clip or blend for blend, cutoff from source, double-sided
from source; no foliage domain override. compile.sh calls Blender then Studio directly; no postpass
in that script. Bird's normalized settings already express blend/base_color_texture/single-sided.

Shiny/Unlit Two Tone extractiondd3a61bf succeeded (sharedassets0.assets:1194, FORWARD).
Pixel program8 computes view-vector/normal dot and light-vector/normal dot, a power-based edge
term and interpolation of two constant colours, and samples reflection cubemaps. It is not a
constant unlit colour despite its name. Current fallback loses this view/light-dependent shading.
Private disassemblies/manifests stored in research/shaders/Shiny_Unlit Two Tone. No speculative
shader implementation added; faithful gravel texture baking needs UV-domain sampling (not merely
changing its normal texture), and Shiny cannot be faithfully collapsed to one constant base colour.
