//! Parts without a material left out of the skate path: a Unity renderer whose material slot is empty (level5's
//! `prototype_e631` part 2, a static-batch submesh) has no surface skate can draw, and the ReSkate writer refuses a part
//! with none. Counted as left out, parts and triangles; a prototype left with no part draws nothing (no mesh).
use primase_core::pass::PassLog;
use std::collections::BTreeSet;

use primase_core::{Entity, Instance, MaterialVariants, Mesh, Name, Topology, World};

pub fn leave_out(world: &mut World) -> Result<PassLog, String> {
    let mut log = PassLog::default();
    let bare: Vec<(Entity, Vec<usize>)> = world
        .iter::<Mesh>()
        .filter_map(|(e, mesh)| {
            let variants = world.get::<MaterialVariants>(e);
            let bare: Vec<usize> = (0..mesh.primitives().len())
                .filter(|&i| bare_part(mesh, variants, i))
                .collect();
            (!bare.is_empty()).then_some((e, bare))
        })
        .collect();
    let placed: BTreeSet<Entity> = world.iter::<Instance>().map(|(_, i)| i.prototype).collect();
    for (e, bare) in bare {
        if placed.contains(&e) {
            log.count("placed prototypes with a part left out", 1);
            if std::env::var_os("BIRD_LIST_UNMATERIALED").is_some() {
                let name = world.get::<Name>(e).map_or("unnamed", |n| n.0.as_str());
                let source = world.get::<primase_core::Source>(e).map(|s| s.0.clone());
                println!("unmaterialed: {e:?} {name} {source:?} parts {bare:?}");
            }
        }
        let mesh = world.get::<Mesh>(e).expect("listed with a mesh");
        for &i in &bare {
            let p = &mesh.primitives()[i];
            log.count("parts without a material left out", 1);
            if p.topology == Topology::Triangles {
                let corners = p.indices.as_ref().map_or(p.vertex_count(), Vec::len);
                log.count(
                    "triangles without a material left out",
                    (corners / 3) as u64,
                );
            }
        }
        if bare.len() == mesh.primitives().len() {
            world.remove::<Mesh>(e);
            log.count("prototypes left with no part", 1);
            continue;
        }
        world.rebuild_primitives(e, |mesh| {
            let from: Vec<usize> = (0..mesh.primitives().len())
                .filter(|i| !bare.contains(i))
                .collect();
            Ok((
                from.iter().map(|&i| mesh.primitives()[i].clone()).collect(),
                from,
            ))
        })?;
    }
    Ok(log)
}

/// Part `i` has no material of its own and none for any variant.
fn bare_part(mesh: &Mesh, variants: Option<&MaterialVariants>, i: usize) -> bool {
    mesh.primitives()[i].material.is_none()
        && variants
            .and_then(|v| v.0.get(i))
            .is_none_or(|v| v.is_empty())
}
