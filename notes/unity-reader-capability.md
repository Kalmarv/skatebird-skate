# primase Unity reader: capability, inputs, gaps

Scope: `ds-modding/primase` at `7da1c6a` (identical to the `modding/primase` checkout; `diff -rq` of
`worlds/unity` is empty). Crate `worlds/unity` (`primase-unity`). Evidence is source line references plus the
Sledding real-data gates in `work/primase/unity-reader/` (read-only reference).

**Bottom line.** The reader ingests a *loose* Unity player data folder (`<Game>_Data` + `_Data/Resources`): strict
SerializedFile v22 only, decoded through an external TPK type-tree package, with `.resS`/`.resource` streaming.
It produces a `World` (hierarchy, meshes, textures, materials, lights, terrain, colliders); lowering to
`pbr.material`, geometry, `core.collider`, `core.light` and `core.environment` is done by passes in the example
`unity_export`. AssetBundles/Addressables, embedded type trees, IL2CPP script fields, shader evaluation and
skinning are not supported. Only one game has been proven: Sledding Game, Unity 6000.3.12f1, container v22, URP.

## 1. Entry points and pipeline

- `worlds/unity/examples/unity_export.rs` is the only end-to-end composition; `primase convert` does **not** have a
  Unity reader yet: `cli/src/convert.rs:39` registers only `("gta", ...)`.
- Required lowering order (exactly what the example runs): `read_level` → `resolve-state` → `select-lod{form=finest}`
  → `unity-drawn` → `lower-unity-pbr` → `lower-unity-terrain` → `lower-unity-collision` → `lower-unity-lights` →
  `normalize-normals` → `tessellate-colliders` → `bake-nonrigid` (`examples/unity_export.rs:74-92`).
- Writers: glTF always (`:109`), web tiles with `--web` (`:123`), skate scene with `--surfaces` (`:160-172`).
- The reader itself takes data only: stored "off" flags become Bool axes (`unity.active.<file>.<pid>`,
  `unity.enabled.<file>.<pid>`), LODGroups become `core.lods` forms or a `unity.lod` axis, and procedural/additive
  filtering is the `unity-drawn` pass (`worlds/unity/src/level.rs:10-28`, `src/drawn.rs:1-24`).
- Unity frame → world: left-handed `(x, y, z)` becomes right-handed `(x, y, -z)`; triangle winding is reversed and
  tangent `w` negated (`src/level.rs:58`, `src/mesh.rs:407-413`).

## 2. SerializedFile reading (`src/serialized.rs`)

- Container versions read: **22 only** — `pub const VERSIONS: 22..=22` (`serialized.rs:21`). No other version is
  accepted; `looks_like` additionally requires ≥48 bytes, first 4 bytes zero, version 22 at offset 8 (`:122-126`).
- Header: big-endian legacy `u32 metadata`, `u32 file size`, `u32 version`, `u32 data offset`, endianness byte + 3
  reserved, then `u32 metadata size`, `u64 file size`, `u64 data offset`, `u64 reserved` (`:128-150`). Metadata is read
  in the file's own endianness; both endiannesses are supported in principle (`:159`).
- Strictness: header file size must equal the actual length; metadata end ≤ data offset ≤ file size; metadata must be
  consumed exactly; duplicate path ids are refused; every object's `(offset, size)` is bounds-checked against the file
  (`:151-160`, `:180-200`, `:216-226`, `:232-245`).
- Metadata read: Unity version string, target platform, embedded-type-tree flag, type table (class id, stripped,
  script type index, script id hash, old type hash), object table (`path id`, `start`, `size`, `type index`,
  aligned to 4), script references, externals (guid/kind/path), reference types, user information
  (`:159-214`).
- **Embedded type trees are refused** with a pointer to use a TPK (`:164-167`).
- Object decode: `decode_with` decodes bytes from the object's start and returns the unconsumed byte count
  (`:255-266`); `decode` additionally requires the tree to cover the object, except class 114 MonoBehaviour
  (`:65-69`, `:268-279`).
- MonoBehaviour: only the base fields are decoded; the script's serialized fields are returned as raw bytes
  (`player.rs:146-158`) because a stripped build has no script type trees. Player builds of Sledding have 16,080 such
  objects / 7,477,440 bytes (`work/primase/unity-reader/baseline-tests.log:79`).
