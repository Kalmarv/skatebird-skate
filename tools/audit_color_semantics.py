"""Audit saved office mesh colours by source shader, without rebuilding any assets."""
import collections
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
path = ROOT / 'out/level3/unity/level3.glb'
blob = path.read_bytes()
size = struct.unpack_from('<I', blob, 12)[0]
doc = json.loads(blob[20:20 + size])
binary = blob[28 + size:]

def accessor(index):
    a = doc['accessors'][index]
    v = doc['bufferViews'][a['bufferView']]
    code, maximum = {5121: ('B', 255), 5123: ('H', 65535), 5125: ('I', 1), 5126: ('f', 1)}[a['componentType']]
    width = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[a['type']]
    fmt = '<' + code * width
    stride = v.get('byteStride', struct.calcsize(fmt))
    start = v.get('byteOffset', 0) + a.get('byteOffset', 0)
    divisor = maximum if a.get('normalized') else 1
    return [tuple(x / divisor for x in struct.unpack_from(fmt, binary, start + i * stride)) for i in range(a['count'])]

records = []
for mesh in doc['meshes']:
    for part_index, part in enumerate(mesh['primitives']):
        if 'COLOR_0' not in part['attributes'] or 'material' not in part:
            continue
        colors = accessor(part['attributes']['COLOR_0'])
        if all(all(abs(x - 1) < 1e-6 for x in c[:3]) for c in colors):
            continue
        material = doc['materials'][part['material']]
        shader = material.get('extras', {}).get('primase', {}).get('unity.material', {}).get('value', {}).get('shader', 'unknown')
        indices = [int(x[0]) for x in accessor(part['indices'])] if 'indices' in part else list(range(len(colors)))
        mixed = sum(len({colors[x][:3] for x in indices[i:i+3]}) > 1 for i in range(0, len(indices) - 2, 3)) if part.get('mode', 4) == 4 else None
        records.append({'mesh': mesh['name'], 'primitive': part_index, 'material': material.get('name'), 'shader': shader,
                        'vertices': len(colors), 'unique_rgb': len({c[:3] for c in colors}),
                        'triangles_with_different_corner_rgb': mixed, 'sample': sorted(set(colors))[:3]})
report = {'artifact': str(path), 'unit': 'saved GLB mesh primitives, not placements',
          'nonwhite_primitives_by_shader': dict(collections.Counter(r['shader'] for r in records)), 'primitives': records}
out = ROOT / 'out/office-color-semantics.json'
out.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report['nonwhite_primitives_by_shader'], indent=2))
print(json.dumps([r for r in records if r['shader'] == 'Houdini/SpecularVertexColor'], indent=2))
