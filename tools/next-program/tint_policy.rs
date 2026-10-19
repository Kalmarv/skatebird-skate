//! Program composition: lower vertex tint only for a shader verified to consume it.
//! Install after raise-skate, before prepare_scene_assets. Kept outside the frozen
//! queued UV candidate until its single-variable office proof has completed.
use primase_core::pass::{Pass, PassLog};
use primase_core::{Entity, MaterialVariants, Mesh, World};
use primase_reskate::dialect::SkateMaterial;
use primase_unity::components::UnityMaterial;

fn consumes_colour(world: &World, material: Entity) -> bool {
    world
        .get::<UnityMaterial>(material)
        .is_some_and(|m| m.shader.as_deref() == Some("Houdini/SpecularVertexColor"))
}

pub fn lower_verified_tints(world: &mut World) -> Result<PassLog, String> {
    // A variant can use a different shader. Refuse mixed semantics rather than
    // let the target splitter silently apply the default shader's rule to it.
    for (entity, mesh) in world.iter::<Mesh>() {
        if let Some(variants) = world.get::<MaterialVariants>(entity) {
            for (i, primitive) in mesh.primitives().iter().enumerate() {
                let selected = primitive
                    .material
                    .is_some_and(|m| consumes_colour(world, m));
                if variants
                    .0
                    .get(i)
                    .is_some_and(|map| map.values().any(|m| consumes_colour(world, *m) != selected))
                {
                    return Err(format!(
                        "{entity:?} primitive {i}: variants disagree on vertex-colour semantics"
                    ));
                }
            }
        }
    }
    // The existing target pass selects primitives through SkateMaterial. Mask
    // unsupported materials for this call, preserving every source stream and
    // component. Restore them on success AND on refusal by the target pass.
    let excluded: Vec<_> = world
        .iter::<SkateMaterial>()
        .filter(|(e, _)| !consumes_colour(world, *e))
        .map(|(e, _)| e)
        .collect();
    let saved: Vec<_> = excluded
        .into_iter()
        .map(|e| {
            (
                e,
                world.remove::<SkateMaterial>(e).expect("selected material"),
            )
        })
        .collect();
    let mut log = PassLog::default();
    let result = primase_reskate::tint::LowerTintReskate::uncapped().run(world, &mut log);
    for (entity, material) in saved {
        world.insert(entity, material);
    }
    result?;
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primase_core::{Primitive, Semantic, Stream, Topology};
    use primase_reskate::dialect::{MaterialClass, Part, SkateParts};
    use primase_unity::components::UnityObject;
    use std::collections::BTreeMap;

    fn material(world: &mut World, shader: &str) -> Entity {
        let entity = world.spawn();
        world.insert(
            entity,
            UnityMaterial {
                material: UnityObject {
                    file: "fixture".into(),
                    path_id: 1,
                },
                shader: Some(shader.into()),
                shader_state: vec![],
                textures: vec![],
                floats: vec![],
                ints: vec![],
                colors: vec![],
                keywords: vec![],
                tags: vec![],
                render_queue: -1,
            },
        );
        world.insert(
            entity,
            SkateMaterial {
                key: shader.into(),
                class: MaterialClass::Opaque,
                base: None,
                normal: None,
                mask: None,
                base_color: None,
                tint: None,
                alpha: None,
                cutoff: None,
                two_sided: None,
                domain: None,
                metallic: None,
                roughness: None,
                wrap: None,
                ocean: None,
                cast_shadows: None,
                emissive: None,
                vertex_alpha: None,
                other: Default::default(),
            },
        );
        entity
    }

    #[test]
    fn splits_houdini_preserves_standard_and_remaps_parts() {
        let mut world = World::new();
        let houdini = material(&mut world, "Houdini/SpecularVertexColor");
        let standard = material(&mut world, "Standard");
        let primitive = |m| Primitive {
            topology: Topology::Triangles,
            indices: None,
            material: Some(m),
            streams: BTreeMap::from([
                (Semantic::Position, Stream::F32x3(vec![[0., 0., 0.]; 6])),
                (
                    Semantic::Color(0),
                    Stream::F32x4(vec![
                        [0.5, 1., 1., 1.],
                        [0.5, 1., 1., 1.],
                        [0.5, 1., 1., 1.],
                        [1.; 4],
                        [1.; 4],
                        [1.; 4],
                    ]),
                ),
            ]),
        };
        let unchanged = primitive(standard);
        let mesh = world.spawn();
        world.insert(mesh, Mesh::new(vec![primitive(houdini), unchanged.clone()]));
        world.insert(mesh, SkateParts(vec![Part::default(); 2]));
        let log = lower_verified_tints(&mut world).unwrap();
        let parts = world.get::<Mesh>(mesh).unwrap().primitives();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[2], unchanged);
        assert_eq!(world.get::<SkateParts>(mesh).unwrap().0.len(), 3);
        assert_eq!(world.get::<SkateMaterial>(standard).unwrap().tint, None);
        assert_eq!(
            world
                .get::<SkateMaterial>(parts[0].material.unwrap())
                .unwrap()
                .tint,
            Some([188, 255, 255])
        );
        assert_eq!(log.counts["tinted primitives split by colour"], 1);
    }
}
