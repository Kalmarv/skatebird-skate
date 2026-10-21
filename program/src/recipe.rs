//! One SkateBIRD level into a ReSkate map and its Studio compile: primase's SkateBIRD reader, the lowering to core
//! meanings, the preparation for skate., primase's map writer, then ReSkate Studio. Every choice is a typed value here;
//! the per-level ones (room shells, ceilings, the pause view, the fallback spawn) are [`crate::choices`].

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use primase_core::convert::convert;
use primase_core::pass::{PassManager, PassRegistry, Pipeline};
use primase_core::pbr::TextureSlot;
use primase_core::state::ResolveState;
use primase_core::{LodChoice, SelectLod};
use primase_gltf::prepare::FillMissingTexcoords;
use primase_image::{
    BakeNormalScale, CapMode, CapTextures, MissingLevels, NormalDownscale, NormalizeNormals, OrmSize, PackOrm,
};
use primase_meshops::{
    BakeNonrigid, Mirrors, OmitMateriallessPrimitives, SpecializeMaterialVariants, TessellateColliders,
    UnusedVariants,
};
use primase_reskate::compile::{Compiled, Studio, compile_map};
use primase_reskate::components::{CollisionSurfaces, CollisionWeld};
use primase_reskate::passes::{
    BlendedSelection, LowerReskateGrindCurves, LowerReskateShadows, OceanMaterials, PauseColour, PauseDrawing,
    PauseExtent, PauseSource, PlanReskateBlendBudget, PrepareReskateBlendOrder, PrepareReskateBlendedSurfaces,
    PrepareReskateCollision, PrepareReskateGrinds, PrepareReskateOverlays, PrepareReskatePauseMap,
    PrepareReskateTextures, PrepareReskateUvRepeats, PrepareReskateUvTransforms, PrepareReskateWater,
    PrepareReskateWrap, RaiseOptions, RaiseSkate, RepeatBlended, RepeatDecals, SamplingConflicts, SnapReskateMarkers,
    UniformAlpha, WaterExtent,
};
use primase_reskate::passes::grind_curves::GrindLeaveOut;
use primase_reskate::tint::LowerTintReskate;
use primase_reskate::{EncodingOptions, MapSink};
use primase_skatebird::SkatebirdReader;
use primase_skatebird::components::SkatebirdRole;
use primase_skatebird::passes::voronoi::LowerSkatebirdVoronoi;
use primase_skatebird::passes::{OmitSkatebirdMissionProps, PlaceSkatebirdMarkers};
use primase_unity::player::Player;
use primase_unity::{drawn::UnityDrawn, lower::LowerUnityPbr, lower_collision::LowerUnityCollision};
use primase_unity::lower_lights::LowerUnityLights;

use crate::choices;
use crate::rails::KeepSkatebirdLineRails;
use crate::levels::{Level, map_name};

use primase_core::convert::Result;

