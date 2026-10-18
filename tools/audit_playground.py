"""Offline normalized leaf records and source custom-shader inventory."""
import collections
import json
import struct
from pathlib import Path
from PIL import Image
import numpy as np

root = Path(__file__).resolve().parents[1]
folder = root / 'out/alpha-fix/level12/map/SkateBIRD_level12'
doc = json.loads((folder / 'map.json').read_text())
leaves = {k: v for k, v in doc['materials'].items() if 'branch_tex' in k}
records = []
def visit(value, path=''):
    if isinstance(value, dict):
        if any(isinstance(v, str) and v in leaves for v in value.values()):
            records.append({'path': path, 'record': value})
        for k, v in value.items():
            visit(v, path + '/' + str(k))
    elif isinstance(value, list):
        for i, v in enumerate(value):
            visit(v, path + '/' + str(i))
visit(doc)
geometry = []
for record in records:
    obj = record['record']
    raw = (folder / obj['mesh']).read_bytes()
    n = struct.unpack_from('<I', raw, 12)[0]
    gltf = json.loads(raw[20:20+n])
    binary = raw[28+n:]
    def accessor(idx):
        a = gltf['accessors'][idx]
        v = gltf['bufferViews'][a['bufferView']]
        width = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}[a['type']]
        dtype = {5121: 'u1', 5123: '<u2', 5125: '<u4', 5126: '<f4'}[a['componentType']]
        item = np.dtype(dtype).itemsize
        return np.ndarray((a['count'], width), dtype=dtype, buffer=binary,
            offset=v.get('byteOffset', 0)+a.get('byteOffset', 0),
            strides=(v.get('byteStride', width*item), item)).copy()
    part = gltf['meshes'][0]['primitives'][obj['primitive']]
    pos = accessor(part['attributes']['POSITION'])
    indices = accessor(part['indices']).reshape(-1, 3)
    geometry.append({'object': obj['name'], 'vertices': len(pos), 'triangles': len(indices),
        'bounds_local': [pos.min(axis=0).tolist(), pos.max(axis=0).tolist()],
        'uv_bounds': [accessor(part['attributes']['TEXCOORD_0']).min(axis=0).tolist(),
                      accessor(part['attributes']['TEXCOORD_0']).max(axis=0).tolist()]})
textures = {}
for mat in leaves.values():
    p = folder / mat['texture']
    im = Image.open(p).convert('RGBA')
    hist = im.getchannel('A').histogram()
    textures[mat['texture']] = {'size': im.size, 'pixels': im.width * im.height,
        'alpha_zero': hist[0], 'alpha_nonzero': sum(hist[1:]), 'alpha_ge_128': sum(hist[128:])}
source = json.loads((root / 'out/visual-source-level12.json').read_text())
custom = {k: v for k,v in source['materials'].items()
          if v['shader'] in ('Bumped Specular Voronoi','Shiny/Unlit Two Tone') or v['name'] == 'branch_tex'}
sections = json.loads((root / 'out/playground-sections.json').read_text())
report = {'map': str(folder / 'map.json'), 'leaf_materials': leaves, 'leaf_records': records,
          'geometry': geometry,
          'textures': textures, 'source_materials': custom,
          'compiled_leaf_named_sections': [r for r in sections if 't_0_0___section_39_' in r['mesh'].lower()],
          'map_top_keys': list(doc)}
(root / 'out/playground-leaf-audit.json').write_text(json.dumps(report, indent=2)+'\n')
print('map keys', list(doc))
print('leaf records', json.dumps(records)[:6000])
print('texture alpha', textures)
print('compiled branch-named sections', len(report['compiled_leaf_named_sections']))
print('compiled keys', sorted({r['key'] for r in report['compiled_leaf_named_sections']}))
print('geometry', geometry)
print('source materials', [(k,v['name'],v['shader']) for k,v in custom.items()])
for k, m in custom.items():
    print(m['name'], 'keywords', m['keywords'], 'slots',
        {s: v for s,v in m['slots'].items() if v['texture']}, 'exported', m['exported'])
print('mesh sample', [r['mesh'] for r in sections[:5]])
