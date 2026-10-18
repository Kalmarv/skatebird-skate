# primase → native ReSkate map: the exact contract

Source: fresh clone `C:/Users/kalmarv/ds-modding/primase` at `7da1c6a584f409cafdf1ad5b1b09f1f0c934c366`
(2026-10-17, "gta: the water lowering's material is used as water (core.material_usage)"). The tree was clean
when this pass started; during it, another process modified `worlds/unity/src/mesh.rs` and
`worlds/unity/src/serialized.rs` (Unity container-version 21 / UV channels), which this note neither made nor uses.
Every reference below is `path:line` relative to `primase/`. Claims are read from source; `[INFERENCE]` marks a
conclusion that follows from the code but is not asserted by a doc comment or test.

The stack is three layers, and only one of them writes Studio's format:

```
game files → reader (core::convert::Reader) → passes (core::pass) → writer (core::convert::Writer)
                                                                     ├─ web  = formats/web
                                                                     └─ reskate = worlds/reskate::build
```

`worlds/reskate` does not read a game. It turns a `primase_core::World` carrying the `skate.*` dialect
(`formats/skate-scene` components) into a Studio map folder. Everything a game reader knows must therefore either be
core data that `raise-skate` understands, or be attached in the `skate.*` dialect directly.

---

## 0. TL;DR (the load-bearing facts)

1. The only function that writes a native ReSkate map is `primase_reskate::build(world, scene_dir, &BuildOptions)`
   (`worlds/reskate/src/build.rs:455`), re-exported at `worlds/reskate/src/lib.rs:28`. It expects a
   `skate.scene` resource and the `skate.*` components; `core.mesh` is the preferred mesh input, `core.mesh_file` the
   legacy fallback (`worlds/reskate/src/scene.rs:487`, `:593-682`).
