//! Every conversion choice of the export, written out in the program: the lowering, the glTF and web writers' options,
//! raise-skate's, the preparation for the ReSkate map and its encoding. Each states the value the program took from
//! primase's `Default`s at the ds-modding fork (primase-work `archive/ds-skatebird` 7b81a686) it was built against
//! until 2026-10-19, so moving to primase main changes no choice; where the fork chose inside a writer, the source is
//! named.
//!
//! What stays with primase because the program cannot state it: the Unity reader's state axes' defaults (the level as
//! stored) and the unit passes' fixed rules (UnityDrawn, the Unity lowerings, NormalizeNormals, TessellateColliders).
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use primase_core::pass::Pass;
use primase_gltf::prepare::{FillMissingTexcoords, PrepareGltfMaterials, UnitValue};
use primase_reskate::EncodingOptions;
use primase_reskate::prepare::{
    OceanMaterials, PauseExtent, PauseSource, PlanReskateBlendBudget, PrepareReskateBlendOrder,
    PrepareReskateGrinds, PrepareReskateOverlays, PrepareReskatePauseMap, PrepareReskateTextures,
    PrepareReskateWater, RaiseOptions, SnapReskateMarkers, WaterExtent,
};

/// raise-skate's light calibration (Studio watts per candela over 4 pi^2): skate's, fitted against San Vansterdam's
/// own lights (primase's `SKATE_LIGHT_SCALE`, the same at the fork and at main).
pub const LIGHT_SCALE: f64 = 3.007e-4;
/// The web tile set's cell, metres.
pub const WEB_TILE: f64 = 200.0;
/// The largest file a web tile may carry before its cell is split, bytes: 96 MiB.
pub const WEB_TILE_BYTES: u64 = 96 * 1024 * 1024;

/// The largest side of a map texture, pixels (the fork's `BuildOptions::tex_cap`, build.rs:66).
pub const MAP_TEXTURE_CAP: u32 = 256;
/// The overlays' stacking step, metres (`overlay_step`, build.rs:71).
pub const OVERLAY_STEP: f32 = 0.004;
/// The grid the blend-chain objects are cut on, metres: Studio's 200 m cell (`blend_grid`, build.rs:73).
pub const BLEND_GRID: f32 = 200.0;
/// The most blend-chain objects (`blend_objects`, build.rs:72): a fixed count, not the cull-ID pool.
pub const BLEND_OBJECTS: usize = 1000;
/// How far past the content the water reaches, metres (the fork writer's water clip, map.rs:1600).
pub const WATER_MARGIN: f32 = 400.0;
/// The pause-map picture's height, pixels (`map_height`, build.rs:76).
pub const PAUSE_MAP_HEIGHT: usize = 2160;

/// The lowering, in order: the level as stored (every state axis at its default: what is off stays off), each
/// prototype's finest level of detail, what Unity draws, its materials, terrain, collision and lights lowered,
/// normal maps to glTF's convention, collision shapes as triangles (the writers here take triangles), and last before
/// the writers authored mirrors and shears baked into geometry, mirrors too, so every placement is rigid.
pub fn lowering() -> Vec<Box<dyn Pass>> {
    vec![
        Box::new(primase_core::state::ResolveState {
            assign: BTreeMap::new(),
            keep: BTreeSet::new(),
        }),
        Box::new(primase_core::SelectLod {
            choice: primase_core::LodChoice::Finest,
        }),
        Box::new(primase_unity::drawn::UnityDrawn),
        Box::new(primase_unity::lower::LowerUnityPbr),
        Box::new(primase_unity::lower_terrain::LowerUnityTerrain),
        Box::new(primase_unity::lower_collision::LowerUnityCollision),
        Box::new(primase_unity::lower_lights::LowerUnityLights),
        Box::new(primase_image::NormalizeNormals),
        Box::new(primase_meshops::TessellateColliders),
        Box::new(primase_meshops::BakeNonrigid {
            mirrors: primase_meshops::Mirrors::Bake,
        }),
    ]
}

/// What the one-file glTF of the level needs that the world does not state, run on a copy of the world for the glTF
/// alone: a material entity without PBR factors drawn light grey (0.8), dielectric, roughness 0.85, and zeros for a
/// texture coordinate set a primitive's material samples but the primitive lacks (the fork's glTF writer did both
/// itself: `fallback_material`, formats/gltf/src/write.rs:216, and its zero coordinates).
pub fn gltf_preparation() -> Result<Vec<Box<dyn Pass>>, String> {
    let grey = UnitValue::new(0.8)?;
    Ok(vec![
        Box::new(PrepareGltfMaterials {
            base_color: [grey, grey, grey, UnitValue::new(1.0)?],
            metallic: UnitValue::new(0.0)?,
            roughness: UnitValue::new(0.85)?,
        }),
        Box::new(FillMissingTexcoords),
    ])
}

/// The one-file glTF of the level: placements grouped by prototype, unplaced prototypes left out, the whole world,
/// texture paths as stored.
pub fn gltf(level: &str) -> primase_gltf::WriteOptions {
    primase_gltf::WriteOptions {
        group_by: primase_gltf::GroupBy::Prototype,
        base_dir: PathBuf::new(),
        include_unplaced: false,
        scene_name: Some(level.to_owned()),
        selection: None,
    }
}

