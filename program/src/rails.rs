//! `keep-skatebird-line-rails{groups=G;G...}`: of SkateBIRD's grind rails (the `GrindableRail` chains primase's reader
//! states as `core.curve` of kind `rail`, each named by its first node's GameObject path), only those of the stated
//! groups stay rails; the others lose their curve, counted. The program's own pass, ahead of primase.
//!
//! Why: skate.'s grind prism is solid and hung under the line, so a rail on a ramp's coping pokes out of the ramp face at
//! the lip, where riders hit it going up (Caleb, 2026-10-20, of 0.3.0). SkateBIRD's rails are snap points with no
//! collision, and its data does not tell a coping from a rope: its `isTwoSided` / `isRaisedPipe` flags are left at their
//! defaults in most levels, and copings, ledges and ropes alike may have no collider of their own. What the game does
//! tell is what each rail runs along: a line it draws (`CurvedLine3D`), a chain (`SFox_Grind_Link`), or an object it
//! names a wire, cable, rope, chain, zipline or line. Those groups, per level, are [`crate::choices::line_rails`]
//! (Caleb: "just detect the actual ziplines/chains and only grind rail those").

use std::collections::BTreeSet;

use primase_core::pass::{Options, Pass, PassLog};
use primase_core::{Component, Curve, Name, World};

/// See the module documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeepSkatebirdLineRails {
    /// Prefixes of the rails' names (bare-`/` GameObject paths, each ending in `/`).
    pub groups: BTreeSet<String>,
}

impl Pass for KeepSkatebirdLineRails {
    fn name(&self) -> &'static str {
        "keep-skatebird-line-rails"
    }
    fn options(&self) -> Options {
        Options::from([(
            "groups".to_owned(),
            self.groups.iter().cloned().collect::<Vec<_>>().join(";"),
        )])
    }
    fn removes(&self) -> &[&'static str] {
        &[Curve::NAME]
    }
    fn run(&self, w: &mut World, log: &mut PassLog) -> Result<(), String> {
        let rails: Vec<(primase_core::Entity, String)> = w
            .iter::<Curve>()
            .filter(|(_, c)| c.kind == "rail")
            .map(|(e, _)| (e, w.get::<Name>(e).map(|n| n.0.clone()).unwrap_or_default()))
            .collect();
        let mut matched: BTreeSet<&str> = BTreeSet::new();
        for (e, name) in rails {
            if let Some(g) = self.groups.iter().find(|g| name.starts_with(g.as_str())) {
                matched.insert(g);
                log.count("rails kept: a rope, cable, chain or zipline", 1);
            } else {
                w.remove::<Curve>(e);
                log.record_removed(Curve::NAME);
                log.omit("rails left out: not a rope, cable, chain or zipline", 1);
            }
        }
        // a group only one mission state has (the clip line during and after its mission) is gone once the state is
        // resolved: counted, not refused
        let gone = self.groups.iter().filter(|g| !matched.contains(g.as_str())).count();
        if gone > 0 {
            log.count("stated groups with no rail in the level as stored", gone as u64);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rail(w: &mut World, name: &str) -> primase_core::Entity {
        let e = w.spawn();
        w.insert(
            e,
            Curve {
                kind: "rail".into(),
                points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                closed: false,
                tangents: None,
                up: None,
            },
        );
        w.insert(e, Name(name.into()));
        e
    }

    #[test]
    fn only_the_stated_groups_stay_rails() {
        let mut w = World::new();
        let rope = rail(&mut w, "Partially Dynamic Level Elements/Rope 1/Extra Rail 2");
        let coping = rail(&mut w, "Level Elements/Under Main Table/quarter - 3_5h 4d 5w - cardboard (20)/Rail");
        let pass = KeepSkatebirdLineRails {
            groups: BTreeSet::from(["Partially Dynamic Level Elements/Rope 1/".to_owned()]),
        };
        let mut log = PassLog::default();
        pass.run(&mut w, &mut log).unwrap();
        assert!(w.get::<Curve>(rope).is_some());
        assert!(w.get::<Curve>(coping).is_none());
    }

    #[test]
    fn a_group_with_no_rail_is_counted() {
        let mut w = World::new();
        let rope = rail(&mut w, "A/Rope/1");
        let pass = KeepSkatebirdLineRails {
            groups: BTreeSet::from(["B/Cable/".to_owned()]),
        };
        let mut log = PassLog::default();
        pass.run(&mut w, &mut log).unwrap();
        assert!(w.get::<Curve>(rope).is_none());
        assert_eq!(log.counts["stated groups with no rail in the level as stored"], 1);
    }
}
