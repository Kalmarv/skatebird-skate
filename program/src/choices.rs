//! The conversion's choices that differ by level, or that need their reason next to them: what the recipe
//! ([`crate::recipe`]) states for each level.
use std::collections::BTreeSet;

use primase_core::pass::Pass;
use primase_meshops::{OmitPlacements, PlacementCollision, SetShadowCasting};
use primase_reskate::compile::{CompileChoices, Streaming, TimeOfDay};
use primase_reskate::passes::{BlendedSelection, LightCalibration, PauseLeaveOut};

/// raise-skate's light calibration: SkateBIRD's lights are in Unity built-in units, relative to its sun (intensity 1),
/// not physical candela, so primase's standard calibration (3.007e-4, fitted to GTA's physical lights) leaves its
/// interiors black. 10,000 times the standard, chosen by in-game comparison of x100, x1000 and x10000 on the office,
/// server room and bedroom 2 (run light1; Caleb, 2026-10-19: "x10000").
pub const LIGHTS: LightCalibration = LightCalibration::Scale(3.007);

/// The spawn of a level with no starting reset position, in the core frame with the scene yaw: level7 (flashback),
/// where the game starts the player at its `Player Birb` placement, Unity (0, 1.189, 0) facing yaw 172.9341. It stood
/// on a mission object the map leaves out, so it drops onto the collision below (Caleb, 2026-10-19: "flash back you
/// spawn in the air").
pub fn fallback_spawn(level: &str) -> Option<([f64; 3], f64)> {
    (level == "level7").then_some(([0.0, 1.189, 0.0], 180.0 - 172.9341))
}

/// An indoor level's room shell: what closes the room in from the sun and sky (walls, ceiling, floor, and in the
/// office its ceiling light panels, window wall, cubicle surround and the CEO office's own walls; in the server room
/// the tower's stacked floors), as prefixes of their placements' names (the Unity reader names a placement by its
/// GameObject path, joined by `/`), found by renderer census (scratch scripts/roots_census.py). The outdoor levels
/// (rooftop, playground, skate heaven) have none.
fn room_shell(level: &str) -> &'static [&'static str] {
    match level {
        "level1" => &["Level Elements/Bounds/"],
        // Caleb, 2026-10-19, of 0.3.0: "office still has a mesh we need to [not] shadow cast"
        "level3" => &[
            "Level Bounds/",
            // the cubicles' tall carpeted surround walls shade the start (sky rays from it: 8 of 48 blocked by them)
            "Level Elements/Cubicle Park/Surround/",
            "Level Elements/Ceiling Lights/",
            "Level Elements/Light Wall/",
            "Level Elements/Window Struts/",
            "Level Elements/CEO Office/CEO Office Bounds/",
        ],
        // the server tower's stacked floors shadow the start beneath them black (skate has little bounce light):
        // Caleb, 2026-10-19: "just make the floors or whatever above not shadow cast"
        "level4" => &["Room Bounds/", "Main Tower/"],
        "level5" => &["Bounds/"],
        "level7" => &["Brick Bedroom/Bounds/"],
        _ => &[],
    }
}

/// The room shell casts no shadow, so skate's sun and sky light the room inside (Caleb, 2026-10-19: "make the room
/// material allow light to pass through and just use skybox lighting"; it replaces 0.2.0's ambient fill lights). The
/// program's choice, added to what the scene itself leaves non-casting (Caleb: the Unity reader's castShadows Off kept,
/// the program's choices on top). `lower-reskate-shadows` gives them non-casting skate materials.
pub fn shell_casts_no_shadow(level: &str) -> Result<Option<Box<dyn Pass>>, String> {
    let shell = room_shell(level);
    if shell.is_empty() {
        return Ok(None);
    }
    Ok(Some(Box::new(SetShadowCasting::new(
        shell.iter().map(|p| (*p).to_owned()).collect(),
        false,
    )?)))
}

/// The ceilings the maps don't draw: the office's and the server room's, high out of reach, cast no shadow but drew
/// as black slabs overhead, lit only from below (Caleb, 2026-10-19: "looks fine... until you look up"); their
/// collision stays. The other ceilings are low and lit, and stay.
pub fn hidden_ceilings(level: &str) -> Option<Box<dyn Pass>> {
    let prefixes: &[&str] = match level {
        "level3" => &["Level Bounds/Ceiling"],
        "level4" => &["Room Bounds/Ceiling"],
        _ => return None,
    };
    Some(Box::new(OmitPlacements {
        prefixes: prefixes.iter().map(|p| (*p).to_owned()).collect(),
        collision: PlacementCollision::Keep,
    }))
}

