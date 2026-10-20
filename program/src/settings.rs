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

use crate::level::{Level, Point};
use primase_gltf::prepare::{FillMissingTexcoords, PrepareGltfMaterials, UnitValue};
use primase_reskate::EncodingOptions;
use primase_reskate::components::{CollisionSurfaces, CollisionWeld};
use primase_reskate::passes::{
    LightCalibration, PrepareReskateCollision, PrepareReskateUvRepeats, RepeatBlended, RepeatDecals,
};
use primase_reskate::prepare::{
    OceanMaterials, PauseExtent, PauseSource, PlanReskateBlendBudget, PrepareReskateBlendOrder,
    PrepareReskateGrinds, PrepareReskateOverlays, PrepareReskatePauseMap, PrepareReskateTextures,
    PrepareReskateWater, RaiseOptions, SnapReskateMarkers, WaterExtent,
};

/// raise-skate's light calibration: SkateBIRD's lights are in Unity built-in units, relative to its sun (intensity 1),
/// not physical candela, so primase's standard calibration (3.007e-4, fitted to GTA's physical lights) leaves its
/// interiors black. 10,000 times the standard, chosen by in-game comparison of x100, x1000 and x10000 on the office,
/// server room and bedroom 2 (run light1; Caleb, 2026-10-19: "x10000").
pub const LIGHTS: LightCalibration = LightCalibration::Scale(3.007);
/// A collision soup larger than this on an axis is not play space (skyline backdrops; the buildings' 360 m sides).
pub const PLAY_SPACE_MAX: f64 = 100.0;
/// Travel markers per level, the starting position included (Caleb, 2026-10-19, of 0.2.0's up to 20: "way too many
/// mission markers").
pub const TRAVEL_POINTS: usize = 5;

/// The levels the program converts: SkateBIRD's skateable scenes (BuildSettings 1-5, 7, 12, 13) and the game's
/// names for them.
pub const LEVELS: [Level; 8] = [
    Level {
        id: "level1",
        name: "bedroom 1",
    },
    Level {
        id: "level2",
        name: "rooftop",
    },
    Level {
        id: "level3",
        name: "office",
    },
    Level {
        id: "level4",
        name: "server room",
    },
    Level {
        id: "level5",
        name: "bedroom 2",
    },
    Level {
        id: "level7",
        name: "flashback",
    },
    Level {
        id: "level12",
        name: "playground",
    },
    Level {
        id: "level13",
        name: "skate heaven",
    },
];

/// The spawn of a level with no starting reset position: level7 (flashback), where the game starts the player at
/// its `Player Birb` placement, Unity (0, 1.189, 0) facing yaw 172.9341.
pub fn fallback_spawn(level: &str) -> Option<Point> {
    (level == "level7").then(|| Point {
        name: "spawn".into(),
        position: [0.0, 1.189, 0.0],
        yaw_deg: crate::level::scene_yaw(172.9341),
    })
}

/// What the maps leave out of the scene (Caleb, 2026-10-19: "any mission only objects should be removed"; "there's a
/// random floating skateboard"): mission collectibles, mission-giver birds and their dressing, unlock pickups, the
/// player's own board and what hangs under the scene camera (the HUD's mission arrow). An `EnableOneAtRandom` group
/// outside those stays whole; objects a mission toggles stay as the scene stores them.
pub fn mission_props() -> primase_skatebird::passes::OmitSkatebirdMissionProps {
    use primase_skatebird::components::SkatebirdRole;
    primase_skatebird::passes::OmitSkatebirdMissionProps {
        roles: BTreeSet::from([
            SkatebirdRole::Collectible,
            SkatebirdRole::MissionGiver,
            SkatebirdRole::BirbDecorator,
            SkatebirdRole::UnlockingCollectible,
            SkatebirdRole::PlayerBoard,
            SkatebirdRole::Camera,
        ]),
    }
}

/// SkateBIRD's grind rails (`GrindableRail` chains, read as curves of kind `rail`) as skate grind curves: 30 mm, on
/// Studio's metal rail surface (Caleb, 2026-10-19: "you can't grind on the clothesline ... I want to").
pub fn grind_curves() -> primase_reskate::passes::LowerReskateGrindCurves {
    primase_reskate::passes::LowerReskateGrindCurves {
        kinds: BTreeSet::from(["rail".to_owned()]),
        radius: 0.03,
        surface: "metal_rail".into(),
    }
}

