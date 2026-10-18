"""Inventory source lighting and its exported representation, without launching games."""
import collections
import json
from pathlib import Path
import sys
import unitypy_skatebird_oracle as source

level_name = sys.argv[1]
source.LEVEL = level_name
env, level = source.load(source.DEFAULT_DATA)
root = Path(__file__).resolve().parent.parent
report = {'level': level_name, 'settings': {}, 'lights': [], 'baked_objects': [], 'renderer_lightmap_indices': {}, 'emissive_materials': []}
counts = collections.Counter()
materials = {}
for obj in level.objects.values():
    kind = obj.type.name
    if kind in ('RenderSettings', 'LightmapSettings'):
        data = obj.read_typetree()
        report['settings'][kind] = data
    elif kind == 'Light':
        data = obj.read_typetree()
        go = obj.read().m_GameObject.read()
        report['lights'].append({'path_id': obj.path_id, 'name': go.m_Name, 'active_self': go.m_IsActive,
            **{k:v for k,v in data.items() if k in ('m_Enabled', 'm_Type', 'm_Color', 'm_Intensity', 'm_Range', 'm_Lightmapping', 'm_BakingOutput', 'm_Shadows', 'm_CullingMask')}})
    elif kind in ('MeshRenderer', 'SkinnedMeshRenderer'):
        data = obj.read_typetree()
        counts[str(data.get('m_LightmapIndex', 'absent'))] += 1
        renderer = obj.read()
        for ptr in renderer.m_Materials:
            if not ptr.m_PathID:
                continue
            ref = ptr.deref()
            key = f'{ref.assets_file.name}:{ref.path_id}'
            if key in materials:
                continue
            mat = ref.read()
            materials[key] = True
            colors = dict(mat.m_SavedProperties.m_Colors)
            color = colors.get('_EmissionColor')
            if color and max(color.r, color.g, color.b) > 0:
                report['emissive_materials'].append({'source':key, 'name': mat.m_Name,
                    'keywords': mat.m_ShaderKeywords, 'emission': [color.r,color.g,color.b,color.a]})

def compact(value):
    if isinstance(value, list):
        return {'count': len(value), 'first': [compact(v) for v in value[:2]]} if len(value) > 32 else [compact(v) for v in value]
    if isinstance(value, dict):
        return {k:compact(v) for k,v in value.items()}
    if isinstance(value, bytes):
        return {'bytes':len(value)}
    return value

for file in env.files.values():
    if not hasattr(file, 'objects'):
        continue
    for obj in file.objects.values():
        if obj.type.name in ('LightProbes', 'LightingDataAsset'):
            report['baked_objects'].append({'file':file.name,'id':obj.path_id,'kind':obj.type.name,'data':compact(obj.read_typetree())})
report['renderer_lightmap_indices'] = dict(counts)
report['settings'] = compact(report['settings'])
scene_path = root / f'out/alpha-fix/{level_name}/unity/skate/scene/scene.json'
scene = json.loads(scene_path.read_text())
report['export_scene_lighting'] = {k:v for k,v in scene.items() if any(s in k for s in ('light','sun','environment'))}
report['export_emissive_materials'] = {k:v for k,v in scene['materials'].items() if v.get('emissive')}
out = root / f'out/source-lighting-{level_name}.json'
out.write_text(json.dumps(report,indent=2)+'\n')
print(out)
print(json.dumps({k:v for k,v in report.items() if k not in ('baked_objects','emissive_materials')},indent=2)[:14000])
print('baked objects',[(v['file'],v['id'],v['kind']) for v in report['baked_objects']])
print('emissive materials',len(report['emissive_materials']))
