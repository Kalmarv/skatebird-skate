"""Record final artifact identity and before/after alpha counts; no runtime verdict inferred."""
import hashlib
import json
from collections import Counter
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent.parent
out = root / 'out' / 'alpha-fix'
record = {'private_commit': subprocess.check_output(['git', '-C', str(root / 'primase'), 'rev-parse', 'HEAD'], text=True).strip(), 'levels': {}}
for level in ('level1', 'level2', 'level3', 'level4', 'level5', 'level7', 'level12', 'level13'):
    scene = out / level / 'unity/skate/scene/scene.json'
    if not scene.exists():
        continue
    entry = {'hashes': {}, 'alpha_count_unit': 'skate-scene material entries; absent alpha means opaque'}
    for tag, path in [('before', root / 'out' / level / 'unity/skate/scene/scene.json'), ('after', scene)]:
        data = json.loads(path.read_text())
        entry[tag] = dict(Counter(m.get('alpha', 'opaque') for m in data['materials'].values()))
    for path in [scene, out / level / 'stage/Patch/layout.toc', out / f'run_{level}/ReSkate.log', out / f'run_{level}/shot_1.png']:
        if path.exists():
            entry['hashes'][str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    record['levels'][level] = entry
(out / 'artifact-manifest.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
