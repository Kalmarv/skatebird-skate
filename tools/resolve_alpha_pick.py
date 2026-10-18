from pathlib import Path
import re

pattern = r'<<<<<<< HEAD\n(.*?)=======\n(.*?)>>>>>>>[^\n]*\n'
p = Path('primase/worlds/unity/src/lower.rs')
s = p.read_text()
def lower(m):
    ours, theirs = m.groups()
    if ours.startswith('//!'):
        return ours
    if 'let built_in' in ours:
        return '        let built_in = matches!(fam, Family::Standard | Family::Legacy);\n' + theirs
    return ours + theirs
s = re.sub(pattern, lower, s, flags=re.S)
start = s.index('/// Whether any pass of the shader blends:')
end = s.index("/// A built-in shader's cull:", start)
s = s[:start] + s[end:]
s = s.replace('        ShaderPass {\n            light_mode:', '        ShaderPass {\n            kind: 0,\n            light_mode:')
p.write_text(s)
p = Path('primase/worlds/unity/src/drawn.rs')
s = p.read_text()
def drawn(m):
    ours, theirs = m.groups()
    if ours.startswith('//!'):
        return ours.replace('forward pass', 'draw pass')
    if 'fn ui_only' in ours:
        start = ours.index('/// Whether a material is on one')
        end = ours.index('/// Whether a material is on a shader the lowering')
        return ours[start:end] + theirs
    return ours[:ours.index('        if !ms.is_empty() && ms.iter().all(|m| m.is_some_and(additive_unmodelled))')] + theirs
s = re.sub(pattern, drawn, s, flags=re.S)
p.write_text(s)