/// `level` read from the game, converted into `<out>/map/<map name>` and, with `studio`, compiled into `<out>/stage`.
/// `surfaces` maps the game's collision surfaces onto skate's (data/skatebird-surfaces.json).
pub fn level(
    player: &Player,
    level: &Level,
    surfaces: &BTreeMap<String, String>,
    out: &Path,
    studio: Option<&Studio>,
) -> Result<(PathBuf, Option<Compiled>)> {
    let name = map_name(level.id);
    let mut reader = SkatebirdReader::new(player, level.id);
    // 200 m render tiles; a prototype placed at least 3 times with at least 150 triangles instanced; Studio's 4,780 m
    // limit; the cull IDs of the level's share of ReSkate's pool
    let e = EncodingOptions {
        out_root: out.join("map"),
        name: Some(name.clone()),
        tile: 200.0,
        min_instances: 3,
        min_instance_tris: 150,
        max_render_edge: None,
        limit: 4780.0,
        offset: [0.0, 0.0],
        cull_ids: EncodingOptions::default().cull_ids,
        scene_warnings: None,
    };

    // Run once on the level as read: SkateBIRD's own data lowered to core meanings.
    let mut lower: Vec<Box<dyn primase_core::pass::Pass>> = vec![
        // mission-only objects, unlocks, the player's own board and the HUD's mission arrow are not the level
        Box::new(OmitSkatebirdMissionProps {
            roles: BTreeSet::from([
                SkatebirdRole::Collectible,
                SkatebirdRole::MissionGiver,
                SkatebirdRole::BirbDecorator,
                SkatebirdRole::UnlockingCollectible,
                SkatebirdRole::PlayerBoard,
                SkatebirdRole::Camera,
            ]),
        }),
        // the level as stored (what is off stays off), each prototype's finest level of detail, what Unity draws
        Box::new(ResolveState {
            assign: BTreeMap::new(),
            assign_class: BTreeMap::from([("lod".to_owned(), "lod0".to_owned())]),
            keep: BTreeSet::new(),
        }),
        Box::new(SelectLod { choice: LodChoice::Finest }),
        Box::new(UnityDrawn),
        Box::new(LowerUnityPbr),
        // Bumped Specular Voronoi (gravel, carpet) baked from its own compiled shader: the forward pass's DIRECTIONAL
        // variant in tier 3, a 2 x 2-cell block, at most 1,024 texels a cell
        Box::new(LowerSkatebirdVoronoi::new(vec!["DIRECTIONAL".to_owned()], 3, 2, 1024)?),
        Box::new(LowerUnityCollision),
        Box::new(LowerUnityLights),
        Box::new(NormalizeNormals),
        Box::new(TessellateColliders),
        Box::new(BakeNonrigid { mirrors: Mirrors::Bake }),
        // only the ropes, cables, chains and ziplines stay grind rails (crate::rails)
        Box::new(KeepSkatebirdLineRails {
            groups: choices::line_rails(level.id).iter().map(|g| (*g).to_owned()).collect(),
        }),
        // the game's start and reset positions as the spawn and 5 travel points spread over the play space (collision
        // soups up to 100 m wide; 10 m margin); a level without a start drops its fallback onto the collision below
        Box::new(PlaceSkatebirdMarkers {
            fallback: choices::fallback_spawn(level.id),
            drop: 0.05,
            margin: 10.0,
            travel: 5,
            play_max_extent: 100.0,
        }),
    ];
    lower.extend(choices::hidden_ceilings(level.id));
    lower.extend(choices::shell_casts_no_shadow(level.id)?);

    // Run before the writer: the core world prepared as the ReSkate writer encodes it.
    let prepare = Pipeline {
        passes: vec![
            // blended surfaces are cutouts at 0.5 (skate draws no blended world geometry): the window glass cut
            // out clear, other glass of uniform alpha frosted (opaque, over 0.8 grey, at least 35% its colour)
            Box::new(PrepareReskateBlendedSurfaces::new(0.5, choices::window_glass(), UniformAlpha::Cutout)?),
            Box::new(PrepareReskateBlendedSurfaces::new(
                0.5,
                BlendedSelection::All,
                UniformAlpha::Frost { opacity: 0.35, grey: 0.8 },
            )?),
            // skate's one texture-coordinate set: a missing set as zeros (Unity reads (0, 0)), unplaced variants
            // dropped, every slot sampled as the base colour (a slot tiling otherwise adopts it, reported)
            Box::new(FillMissingTexcoords),
            Box::new(SpecializeMaterialVariants::new(UnusedVariants::Discard)),
            Box::new(PrepareReskateUvTransforms::new(TextureSlot::BaseColor, SamplingConflicts::Adopt)),
            Box::new(OmitMateriallessPrimitives),
            // one wrap and one mask image per material: occlusion, roughness and metallic packed
            Box::new(PrepareReskateWrap::new(TextureSlot::BaseColor, 0)?),
            Box::new(PackOrm::new(OrmSize::MetallicRoughness, MissingLevels::Generate)),
            // the rails left (ropes, cables, chains, ziplines) as 30 mm grind curves on Studio's metal rail surface
            Box::new(LowerReskateGrindCurves {
                kinds: BTreeSet::from(["rail".to_owned()]),
                radius: 0.03,
                surface: "metal_rail".into(),
                leave_out: GrindLeaveOut::Nothing,
            }),
            Box::new(BakeNormalScale),
            Box::new(RaiseSkate::new(
                RaiseOptions::new(level.id, choices::LIGHTS)?.with_surfaces(surfaces.clone())?,
            )),
            Box::new(LowerReskateShadows),
            // vertex colours as tinted copies where the source draws them (Houdini's SpecularVertexColor)
            Box::new(LowerTintReskate::uncapped()),
            Box::new(PrepareReskateGrinds::studio()),
            Box::new(SnapReskateMarkers::for_encoding(&e)?),
            Box::new(PrepareReskateWater::new(WaterExtent::Content { margin: 400.0 }, OceanMaterials::Stated)?),
            // the pause map: a 2,160 px textured view from above the play space, walls and ceilings left out
            Box::new(PrepareReskatePauseMap::new(PauseSource::Geometry {
                height: 2160,
                extent: PauseExtent::Collision { max_extent: 100.0 },
                drawing: PauseDrawing { leave_out: choices::pause_leave_out(level.id), colour: PauseColour::Texture },
            })?),
            Box::new(CapTextures::new(256)?.with_mode(CapMode { normals: NormalDownscale::Box, ..CapMode::default() })),
            Box::new(PrepareReskateTextures),
            Box::new(PrepareReskateUvRepeats::new(RepeatDecals::Omit, RepeatBlended::Cutout)),
            Box::new(PrepareReskateOverlays::new(0.004)?.for_encoding(&e)?),
            Box::new(PrepareReskateBlendOrder::new(200.0)?.for_encoding(&e)?),
            Box::new(PrepareReskateCollision::new(CollisionSurfaces::Coalesce, CollisionWeld::Millimetre, &e)?),
            Box::new(PlanReskateBlendBudget::objects(1000)),
        ],
    };

    // Read, lower, prepare, write `<out>/map/<name>`; what was left out or approximated comes back as notices.
    let writer = Box::new(MapSink::new(Path::new(""), &e));
    let manager = PassManager::new(PassRegistry::new());
    let report = convert(&mut reader, writer, &manager, &Pipeline { passes: lower }, &prepare)?;
    for notice in report.notices() {
        println!("{notice}");
    }
    let map = out.join("map").join(&name);

    // ReSkate Studio compiles the map folder into a mod, against an empty stage folder (it refuses one in use).
    let Some(studio) = studio else {
        return Ok((map, None));
    };
    let stage = out.join("stage");
    if stage.exists() {
        std::fs::remove_dir_all(&stage)?;
    }
    let compiled = compile_map(studio, &map, &stage, &name, &choices::studio_compile())?;
    Ok((map, Some(compiled)))
}