- PPtr resolution: `m_FileID == 0` is the same file; otherwise `externals[fileID - 1].path`'s basename is lowercased
  and used as the target file key; a missing external file fails with `no serialized file ...`
  (`player.rs:238-269`, keys lowercased in `player.rs:82-85`).

## 3. Type trees and TPK packages

Decode rules (`src/typetree.rs:1-9`):

- primitives by type name (`bool`, `SInt8/UInt8/char`, `short/SInt16`, `UInt16`, `int/SInt32`,
  `UInt32/Type*`, `SInt64`, `UInt64/FileSize`, `float`, `double`) (`typetree.rs:40-74`);
- `string`: 32-bit length, UTF-8 bytes, align to 4; `TypelessData`: 32-bit length + bytes; `pair`: two children;
- a node whose **first child is `Array`** is a vector: 32-bit count then elements by the array's second child
  (`typetree.rs:126-139`, `:178-184`);
- everything else is a struct of children in order (`:191-197`);
- `ReferencedObject` (`[SerializeReference]`) is **refused**: it needs the file's own reference type trees
  (`:185-189`);
- meta flag `ALIGN = 0x4000` aligns to 4 bytes after the node and for aligned vectors (`:15`, `:199-202`);
- depth limits: TPK tree build 64 (`tpk.rs:258-261`), value recursion 128 (`typetree.rs:153-156`);
- every child/string index is validated when the tree is built or the package parsed (`typetree.rs:96-110`,
  `tpk.rs:204-218`).

TPK package (`src/tpk.rs`):

- magic `0x2A4B5054` ("TPK*"), format versions 1 and 2, data type 0 (type tree information) only
  (`tpk.rs:13`, `:66-83`); compression 0 = uncompressed, 1 = LZ4; **LZMA and Brotli are refused** with "re-pack them
  uncompressed" (`:75-101`).
- Blob: created time, version list, per-class entries (id, per-version `(unity version, class)`; class name string,
  base, flags; editor root skipped when flag 64, release root used when flag 128), common-string tables skipped,
  node table (`type name`, `name`, `byte_size`, `version`, meta flag, children), varint string table; trailing bytes
  refused (`:115-201`).
- Lookup: for class id + file Unity version, the newest listed version **≤** the file's version (`:220-228`), then its
  **release root** tree; a missing class or a class without a release tree is an error (`:230-250`).
  Consequence: a package older than the game silently uses the nearest older layout, producing either wrong values or
  the "N bytes after its type tree" error. There is no explicit "Unity version not in package" warning.

Where the TPK comes from (`UnityPy` 1.25.4 / `tpk_ar` 0.2.4 in `work/primase/pm-core/unitypy-venv`):

- The package is Unity's per-version class-layout database produced by the **AssetRipper/Tpk** project
  (`https://github.com/AssetRipper/Tpk`, workflow "Type Tree Tpk", weekly artifacts `uncompressed_file`, `lz4_file`,
  `lzma_file`, `brotli_file`; last successful run 2026-10-18). UnityPy vendors a copy at
  `.../Lib/site-packages/UnityPy/resources/lzma.tpk`; `tpk_ar.utils.download_tpk` fetches the nightly through
  `nightly.link` (`tpk_ar/utils.py:10`).
- Verified: `work/primase/pm-core/w9/typetrees.tpk` is that UnityPy file decompressed from LZMA and re-headered
  uncompressed — its blob is byte-identical to `decompress_lzma(UnityPy/resources/lzma.tpk)` (1,538,989 bytes;
  TPK format 2, compression 0, data type 0, **1,420 versions** 3.4.0.3.1 → 6000.7.0.0.3, **415 classes**).
  Sledding's `6000.3.12f1` is present as `6000.3.12.3.1`.
- Check coverage of a package (tpk_ar, same venv):
  `python -c "from tpk_ar import TpkFile; from io import BytesIO; b=TpkFile.parse(BytesIO(open('typetrees.tpk','rb').read())).GetDataBlob(); print(len(b.Versions), str(sorted(b.Versions)[-1]))"`.
