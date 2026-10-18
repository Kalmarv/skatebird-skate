from pathlib import Path
s = Path('primase/worlds/unity/examples/unity_export.rs').read_text()
start = s.index('    show(\n        "bake-texture-transforms"')
end = s.index("    // skate's materials carry", start)
s = s[:start] + '    show("preserve-base-tiling", &uv_policy::preserve_base_tiling(&mut w)?);\n' + s[end:]
s = s.replace('use std::collections::BTreeMap;', 'mod uv_policy;\n\nuse std::collections::BTreeMap;', 1)
Path('program/src/main.rs').write_text(s)
