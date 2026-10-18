"""Extract one shader's stored D3D programs with UnityPy; private research only."""
import ctypes as C
import hashlib
import json
from pathlib import Path
import sys
from UnityPy.export.ShaderConverter import ShaderProgram
from UnityPy.helpers import CompressionHelper
from UnityPy.streams import EndianBinaryReader
import unitypy_skatebird_oracle as source

source.LEVEL = sys.argv[2] if len(sys.argv) > 2 else 'level3'
env, level = source.load(source.DEFAULT_DATA)
shader_name = sys.argv[1]
trace = json.loads(Path(f'out/visual-source-{source.LEVEL}.json').read_text())
key = next(k for k,m in trace['materials'].items() if m['shader'] == shader_name)
filename, pid = key.rsplit(':',1)
file = next(f for f in env.files.values() if hasattr(f,'objects') and Path(f.name).name == filename)
shader = file.objects[int(pid)].read().m_Shader.read()
out = Path('research/shaders') / shader_name.replace('/','_')
out.mkdir(parents=True, exist_ok=True)
dis = C.WinDLL('d3dcompiler_47.dll').D3DDisassemble
dis.argtypes = [C.c_void_p,C.c_size_t,C.c_uint,C.c_char_p,C.POINTER(C.c_void_p)]
dis.restype = C.c_long
def disassemble(code):
    blob = C.c_void_p()
    buf = C.create_string_buffer(code)
    status = dis(buf,len(code),0,None,C.byref(blob))
    if status < 0: raise RuntimeError(f'D3DDisassemble {status:#x}')
    table = C.cast(blob,C.POINTER(C.POINTER(C.c_void_p))).contents
    pointer = C.WINFUNCTYPE(C.c_void_p,C.c_void_p)(table[3])(blob)
    size = C.WINFUNCTYPE(C.c_size_t,C.c_void_p)(table[4])(blob)
    text = C.string_at(pointer,size).decode('utf8').rstrip('\0')
    C.WINFUNCTYPE(C.c_ulong,C.c_void_p)(table[2])(blob)
    return text

def entry(values, i):
    v = values[i]
    return v[0] if isinstance(v,list) else v
records = []
base_pass = next(p for sub in shader.m_ParsedForm.m_SubShaders for p in sub.m_Passes
    if any(k.lower() == 'lightmode' and v.lower() == 'forwardbase' for k,v in p.m_State.m_Tags.tags))
base_indices = {p.m_BlobIndex for stage in (base_pass.progVertex,base_pass.progFragment) for p in stage.m_SubPrograms}
for i, platform in enumerate(shader.platforms):
    offset = entry(shader.offsets,i)
    length = entry(shader.compressedLengths,i)
    data = CompressionHelper.decompress_lz4(bytes(shader.compressedBlob)[offset:offset+length],entry(shader.decompressedLengths,i))
    program = ShaderProgram(EndianBinaryReader(data,endian='<'),shader.object_reader.version)
    # Choose least-keyword programs only within the surface ForwardBase pass, never shadow/depth passes.
    chosen = {}
    for index, sub in enumerate(program.m_SubPrograms):
        if index not in base_indices: continue
        code = bytes(sub.m_ProgramCode)
        at = code.find(b'DXBC')
        if at < 0: continue
        kind = str(sub.m_ProgramType)
        score = len(sub.m_Keywords) + len(sub.m_LocalKeywords or [])
        if kind not in chosen or score < chosen[kind][0]: chosen[kind] = (score,index,sub,code[at:])
    for kind, (_,index,sub,code) in chosen.items():
        name = f'platform{platform}_program{index}'
        (out / (name+'.dxbc')).write_bytes(code)
        (out / (name+'.asm')).write_text(disassemble(code))
        records.append({'file':name,'kind':kind,'keywords':sub.m_Keywords,'local_keywords':sub.m_LocalKeywords,'sha256':hashlib.sha256(code).hexdigest()})
metadata = {'shader':shader_name,'material':key,'pass':base_pass.m_State.m_Name,'shader_file':shader.object_reader.assets_file.name,'shader_path_id':shader.object_reader.path_id,'programs':records}
(out/'manifest.json').write_text(json.dumps(metadata,indent=2)+'\n')
print(json.dumps(metadata,indent=2))