- Generation for a new Unity version (no TPK is generated by primase; a player build has stripped its type trees):
  1. get a newer TPK from AssetRipper (or refresh UnityPy's copy):
     `UnityPy.exe update_tpk` (same as `python -m UnityPy.cli update_tpk`), which overwrites
     `UnityPy/resources/lzma.tpk`;
  2. re-pack it uncompressed with `work/primase/pm-core/w9/repack_tpk.py IN.tpk OUT.tpk` (it parses with `tpk_ar`,
     decompresses LZMA, writes header `<IBBBBIII` magic/version/compression 0/data type/reserved/reserved2/size/size).
- Caveat observed 2026-10-18: every `nightly.link/AssetRipper/Tpk/workflows/type_tree_tpk/master/*_file.zip` URL
  returned HTTP 404, while the same artifacts exist in the latest Actions run. `update_tpk` may therefore fail today;
  use the UnityPy-bundled copy (verified current through 6000.7) or download the run artifact with an authenticated
  `gh run download`.
- If the game's Unity version is newer than everything in the package, you need an updated upstream TPK; primase
  cannot synthesize the layouts.

## 4. AssetBundles/Addressables vs loose `*_Data` folders

- `Player::open(dir, package)` scans exactly two folders: `dir` and `dir/Resources`, top level only; files that do not
  look like SerializedFile v22 are skipped ("resources, bundles, configs") (`player.rs:62-96`).
- There is **no UnityFS/AssetBundle/Addressables/catalog reader** anywhere in primase: no `UnityFS`, `AssetBundle`,
  `Addressables` or CAB parser exists in the tree (`grep -rni "assetbundle|addressable|UnityFS" primase` → no hits).
  Sledding is read from its loose `_Data` files, not bundles (the same conclusion as the prior review of BundleRipper:
  `work/bundleripper/UNITY_GENERAL_FINDINGS.md:66-69`).
- Extracted bundle SerializedFiles can in principle be read if every `m_FileID` external and every `.resS` basename is
  co-located with the file being opened; nothing resolves a bundle path or an Addressables key.
- `Resources/unity default resources` and `Resources/unity_builtin_extra` are opened as ordinary SerializedFiles
  (they supply built-in shaders). Sledding: 9 files read (`baseline-tests.log:76`).
- Streaming: `StreamingInfo { offset, size, path }`; `path`'s basename is used, so `archive:/<cab>/<name>` works if
  `<name>` exists in the SerializedFile's folder; the range is bounds-checked against the file (`resource.rs:1-3`,
  `:51-77`). Texture mips and mesh vertex streams (`m_StreamData`) both use this (`texture.rs:93-105`,
  `mesh.rs:296-309`).

## 5. What is read and lowered

`components.rs` defines the dialect (`unity.*`). Supported serialized classes:

| Unity class | Read | Lowered / output |
|---|---|---|
| `GameObject`, `Transform`, `RectTransform` | hierarchy, local TRS → world matrices; RectTransform anchors/pivot (`scene.rs:190-330`) | `core.transform` |
| `MeshFilter`, `MeshRenderer` | mesh, materials, `m_StaticBatchInfo` slice, `m_CastShadows`, `m_ReceiveShadows`, layer (`scene.rs:482-563`) | instances/prototypes; `core.shadows` |
| `LODGroup` | levels; folds into `core.lods` forms when levels share LOD 0's matrix within 4 f32 steps, else a `unity.lod` axis (`scene.rs:434-470`, `level.rs:10-28`) | `select-lod` |
| `Mesh` | interleaved streams or compressed mesh; 16/32-bit indices; submeshes (`mesh.rs:194-397`) | `core.mesh` |
| `Texture2D` | formats, all mips, colour space, usage, wrap/filter, normal convention (`texture.rs:83-205`) | `core.texture` |
| `Material`, `Shader` | every saved property (textures + `_ST`, floats, ints, colours), keywords, tags, render queue; shader `m_ParsedForm` subshaders/passes (`LightMode`, blend factors, zWrite, cull) (`material.rs:1-6`, `:19-31`, `:144-170`) | `pbr.material` for URP Lit/Unlit/SpeedTree7; base colour otherwise (`lower.rs:3-22`, `:47-63`) |
| `Light`, `RenderSettings` | type, colour, intensity, range, spot/inner angles, shadow type, colour temperature, bounce, culling mask, cookie, baked, render mode; `m_Sun` (`scene.rs:699-767`, `components.rs:439-497`) | `core.light`, `core.environment` (`lower_lights.rs:1-44`) |
| `BoxCollider`, `SphereCollider`, `CapsuleCollider`, `MeshCollider` | centre/size/radius/height/direction, convex, trigger, physic material, Rigidbody kinematic (`scene.rs:596-663`) | `core.collider` / `core.collision` (`lower_collision.rs:1-20`) |
| `Terrain`, `TerrainData`, `TerrainLayer`, `TerrainCollider` | heightmap (513² i16), holes, splat layers + alphamaps, tree prototypes/instances, collider flags (`terrain.rs:1-9`, `:109-176`) | terrain mesh + base-colour image, tree placements, `core.heightfield` (`lower_terrain.rs:1-13`, `lower_collision.rs:205-230`) |
| `Rigidbody`, `PhysicMaterial` | kinematic flag; friction/bounce values and combine modes | collider filtering / surfaces (`scene.rs:566-590`, `level.rs:582-600`) |
| `MonoBehaviour` | base fields + script namespace/class + raw field bytes (`player.rs:146-158`, `components.rs:245-273`) | `unity.behaviours` (used to identify procedural Shapes) |

