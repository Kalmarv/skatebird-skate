# SkateBIRD -> skate. map port: plan

Status: **M0 complete, M1 in progress** (2026-10-18). Pipeline: `SkateBIRD_Data -> primase worlds/unity -> primase worlds/reskate -> skate-scene -> normalized map -> reskate_cli compile-map -> ReSkate mod`.

## What SkateBIRD is (verified from the install)

- Unity **2019.4.20f1**, Mono, built-in render pipeline, container **SerializedFile v21**, no
  anti-cheat, **no AssetBundles/Addressables** (loose `levelN` + `sharedassetsN.assets[.resS]`).
- 14 scenes; the skateable ones: `level1` bedroom 1, `level2` rooftop, `level3` office,
  `level4` server room, `level5` bedroom 2, `level7` flashback, `level12` playground,
  `level13` skate heaven.
- Geometry: ProBuilder / static-batch meshes, **no LODs, no terrain, no lightmaps**, 17 lights on
  the rooftop, ~500-4,000 MeshColliders per level, PhysX colliders for the rest. UV0 is often
  **4 components**; some meshes carry no tangents.
- Shaders: built-in **Standard**, **Standard (Specular setup)**, **Legacy Shaders/Bumped
  Diffuse/Specular/Transparent**, Houdini/SpecularVertexColor, Unlit Two Tone, particles. Shader
  objects ship in `sharedassets0` (so names resolve; they are not stripped builtins).
- 380 materials, 447 Texture2D + 2 cubemaps in `sharedassets0`.

## Level sizes (UnityPy measurement, metres; meshes over 100 m dropped as backdrops)

| level | scene | extent (x y z) |
|---|---|---|
| 1 | bedroom 1 | 280 x 400 x 255 |
| 2 | rooftop | 247 x 392 x 252 |
| 3 | office | 275 x 391 x 280 |
| 4 | server room | 198 x 382 x 194 |
| 5 | bedroom 2 | 288 x 380 x 265 |
| 7 | flashback | 374 x 364 x 416 |
| 12 | playground | 357 x 407 x 350 |
| 13 | skate heaven | 266 x 544 x 317 |

All eight are **small** (<= ~400 m across, ~150k triangles each), so a combined map is cheap:
8 levels spaced ~600 m on a grid = ~1.2 M triangles, well inside Studio's compile budget (73 M
triangles at 31 GB) and skate's 9.6 km root.

Plan: **per-level maps first** (rooftop as the proving level), then a combined
"SkateBIRD - All Levels" map with travel points as the headline release.

## primase work needed (all local commits in `ds-modding/primase`)

Done (M0, compiles + exports rooftop):
1. `worlds/unity/src/serialized.rs`: **container version 21** header, metadata-at-20 layout,
   32-bit object offsets, `looks_like`. Gate: UnityPy census (pending test).
2. `worlds/unity/src/mesh.rs`: UV0/UV1 with 3-4 components use xy (zw unused by any texture).
3. `worlds/unity/src/level.rs`: non-finite material numbers (SkateBIRD `_CameraFadeParams` g=inf)
   become 0 with a counted note.

Next (M1):
4. `--surfaces` map for SkateBIRD's PhysicMaterials -> skate surface names; check what
   PhysicMaterials exist per level.
5. Feed the skate scene through `build_map` -> `reskate_cli compile-map` -> install -> load in
   skate. This is the first end-to-end gate.
6. Fix whatever the ReSkate writer/compile rejects (collision surfaces, min_height, tiles).

Then (M2 materials):
7. Add **built-in Standard** and **Legacy Bumped/Specular** families to
   `worlds/unity/src/lower.rs`: `_MainTex`/`_Color`, `_BumpMap`, `_MetallicGlossMap`
   (R metallic, A smoothness) or `_Glossiness`/`_SpecularColor`, `_OcclusionMap`, `_EmissionMap`
   + `_EmissionColor`, cutout/transparent modes from keywords. Without this every material is
   base colour only (flat lighting, no normal maps).

Then (M3+):
8. Lights: the reader marks "no sun: none named and pipeline is not URP"; SkateBIRD's directional
   light is RenderSettings `m_Sun`? verify and lower it (needs a non-URP rule).
9. Spawn/teleports: spawn at the level's player start, teleport markers at landmarks.
10. Scale-out: all 8 levels, texture budget per map, packaging (`install_mod.py` shape).

## Risks / open questions

- **Tangents**: meshes with UV0 4D may lack tangents; skate's native decl wants TangentSpace.
  Mitigation: compute tangents in the writer/pass (mesh already has normals + UV).
- **Static batched meshes** ("Combined Mesh (root: scene)") can carry bad AABBs at huge
  coordinates; they must be kept but their outliers excluded from spawn/framing.
- **Skate surface table**: SkateBIRD has few PhysicMaterials; default mapping and water/glass
  decisions need a decision.
- **Level scale**: SkateBIRD's bird/board scale vs skate's human scale; likely keep 1:1 (the game
  is metres) but the skateable bowls/rails will be tiny for a human skater. Decide: 1:1 faithful
  vs scaled up 2-4x for skate gameplay.
- **Collision coverage** on meshes that are render-only (some props) - accept or drop.

## Verification

- M0: export log counts vs UnityPy census; GLB re-imported/rendered.
- M1: map loads in skate. (screenshot), skater can roll on each level, spawn on ground.
- M2: material screenshots vs SkateBIRD screenshots (paired shots are the oracle).
