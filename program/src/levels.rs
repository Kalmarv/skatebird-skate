//! The SkateBIRD levels the program converts, and the names their maps, mods and zips go by. The conversion and the
//! tooling (`tools/skatebird-tools`) both name levels this way.

/// A level the program converts: its scene file's name (`level2`) and the game's name for it (`rooftop`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub id: &'static str,
    pub name: &'static str,
}

/// SkateBIRD's skateable scenes (BuildSettings 1-5, 7, 12, 13) and the game's names for them.
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

/// The levels of [`LEVELS`] named in `wanted` (all when it is empty).
pub fn chosen(wanted: &[String]) -> Result<Vec<Level>, String> {
    if wanted.is_empty() {
        return Ok(LEVELS.to_vec());
    }
    wanted
        .iter()
        .map(|id| {
            LEVELS.iter().find(|l| l.id == id).copied().ok_or_else(|| {
                let ids: Vec<&str> = LEVELS.iter().map(|l| l.id).collect();
                format!("no level {id} (the program converts {ids:?})")
            })
        })
        .collect()
}

/// The map's (and the mod folder's, and the zip's) name for a level, by the game's name for it: `SkateBIRD_Rooftop`,
/// `SkateBIRD_ServerRoom` (Caleb, 2026-10-19: the scene numbers confused).
pub fn map_name(level: &str) -> String {
    let name: String = LEVELS
        .iter()
        .find(|l| l.id == level)
        .map_or(level.to_owned(), |l| {
            l.name
                .split(' ')
                .map(|w| {
                    let mut c = w.chars();
                    c.next()
                        .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
                })
                .collect()
        });
    format!("SkateBIRD_{name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_program_converts_eight_levels() {
        assert_eq!(chosen(&[]).unwrap().len(), 8);
        assert!(chosen(&["level6".into()]).is_err());
    }

    #[test]
    fn maps_are_named_by_the_games_level_names() {
        assert_eq!(map_name("level2"), "SkateBIRD_Rooftop");
        assert_eq!(map_name("level4"), "SkateBIRD_ServerRoom");
        assert_eq!(map_name("level1"), "SkateBIRD_Bedroom1");
    }
}