Meshes: position, normal, tangent, colour and UV0–7 only (blend weights/indices are not decoded); vertex formats
f32×2/3/4, half×2/4, unorm8×4, unorm16×2, snorm16×2/4; other formats are refused (`mesh.rs:105-126`, `:322-337`).
Compressed meshes decode bit-packed vertices, UVs, normals and tangents (x/y + sign, `z = sqrt(...)`), float colours,
and bit-packed `m_Triangles` (`mesh.rs:8-10`, `:181-192`, `:206-276`, `:354-367`). Topologies: triangles (0), lines (3),
points (5); quads and line strips are refused (`mesh.rs:398-408`). UV channels must be dimension 2.

Textures: formats 1 (A8), 2 (ARGB4444), 3 (RGB24 widened to RGBA8), 4 (RGBA8), 10 (BC1), 12 (BC3), 24 (BC6H UF16),
25 (BC7), 26 (BC4), 27 (BC5), 63 (R8); **crunched 28/29/64/65 and every other id are refused** (`texture.rs:59-81`).
Normal maps: usage modes 3 (DXT5nm, X in alpha) and 4 (X in red), green up, Z rebuilt (`:123-137`). Rows are flipped
to top-first (`:1-8`).

Materials: only URP Lit, URP Unlit and URP Nature/SpeedTree7 are modelled; every other shader gets a base-colour
fallback from `_BaseMap/_MainTex/_Main_Tex/_Texture2D` × `_BaseColor/_Color/_Default_Color/_Base_Color`
(`lower.rs:47-99`). URP Lit handles base colour, metallic/roughness (factor or map), occlusion, normal, emission (HDR
→ `emissive_strength`), alpha blend/clip, `_Cull`, and the base map's tiling for all slots (`lower.rs:3-22`).

Lights: spot/directional/point are lowered to `KHR_lights_punctual` semantics; area lights are counted, not lowered;
colour temperature is not applied; URP intensity is taken as candela (lux for directional) (`lower_lights.rs:1-44`).
The sun is `RenderSettings.m_Sun`, else for a URP player the strictly brightest shining directional light above zero
(URP `GetBrightestDirectionalLightIndex`); ties, zero intensity and a fully baked winner give no sun
(`lower_lights.rs:20-44`).

Terrain: full-resolution heightmap geometry (2 triangles per cell, one diagonal), holes, normals by central
differences, UVs spanning the terrain, one base-colour image at alphamap resolution from each layer's mean diffuse
colour; tree prototypes' prefab renderers and colliders are placed (`lower_terrain.rs:1-13`, `level.rs:530-585`).

## 6. Versions tested / gate evidence

- Only **Sledding Game** (Steam, Unity **6000.3.12f1**, IL2CPP, URP), `Sledding Game_Data`, levels `level0` (empty
  boot scene: 0 renderers, 0 lights) and `level1` (`work/primase/unity-reader/STATUS.md:7-11`). Container version 22,
  little-endian (verified by parsing the installed headers). TPK: `work/primase/pm-core/w9/typetrees.tpk`.
- No other Unity title has been read; the 11 titles named in `STATUS.md:9-10` and ~15 more installed on the desktop are
  untried and may be refused by the container-version gate.
- Oracle: UnityPy 1.25.4. Census: 9 SerializedFiles, 189,299 objects decoded exactly, 1,238 streamed resources,
  496 sampled objects / 83 classes / 1,553,636 numbers with 0 differences, 1 object short of its tree
  (`baseline-tests.log:76-82`, `:310`).
