//! `unify-texture-wrap`: one sampling wrap for every slot of a material.
//!
//! A program step, run by the pass manager: copied unchanged from the ds-modding fork's `formats/skate-scene`
//! (primase-work archive/ds-skatebird 7b81a686), which primase main does not have; a candidate to promote into primase.
//!
//! skate's compiled materials carry one wrap for the material's textures, so a reader's per-slot wrap modes (Unity
//! stores wrap on the texture: `_MainTex` repeat, `_BumpMap` clamp, ...) cannot be written as they are. Every slot
//! takes the base colour texture's wrap mode (per axis: `u`, then `v`; a slot with different modes per axis keeps its
//! `u`), counted as an approximation.

use primase_core::pass::PassLog;
use primase_core::pbr::PbrMaterial;
use primase_core::{Entity, World};

/// Give every texture slot of every material the base colour slot's wrap mode.
pub(crate) fn unify_texture_wrap(world: &mut World) -> PassLog {
    let mut log = PassLog::default();
    let entities: Vec<Entity> = world.iter::<PbrMaterial>().map(|(e, _)| e).collect();
    let mut changed = 0u64;
    for e in entities {
        let Some(m) = world.get_mut::<PbrMaterial>(e) else {
            continue;
        };
        // the base colour's wrap when it has a texture, else the first slot's, else nothing to unify
        let want = m
            .base_color_texture
            .map(|r| r.wrap)
            .or_else(|| m.textures().next().map(|r| r.wrap));
        let Some(want) = want else { continue };
        let want = [want[0], want[0]];
        for slot in [
            &mut m.base_color_texture,
            &mut m.metallic_roughness_texture,
            &mut m.normal_texture,
            &mut m.occlusion_texture,
            &mut m.emissive_texture,
        ] {
            let Some(r) = slot.as_mut() else { continue };
            if r.wrap != want {
                r.wrap = want;
                changed += 1;
            }
        }
        if let Some(s) = m.specular.as_mut() {
            for r in [&mut s.texture, &mut s.color_texture] {
                let Some(r) = r.as_mut() else { continue };
                if r.wrap != want {
                    r.wrap = want;
                    changed += 1;
                }
            }
        }
    }
    if changed > 0 {
        log.count("texture slots given the base colour's wrap", changed);
        log.approximate(
            "texture wrap modes that differ between a material's slots use the base colour's",
            changed,
        );
    }
    log
}

/// [`unify_texture_wrap`] as a pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnifyTextureWrap;

impl primase_core::pass::Pass for UnifyTextureWrap {
    fn name(&self) -> &'static str {
        "unify-texture-wrap"
    }
    fn requires(&self) -> &[&'static str] {
        &["pbr.material"]
    }
    fn modifies(&self) -> &[&'static str] {
        &["pbr.material"]
    }
    fn approximates(&self) -> &'static str {
        "slots whose texture wrap differs from the base colour's take the base colour's wrap"
    }
    fn run(
        &self,
        world: &mut World,
        log: &mut primase_core::pass::PassLog,
    ) -> std::result::Result<(), String> {
        let r = unify_texture_wrap(world);
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
    use primase_core::pbr::{TextureRef, Wrap};
    use primase_core::{ColorSpace, TexFormat, Texture};

    fn tex(w: &mut World) -> Entity {
        let t = w.spawn();
        w.insert(
            t,
            Texture {
                width: 1,
                height: 1,
                format: TexFormat::Rgba8Unorm,
                levels: vec![vec![0; 4]],
                color_space: ColorSpace::Srgb,
                role: primase_core::TexRole::Color,
                normal: None,
            },
        );
        t
    }

    #[test]
    fn slots_take_the_base_wrap() {
        let mut w = World::new();
        let (a, b) = (tex(&mut w), tex(&mut w));
        let e = w.spawn();
        let mut m = PbrMaterial::default();
        let mut base = TextureRef::new(a);
        base.wrap = [Wrap::Clamp, Wrap::Clamp];
        let mut normal = TextureRef::new(b);
        normal.wrap = [Wrap::Repeat, Wrap::Repeat];
        m.base_color_texture = Some(base);
        m.normal_texture = Some(normal);
        w.insert(e, m);
        let log = unify_texture_wrap(&mut w);
        let m = w.get::<PbrMaterial>(e).unwrap();
        assert_eq!(m.normal_texture.unwrap().wrap, [Wrap::Clamp, Wrap::Clamp]);
        assert_eq!(
            m.base_color_texture.unwrap().wrap,
            [Wrap::Clamp, Wrap::Clamp]
        );
        assert_eq!(
            log.counts.get("texture slots given the base colour's wrap"),
            Some(&1)
        );
    }

    #[test]
    fn a_material_without_textures_is_unchanged() {
        let mut w = World::new();
        let e = w.spawn();
        w.insert(e, PbrMaterial::default());
        let log = unify_texture_wrap(&mut w);
        assert!(log.counts.is_empty());
    }
}
