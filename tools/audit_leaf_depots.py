"""Read compiled leaf-related keys using the existing GPL Patch/depot readers."""
import hashlib
import json
import struct
import sys
from pathlib import Path

ports = Path('C:/Users/kalmarv/modding/reskate/reskate-ports')
sys.path[:0] = [str(ports / 'kit/gpl'), str(ports / 'ports/gta/stitch')]
import fbpatch as fp
import fbebx
import sbdfmt
import blk

root = Path(__file__).resolve().parents[1]
patch = root / 'out/alpha-fix/level12/stage/Patch'
game = root / 'game'
F, B = fp.Folder(str(patch)), fp.Folder(str(game / 'Data'))
audit = json.loads((root / 'out/playground-leaf-audit.json').read_text())
keys = {int(r['key'], 16) for r in audit['compiled_leaf_named_sections']}
output_name = 'playground-leaf-depots.json'
if len(sys.argv) > 1 and sys.argv[1] == 'sledding':
    patch = Path('C:/Users/kalmarv/modding/work/sledding/lab/v2/stage/Patch')
    F = fp.Folder(str(patch))
    census = json.loads((root / 'out/sledding-sections.json').read_text())
    keys = {int('eb6d7639ba8a93a4', 16)}
    for token in ('stone_character_mouth', 'glowing_eyes', 'text_000'):
        matches = [r for r in census if token in r['mesh'].lower() and r['lod'] == 0]
        keys.update(int(r['key'],16) for r in matches[:4])
    output_name = 'sledding-reference-depots.json'
textures, blocks, programs = {}, [], []
def payload(f):
    raw = fp.read_any(F, B, *f)
    return fp.decode_cas(raw, str(game))

for rel in F.tocs:
    toc = fp.read_toc((patch / rel).read_bytes())
    for bundle, region in toc.bundles:
        files, b, first, _ = fp.list_bundle(F, region, str(game), B)
        if b is None or first != 1 or not files[0][0].patch:
            continue
        for i, a in enumerate(b.ebx):
            if '/textures/' in a.name:
                f = files[first + i]
                if f[0].patch:
                    textures[bytes(fbebx.read_file_guid(payload(f))).hex()] = a.name
        for i, a in enumerate(b.res):
            if a.rtype not in (0x73312045, fp.PROGRAM_LOOKUP):
                continue
            data = payload(files[first + len(b.ebx) + i])
            if a.rtype == fp.PROGRAM_LOOKUP:
                for k, value in fp._program_rows(data, a.rmeta).items():
                    if k in keys:
                        programs.append({'bundle': bundle, 'key': f'{k:016x}',
                            'programs': struct.unpack('<%dI' % (len(value)//4), value)})
                continue
            depot = sbdfmt.read(data, a.rmeta)
            for key, bi in depot.entries:
                if key not in keys:
                    continue
                block = depot.blocks[bi]
                params, pos, error = blk.parse(block.data)
                if error or pos != len(block.data):
                    raise ValueError((a.name, key, error, pos, len(block.data)))
                values = []
                for name, typ, count, x, value in params:
                    v = {'name': f'{name:016x}', 'type': f'{typ:08x}'}
                    if value[0] == 'tex':
                        v.update(file_guid=value[3].hex(), instance_guid=value[2].hex())
                    else:
                        v['bytes'] = value[2].hex()
                    values.append(v)
                blocks.append({'bundle': bundle, 'depot': a.name, 'key': f'{key:016x}',
                    'sha256': hashlib.sha256(block.data).hexdigest(), 'parameters': values})
for b in blocks:
    for v in b['parameters']:
        if 'file_guid' in v:
            v['texture_name'] = textures.get(v['file_guid'])
out = {'keys': sorted(f'{k:016x}' for k in keys), 'blocks': blocks, 'programs': programs}
(root / 'out' / output_name).write_text(json.dumps(out, indent=2)+'\n')
print('keys', out['keys'], 'blocks', len(blocks), 'program rows', len(programs))
for key in out['keys']:
    selected = [b for b in blocks if b['key'] == key]
    names = sorted({v['texture_name'] or v['file_guid'] for b in selected for v in b['parameters'] if 'file_guid' in v})
    print(key, 'textures', names[:12], 'distinct blocks', len({b['sha256'] for b in selected}))
print('program tuples', sorted({(p['key'], tuple(p['programs'])) for p in programs}))