- Geometry gates: 3,944 renderers and 225 mesh/batch prototypes bit-exact back in Unity's frame; 552 stored + 74
  compressed meshes; 918 textures; 141 level materials; 9 terrains / 12,321 trees / 144 terrain pieces; 54 lights;
  15,529 colliders and 15,513 collision shapes (`baseline-tests.log:190-343`).
- Current output (level1 web): 214 tiles, 7.33 M triangles, 16,072 placements, 53 tile lights + sun, 15,522 collision
  records, 118 images, 133 MB, format 2.1, 3 min 15 s (`STATUS.md:36-39`).

## 7. Exact `unity_export` invocation and inputs

Usage (`examples/unity_export.rs:3`, `:18`; README `primase/README.md:69-70`):

```
cargo run --release -p primase-unity --example unity_export -- \
  <Game>_Data <typetrees.tpk> <level> <output folder> [--web] [--ktx2] [--surfaces <map.json>]
```

Required positional inputs:

1. `<Game>_Data` — the player's data folder, e.g.
   `C:/Program Files (x86)/Steam/steamapps/common/Sledding Game/Sledding Game_Data`. It must contain the
   `globalgamemanagers`/`*.assets`/`levelN` SerializedFiles and the `.resS`/`.resource` streams they name; its
   `Resources/` subfolder is read too (`player.rs:62-96`).
2. `typetrees.tpk` — an uncompressed or LZ4 TPK whose version list covers the game's Unity version
   (`CONTRIBUTING.md:215-216`, `src/lib.rs:6-7`).
3. `<level>` — the SerializedFile *name* (lowercase, no extension), e.g. `level1`; it is used both as the lookup key
   and the output name (`level.rs:801-803`, `scene.rs:190-191`).
4. `<output folder>` — created if missing.

Options: `--web` writes the tiled web set to `<out>/web`; `--ktx2` (requires `--web`) writes KTX2 images;
`--surfaces <map.json>` additionally bakes texture transforms and writes a skate scene to `<out>/skate/{assets,scene}`
(`examples/unity_export.rs:30-44`, `:105-172`). The surface map is a JSON object of the game's physic-material names
(`default` for none) to skate surface names (`:5-6`, `:147-149`).

Sledding example:

```
cd C:/Users/kalmarv/ds-modding/primase
cargo run --release -p primase-unity --example unity_export -- \
  "C:/Program Files (x86)/Steam/steamapps/common/Sledding Game/Sledding Game_Data" \
  C:/Users/kalmarv/modding/work/primase/pm-core/w9/typetrees.tpk \
  level1 out/sledding --web --ktx2
```

Always writes `<out>/<level>.glb`; prints counts and pass approximations. There is no environment variable needed;
the real-data test suite uses `PRIMASE_UNITY_DATA` / `PRIMASE_UNITY_TPK` plus optional oracle variables
(`CONTRIBUTING.md:215-280`, `primase-data.example.toml:6-7`). The census test walks all subfolders, but
`Player::open` does not.

## 8. Not supported

- **AssetBundles / Addressables / UnityFS / CAB**: no reader; only loose `_Data` (+ `_Data/Resources`) files.
  SerializedFiles extracted from bundles work only if externals and `.resS` names are co-located.
- **Embedded type trees**: refused (`serialized.rs:164-167`); a TPK is mandatory.
- **Container versions other than 22** (`serialized.rs:21`); anything else fails with
  `SerializedFile version N (read: 22..=22)`.
- **TPK format/compression**: only data type 0, formats 1–2, uncompressed or LZ4; LZMA/Brotli refused
  (`tpk.rs:71-101`).
- **IL2CPP script fields**: MonoBehaviour fields stay bytes; no generated IL2CPP type trees, no runtime script layout
  (`player.rs:146-158`).
- **Managed references** (`[SerializeReference]`, `ReferencedObject`): refused (`typetree.rs:185-189`).
- **Shader Graphs / custom shaders**: not evaluated. Only URP Lit/Unlit/SpeedTree7 are modelled; Shader Graph and
  custom URP shaders get base-colour only. Compiled programs are read as data (`program.rs:1-8`, `:86-96`, `:224-240`), but no
  pass evaluates them; `primase-dxbc` covers only 13 of 988 Sledding ps_5_0 programs
  (`work/primase/unity-reader/UNITY-SHADER-MAP-PLAN.md:1-20`, `SHADER-SLICING-NOTE.md:1-36`).
