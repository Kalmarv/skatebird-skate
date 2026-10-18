"""Inspect the earlier Sledding export; do not infer its lineage from main."""
import collections
import json
import hashlib
import zipfile
from pathlib import Path
from PIL import Image
root = Path(__file__).resolve().parents[1]
lab = Path('C:/Users/kalmarv/modding/work/sledding/lab/v2')
doc = json.loads((lab / 'out/manifest.json').read_text())
mats = doc['materials']
trees = {k:v for k,v in mats.items() if 'tree' in v.get('name','').lower() or 'speedtree' in v.get('shader','').lower()}
alpha = {k:v for k,v in mats.items() if v.get('alpha','opaque') != 'opaque'}
sections = json.loads((root / 'out/sledding-sections.json').read_text())
matches = [r for r in sections if any(x in r['mesh'].lower() for x in ('tree','evergreen','pine'))]
out = {'manifest': str(lab / 'out/manifest.json'), 'top_keys': list(doc),
       'tree_materials': trees, 'nonopaque_materials': alpha, 'tree_named_sections': matches,
       'alpha_material_counts': dict(collections.Counter(m.get('alpha','opaque') for m in mats.values()))}
out['tree_texture_alpha'] = {}
for key, mat in trees.items():
    im = Image.open(lab / 'out/textures' / mat['texture']).convert('RGBA')
    hist = im.getchannel('A').histogram()
    out['tree_texture_alpha'][key] = {'size': im.size, 'alpha_255': hist[255], 'pixels': im.width*im.height}
archive = lab.parent / 'release/Kalmarv-Sledding_Game_Mountain-0.2.1.zip'
out['release_comparison'] = {'archive': str(archive), 'files': []}
with zipfile.ZipFile(archive) as z:
    out['release_comparison']['member_sample'] = z.namelist()[:12]
    for rel in ('layout.toc', 'Win32/levels/game/dingolevel_reskate_sledding/dingolevel_reskate_sledding.toc'):
        candidates = [name for name in z.namelist() if name.replace('\\','/') == rel or name.replace('\\','/').endswith('/'+rel)]
        for name in candidates:
            local = lab / 'stage/Patch' / rel
            out['release_comparison']['files'].append({'name': name, 'matches_stage':
                hashlib.sha256(z.read(name)).digest() == hashlib.sha256(local.read_bytes()).digest()})
(root / 'out/sledding-reference-audit.json').write_text(json.dumps(out,indent=2)+'\n')
print('top keys', list(doc))
print('tree materials', json.dumps(trees))
print('alpha counts', out['alpha_material_counts'], 'tree texture', out['tree_texture_alpha'])
print('release identity', out['release_comparison'])
print('tree named section rows', len(matches), 'sample', matches[:3])
for key,v in doc.items():
    if key not in ('materials',) and isinstance(v,list):
        print(key, len(v), 'sample',str(v[:1])[:1600])
