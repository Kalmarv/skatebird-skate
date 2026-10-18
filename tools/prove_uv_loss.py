"""Compare saved pre-bake and skate-scene UVs for the office desk material."""
import json
import struct
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
target = Path(sys.argv[1]) if len(sys.argv) > 1 else root / 'out/level3'
def glb(path):
    raw = path.read_bytes()
    size, kind = struct.unpack_from('<II', raw, 12)
    assert kind == 0x4e4f534a
    doc = json.loads(raw[20:20+size])
    length, kind = struct.unpack_from('<II', raw, 20+size)
    assert kind == 0x004e4942
    return doc, raw[28+size:28+size+length]

def uv(doc, data, primitive):
    accessor = doc['accessors'][primitive['attributes']['TEXCOORD_0']]
    assert accessor['componentType'] == 5126 and accessor['type'] == 'VEC2'
    view = doc['bufferViews'][accessor['bufferView']]
    offset = view.get('byteOffset',0) + accessor.get('byteOffset',0)
    stride = view.get('byteStride',8)
    return [struct.unpack_from('<ff',data,offset+i*stride) for i in range(accessor['count'])]

before, data = glb(root / 'out/level3/unity/level3.glb')
records = []
for name in ('prototype_e87','prototype_e97'):
    mesh = next(m for m in before['meshes'] if m.get('name') == name)
    primitive = mesh['primitives'][0]
    material = before['materials'][primitive['material']]
    after, payload = glb(target / f'unity/skate/scene/prototypes/{name}.glb')
    a = uv(before,data,primitive)
    b = uv(after,payload,after['meshes'][0]['primitives'][0])
    records.append({'prototype':name,'material':material['name'],'uv_count_before':len(a),'uv_count_after':len(b),
        'uv_arrays_identical':a==b, 'first_uvs':a[:4],
        'base_transform_before':material['pbrMetallicRoughness']['baseColorTexture']['extensions']['KHR_texture_transform'],
        'max_difference_from_required_base_uv':max(max(abs(x[0]*.1-y[0]),abs(x[1]*.1+.9-y[1])) for x,y in zip(a,b))})
out = target / 'uv-proof.json' if len(sys.argv) > 1 else root / 'out/office-uv-loss-proof.json'
out.write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps(records,indent=2))
if '--expect-fixed' in sys.argv:
    assert all(r['max_difference_from_required_base_uv'] < 1e-5 for r in records), 'Base UV transform was not preserved'
