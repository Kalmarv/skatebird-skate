//! SkateBIRD program policy for the target's single texture-coordinate stream.
//! Preserve albedo tiling. Other slots with a different sampling transform adopt it,
//! with a material/slot-specific approximation; never silently erase albedo transforms.
use primase_core::pass::PassLog;
use primase_core::pbr::{PbrMaterial, TextureRef, UvTransform};
use primase_core::{MaterialVariants, Mesh, Name, World};
use std::collections::BTreeSet;

fn slots(m: &mut PbrMaterial) -> Vec<(&'static str, &mut Option<TextureRef>)> {
    let mut result = vec![
        ("base", &mut m.base_color_texture),
        ("normal", &mut m.normal_texture),
        ("metallic-roughness", &mut m.metallic_roughness_texture),
        ("occlusion", &mut m.occlusion_texture),
        ("emissive", &mut m.emissive_texture),
    ];
    if let Some(s) = m.specular.as_mut() {
        result.push(("specular", &mut s.texture));
        result.push(("specular-color", &mut s.color_texture));
    }
    result
}

pub fn preserve_base_tiling(world: &mut World) -> Result<PassLog, String> {
    let mut log = PassLog::default();
    let materials: Vec<_> = world
        .iter::<PbrMaterial>()
        .map(|(e, m)| (e, m.clone()))
        .collect();
    for (entity, mut material) in materials {
        let anchor = material
            .base_color_texture
            .or_else(|| material.textures().next());
        let Some(anchor) = anchor else { continue };
        if anchor.texcoord != 0 {
            return Err(format!("material {entity:?}: target policy requires UV0"));
        }
        let label = world
            .get::<Name>(entity)
            .map(|n| n.0.as_str())
            .unwrap_or("unnamed");
        for (name, slot) in slots(&mut material) {
            let Some(slot) = slot else { continue };
            if slot.texcoord != 0 {
                return Err(format!("{label}/{name}: target policy requires UV0"));
            }
            if slot.transform.unwrap_or(UvTransform::IDENTITY)
                != anchor.transform.unwrap_or(UvTransform::IDENTITY)
            {
                log.approximate(&format!("{label}/{name}: distinct tiling replaced by base tiling (single target UV set)"), 1);
            }
            slot.transform = anchor.transform;
        }
        world.insert(entity, material);
    }
    let baked = primase_meshops::bake_texture_transforms(world);
    for (key, count) in baked.counts {
        log.count(&key, count);
    }
    for (key, count) in baked.approximations {
        log.approximate(&key, count);
    }
    let mut used = BTreeSet::new();
    for (entity, mesh) in world.iter::<Mesh>() {
        for p in mesh.primitives() {
            used.extend(p.material);
        }
        if let Some(variants) = world.get::<MaterialVariants>(entity) {
            for part in &variants.0 {
                used.extend(part.values().copied());
            }
        }
    }
    for entity in used {
        if let Some(m) = world.get::<PbrMaterial>(entity)
            && m.textures()
                .any(|r| r.transform.is_some_and(|t| t != UvTransform::IDENTITY))
        {
            let name = world
                .get::<Name>(entity)
                .map(|n| n.0.as_str())
                .unwrap_or("unnamed");
            let placed: BTreeSet<_> = world
                .iter::<primase_core::Instance>()
                .map(|(_, instance)| instance.prototype)
                .collect();
            let mut uses = Vec::new();
            for (mesh_entity, mesh) in world.iter::<Mesh>() {
                let variants = world.get::<MaterialVariants>(mesh_entity);
                for (i, primitive) in mesh.primitives().iter().enumerate() {
                    let variant_use = variants
                        .and_then(|v| v.0.get(i))
                        .is_some_and(|v| v.values().any(|m| *m == entity));
                    if primitive.material == Some(entity) || variant_use {
                        uses.push(format!(
                            "{mesh_entity:?}/{i}: {:?}, placed={}, uv0={}, default={:?}, variants={:?}",
                            primitive.topology,
                            placed.contains(&mesh_entity),
                            primitive
                                .streams
                                .get(&primase_core::Semantic::TexCoord(0))
                                .map_or("absent", |s| {
                                    if matches!(s, primase_core::Stream::F32x2(_)) {
                                        "f32x2"
                                    } else {
                                        "other"
                                    }
                                }),
                            primitive.material,
                            variants.and_then(|v| v.0.get(i))
                        ));
                    }
                }
            }
            return Err(format!(
                "{name}: base tiling still unbaked; refusing to erase it. Bake counts: {:?}. Uses: {uses:?}",
                log.counts
            ));
        }
    }
    Ok(log)
}
