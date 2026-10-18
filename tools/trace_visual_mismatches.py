"""Read source material bindings and named vegetation renderers beside an existing export."""
import collections
import json
from pathlib import Path
import sys
import unitypy_skatebird_oracle as source

level_name = sys.argv[1]
source.LEVEL = level_name
env, level = source.load(source.DEFAULT_DATA)
root = Path(__file__).resolve().parent.parent
scene = json.loads((root / 'out' / level_name / 'unity/skate/scene/scene.json').read_text())
objects = {o.path_id: o.read() for o in level.objects.values() if o.type.name == 'GameObject'}
types = collections.Counter(o.type.name for o in level.objects.values())
materials = {}
vegetation = []
for obj in level.objects.values():
    if obj.type.name not in ('MeshRenderer','SkinnedMeshRenderer','BillboardRenderer','SpriteRenderer','ParticleSystemRenderer'):
        continue
    renderer = obj.read()
    go = objects.get(renderer.m_GameObject.m_PathID)
    name = go.m_Name if go else '?'
    refs = []
    for ptr in getattr(renderer, 'm_Materials', []):
        if not ptr.m_PathID:
            continue
        matobj = ptr.deref()
        mat = matobj.read()
        key = f'{matobj.assets_file.name}:{matobj.path_id}'
        refs.append(key)
        if key not in materials:
            shader = mat.m_Shader.read_typetree()['m_ParsedForm']['m_Name']
            slots = {}
            for slot, binding in mat.m_SavedProperties.m_TexEnvs:
                tex = binding.m_Texture.read() if binding.m_Texture.m_PathID else None
                slots[slot] = {'texture': tex.m_Name if tex else None,
                    'scale': [binding.m_Scale.x,binding.m_Scale.y],
                    'offset': [binding.m_Offset.x,binding.m_Offset.y]}
            materials[key] = {'name': mat.m_Name, 'shader': shader,
                'keywords': mat.m_ShaderKeywords, 'slots': slots,
                'floats': dict(mat.m_SavedProperties.m_Floats), 'renderer_examples': [],
                'exported': {k:v for k,v in scene['materials'].items() if v.get('label') == mat.m_Name}}
        if len(materials[key]['renderer_examples']) < 5:
            materials[key]['renderer_examples'].append(name)
    if any(s in name.lower() for s in ('tree','leaves','foliage','bush')):
        vegetation.append({'name':name, 'renderer':obj.type.name, 'path_id':obj.path_id,
            'enabled':getattr(renderer,'m_Enabled',None), 'active_self':go.m_IsActive if go else None,
            'materials':refs})
result = {'level':level_name, 'renderer_object_counts':{k:v for k,v in types.items() if 'Renderer' in k},
    'scope':'stored renderer references, including inactive; not a claim of runtime visibility',
    'vegetation':vegetation, 'materials':materials}
out = root / 'out' / f'visual-source-{level_name}.json'
out.write_text(json.dumps(result,indent=2)+'\n')
print(out)
print('renderer classes',result['renderer_object_counts'])
print('vegetation',json.dumps(vegetation)[:3500])
for key,m in materials.items():
    if m['exported'] and any(s in m['name'].lower() for s in ('wood','floor','ramp','tree','leav','asphalt')):
        print(key,m['name'],m['shader'],json.dumps({k:v for k,v in m['slots'].items() if k in ('_MainTex','_BumpMap')}))
