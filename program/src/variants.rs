//! Give differently textured placements independent geometry before baking UVs; a prototype nobody places keeps
//! only its default materials.
use primase_core::pass::PassLog;
use primase_core::{Entity, Instance, Lods, MaterialVariants, Mesh, MeshFile, Name, World};
use std::collections::BTreeMap;

pub fn specialize(world: &mut World) -> Result<PassLog, String> {
    let mut placements: BTreeMap<Entity, BTreeMap<u32, Vec<Entity>>> = BTreeMap::new();
    for (e, i) in world.iter::<Instance>() {
        if world.has::<MaterialVariants>(i.prototype) {
            placements
                .entry(i.prototype)
                .or_default()
                .entry(i.variant)
                .or_default()
                .push(e);
        }
    }
    // Validate before changing geometry or placements.
    for prototype in placements.keys() {
        let mesh = world
            .get::<Mesh>(*prototype)
            .ok_or("variant prototype has no loaded mesh")?;
        let variants = world.get::<MaterialVariants>(*prototype).unwrap();
        if variants.0.len() != mesh.primitives().len() || world.has::<Lods>(*prototype) {
            return Err(format!(
                "{prototype:?}: variant specialization needs matching parts and no LOD graph"
            ));
        }
    }
    let mut log = PassLog::default();
    // Variants are per-placement choices: a prototype nobody places chooses none, and keeping them would join its
    // materials into one texture-coordinate group with every alternative's tiling.
    let unplaced: Vec<Entity> = world
        .iter::<MaterialVariants>()
        .map(|(e, _)| e)
        .filter(|e| !placements.contains_key(e))
        .collect();
    for prototype in unplaced {
        world.remove::<MaterialVariants>(prototype);
        log.approximate(
            "material variants of a prototype nobody places left out (a program choice: none is chosen)",
            1,
        );
    }
    for (prototype, used) in placements {
        let source = world.get::<Mesh>(prototype).unwrap().clone();
        let variants = world.get::<MaterialVariants>(prototype).unwrap().clone();
        let name = world.get::<Name>(prototype).map(|n| n.0.clone());
        // Clone before editing the original. Reuse the original for one used
        // variant so no unplaced collision prototype remains behind.
        let mut targets = Vec::new();
        for (index, (variant, instances)) in used.into_iter().enumerate() {
            let target = if index == 0 {
                prototype
            } else {
                let target = world.spawn();
                world.copy_components(prototype, target);
                log.count("prototype copies for material variants", 1);
                target
            };
            targets.push((target, variant, instances));
        }
        for (target, variant, instances) in targets {
            let mut mesh = source.clone();
            for (mut part, choices) in mesh.primitives_mut().zip(&variants.0) {
                if let Some(material) = choices.get(&variant) {
                    *part.material_mut() = Some(*material);
                }
            }
            world.insert(target, mesh);
            world.remove::<MaterialVariants>(target);
            world.remove::<MeshFile>(target);
            if let Some(name) = &name {
                world.insert(target, Name(format!("{name} variant {variant}")));
            }
            for instance in instances {
                world.insert(
                    instance,
                    Instance {
                        prototype: target,
                        variant: 0,
                    },
                );
                log.count("placements with resolved material variants", 1);
            }
        }
    }
    Ok(log)
}
