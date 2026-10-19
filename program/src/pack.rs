//! `pack-orm`: one occlusion/roughness/metallic image from a reader's separate occlusion and metallic-roughness
//! textures.
//!
//! A program step, run by the pass manager: copied unchanged from the ds-modding fork's `formats/skate-scene`
//! (primase-work archive/ds-skatebird 7b81a686), which primase main does not have; a candidate to promote into primase.
//!
//! glTF-style readers keep occlusion in its own texture (R, re-encoded from Unity's G) and metallic (B) with
//! roughness (G) in another. skate's compiled material has one MSK image, so `raise_skate` needs both slots to name
//! the same texture: R occlusion, G roughness, B metallic, A 255. Materials sharing a pair share the packed texture.
//! Textures of different sizes are resampled nearest to the metallic-roughness texture's size (counted). Both slots
//! must sample the same coordinate set and wrap; a differing layout is refused, not guessed.

use std::collections::BTreeMap;

use primase_core::pass::{Pass, PassLog};
use primase_core::pbr::PbrMaterial;
use primase_core::{ColorSpace, Entity, TexFormat, TexRole, Texture, World};

/// Pack every material's separate occlusion and metallic-roughness textures.
pub(crate) fn pack_orm(world: &mut World) -> Result<PassLog, String> {
    let mut log = PassLog::default();
    let materials: Vec<(Entity, PbrMaterial)> = world
        .iter::<PbrMaterial>()
        .map(|(e, m)| (e, m.clone()))
        .collect();
    // (metallic-roughness texture, occlusion texture) -> packed texture
    let mut cache: BTreeMap<(Entity, Entity), Entity> = BTreeMap::new();
    let mut packed = 0u64;
    let mut resampled = 0u64;
    for (e, m) in materials {
        let (Some(mr), Some(ao)) = (m.metallic_roughness_texture, m.occlusion_texture) else {
            continue;
        };
        if mr.texture == ao.texture {
            continue;
        }
        if mr.texcoord != ao.texcoord || mr.wrap != ao.wrap {
            return Err(format!(
                "material {e:?}: occlusion and metallic-roughness sample different coordinate sets or wraps"
            ));
        }
        let tex = if let Some(&t) = cache.get(&(mr.texture, ao.texture)) {
            t
        } else {
            // the packed texture is built from borrowed source pixels, then the world is mutated to hold it
            let (width, height, levels) = {
                let src_mr = world.get::<Texture>(mr.texture).ok_or_else(|| {
                    format!("material {e:?}: metallic-roughness texture has no pixels")
                })?;
                let src_ao = world
                    .get::<Texture>(ao.texture)
                    .ok_or_else(|| format!("material {e:?}: occlusion texture has no pixels"))?;
                for t in [src_mr, src_ao] {
                    if t.format != TexFormat::Rgba8Unorm || t.color_space != ColorSpace::Linear {
                        return Err(format!(
                            "material {e:?}: mask textures must be linear RGBA8 before packing"
                        ));
                    }
                }
                let mut levels = Vec::with_capacity(src_mr.levels.len());
                for (i, mr_level) in src_mr.levels.iter().enumerate() {
                    let (w, h) = (
                        (src_mr.width >> i).max(1) as usize,
                        (src_mr.height >> i).max(1) as usize,
                    );
                    let ao_level = src_ao.levels.get(i);
                    let (aw, ah) = (
                        (src_ao.width >> i).max(1) as usize,
                        (src_ao.height >> i).max(1) as usize,
                    );
                    if ao_level.is_some() && (aw != w || ah != h) {
                        resampled += 1;
                    }
                    let mut out = Vec::with_capacity(w * h * 4);
                    for y in 0..h {
                        for x in 0..w {
                            let at = (y * w + x) * 4;
                            let mr_px = &mr_level[at..at + 4];
                            let occlusion = match ao_level {
                                Some(a) if aw == w && ah == h => a[at],
                                Some(a) => {
                                    let (sx, sy) = (x * aw / w, y * ah / h);
                                    let oat = (sy * aw + sx) * 4;
                                    a.get(oat).copied().unwrap_or(255)
                                }
                                None => 255,
                            };
                            out.extend_from_slice(&[occlusion, mr_px[1], mr_px[2], 255]);
                        }
                    }
                    levels.push(out);
                }
                (src_mr.width, src_mr.height, levels)
            };
            let t = world.spawn();
            world.insert(
                t,
                Texture {
                    width,
                    height,
                    format: TexFormat::Rgba8Unorm,
                    levels,
                    color_space: ColorSpace::Linear,
                    role: TexRole::Mask,
                    normal: None,
                },
            );
            cache.insert((mr.texture, ao.texture), t);
            packed += 1;
            t
        };
        let mut mr = mr;
        mr.texture = tex;
        let mut ao = ao;
        ao.texture = tex;
        if let Some(m) = world.get_mut::<PbrMaterial>(e) {
            m.metallic_roughness_texture = Some(mr);
            m.occlusion_texture = Some(ao);
        }
        log.count(
            "materials given a packed occlusion/roughness/metallic texture",
            1,
        );
    }
    if packed > 0 {
        log.count("packed ORM textures", packed);
    }
    if resampled > 0 {
        log.approximate(
            "occlusion levels resampled to the metallic-roughness texture's size",
            resampled,
        );
    }
    Ok(log)
}