2. Two entry paths exist:
   - **direct**: `primase_skate_scene::read` → optional `select_lod`/`leave_out_optional` → `build`
     (`worlds/reskate/examples/build_map.rs:48-57`).
   - **convert loop**: any `Reader` + passes (`…, bake-normal-scale, raise-skate{…}, lower-layered-reskate`) +
     the CLI's `Reskate` writer (`cli/src/reskate.rs:73-107`); the writer calls `build(part, Path::new(""),
     &options)` on one in-memory part (`cli/src/reskate.rs:86-89`).
3. The ReSkate target is deliberately weak: **only `core.layered` is illegal; every other name is carried**
   (`cli/src/reskate.rs:75-77`, `core/src/pass.rs:385-484`). The target cannot prove a world is a valid skate scene;
   `build` does that at runtime.
4. Nothing turns `core.marker` into a travel point, and nothing turns a source hull collider into skate hull
   collision. Spawn is the only marker `raise-skate` makes (`formats/skate-scene/src/raise.rs:659-712`), and a spawn
   is inserted automatically if the world has none (`worlds/reskate/src/map.rs:1945-1963`).
5. `min_height` is not a core component. It comes from `skate.scene.min_height` or `BuildOptions`
   (`worlds/reskate/src/scene.rs:910-923`, `worlds/reskate/src/build/floor.rs:70-100`); the CLI exposes neither.

---

## 1. The programmatic composition that produces a native map

### 1.1 The convert loop (the generic path)

Types and functions:

| piece | item | reference |
|---|---|---|
| reader trait | `primase_core::convert::Reader` (`read`, `load`, `default_select`, `default_passes`, `emits`, `prototype_box`) | `core/src/convert.rs:20-36` |
| writer trait | `primase_core::convert::Writer` (`target`, `plan`, `next_part`, `write`, `finish`) | `core/src/convert.rs:53-67` |
| driver | `primase_core::convert::convert(reader, writer, manager, select, passes)` | `core/src/convert.rs:125-133` |
| passes | `PassRegistry`, `PassManager`, `Pipeline`; `PassManager::run` and `PassManager::convert` | `core/src/pass.rs:143-310`, `:821`, `:1360` |
| target check | `Target`, `Legality`, `Mode::Full`; the loop calls `manager.convert(..., Mode::Full, ...)` after `retain` | `core/src/pass.rs:324-484`, `core/src/convert.rs:185` |

Ordering is fixed by `convert` (`core/src/convert.rs:125-195`):

1. `reader.read()` → whole world (placements, prototypes, state; geometry/textures not loaded).
2. `check_emits` — every component namespace must be in `Reader::emits()` (`core/src/convert.rs:39-49`, `:136`).
3. `manager.run(world, select)` — the reader's selection (region, state, hour).
4. `writer.plan(&world, boxes)` — geometry is still unloaded; `boxes` is `reader.prototype_box`.
5. Per part from `writer.next_part()`: `reader.load` → `check_emits` → `manager.run(passes)` → keep the closure of
   the roots plus `PassLog::placed` entities → `manager.convert(target, Mode::Full)` → `writer.write(&part)`.
6. `writer.finish()`.

The root type is `primase_core::Entity`; the writer declares a `Target`; passes are text (`name{key=value,…}`)
(`core/src/pass.rs:202-288`).

The CLI is a thin composition of this loop (`cli/src/main.rs:16-21`, `cli/src/convert.rs:87-120`):

- readers table: `gta` only — `cli/src/convert.rs:39`.
- writers table: `web`, `reskate` — `cli/src/convert.rs:41-42`.
- registry: core `select-lod`, `drop-optional`, `resolve-state`; `primase_image::register` (`cap-textures`,
  `bake-layer-channels`, `normalize-normals`, `bake-normal-scale`); `primase_meshops::register` (`bake-nonrigid`,
  `tessellate-colliders`, …); `primase_gta::register`; `primase_skate_scene::register` (`raise-skate` only);
  `primase_reskate::register` (`lower-layered-reskate`, `lower-tint-reskate`,
  `lower-vertex-alpha-reskate`) — `cli/src/convert.rs:21-37`, `toolkit/image/src/layers.rs:18-32`,
  `formats/skate-scene/src/raise.rs:858-860`, `worlds/reskate/src/lib.rs:30-48`.

Not registered in the CLI: `lower-skate-pbr`, `lower-skate-lights` (their `register` in
`formats/skate-scene/src/lower.rs:17-35` is called only by examples/tests), `load-mesh-files`
(`formats/gltf/src/load.rs:172`), `load-textures`. Readers are expected to load meshes/textures in `Reader::load`
instead.

The documented ReSkate pipeline for a conversion is in the CLI usage:

```
<reader's default passes>, bake-normal-scale,
raise-skate{name=N,spawn=centre}, lower-layered-reskate
```

`cli/src/convert.rs:11-18`. Why each step:

- `bake-normal-scale` — skate has no normal-strength scalar (`cli/src/reskate.rs:1-8`).
- `raise-skate` — creates `skate.texture/material/prototype/parts/placement/light/marker` and the `skate.scene`
  resource from core (`formats/skate-scene/src/raise.rs:186-720`, `:828-840`).
- `lower-layered-reskate` — the writer refuses `MaterialClass::Layered` (`worlds/reskate/src/scene.rs:562-568`);
  the pass removes `core.layered`, provides `skate.material`, `reskate.vertex_blend`, `reskate.blend_opacity` and
  modifies `core.mesh`/`core.material_variants`/`skate.prototype` (`worlds/reskate/src/layered.rs:405-427`).

`Scene::from_world` additionally names `bake-nonrigid` when a placement is mirrored or sheared
(`worlds/reskate/src/scene.rs:690-698`) and `select-lod` when any `core.lods` exists (`:489-491`). The CLI usage
string omits `bake-nonrigid`; a source with mirrored/sheared placements needs it appended, or `build` errors.
`[INFERENCE]` The converter's own target check would not add it: `reskate` carries unknown names, and
`bake-nonrigid` legalizes `core.transform`, which is never declared illegal.

Minimal code (mirrors `cli/src/convert.rs:95-117`):

```rust
use primase_core::convert::{convert, Reader, Writer};
use primase_core::pass::{PassManager, PassRegistry};

let registry: PassRegistry = /* wire the passes as cli/src/convert.rs:21-37 does */;
let mut reader: Box<dyn Reader> = /* your reader */;
let writer: Box<dyn Writer> = /* web or reskate adapter */;
let select_text = reader.default_select();
let pass_text = format!(
    "{}, bake-normal-scale, raise-skate{{name=mymap,spawn=centre}}, lower-layered-reskate",
    reader.default_passes()
);
let select = registry.parse(&select_text)?;
let passes = registry.parse(&pass_text)?;
let manager = PassManager::new(registry);
let report = convert(reader.as_mut(), writer, &manager, &select, &passes)?;
```

There is **no library `Reskate` writer type**. The adapter lives in the binary crate as a private module
(`cli/src/main.rs:12`, `cli/src/reskate.rs`). A second consumer either copies that 100-line adapter or calls
`build` directly.

A working CLI shape for GTA → native map (GTA's defaults already include `select-lod`, the GTA lowerings and
`normalize-normals`, `worlds/gta/src/pipeline.rs:28`; add `bake-nonrigid` if any placement is mirrored/sheared):

```
primase convert --from gta --gta <install> --keys <key folder> --region legion \
  --passes "select-lod{form=finest}, lower-gta-embedded-collision, lower-gta-surfaces, gta-pose-fragments, \
lower-gta-pbr, gta-winding, gta-tint, gta-terrain-layers, gta-vertex-colours, lower-gta-water, normalize-normals, \
bake-nonrigid, bake-normal-scale, raise-skate{name=mymap,spawn=centre}, lower-layered-reskate" \
  --to reskate C:/out --name mymap