- **Custom SRP / HDRP / built-in pipeline**: `render_pipeline()` only classifies Built-in/Universal/Other by the
  pipeline asset's script class (`player.rs:20-41`, `:161-236`); lowering and the sun fallback are URP rules only.
  HDRP/built-in materials fall back to base colour, light units stay URP.
- **ECS / DOTS**: no Entities, ISystem or baker support; only GameObjects/MonoBehaviours are read.
- **SkinnedMeshRenderer / skinning**: not read; blend weights/indices are not decoded (`mesh.rs:26-41`, `:70-77`).
- **SpriteRenderer, ParticleSystemRenderer, LineRenderer, Canvas/UI, Camera, Animator/AnimationClip, Audio\*,
  ReflectionProbe, LightProbes, DecalProjector, Splines**: not read. Sledding counts: 2 active SkinnedMeshRenderer,
  5 SpriteRenderer, 74 ParticleSystemRenderer, 20 LineRenderer, 587 Canvas, 12 Camera (`STATUS.md:47`, census
  class list in `baseline-tests.log:83-166`).
- **Terrain detail/grass prototypes**, terrain tiled splat textures and the terrain material template: not
  reproduced; one diagonal per cell; terrain rotation/scale ignored (Unity places by position) (`lower_terrain.rs:1-13`,
  `level.rs:876-882`; `baseline-tests.log:335`).
- **Crunched textures** (formats 28/29/64/65) and empty 0×0 textures: not decoded (`texture.rs:74-77`).
- **Other mesh topologies/formats**: quads (2) and line strips (4) refused; UVs must be 2-D; only the listed vertex
  formats (`mesh.rs:105-126`, `:398-408`, `:438-470`).
- **Multiple scenes / additive loading**: `read_level` reads one SerializedFile by name; no scene list, no additive
  scene graph. Only terrain tree prefabs are opened as extra scenes (`level.rs:530-585`).
- **Physics semantics**: convex hulls over 255 triangles are counted; collider-owned TerrainData that differs from the
  drawn one is not read; sphere/capsule scaling follows PhysX and is counted (`lower_collision.rs:6-17`).
- **Not registered in `primase convert`** (`cli/src/convert.rs:39`); `unity_export` is the working composition.

## 9. Failure modes observed on Sledding Game

1. **`level0` is an empty boot scene** (0 renderers/0 lights) — exporting it produces nothing; use `level1`
   (`STATUS.md:7-8`).
2. **PlayerSettings is 4 bytes short of the TPK tree**: `globalgamemanagers/1 (class 129): 4 bytes after the tree`;
   the census lists it as the one uncovered object, so it is a package/version gap rather than a parse bug
   (`baseline-tests.log:80-81`).
3. **16,080 MonoBehaviours undecoded** (7,477,440 bytes), including URP pipeline/quality assets. The URP asset's
   main-light mode is therefore assumed per-pixel and counted (`baseline-tests.log:79`, `lower_lights.rs:28-29`, `:136`).
4. **Textures**: 13 empty `0x0` textures and 1 crunched format 28 texture are not decoded (`baseline-tests.log:349`);
   material slots that need them are left out and counted.
5. **Procedural Shapes renderers**: 388 renderers have placeholder meshes and are dropped by `unity-drawn`
   (`READER-AS-DATA.md:12-16`, `baseline-tests.log:193`, `drawn.rs:78-101`). Shapes are `Shapes/*` shaders with
   MonoBehaviour parameters.
6. **Additive aurora drawn as an opaque 3 km slab**: Shader Graph blending lives in `m_ParsedForm`, not the
   material's `_Surface`; the first web build drew 13 additive aurora planes opaque. `unity-drawn` now drops additive
   unmodelled shaders (`drawn.rs:41-55`, `LANDING.md:22-24`).
7. **Empty mesh**: "fireplace curve" has 0 vertices and one 0-index submesh; empty submeshes are skipped because glTF
   forbids 0-count accessors (`project/primase` KB note "Unity reader numerics", `mesh.rs:474-500`).
8. **Compressed-mesh rounding**: 74 compressed meshes; f32 dequantisation differs from the f64 oracle by up to
   `3.9e-4`, bounded by the packed vector's ends rather than the value; the normal/tangent Z rebuild needs the
   `sqrt` error bound (`baseline-tests.log:298-301`, KB note 6bad8e51).
9. **LOD/state counts**: 12,942 renderers stored inactive/disabled and 1,183 LOD 1+ excluded by default; 2,102
   LODGroups folded, 3 kept as a `unity.lod` axis, 405 single-level (`baseline-tests.log:192`, `RESUME.md:20-22`).