/// [`pack_orm`] as a pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackOrm;

impl Pass for PackOrm {
    fn name(&self) -> &'static str {
        "pack-orm"
    }
    fn requires(&self) -> &[&'static str] {
        &["pbr.material"]
    }
    fn modifies(&self) -> &[&'static str] {
        &["pbr.material"]
    }
    /// The packed image is a new texture entity.
    fn provides(&self) -> &[&'static str] {
        &["core.texture"]
    }
    fn approximates(&self) -> &'static str {
        "an occlusion map of another size is resampled nearest to the metallic-roughness texture"
    }
    fn run(&self, world: &mut World, log: &mut PassLog) -> Result<(), String> {
        let r = pack_orm(world)?;
        for (k, n) in r.counts {
            log.count(&k, n);
        }
        for (k, n) in r.approximations {
            log.approximate(&k, n);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primase_core::pbr::TextureRef;

    fn px(w: u32, h: u32, f: impl Fn(usize) -> [u8; 4]) -> Texture {
        let n = (w * h) as usize;
        Texture {
            width: w,
            height: h,
            format: TexFormat::Rgba8Unorm,
            levels: vec![(0..n).flat_map(f).collect()],
            color_space: ColorSpace::Linear,
            role: TexRole::Mask,
            normal: None,
        }
    }

    #[test]
    fn occlusion_moves_to_red_and_roughness_and_metallic_are_kept() {
        let mut w = World::new();
        let mr = w.spawn();
        w.insert(mr, px(2, 1, |i| [0, 100 + i as u8, 50 + i as u8, 255]));
        let ao = w.spawn();
        w.insert(ao, px(2, 1, |i| [200 + i as u8, 0, 0, 255]));
        let e = w.spawn();
        let m = PbrMaterial {
            metallic_roughness_texture: Some(TextureRef::new(mr)),
            occlusion_texture: Some(TextureRef::new(ao)),
            ..Default::default()
        };
        w.insert(e, m);
        pack_orm(&mut w).unwrap();
        let m = w.get::<PbrMaterial>(e).unwrap();
        let (a, b) = (
            m.metallic_roughness_texture.unwrap(),
            m.occlusion_texture.unwrap(),
        );
        assert_eq!(a.texture, b.texture);
        let t = w.get::<Texture>(a.texture).unwrap();
        assert_eq!(t.levels[0], vec![200, 100, 50, 255, 201, 101, 51, 255]);
    }

    #[test]
    fn a_shared_pair_is_packed_once() {
        let mut w = World::new();
        let mr = w.spawn();
        w.insert(mr, px(1, 1, |_| [0, 10, 20, 255]));
        let ao = w.spawn();
        w.insert(ao, px(1, 1, |_| [30, 0, 0, 255]));
        for _ in 0..3 {
            let e = w.spawn();
            let m = PbrMaterial {
                metallic_roughness_texture: Some(TextureRef::new(mr)),
                occlusion_texture: Some(TextureRef::new(ao)),
                ..Default::default()
            };
            w.insert(e, m);
        }
        let log = pack_orm(&mut w).unwrap();
        assert_eq!(log.counts.get("packed ORM textures"), Some(&1));
        assert_eq!(
            log.counts
                .get("materials given a packed occlusion/roughness/metallic texture"),
            Some(&3)
        );
    }
}
