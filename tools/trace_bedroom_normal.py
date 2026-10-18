"""Check the actual bedroom tabletop normal pixels across saved export stages."""
import hashlib
import io
import json
import struct
from pathlib import Path
import numpy as np
from PIL import Image
import UnityPy

root = Path(__file__).resolve().parent.parent
path = root / 'out/level1/unity/level1.glb'
blob = path.read_bytes()
size = struct.unpack_from('<I', blob, 12)[0]
doc = json.loads(blob[20:20+size])
binary = blob[28+size:]
material = next(m for m in doc['materials'] if m['name'] == 'hardwood gross - hawaiian koa')
texture = doc['textures'][material['normalTexture']['index']]
img = doc['images'][texture['source']]
view = doc['bufferViews'][img['bufferView']]
start = view.get('byteOffset', 0)
source = Image.open(io.BytesIO(binary[start:start+view['byteLength']])).convert('RGB')
scene_path = root / 'out/alpha-fix/level1/unity/skate/scene/textures/detail_wood_a_01_norm.png'
map_path = root / 'out/alpha-fix/level1/map/SkateBIRD_level1/textures/detail_wood_a_01_norm_f985fcad.png'
arrays = [np.asarray(source), np.asarray(Image.open(scene_path).convert('RGB')), np.asarray(Image.open(map_path).convert('RGB'))]
reference = source
while reference.width > arrays[2].shape[1] and reference.height > arrays[2].shape[0]:
    reference = reference.resize((reference.width // 2, reference.height // 2), Image.Resampling.BOX)
resized = np.asarray(reference).astype(np.int16)
target = arrays[2].astype(np.int16)
report = {'material': material['name'], 'normal_image': img['name'],
          'base_sampling': material['pbrMetallicRoughness']['baseColorTexture'],
          'normal_sampling': material['normalTexture'],
          'sizes': [list(a.shape) for a in arrays],
          'rgb_pixel_sha256': [hashlib.sha256(a.tobytes()).hexdigest() for a in arrays],
          'source_glb_to_scene_identical_rgb': bool(np.array_equal(arrays[0], arrays[1])),
          'scene_to_map_identical_rgb': bool(np.array_equal(arrays[1], arrays[2])),
          'downsampled_green_mae_same_sign': float(np.abs(resized[:,:,1] - target[:,:,1]).mean()),
          'downsampled_green_mae_inverted_sign': float(np.abs(255 - resized[:,:,1] - target[:,:,1]).mean()),
          'scope': 'normalized source GLB to final normalized map; does not validate Unity decoder or native compiler tangent basis'}
env = UnityPy.load('C:/Program Files (x86)/Steam/steamapps/common/SkateBIRD/SkateBIRD_Data/sharedassets0.assets')
raw = next(o for o in env.objects if o.path_id == 509).read()
packed = np.asarray(raw.image.convert('RGBA')).astype(np.int16)
decoded = arrays[1].astype(np.int16)
report['source_texture'] = 'sharedassets0.assets:509, DXT5, usage=3 (DXT5nm)'
report['source_alpha_to_export_red_mae'] = float(np.abs(packed[:,:,3] - decoded[:,:,0]).mean())
report['source_green_to_export_green_mae'] = float(np.abs(packed[:,:,1] - decoded[:,:,1]).mean())
report['scope'] = 'source DXT5nm channel decode and normalized-map sign; native compiler tangent basis not validated'
out = root / 'out/bedroom-normal-trace.json'
out.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