10. **Terrain**: 9 terrains, 513×513, 3 layers each, 4,718,592 triangles. Tiled splat textures are replaced by layer
    mean colours, the BruteForce/SnowIceURP material template is not lowered, and Unity's per-cell diagonal is not
    read (`baseline-tests.log:335`). Terrain rotations/scales are ignored (`level.rs:876-882`).
11. **19 level materials on unmodelled shaders** (Shine Graph ×6, SnowIceURP, ColorMask, GlowIntensity, gold, eyes,
    FlatKit/Water, Aurora, Correcting Quad Normals): base colour only; the water is a flat colour, worst base-colour
    difference 235 (`baseline-tests.log:231-246`, `:282-284`).
12. **No sun in RenderSettings** (`m_Sun` path id 0); the URP brightest-directional rule was added, with tie/zero/
    baked refusals (`SUN-DECISION.md:1-32`, `baseline-tests.log:212`).
13. **Directional light handling in the viewer**: the fallback sun goes to `core.environment`, but the viewer's light
    pool takes only point/spot; a non-sun directional light becomes a second unshadowed tile light, and the viewer
    currently ignores `environment.sun` (`STATUS.md:42-43`).
14. **Viewer framing**: Sledding's 9 terrains span 13×13 km, so the default view is ~6 km from the playable area;
    `scenes.json` links cannot carry a camera (`STATUS.md:53-54`).
15. **One mesh placed both as LOD 0 and by a renderer outside the group** gets two prototypes (one with forms); the
    output gates classify this as equivalent (`LANDING.md:23-24`).
16. **Other Unity games**: container versions are gated to 22 and layouts to Sledding's; untried titles may fail to
    open or decode (`STATUS.md:9-11`).

Archive reference (older UnityPy exporter for the same game, useful as independent evidence of Sledding's traps; it is
not primase code):

- IL2CPP `global-metadata` v39 could not be loaded by UnityPy's `TypeTreeGenerator` (LibCpp2IL), so all script fields
  (Shapes, `FenceLayout`, `SkiLift`, `BF_AddSnow`, TMP) are unreadable; primase has the same limit and keeps bytes
  (`archive/unity-reskate/NOTES.md:13-16`, `MODLOG.md:10-12`).
- That exporter's UnityPy `get_triangles` dropped `SubMesh.baseVertex`; primase adds `base_vertex` explicitly
  (`archive/unity-reskate/NOTES.md:29-30`, `MODLOG.md:65`, `worlds/unity/src/mesh.rs:493-504`).
- SpeedTree LOD1 is a billboard quad (`NOTES.md:76`), which is why `select-lod{form=finest}` is the only sane default.
- Rendering the four 5 km `World/Edge Of World` slabs at y = 300 stretched the downstream skate streaming grid to
  2,304 cells and crashed Skate twice right after loading (`NOTES.md:145-147`); a downstream streaming risk, not a
  reader failure, and the reason those slabs were skipped in that port.
- LineRenderer chairlift cables, `BF_AddSnow` roof snow, TMP sign text and Shapes decals were reconstructed from raw
  bytes/names by that exporter (`NOTES.md:91-130`); primase reads none of them (LineRenderer, TMP and arbitrary
  MonoBehaviour layouts are unsupported), so a skate port driven only by primase loses those unless they are added.

## 10. Reference notes read

- `work/primase/unity-reader/STATUS.md`, `RESUME.md`, `LANDING.md`, `READER-AS-DATA.md`, `SUN-DECISION.md`,
  `UNITY-SHADER-MAP-PLAN.md`, `SHADER-SLICING-NOTE.md`, `baseline-tests.log`, `env.sh`.
- `work/bundleripper/UNITY_GENERAL_FINDINGS.md:66-69` (Unity reader comparison; primase uses the UnityPy-based loose
  file frontend, not UnityFS).
- `archive/unity-reskate/NOTES.md`, `MODLOG.md` (the earlier UnityPy -> skate. port; independent evidence of
  Sledding's data traps, not primase code).
- Orca KB `projects/primase.md`, sections "Unity reader numerics" and "Unity reader (Sledding Game, Unity 6000.3
  URP; note:35bd0937)".
- There is no `C:/Users/kalmarv/modding/topics` directory; the topic pages live in the Orca KB and were read there.