/// The web tile set: [`WEB_TILE`] cells split to [`WEB_TILE_BYTES`], GPU instancing, compressed vertex streams;
/// with `ktx2` textures as KTX2, encoded every run, encodings held in at most 512 MiB; no viewer records, no camera.
/// The textures' sizes are [`web_textures`]'.
pub fn web(ktx2: bool) -> Result<primase_web::Options, String> {
    Ok(primase_web::Options {
        ktx2,
        encoding_memory: 512 << 20,
        tile_byte_budget: Some(WEB_TILE_BYTES),
        ..primase_web::Options::new(
            primase_web::TileSize::new(WEB_TILE).map_err(|e| e.to_string())?,
        )
    })
}

/// With KTX2, the web textures at most 512 px (colour, normal) and 256 px (mask), the fork's KTX2 caps (the fork's
/// web writer applied them while encoding KTX2 and left PNGs at their size; main's writer never resizes, so the
/// program runs this pass before writing KTX2 tiles).
pub fn web_textures() -> Result<primase_web::prepare::PrepareWebTextures, String> {
    use primase_web::prepare::{PrepareWebTextures, RoleCaps, WebCap};
    Ok(PrepareWebTextures::new(RoleCaps {
        color: WebCap::new(512)?,
        normal: WebCap::new(512)?,
        mask: WebCap::new(256)?,
    }))
}

/// raise-skate's options: the scene `name`, the game's surfaces onto skate's (`surfaces`; an unmapped surface with no
/// class has its triangles left out, reported), lights at [`LIGHT_SCALE`].
pub fn raise(name: &str, surfaces: BTreeMap<String, String>) -> Result<RaiseOptions, String> {
    RaiseOptions::new(name)?
        .with_surfaces(surfaces)?
        .with_light_scale(LIGHT_SCALE)
}

/// How the map is encoded (the fork's `BuildOptions` defaults, build.rs:63-87, and the map name the program gives):
/// 200 m render tiles, a prototype placed at least 3 times with at least 150 triangles instanced, Studio's 4,780 m
/// limit, no offset, the scene's longest render edge, the cull IDs of the level's share of ReSkate's pool.
pub fn encoding(out_root: PathBuf, name: &str) -> EncodingOptions {
    EncodingOptions {
        out_root,
        name: Some(name.to_owned()),
        tile: 200.0,
        min_instances: 3,
        min_instance_tris: 150,
        max_render_edge: None,
        limit: 4780.0,
        offset: [0.0, 0.0],
        cull_ids: EncodingOptions::default().cull_ids,
        scene_warnings: None,
    }
}

/// The preparation for the ReSkate map, in order, as the fork's map writer made it inside itself (a SkateBIRD scene
/// states no region, floor or texture budget, and the program places its spawn and travel markers): grind curves'
/// prisms as Studio builds them; the markers snapped for this encoding; the water surfaces of the content, 400 m past
/// it, no material drawn as ocean unless it says so; the pause-map picture drawn from the geometry over the content,
/// 2,160 px high; textures capped at 256 px (stored levels, else the 2 x 2 box, normal maps alike); tints multiplied
/// into texture copies; the overlays stacked 4 mm apart; the blend chains cut on the 200 m grid; at most 1,000 chain
/// objects. No light cap.
pub fn map_preparation(encoding: &EncodingOptions) -> Result<Vec<Box<dyn Pass>>, String> {
    let cap = primase_image::CapTextures::new(MAP_TEXTURE_CAP)?.with_mode(primase_image::CapMode {
        normals: primase_image::NormalDownscale::Box,
        ..Default::default()
    });
    Ok(vec![
        Box::new(PrepareReskateGrinds::studio()),
        Box::new(SnapReskateMarkers::for_encoding(encoding)?),
        Box::new(PrepareReskateWater::new(
            WaterExtent::Content {
                margin: WATER_MARGIN,
            },
            OceanMaterials::Stated,
        )?),
        Box::new(PrepareReskatePauseMap::new(PauseSource::Geometry {
            height: PAUSE_MAP_HEIGHT,
            extent: PauseExtent::Content,
        })?),
        Box::new(cap),
        Box::new(PrepareReskateTextures),
        Box::new(PrepareReskateOverlays::new(OVERLAY_STEP)?.for_encoding(encoding)?),
        Box::new(PrepareReskateBlendOrder::new(BLEND_GRID)?.for_encoding(encoding)?),
        Box::new(PlanReskateBlendBudget::objects(BLEND_OBJECTS)),
    ])
}

#[cfg(test)]
mod tests {
    use primase_core::pass::Pipeline;

    use super::*;

    /// The preparation as the pass manager records it: each choice with its value.
    #[test]
    fn the_map_preparation_states_its_choices() {
        let e = encoding(PathBuf::from("out"), "SkateBIRD_level2");
        let text = Pipeline {
            passes: map_preparation(&e).unwrap(),
        }
        .to_string();
        for want in [
            "prepare-reskate-grinds",
            "snap-reskate-markers",
            "prepare-reskate-water{",
            "margin=400",
            "prepare-reskate-pause-map{",
            "height=2160",
            "cap-textures{",
            "max=256",
            "prepare-reskate-textures",
            "prepare-reskate-overlays{",
            "step=0.004",
            "prepare-reskate-blend-order{",
            "grid=200",
            "plan-reskate-blend-budget{objects=1000}",
        ] {
            assert!(text.contains(want), "{want} not in {text}");
        }
    }

    #[test]
    fn the_raise_options_state_the_light_scale() {
        let surfaces = BTreeMap::from([("default".to_owned(), "concrete".to_owned())]);
        let o = raise("level2", surfaces).unwrap();
        assert_eq!(o.options()["light_scale"], LIGHT_SCALE.to_string());
        assert!(raise("level 2", BTreeMap::new()).is_err());
    }
}