/// An indoor level's room shell: the hierarchy under which its walls, ceiling and floor are placed, as the prefix of
/// their placements' names (the Unity reader names a placement by its GameObject path, joined by `/`), found by
/// renderer census (scratch scripts/roots_census.py). The outdoor levels (rooftop, playground, skate heaven) have none.
pub fn room_shell(level: &str) -> Option<&'static str> {
    match level {
        "level1" => Some("Level Elements/Bounds/"),
        "level3" => Some("Level Bounds/"),
        "level4" => Some("Room Bounds/"),
        "level5" => Some("Bounds/"),
        "level7" => Some("Brick Bedroom/Bounds/"),
        _ => None,
    }
}

/// The room shell casts no shadow, so skate's sun and sky light the room inside (Caleb, 2026-10-19: "make the room
/// material allow light to pass through and just use skybox lighting"; it replaces 0.2.0's ambient fill lights). A
/// shell material drawn outside the shell too gets a copy for the shell.
pub fn shadow_casting(
    level: &str,
) -> Result<Option<primase_reskate::passes::SetReskateShadowCasting>, String> {
    room_shell(level)
        .map(|shell| {
            primase_reskate::passes::SetReskateShadowCasting::new(
                BTreeSet::new(),
                BTreeSet::from([shell.to_owned()]),
                primase_reskate::passes::SharedMaterials::Copy,
            )
        })
        .transpose()
}
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

/// The pause map's ground over a level's play space (core frame; the picture's north is -z): the map's content extent
/// reaches the skyline backdrops kilometres out, which left the level a speck (Caleb, 2026-10-19: "map is fucked we
/// need the actual overhead view").
pub fn pause_extent(space: [[f64; 3]; 2]) -> PauseExtent {
    let [lo, hi] = space;
    PauseExtent::Rect([lo[0] as f32, -hi[2] as f32, hi[0] as f32, -lo[2] as f32])
}

