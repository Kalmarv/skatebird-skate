//! Blended surfaces drawn as alpha-tested cutouts. Studio compiles an alpha-blended material to the stock decal/blend
//! shader pair, which skate does not draw as standalone world geometry: level12's tree canopies (Standard Fade leaf
//! cards) vanish, as did rooftop's surfaces while they were wrongly blended (job 73cde79a); alpha-tested materials
//! compile to the cutout programs and draw (level12's drain grid). So every blended material that is not a decal
//! (drawn over the surface beneath it, which skate's decal path does draw) and not water becomes a cutout at alpha
//! 0.5: counted, an approximation (soft edges become hard; a mostly transparent surface such as glass mostly vanishes,
//! as it already did).
use primase_core::pass::PassLog;
use primase_core::pbr::{AlphaMode, PbrMaterial};
use primase_core::{DrawLayer, DrawOrder, MaterialUsage, Name, World};

/// The cutoff blended surfaces are tested at.
pub const CUTOFF: f32 = 0.5;

pub fn blended_as_cutout(w: &mut World) -> PassLog {
    let mut log = PassLog::default();
    let blended: Vec<_> = w
        .iter::<PbrMaterial>()
        .filter(|(_, m)| m.alpha_mode == AlphaMode::Blend)
        .map(|(e, _)| e)
        .collect();
    for e in blended {
        if DrawOrder::of(w, e).layer == DrawLayer::Decal {
            log.count("blended decals kept blended", 1);
            continue;
        }
        if w.get::<MaterialUsage>(e).copied() == Some(MaterialUsage::Water) {
            log.count("blended water kept blended", 1);
            continue;
        }
        let name = w
            .get::<Name>(e)
            .map_or_else(|| format!("{e:?}"), |n| n.0.clone());
        if let Some(m) = w.get_mut::<PbrMaterial>(e) {
            m.alpha_mode = AlphaMode::Mask { cutoff: CUTOFF };
        }
        log.approximate(
            &format!("blended surface drawn as a cutout at {CUTOFF}: {name}"),
            1,
        );
        log.count("blended surfaces drawn as cutouts", 1);
    }
    log
}
