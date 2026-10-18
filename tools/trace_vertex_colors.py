"""Trace one supported source shader's mesh colours through saved export stages."""
import json
import struct
from pathlib import Path

root = Path(__file__).resolve().parent.parent
def read_glb(path):
    b = path.read_bytes()
    n = struct.unpack_from('<I', b, 12)[0]
    return json.loads(b[20:20+n]), b[28+n:]
def colors(d, b, index):
    a = d['accessors'][index]
    v = d['bufferViews'][a['bufferView']]
    width = {'VEC3':3,'VEC4':4}[a['type']]
    code = {5121:'B',5123:'H',5126:'f'}[a['componentType']]
    fmt = '<'+code*width
    step = v.get('byteStride',struct.calcsize(fmt))
    offset = v.get('byteOffset',0)+a.get('byteOffset',0)
    return [struct.unpack_from(fmt,b,offset+i*step) for i in range(a['count'])]
d,b = read_glb(root/'out/level3/unity/level3.glb')
materials = {i:m for i,m in enumerate(d['materials']) if m.get('extras',{}).get('primase',{}).get('unity.material',{}).get('value',{}).get('shader') == 'Houdini/SpecularVertexColor'}
records = []
for mesh in d['meshes']:
    for part in mesh['primitives']:
        if part.get('material') not in materials or 'COLOR_0' not in part['attributes']: continue
        c = colors(d,b,part['attributes']['COLOR_0'])
        unique = sorted(set(c))
        if len(unique) == 1 and all(x in (1,255,65535) for x in unique[0][:3]): continue
        path = root/f"out/level3/unity/skate/scene/prototypes/{mesh['name']}.glb"
        if not path.exists(): continue
        sd,sb = read_glb(path)
        records.append({'prototype':mesh['name'],'material':materials[part['material']]['name'],
            'vertices':len(c),'unique_colors':len(unique),'sample':unique[:5],
            'scene_has_color':any('COLOR_0' in p['attributes'] for m in sd['meshes'] for p in m['primitives'])})
out = root/'out/office-vertex-color-trace.json'
out.write_text(json.dumps(records,indent=2)+'\n')
print('Affected saved primitives:',len(records))
print(json.dumps(records[:4],indent=2))
