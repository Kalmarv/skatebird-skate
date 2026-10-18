//! Every conversion choice of the export, written out in the program: the lowering, the glTF and web writers' options
//! and raise-skate's. Until 2026-10-18 the program took the rest of each from primase's `Default`s; they were frozen
//! with the values they had at the primase it builds against (the ds-modding fork, archived as primase-work
//! `archive/ds-skatebird` 7b81a686), and the tests below show each equals the default it replaces there, so the freeze
//! changed no output.
//!
//! What stays with primase because the program cannot state it: the Unity reader's state axes' defaults (the level as
//! stored), the unit passes' fixed rules (UnityDrawn, the Unity lowerings, NormalizeNormals, TessellateColliders), and
//! the skate passes that take no options (unify-texture-wrap, pack-orm, bake-normal-scale, prepare-scene-assets, the
//! scene writer).
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use primase_core::pass::Pass;

/// raise-skate's light calibration (Studio watts per candela over 4 pi^2): skate's, fitted against San Vansterdam's
/// own lights (primase's `SKATE_LIGHT_SCALE` at the pin).
pub const LIGHT_SCALE: f64 = 3.007e-4;
/// The web tile set's cell, metres.
pub const WEB_TILE: f64 = 200.0;
/// The largest file a web tile may carry before its cell is split, bytes: 96 MiB.
pub const WEB_TILE_BYTES: u64 = 96 * 1024 * 1024;

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
            keep_mirrors: false,
        }),
    ]
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
/// with `ktx2` textures as KTX2 at most 512 px (colour, normal) and 256 px (mask), encoded every run; no viewer
/// records, no camera.
pub fn web(ktx2: bool) -> primase_web::Options {
    primase_web::Options {
        tile_size: WEB_TILE,
        gpu_instancing: true,
        compress: true,
        ktx2,
        texture_caps: primase_web::ktx2::TextureCaps {
            colour: 512,
            normal: 512,
            mask: 256,
        },
        texture_caps_by_name: Vec::new(),
        cache: None,
        cache_verify: false,
        viewer_items: Vec::new(),
        split_by_tile: BTreeSet::new(),
        base_dir: PathBuf::new(),
        tile_byte_budget: Some(WEB_TILE_BYTES),
        camera: None,
    }
}

/// raise-skate's options: the scene `name`, the game's surfaces onto skate's (`surfaces`, an unmapped surface with no
/// class refused), the spawn and its yaw (none: no marker, yaw 0), lights at [`LIGHT_SCALE`].
pub fn raise(
    name: String,
    surfaces: BTreeMap<String, String>,
    spawn: Option<(primase_skate_scene::Spawn, f64)>,
) -> primase_skate_scene::RaiseOptions {
    primase_skate_scene::RaiseOptions {
        name,
        surfaces,
        spawn: spawn.map(|s| s.0),
        spawn_yaw: spawn.map_or(0.0, |s| s.1),
        unclassified: None,
        light_scale: LIGHT_SCALE,
    }
}

#[cfg(test)]
mod tests {
    //! The frozen settings against the primase defaults they replace, at the pin.
    use primase_core::pass::Pipeline;

    use super::*;

    /// The lowering the program listed before the freeze, on the passes' `Default`s.
    fn inherited_lowering() -> Vec<Box<dyn Pass>> {
        vec![
            Box::new(primase_core::state::ResolveState::default()),
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
            Box::new(primase_meshops::BakeNonrigid::default()),
        ]
    }

    /// The same passes in the same order with the same effective options (each printed as the pass manager records
    /// it), and the two option-carrying ones equal as values.
    #[test]
    fn the_lowering_is_the_inherited_one() {
        let frozen = Pipeline { passes: lowering() };
        let inherited = Pipeline {
            passes: inherited_lowering(),
        };
        assert_eq!(frozen.to_string(), inherited.to_string());
        assert_eq!(
            primase_meshops::BakeNonrigid {
                keep_mirrors: false
            },
            primase_meshops::BakeNonrigid::default()
        );
        let state = primase_core::state::ResolveState::default();
        assert!(state.assign.is_empty() && state.keep.is_empty());
    }

    /// The writers' options equal the program's `..Default::default()` ones.
    #[test]
    fn the_writer_options_are_the_inherited_ones() {
        let inherited = primase_gltf::WriteOptions {
            scene_name: Some("level2".into()),
            ..Default::default()
        };
        assert_eq!(format!("{:?}", gltf("level2")), format!("{inherited:?}"));
        for ktx2 in [false, true] {
            let inherited = primase_web::Options {
                ktx2,
                ..Default::default()
            };
            assert_eq!(format!("{:?}", web(ktx2)), format!("{inherited:?}"));
        }
        assert_eq!(WEB_TILE_BYTES, primase_web::DEFAULT_TILE_BYTE_BUDGET);
    }

    /// raise-skate's options equal the program's `..Default::default()` ones, with and without a spawn.
    #[test]
    fn the_raise_options_are_the_inherited_ones() {
        let surfaces = BTreeMap::from([("default".to_owned(), "concrete".to_owned())]);
        for spawn in [
            None,
            Some((primase_skate_scene::Spawn::At([1.0, 2.0]), 90.0)),
            Some((
                primase_skate_scene::Spawn::AtWithHeight([1.0, 2.0, 3.0]),
                -45.0,
            )),
        ] {
            let inherited = primase_skate_scene::RaiseOptions {
                name: "level2".into(),
                surfaces: surfaces.clone(),
                spawn: spawn.map(|s| s.0),
                spawn_yaw: spawn.map_or(0.0, |s| s.1),
                ..Default::default()
            };
            assert_eq!(
                format!("{:?}", raise("level2".into(), surfaces.clone(), spawn)),
                format!("{inherited:?}")
            );
        }
        assert_eq!(LIGHT_SCALE, primase_skate_scene::SKATE_LIGHT_SCALE);
    }
}