/// The lowering, in order: the level as stored (every state axis at its default: what is off stays off), each
/// prototype's finest level of detail, what Unity draws, its materials, terrain, collision and lights lowered,
/// normal maps to glTF's convention, collision shapes as triangles (the writers here take triangles), and last before
/// the writers authored mirrors and shears baked into geometry, mirrors too, so every placement is rigid.
pub fn lowering() -> Result<Vec<Box<dyn Pass>>, String> {
    Ok(vec![
        Box::new(primase_core::state::ResolveState {
            assign: BTreeMap::new(),
            assign_class: BTreeMap::from([("lod".to_owned(), "lod0".to_owned())]),
            keep: BTreeSet::new(),
        }),
        Box::new(primase_core::SelectLod {
            choice: primase_core::LodChoice::Finest,
        }),
        Box::new(primase_unity::drawn::UnityDrawn),
        Box::new(primase_unity::lower::LowerUnityPbr),
        // SkateBIRD's Bumped Specular Voronoi (gravel, carpet) baked from its own compiled shader into textures
        // (the forward pass's DIRECTIONAL variant in tier 3; a 2 x 2-cell block, at most 1,024 texels a cell)
        Box::new(
            primase_skatebird::passes::voronoi::LowerSkatebirdVoronoi::new(
                vec!["DIRECTIONAL".to_owned()],
                3,
                2,
                1024,
            )?,
        ),
        Box::new(primase_unity::lower_terrain::LowerUnityTerrain),
        Box::new(primase_unity::lower_collision::LowerUnityCollision),
        Box::new(primase_unity::lower_lights::LowerUnityLights),
        Box::new(primase_image::NormalizeNormals),
        Box::new(primase_meshops::TessellateColliders),
        Box::new(primase_meshops::BakeNonrigid {
            mirrors: primase_meshops::Mirrors::Bake,
        }),
        // the scene's ambient as core's (a linear gradient); a skybox scene by its probe's average
        Box::new(primase_unity::lower_ambient::LowerUnityAmbient {
            probe: primase_unity::lower_ambient::UnityAmbientProbe::ProbeAverage,
        }),
    ])
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
/// class has its triangles left out, reported), lights calibrated as `lights` ([`LIGHTS`] unless an experiment says).
pub fn raise(
    name: &str,
    surfaces: BTreeMap<String, String>,
    lights: LightCalibration,
) -> Result<RaiseOptions, String> {
    RaiseOptions::new(name, lights)?.with_surfaces(surfaces)
}

/// The window glass: drawn clear (Caleb, 2026-10-19, of frosted windows: "Fully clear windows").
pub const WINDOW_GLASS: [&str; 1] = ["ClearGlass"];

/// Blended surfaces (not decals, not water) as alpha-tested cutouts at 0.5: Studio compiles a blended material to the
/// stock decal/blend programs, which skate doesn't draw as world geometry (level12's canopies vanished). Window glass
/// ([`WINDOW_GLASS`]) cuts out below the cutoff, so the windows are clear; other glass (a uniform alpha below the
/// cutoff: level1's glass tabletop, which collides) is drawn opaque as frosted glass, over a light grey (0.8 linear)
/// at least 35% its own colour (Caleb, 2026-10-19: "I get it's glass but lets make it slightly more cloudy").
pub fn blended_surfaces() -> Result<Vec<Box<dyn Pass>>, String> {
    use primase_reskate::passes::{BlendedSelection, PrepareReskateBlendedSurfaces, UniformAlpha};
    let windows = BlendedSelection::Named(WINDOW_GLASS.iter().map(|s| (*s).to_owned()).collect());
    Ok(vec![
        Box::new(PrepareReskateBlendedSurfaces::new(0.5, windows, UniformAlpha::Cutout)?),
        Box::new(PrepareReskateBlendedSurfaces::new(
            0.5,
            BlendedSelection::All,
            UniformAlpha::Frost {
                opacity: 0.35,
                grey: 0.8,
            },
        )?),
    ])
}

/// skate's one texture-coordinate set, as the program chooses it: a primitive stored without the set its texture
/// samples reads (0, 0) in Unity, so it gets the set as zeros; variants nobody places are discarded (they choose
/// nothing) and each placement's material variant gets geometry of its own; then every slot samples as the base
/// colour does (a slot that tiles otherwise adopts it, reported), the tiling baked into the coordinates.
pub fn one_uv_set() -> Vec<Box<dyn Pass>> {
    vec![
        Box::new(FillMissingTexcoords),
        Box::new(primase_meshops::SpecializeMaterialVariants::new(
            primase_meshops::UnusedVariants::Discard,
        )),
        Box::new(primase_reskate::passes::PrepareReskateUvTransforms::new(
            primase_core::pbr::TextureSlot::BaseColor,
            primase_reskate::passes::SamplingConflicts::Adopt,
        )),
    ]
}

/// One wrap per material: every slot wraps as the base colour wraps along u.
pub fn one_wrap() -> Result<primase_reskate::passes::PrepareReskateWrap, String> {
    primase_reskate::passes::PrepareReskateWrap::new(primase_core::pbr::TextureSlot::BaseColor, 0)
}

/// The map's (and the mod folder's, and the zip's) name for a level, by the game's name for it: `SkateBIRD_Rooftop`,
/// `SkateBIRD_ServerRoom` (Caleb, 2026-10-19: the scene numbers confused).
pub fn map_name(level: &str) -> String {
    // a variant built with `--tag T` is `<level>_T`
    let (id, tag) = level
        .split_once('_')
        .map_or((level, None), |(i, t)| (i, Some(t)));
    let name = LEVELS
        .iter()
        .find(|l| l.id == id)
        .map_or(id.to_owned(), |l| {
            l.name
                .split(' ')
                .map(|w| {
                    let mut c = w.chars();
                    c.next()
                        .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
                })
                .collect()
        });
    match tag {
        Some(t) => format!("SkateBIRD_{name}_{t}"),
        None => format!("SkateBIRD_{name}"),
    }
}

/// The Studio compile of a level's map: resident (`--no-stream`: these levels are small, so no WorldPartition cells
/// and none of Studio's own vista proxies, which hung the GPU on large maps), the game's own geometry at every distance
/// (`--no-lods`: SkateBIRD has no LODs, Studio's would be decimations), no baked GI, compiled at noon.
pub fn studio_compile() -> primase_reskate::compile::CompileChoices {
    primase_reskate::compile::CompileChoices {
        streaming: Some(primase_reskate::compile::Streaming::Resident),
        lods: false,
        gi: false,
        time_of_day: Some(primase_reskate::compile::TimeOfDay::Noon),
        pause_map: None,
    }
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

/// The preparation for the ReSkate map, in order: vertex tints as tinted material copies where the source draws
/// them; then as the fork's map writer made it inside itself (a SkateBIRD scene
/// states no region, floor or texture budget, and the program places its spawn and travel markers): grind curves'
/// prisms as Studio builds them; the markers snapped for this encoding; the water surfaces of the content, 400 m past
/// it, no material drawn as ocean unless it says so; the pause-map picture drawn from the geometry over `pause`
/// ([`pause_extent`] for a level), 2,160 px high; textures capped at 256 px (stored levels, else the 2 x 2 box, normal maps alike); tints multiplied
/// into texture copies; triangles over Studio's 256 texture repeats left out of repeating decals, and a repeating
/// blended surface with such a triangle drawn as a cutout twin (what the fork's writer did inside itself); the
/// overlays stacked 4 mm apart; the blend chains cut on the 200 m grid; the collision laid out as the fork's writer
/// did (a leaf's rare surfaces coalesced into its common one, corners welded to the millimetre); at most 1,000 chain
/// objects (a fixed count, so no map summary). No light cap.
pub fn map_preparation(
    encoding: &EncodingOptions,
    pause: PauseExtent,
) -> Result<Vec<Box<dyn Pass>>, String> {
    let cap = primase_image::CapTextures::new(MAP_TEXTURE_CAP)?.with_mode(primase_image::CapMode {
        normals: primase_image::NormalDownscale::Box,
        ..Default::default()
    });
    Ok(vec![
        // per-vertex colours as tinted material copies, for the materials whose source takes them (Houdini's
        // SpecularVertexColor; Standard and Legacy ignore COLOR_0, core.vertex_color says so)
        Box::new(primase_reskate::tint::LowerTintReskate::uncapped()),
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
            extent: pause,
        })?),
        Box::new(cap),
        Box::new(PrepareReskateTextures),
        Box::new(PrepareReskateUvRepeats::new(
            RepeatDecals::Omit,
            RepeatBlended::Cutout,
        )),
        Box::new(PrepareReskateOverlays::new(OVERLAY_STEP)?.for_encoding(encoding)?),
        Box::new(PrepareReskateBlendOrder::new(BLEND_GRID)?.for_encoding(encoding)?),
        Box::new(PrepareReskateCollision::new(
            CollisionSurfaces::Coalesce,
            CollisionWeld::Millimetre,
            encoding,
        )?),
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
            passes: map_preparation(&e, PauseExtent::Content).unwrap(),
        }
        .to_string();
        for want in [
            "lower-tint-reskate",
            "prepare-reskate-grinds",
            "snap-reskate-markers",
            "prepare-reskate-water{",
            "margin=400",
            "prepare-reskate-pause-map{",
            "height=2160",
            "cap-textures{",
            "max=256",
            "prepare-reskate-textures",
            "prepare-reskate-uv-repeats{",
            "decals=omit",
            "blended=cutout",
            "prepare-reskate-overlays{",
            "step=0.004",
            "prepare-reskate-blend-order{",
            "grid=200",
            "prepare-reskate-collision{",
            "surfaces=coalesce",
            "weld=mm",
            "plan-reskate-blend-budget{objects=1000}",
        ] {
            assert!(text.contains(want), "{want} not in {text}");
        }
    }

    #[test]
    fn maps_are_named_by_the_games_level_names() {
        assert_eq!(map_name("level2"), "SkateBIRD_Rooftop");
        assert_eq!(map_name("level4"), "SkateBIRD_ServerRoom");
        assert_eq!(map_name("level1"), "SkateBIRD_Bedroom1");
        assert_eq!(map_name("level3_s1k"), "SkateBIRD_Office_s1k");
    }

    #[test]
    fn the_raise_options_state_the_light_calibration() {
        let surfaces = BTreeMap::from([("default".to_owned(), "concrete".to_owned())]);
        let o = raise("level2", surfaces, LIGHTS).unwrap();
        assert_eq!(o.options()["light_scale"], "3.007");
        assert!(raise("level 2", BTreeMap::new(), LIGHTS).is_err());
    }
}