```

This writes `C:/out/mymap` (`out_root` + `name`, `worlds/reskate/src/build.rs:520`).

### 1.2 The direct path: a skate-scene folder → map folder

`worlds/reskate/examples/build_map.rs` is the canonical composition:

```rust
let mut w = World::new();
primase_skate_scene::read(&scene, &mut w)?;                       // read.rs:436
let lods = primase_reskate::select_lod(&mut w, lod)?;             // build.rs:423
let (warnings, dropped) =
    primase_reskate::leave_out_optional(&mut w, &scene, max_priority)?; // build.rs:430
o.scene_warnings = Some(warnings);
let r = primase_reskate::build(&w, &scene, &o)?;                  // build.rs:455
```

`build` re-creates the intermediate `Scene` with `Scene::from_world(world, dir)` (`worlds/reskate/src/build.rs:456`),
so the world must already hold the skate dialect and a `skate.scene` resource.

### 1.3 The in-memory path: core world → raise → build

`cli/src/reskate.rs:86-89` is the working example: because the world carries `core.mesh`/`core.texture` in memory,
`build` is given an empty directory. The rest of `build` (folder writes) is unchanged.

```rust
primase_skate_scene::raise_skate(&mut w, &RaiseOptions {
    name: "mymap".into(),
    surfaces,                       // source surface name → skate surface
    spawn: Some(Spawn::Centre),
    unclassified: None,             // or Some("concrete")
    light_scale: SKATE_LIGHT_SCALE,
    ..Default::default()
})?;
// then, if needed: normalize-normals, bake-normal-scale, lower-layered-reskate, bake-nonrigid
let r = primase_reskate::build(&w, std::path::Path::new(""), &BuildOptions {
    out_root: out.into(),
    name: Some("mymap".into()),
    ..Default::default()
})?;
```

### 1.4 What the writer emits

`build` writes `<out_root>/<name>` (`worlds/reskate/src/build.rs:520`):

- `meshes/*.glb` (merged tile meshes and instanced prototypes) — `build.rs:549-566`,
- `textures/*` via `TexLib::save` — `build.rs:574`, `worlds/reskate/src/texture.rs`,
- `map.json`, `pieces.json`, `textures_index.json`, `trainer.json`, `ui/*` fitted in `MapWriter::finish`
  (`worlds/reskate/src/map.rs:1941-2387`),
- `stats.json` (`build.rs:591-592`).

The `BuildReport` carries the numbers a caller should print (`build.rs:93-111`): tiles, merged placements/tris,
instances, prototypes, materials, textures, collision tris, lights without tangent, overlay lifts, blend stats.

---

## 2. What the ReSkate writer demands of a world

Everything below is enforced in `Scene::from_world` (`worlds/reskate/src/scene.rs:487-940`) or `build`/`map.rs`,
not by the convert target. The order is the order the writer reads it.

### 2.1 The scene resource

`skate.scene` (`SkateScene`) is mandatory: `Scene::from_world` errors "the world holds no skate scene"
(`worlds/reskate/src/scene.rs:506-508`); `build` then `expect`s it for the pause map (`build.rs:510-512`).
Fields that change output: `name`, `display_name`, `generator`, `region`, `surfaces`, `streaming_distances`,
`texture_budget`, `min_height`, `max_render_edge`, `pause_map`, `trainer_note`, `light_prefix`, `warnings`, `stats`
(`formats/skate-scene/src/components.rs:182-213`).

`raise-skate` creates a minimal one: `name`, `generator = "primase"`, placements table, lights table when lights
exist (`formats/skate-scene/src/raise.rs:714-720`). Region/min_height/etc. are left `None`.

### 2.2 Prototypes and meshes

- `skate.prototype` marks a prototype; `skate.parts` gives one part per primitive in draw order
  (`worlds/reskate/src/scene.rs:593-682`; raise creates both at `raise.rs:527-583`).
- Mesh input precedence: `core.mesh` first; `core.mesh_file` (GLB re-read) only when there is no `core.mesh`
  (`scene.rs:593-600`, `:115-120`). A prototype with only `core.collision` collides without drawing
  (`scene.rs:625-631`, `:1257-1266`).
- Primitive contract: positions required; normals optional (flat normals are built); UV0 optional (zeros); indices
  optional (sequential). Per-vertex blend weights travel beside `core.mesh` as `reskate.blend_opacity` on the
  prototype (one `Option<Vec<f32>>` per primitive, `worlds/reskate/src/layered.rs:82-90`), are attached at
  `scene.rs:660-668`, and survive flat-normal rebuilding (`scene.rs:1399-1428`).
- A primitive's material is `Primitive::material` (or `core.mesh_file.materials[i]`), mapped to a `skate.material`
  key; `core.material_variants` supplies per-variant materials (`scene.rs:615-632`, `:1450-1487`).
- Foliage/ground flags, `core.name`, `core.source` come from the prototype entity (`scene.rs:660-680`).

### 2.3 Materials and textures

- Materials are `skate.material` entities keyed by `SkateMaterial.key` (`scene.rs:549-576`).
  `MaterialClass::Layered` is refused until lowered (`scene.rs:562-568`).
- Textures are `skate.texture` entities keyed by `SkateTexture.key`. Pixels come from `core.texture` when present,
  else from `core.texture_file`; neither is an error (`scene.rs:526-547`).
- `raise-skate` only raises materials that a placed primitive actually draws (own, a variant's, or a layer of a
  drawn blend), and only the base/normal textures they use; packing metallic/roughness/occlusion into Studio masks
  creates new `skate.texture` entities (`raise.rs:260-330`, `:350-370`, `:380-410`). A used texture
  without `core.texture` pixels is refused (`raise.rs:356-360`). Normal maps must already be
  `NormalConvention::GLTF` (`raise.rs:361-365`); `normalize-normals` does that.
- Class mapping comes from alpha mode + `core.draw_order` decal + water usage (`raise.rs:420-500`).

### 2.4 Collision

- `core.collision` on a prototype becomes that prototype's soup, placed by each instance
  (`scene.rs:726-748`; `raise.rs:530-552`). It becomes water mode when every triangle is on a water-class surface,
  else triangle mesh.
- `core.collision` on an entity that is not a prototype (and not `skate.collision`) becomes a world soup through
  that entity's transform, with winding flipped under a mirror (`scene.rs:752-775`). Those are always triangle-mesh,
  ground, region-cut (`scene.rs:723-740` `soup_def` with `c: None` → `mode_of(None)` at `:356-361`).
- Hull collisions exist only in the dialect (`SkateCollision` with `mode = Hull`, `:356-361`, `:724-730`).
- Surfaces: each `Collision.surfaces` index is resolved through `core.surfaces` to a name, and that name must be in
  the scene's surface list (`formats/skate-scene/src/collision.rs:82-109`). `raise-skate` rewrites names onto the
  17 standard skate surfaces using the caller's map, then `Surface.class`, then "already a skate name", else
  `unclassified=`; otherwise it refuses (`raise.rs:210-252`). The 17 names are `default, concrete, asphalt, metal,
  wood, marble, grass, brick, glass, earth, sand, gravel, plastic, rubber, metal_grate, metal_thin, roofing`
  (`worlds/reskate/src/scene.rs:42-60`).

### 2.5 Lights

- Input is `skate.light`; `raise-skate` converts point and spot `core.light`s and drops directional ones, counted
  (`raise.rs:590-640`). Energy becomes Studio watts (`studio_watts`, `raise.rs:145-149`); a spot's basis comes from
  its tangent, with a documented fallback when absent.
- The writer keeps at most `lights_max_per_cell` per 200 m cell (`map.rs:1464-1501`); default is `0` = no cap
  (`build.rs:79`). Lights are named `<light_prefix><n>` (`map.rs:1502-1550`).

### 2.6 Spawn and travel points

- Input markers are `skate.marker` (`scene.rs:824-866`). `raise-skate` creates at most one marker, and only when
  `RaiseOptions::spawn` is set (the CLI sets `spawn=centre`): a `spawn` at the caller's point or the per-axis median
  of placements, snapped to ground (`raise.rs:659-712`).
- If the world still has no spawn, `MapWriter::finish` inserts one at the region centre with street snap
  (`map.rs:1945-1963`). So spawn is optional, travel is not inferred.
- Travel points are only `SkateMarker { kind: Travel }`; more than 25 is a warning imported from the reference
  (`build.rs:399-403`, `map.rs:2169-2173`). Trainer spots are `Trainer` markers, capped at 64
  (`map.rs:2289-2347`). `core.marker` and `core.curve` are not read by this writer at all.
- The one reader that produces travel points today is the FromSoft/Sekiro reader, which attaches `SkateMarker`
  directly (`worlds/fromsoft/src/sekiro/extras.rs:120-160`); that is an architecture exception, not the core path.

### 2.7 `min_height` / floor policy

- `BuildOptions.min_height: Option<f32>` overrides `Scene.min_height`, which comes from
  `SkateScene.min_height.height` (default `100.01` when the object exists without a height) (`build.rs:26-58`,
  `scene.rs:910-923`).
- If the resulting height is NaN (no config), the policy is off (`worlds/reskate/src/build/floor.rs:70-75`).
- Otherwise: floor = pinned `floor` or the 1st-percentile lowest vertex of placed parts, `lift = (height - floor).max(0)`
  applied as the writer's z offset, `drop_below = floor - stub_below`; parts below it are left out, whole
  placements when all parts are (`worlds/reskate/src/build/floor.rs:40-68`, `:76-104`; offset applied at `build.rs:516-521`).
- Pitfall: the span measurement re-reads each prototype's GLB when `proto.glb` is set (`floor.rs:93-99`), even in an
  in-memory build. A `core.mesh_file`-carrying world built with `dir = ""` and floor enabled will fail unless those
  paths resolve or the prototypes have no `MeshFile`.
- The CLI exposes no `--min-height`, `--floor`, or `--stub-below` (`cli/src/reskate.rs:35-58`).

### 2.8 Tiles and cells

- The writer frame is `(east, north, up) = (x, -z, y)` of the world frame; all positions go through
  `scene::w`/`w_at` (`worlds/reskate/src/scene.rs:28-34`).
- Map tile = `floor(pos / tile)` in that frame, with `tile = 200.0` by default (`build.rs:68`, `:145-147`).
  A `core.tile` on a placement overrides it verbatim (`build.rs:141-143`, `scene.rs:716`). Consequently a reader
  must not attach source streaming tiles unless they are already the writer's 200 m grid; the safest is to attach
  none.
- Region: `skate.scene.region` if set, else the floored bounds of placements/collision soups (`build.rs:513-515`,
  `scene.rs:880`). Placements outside are clipped by triangle test at the region edge, not dropped blindly
  (`build.rs:113-139`, `worlds/reskate/src/map.rs:1068-1120`).
- Bands: placements are processed in 8-row bands (`band_rows: 8`, `build.rs:473`, `map.rs:998-1040`).
- Instancing vs merging: a prototype is instanced when its archetype count >= `min_instances` (3) and its
  triangle count >= `min_instance_tris` (150); otherwise its geometry is merged per material into the tile mesh
  (`build.rs:69-70`, `map.rs:1090-1110`). `--min-instances` is the only one exposed by the CLI.
- Blend-chain overlays are cut on `blend_grid` (default 200 m = Studio's WorldPartition cell), capped by
  `blend_objects` (1000) (`build.rs:72-74`, `map.rs:27-45`).
- `core.priority`, when present, must be in 0..=15 (`scene.rs:492-497`).

### 2.9 Hard refusals (writer-side)

- `core.lods` anywhere: "choose one form first (select-lod)" (`scene.rs:489-491`).
- `core.priority > 15` (`scene.rs:492-497`).
- A placement with a mirrored or sheared transform (`scene.rs:690-698`).
- `placements split by tile` (`scene.rs:460-463`).
- A layered material not lowered (`scene.rs:562-568`).
- A texture with neither pixels nor a file (`scene.rs:532-541`).
- A surface name not in the scene's list (`formats/skate-scene/src/collision.rs:90-102`).
- Raise-side: already a skate scene, `core.lods`, `core.collider`, unmapped surfaces, a layered entity that is also
  PBR, a layer without PBR, used texture without pixels, a normal map not in glTF convention
  (`formats/skate-scene/src/raise.rs:193-250`, `:300-370`).

---

## 3. What the web viewer writer needs

Module: `formats/web`. The `Writer` adapter is `primase_web::convert::TileWriter` (`formats/web/src/convert.rs:21`,
`:57-150`), created by the CLI at `cli/src/web.rs:12-46`.

### 3.1 Target

`formats/web/src/lib.rs:428-456` (`pub fn target()`):

| rule | names |
|---|---|
| legal | `core.instance`, `core.name`, `core.mesh`, `core.material_variants`, `core.shadows`, `core.light`, `core.collision`, `core.marker`, `core.curve`, `pbr.material`, `core.layered`, `core.texture` |
| dynamic | `core.transform` (regular or degenerate only); `core.mesh_file` only when `core.mesh` exists; `core.texture_file` only when `core.texture` exists; `core.curve` only with `tangents: None` (a polyline) |
| illegal | `core.lods`, `core.collider` |
| unlisted | carried (`carry_unknown`) |

Resources: `core.environment` is read for the sun (`lib.rs:493`, `:686-710`); its `verify` requires the sun entity to
be a directional `core.light` (`core/src/environment.rs:17-30`).

### 3.2 Demands

- A drawn prototype needs a loaded `core.mesh` at write time; a missing mesh is an error naming the entity
  (`formats/web/src/lib.rs:1338-1345`, `write_cell` at `:782-800`). `MeshFile` is allowed only because the mesh must also be
  present.
- Placements: `core.instance` + `core.transform`; an entity with `core.mesh` and a transform but no instance is also
  treated as a placement (`lib.rs:496-516`). Transforms must be rigid/degenerate; degenerate placements are counted
  and skipped.
- Materials: `pbr.material` is what the GLB writer encodes; a material entity without it is the glTF fallback
  material, counted (`lib.rs:423-427`). `core.layered` is written as a `PRIMASE_layered_material` extension
  (`lib.rs:128`, `formats/web/examples/scene_to_tiles.rs:30-33`).
- Textures: loaded `core.texture`; PNG by default, KTX2 with the feature and caps.
- `core.marker` and `core.curve` become viewer records in `tiles.json`, not geometry; a Bezier curve must be
  flattened first (`lib.rs:616-684`).
- Collision is written as triangle records per tile (`formats/web/src/collision.rs`; planned at
  `lib.rs:534-538`).
- The writer is streaming: `plan` computes cells from pivots or triangle centres (`spanning`, `lib.rs:458-470`);
  `next_part` first asks for each split placement plus its prototype, then for each cell's roots
  (`convert.rs:85-104`). `write_cell` requires the handed world to be **exactly** the closure of that cell's roots
  (`lib.rs:794-800`), which the convert loop guarantees.
- Output: one GLB per tile / subcell, collision sidecars, shared `images/<sha>.<ext>`, and `tiles.json` published
  last with length checks (`lib.rs:1406-1512`). The optional `--plain` set is a second, uncompressed,
  non-instanced copy (`cli/src/web.rs:33-46`).

Options and defaults (`lib.rs:41-100`): `tile_size = 200`, `gpu_instancing = true`, `compress = true`,
`tile_byte_budget = 96 MiB`, quadtree subcells to level 8 (`lib.rs:76-81`), optional KTX2, `camera`, `viewer_items`,
`base_dir`.

---

## 4. What a new game reader must produce to reach the ReSkate writer

### 4.1 Reader trait contract

Implement `primase_core::convert::Reader` (`core/src/convert.rs:20-36`):

- `read()`: whole-world placements/prototypes/state, geometry and textures **not** loaded.
- `load(world, roots)`: the subset reaching `roots`, with every mesh/texture the writer needs loaded; entity ids
  must stay the world's (`:24-27`).
- `default_select()` / `default_passes()`: pipelines; the CLI appends the ReSkate steps after the latter.
- `emits()`: component namespaces it may attach, checked by `check_emits` after read and after every load (`:39-49`).
  GTA uses `["core", "gta"]` (`worlds/gta/src/convert.rs:85-87`). The architecture rule is core plus the reader's
  own dialect: a non-PBR source must not emit `pbr.*`, and lowering to `skate.*` belongs in a pass. The
  FromSoft/Sekiro reader, which attaches `skate.*` itself, is the recorded exception (`architecture.toml`
  `[[dependency]] from = "worlds/fromsoft"`); `check_emits` enforces only the list the reader declares.
- `prototype_box`: a prototype's box in the core frame before geometry loads; the web writer uses it to plan cells
  (`core/src/convert.rs:34-35`).

### 4.2 Core components the ReSkate path actually consumes

Minimum to produce a non-empty map through `raise-skate` → `build`:

| core item | role in the writer | reference |
|---|---|---|
| `core.instance` + `core.transform` | placements of prototypes; transform must classify regular (no mirror/shear) | `scene.rs:687-717`, `:690-698` |
| `core.mesh` | prototype geometry; positions required, normals/UV optional | `scene.rs:115-120`, `:982-1006` |
| `Primitive::material` / `core.material_variants` | material assignment per part and variant | `scene.rs:615-632`, `:1450-1487` |
| `pbr.material` | raised to `skate.material`; base/normal textures, alpha mode, metallic/roughness, wrap, double-sided, cast shadows | `raise.rs:317-500` |
| `core.texture` (pixels) | raised to `skate.texture`; required for every used base/normal texture | `raise.rs:350-370` |
| `core.surfaces` + `Surface.class` | collision surface names/classes mapped onto the 17 skate surfaces | `raise.rs:210-252`, `formats/skate-scene/src/collision.rs:82-109` |
| `core.collision` | prototype soup or world soup | `scene.rs:726-775`, `raise.rs:530-552` |
| `core.light` (point/spot) | raised to `skate.light`; directional dropped | `raise.rs:590-640` |
| `core.name` | prototype/material/texture/marker keys and labels | `scene.rs:660-680`, `raise.rs:255-258` |
| `core.draw_order` | decal material class | `raise.rs:429-435` |
| `core.priority` (optional) | 0..=15 enforced by the writer | `scene.rs:492-497` |
| `core.layered` (optional) | must be lowered by `lower-layered-reskate` | `scene.rs:562-568`, `worlds/reskate/src/layered.rs:405-427` |

Must be lowered before `build`:

- `core.lods` → `select-lod` (`scene.rs:489-491`).
- `core.collider` → `tessellate-colliders` (`raise.rs:201-206`).
- `core.layered` → `lower-layered-reskate` (`cli/src/reskate.rs:73-77`).
- mirrored/sheared `core.transform` → `bake-nonrigid` (`scene.rs:690-698`).
- non-glTF normals → `normalize-normals`; normal strength → `bake-normal-scale` (`raise.rs:361-365`).

Optional but read: `core.mesh_file` (only as a fallback), `reskate.blend_opacity`, `reskate.vertex_blend`
(`worlds/reskate/src/layered.rs:38-90`, used at `scene.rs:570`, `:660-668`).

Not consumed at all by the ReSkate writer: `core.marker`, `core.curve`, `core.shadows`, `core.environment`,
`core.material_usage` (today). `[INFERENCE]` A Sekiro-style port that attaches `core.marker` will produce a map
with only the auto-inserted spawn; travel/trainer/vfx markers require the `skate.marker` dialect.

### 4.3 Target legality names

The ReSkate target is `Target::new("reskate").illegal("core.layered").carry_unknown()`
(`cli/src/reskate.rs:75-77`). Therefore:

- **illegal**: `core.layered` only. `PassManager::convert` may plan `lower-layered-reskate` for it, since that pass
  legalizes `core.layered` and requires `core.layered`, `core.mesh`, `skate.material` (`worlds/reskate/src/lib.rs:31-32`,
  `worlds/reskate/src/layered.rs:416-427`).
- **carried**: every other name, known or unknown (`core/src/pass.rs:474-477`, `:484-497`).
- There are no `Target::legal`, `carry`, `dynamic`, `check`, or `check_resource` calls for this writer.

Consequences: the target check is not a validity check for this writer. A world missing `skate.scene`,
`skate.prototype`, `skate.material`, or with an unresolvable collision surface passes `Mode::Full` and fails inside
`build`. `[INFERENCE]` Any new reader that reaches `--to reskate` without `raise-skate` will fail at runtime, not at
the target gate.

### 4.4 Reporting and gaps to plan around

A new reader cannot express, in core alone, these runtime scene fields: region, `min_height` floor policy,
streaming distances, texture budget, light groups/prefix, pause map, trainer note, travel/trainer/vfx marker kinds,
and hull collision mode. They live in `SkateScene`/`skate.*` (`formats/skate-scene/src/components.rs:182-220`).
Water collision *is* reachable from core: a prototype whose collision is entirely on
`SurfaceClass::Water` surfaces becomes a water-mode soup (`raise.rs:266-282`, `:530-552`).
Options:

1. Extend `raise-skate` (or add a lowering pass) from `core.marker` to `skate.marker` (kinds travel/spawn/trainer),
   and add `min_height`/`region` options to `RaiseOptions` — the architecture-clean path.
2. Have the reader attach `skate.*` directly (Sekiro precedent) and declare it in `emits()`.
3. Pass a prebuilt `SkateScene` through a pass, not a reader.

`raise-skate` refuses to run on a world that already has `SkateScene` (`raise.rs:194-196`), so option 2 and 1 cannot
be mixed in one pipeline.

---

## 5. Current state of `primase convert` and examples to copy

### 5.1 CLI

- Only subcommand is `convert` (`cli/src/main.rs:16-21`).
- Usage and option grammar: `cli/src/convert.rs:11-18`; parsing requires `--from` first and `--to` after it
  (`cli/src/convert.rs:87-119`).
- Readers: `gta` only (`cli/src/convert.rs:39`); it emits `core` + `gta` and loads drawables/textures per part
  (`worlds/gta/src/convert.rs:43-89`).
- Writers: `web` and `reskate` (`cli/src/convert.rs:41-42`).
- reskate options actually accepted: `--name`, `--min-instances`, `--blend-objects`, `--blend-grid`
  (`cli/src/reskate.rs:35-58`). Not exposed: `tile`, `tex_cap`, `tex_caps`, `overlay_step`, `lights_max_per_cell`,
  water flags, `piece_log`, `limit`, `map_height`, `min_height`, `floor`, `stub_below`, `max_render_edge`,
  `tex_budget`, `offset` (`worlds/reskate/src/build.rs:26-89`).
- Output path quirk: the CLI sets `out_root = <out>` and defaults `name = <out>.file_name()`, and `build` writes
  `<out_root>/<name>` (`cli/src/reskate.rs:23-31`, `build.rs:520`). `--to reskate C:/x/mymap` therefore writes
  `C:/x/mymap/mymap` unless `--name` says otherwise.
- The reskate writer plans one part containing every selected entity (`cli/src/reskate.rs:78-84`); the whole
  selection is resident during `build`.
- Pipeline declaration test (no game data) parses the reader defaults plus the documented ReSkate suffix and checks
  `Pipeline::check` (`cli/src/convert.rs:122-160`).

### 5.2 Examples

| example | what it demonstrates | reference |
|---|---|---|
| `worlds/reskate/examples/build_map.rs` | skate-scene folder → native map, with `select_lod`, `leave_out_optional`, `build` | `build_map.rs:48-57` |
| `cli/src/reskate.rs` | the `Writer` adapter to copy: one part, `build` with an empty dir, target | `reskate.rs:23-110` |
| `formats/web/examples/scene_to_tiles.rs` | per-cell streaming: subset → load meshes/textures → lower materials → web write | `scene_to_tiles.rs:22-36`, `:66-120` |
| `formats/gltf/examples/scene_to_glb.rs` | explicit pass registry for a scene folder (`load-mesh-files`, `load-textures`, `lower-skate-pbr`, …) | `scene_to_glb.rs:24-47` |
| `worlds/unity/examples/unity_export.rs` | Unity level → lowering pipeline → glTF/web/skate-scene folder; ends with `raise-skate` + `prepare_scene_assets` + `write` (not `build`) | `unity_export.rs:71-100`, `:152-174` |
| `core/tests/convert.rs` | minimal `Reader`/`Writer` pair showing the loop's order and `emits` | `core/tests/convert.rs:50-150` |

### 5.3 Other readers

- `worlds/unity`: a full Unity player reader (`Player`, `read_level` at `worlds/unity/src/level.rs:801`), producing
  `core.mesh`, `core.instance`, `core.transform`, `core.name`, `core.shadows`, `core.lods`, `core.material_variants`,
  `core.texture` and `unity.*`; materials become `pbr.material` only through `lower-unity-pbr`. It has **no**
  `Reader` impl and **no** `register()` (`architecture.toml` pass exceptions), so `primase convert` cannot use it.
  To reach the ReSkate writer today: run the lowering pipeline in `worlds/unity/examples/unity_export.rs:71-100`
  (resolve-state, select-lod, unity-drawn, lower-unity-pbr/terrain/collision/lights, normalize-normals, then
  `tessellate-colliders` because `raise-skate` refuses `core.collider`), then `raise-skate` and `build`. Unity's
  collision surfaces are physic-material names with no `Surface.class` (`worlds/unity/src/lower_collision.rs:237-251`),
  so `raise-skate` needs an explicit `map=` or `unclassified=` for them (`raise.rs:210-252`). The example itself ends
  at `write` (a skate-scene folder, `unity_export.rs:172-174`), not at `build`.
- `worlds/fromsoft`: attaches `skate.*` components directly (including travel/spawn markers) but has no
  `Reader` impl and is not in `READERS`.
- `formats/skate-scene`: has `read`/`write` functions, no `Reader` impl, so folders are not a CLI source either.

---

## 6. Verification done for this note

- Repository identity: `git -C primase log -1` = `7da1c6a…` (2026-10-17); `git status --porcelain` was empty at
  the start of the pass and showed two modified Unity files later (another writer, as noted above).
- Every cited `file:line` was checked against the source in this clone; no code was executed and no Rust build was
  run, because the deliverable is a source-derived contract, not a code change.
- Read-only context consulted: `C:/Users/kalmarv/modding/work/ONE-PIPELINE.md` (one reader → passes → writer; no
  side scripts, no scene folders as input) and `C:/Users/kalmarv/modding/work/AGENT-RULES.md`. No `primase/*RESUME*.md`
  exists in that work folder at the time of writing.