/// The rails that stay grind rails, by level: the groups (bare-`/` GameObject paths) whose rails run along a line the
/// game draws (`CurvedLine3D`), a chain (`SFox_Grind_Link`) or an object named a wire, cable, rope, chain, zipline or
/// line, found by census of the game's objects (scratch scripts/rail_kinds.py, 2026-10-20). Every other rail is left
/// out ([`crate::rails`]; Caleb: "just detect the actual ziplines/chains and only grind rail those"). Flashback has none.
pub fn line_rails(level: &str) -> &'static [&'static str] {
    match level {
        "level1" => &[
            "Partially Dynamic Level Elements/Back Of Overpass Table Magazine Span Rail/",
            "Partially Dynamic Level Elements/Front Of Overpass To Bowl Rail/",
            "Partially Dynamic Level Elements/Rope 1/",
            "Partially Dynamic Level Elements/Rope From Big Vertical Ramp/",
        ],
        "level2" => &[
            "Level Elements/Separate Building 1/Baked Houdini Objects/Tangled Mess O Wires 11/",
            "Level Elements/Separate Building 1/Baked Houdini Objects/Tangled Mess O Wires 12/",
            "Level Elements/Separate Building 1/Baked Houdini Objects/Tangled Mess O Wires 13/",
            "Level Elements/Separate Building 1/Baked Houdini Objects/Tangled Mess O Wires 14/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 1/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 2/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 3/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 4/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 5/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 6/",
            "Level Elements/Shorter Building/Baked Houdini Objects/Tangled Mess O Wires 7/",
            "Level Elements/Taller Building/Baked Houdini Objects/Tangled Mess O Wires 10/",
            "Level Elements/Taller Building/Baked Houdini Objects/Tangled Mess O Wires 8/",
            "Level Elements/Taller Building/Baked Houdini Objects/Tangled Mess O Wires 9/",
        ],
        "level3" => &[
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Cable_Run20/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Cable_Run24/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Cable_Run28 Right Curve/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Grind_Link1/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Grind_Link3/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Grind_Link4/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Grind_Link5/",
            "Level Elements/CEO Office/Baked Houdini Objects/SFox_Grind_Link6/",
            "Level Elements/Cubicle Park/Computers Side/Baked Houdini Prefabs/SFox_Grind_Link2/",
            "Level Elements/Cubicle Park/Snack Bowl Side/Baked Houdini Prefabs/SFox_Grind_Link1 cubicle bowl side/",
            "Level Elements/Cubicle Park/Snack Bowl Side/Baked Houdini Prefabs/SFox_Grind_Link4 cubicle bowl side/",
            "Level Elements/Snack Bar/Baked Houdini Objects/SFox_Grind_Link1/",
            "Partially Static Level Elements/Clip-line enabled DURING clip-line mission/SFox_Grind_Link2/",
            "Partially Static Level Elements/Clip-line enabled after clip-line mission/SFox_Grind_Link2/",
        ],
        "level4" => &[
            "Main Tower/Second Level/Baked Houdini Objects/SFox_Cable_Run1/",
            "Main Tower/Second Level/Baked Houdini Objects/SFox_Cable_Run2/",
            "Main Tower/Second Level/Baked Houdini Objects/SFox_Cable_Run3/",
            "Main Tower/Second Level/Baked Houdini Objects/SFox_Cable_Run5/",
            "Main Tower/Second Level/Baked Houdini Objects/sfox_grind_cable1/",
            "Main Tower/Second Level/Baked Houdini Objects/sfox_grind_cable2/",
            "Main Tower/Third Level/Baked Houdini Objects/sfox_grind_cable1/",
        ],
        "level5" => &[
            "Level Elements/Bathroom/bathroom bird toy chain/",
            "Level Elements/By The Door/ceiling to floor rope coil/",
            "Missions - Bedroom 2/Streaming Profits/Mission Enabled/Contents/bed to waterfall money line/",
            "Missions - Bedroom 2/Streaming Profits/Mission Enabled/Contents/couch to waterfall money line/",
            "Missions - Bedroom 2/Streaming Profits/bed to waterfall no-money line/",
            "Missions - Bedroom 2/Streaming Profits/couch to waterfall no-money line/",
        ],
        "level12" => &[
            "Level Elements/Second Level/Bird added ropes/Rope After Mission/ceiling to floor rope coil/",
            "Level Elements/Second Level/Bird added ropes/Rope During Mission/ceiling to floor rope coil/",
            "Non-Static Level Elements/Bird Added Zipline 1/",
            "Non-Static Level Elements/Bird Added Zipline 2/",
        ],
        "level13" => &[
            "Level Contents/Connecting Grinds Etc Main Area/SFox_Grind_Link1 (1)/",
            "Level Contents/Connecting Grinds Etc Main Area/SFox_Grind_Link1/",
            "Level Contents/Connecting Grinds Etc Main Area/SFox_Grind_Link2 (1)/",
            "Level Contents/Connecting Grinds Etc Main Area/SFox_Grind_Link2/",
            "Level Contents/Connecting Grinds Etc Main Area/SFox_Grind_Link4 cubicle bowl side/",
            "Level Contents/Connection To High Roof/SFox_Grind_Link2 (2)/",
            "Level Contents/Connection To High Roof/SFox_Grind_Link2 (3)/",
            "Level Contents/Heaven Is A Halfpipe/Book Based Funbox/SFox_Grind_Link2 (3)/",
            "Level Contents/Heaven Is A Halfpipe/Book Based Funbox/SFox_Grind_Link2 (4)/",
            "Level Contents/Heaven Is A Halfpipe/SFox_Grind_Link2 (3)/",
            "Level Contents/Heaven Is A Halfpipe/SFox_Grind_Link2 (6)/",
            "Level Contents/Heaven Is A Halfpipe/SFox_Grind_Link4 cubicle bowl side (2)/",
            "Level Contents/Ivory Mega Ramp Maze Exterior/SFox_Grind_Link4 cubicle bowl side (1)/",
            "Level Contents/Pizza Box Ramp Far End Surrounds/Golden Grid Cable Big Loop/sfox_grind_cable1/",
            "Level Contents/Pizza Box Ramp Surrounds/Golden Cable Half 1/sfox_grind_cable1 (3)/",
            "Level Contents/Pizza Box Ramp Surrounds/Golden Cable Half 2/sfox_grind_cable1 (2)/",
            "Level Contents/Simple Magazine Half Pipes Surrounds/Grind Cable Short Hoop (1)/sfox_grind_cable1/",
            "Level Contents/Simple Magazine Half Pipes Surrounds/Grind Cable Short Hoop/sfox_grind_cable1/",
        ],
        _ => &[],
    }
}

/// The window glass, drawn clear (Caleb, 2026-10-19, of frosted windows: "Fully clear windows"; "just the big windows
/// clear"): the bedrooms' ClearGlass.
pub fn window_glass() -> BlendedSelection {
    BlendedSelection::Named(BTreeSet::from(["ClearGlass".to_owned()]))
}

/// What the pause map's overhead view leaves out: an indoor level's walls and ceiling (which hide the room from
/// above), its floor and the structures that cast no shadow kept; nothing in the outdoor levels.
pub fn pause_leave_out(level: &str) -> PauseLeaveOut {
    let prefixes: &[&str] = match level {
        "level1" => &["Level Elements/Bounds/"],
        "level3" => &[
            "Level Bounds/Longer Wall",
            "Level Bounds/Shorter Wall",
            "Level Elements/Ceiling Lights/",
        ],
        "level4" => &["Room Bounds/Wall"],
        "level5" => &["Bounds/"],
        "level7" => &["Brick Bedroom/Bounds/"],
        _ => &[],
    };
    if prefixes.is_empty() {
        PauseLeaveOut::Nothing
    } else {
        PauseLeaveOut::Placements(prefixes.iter().map(|p| (*p).to_owned()).collect())
    }
}

/// The Studio compile of a level's map: resident (`--no-stream`: these levels are small, so no WorldPartition cells
/// and none of Studio's own vista proxies, which hung the GPU on large maps), the game's own geometry at every distance
/// (`--no-lods`: SkateBIRD has no LODs, Studio's would be decimations), no baked GI, compiled at noon (Caleb,
/// 2026-10-20: "default tod is noon, noon is fine").
pub fn studio_compile() -> CompileChoices {
    CompileChoices {
        streaming: Some(Streaming::Resident),
        lods: false,
        gi: false,
        time_of_day: Some(TimeOfDay::Noon),
        pause_map: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fallback_spawn_faces_the_games_yaw() {
        let (p, yaw) = fallback_spawn("level7").unwrap();
        assert_eq!(p, [0.0, 1.189, 0.0]);
        assert!((yaw - 7.0659).abs() < 1e-9);
        assert!(fallback_spawn("level2").is_none());
    }

    #[test]
    fn indoor_levels_have_a_shell_and_outdoor_ones_none() {
        for l in ["level1", "level3", "level4", "level5", "level7"] {
            assert!(shell_casts_no_shadow(l).unwrap().is_some(), "{l}");
        }
        for l in ["level2", "level12", "level13"] {
            assert!(shell_casts_no_shadow(l).unwrap().is_none(), "{l}");
            assert_eq!(pause_leave_out(l), PauseLeaveOut::Nothing, "{l}");
        }
    }
}
