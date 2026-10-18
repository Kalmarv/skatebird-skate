"""Read stored shader states with UnityPy; compare labels to existing export (not an oracle).

This audit never changes assets or infers an alpha mode for an unmodelled shader.
Usage: python tools/audit_material_alpha.py level4
"""
import collections
import hashlib
import json
from pathlib import Path
import sys

import UnityPy
import unitypy_skatebird_oracle as source


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    level_name = sys.argv[1] if len(sys.argv) > 1 else "level4"
    source.LEVEL = level_name
    env, level = source.load(source.DEFAULT_DATA)
    refs = {}
    for obj in level.objects.values():
        if obj.type.name != "MeshRenderer":
            continue
        for ptr in obj.read().m_Materials:
            if not ptr.m_PathID:
                continue
            mat = ptr.deref()
            if mat is None:
                raise ValueError(f"unresolved material reference: {ptr}")
            refs[(mat.assets_file.name, mat.path_id)] = mat
    root = Path(__file__).resolve().parents[1]
    exported_path = root / "out" / level_name / "unity/skate/scene/scene.json"
    exported = json.loads(exported_path.read_text())
    materials = []
    for (file, pid), obj in sorted(refs.items()):
        mat = obj.read()
        shader = mat.m_Shader.read_typetree()["m_ParsedForm"]
        floats = dict(mat.m_SavedProperties.m_Floats)
        subshaders = []
        for sub in shader["m_SubShaders"]:
            passes = []
            for shader_pass in sub["m_Passes"]:
                state = shader_pass["m_State"]
                blend = state["rtBlend0"]
                passes.append({
                    "name": state["m_Name"],
                    "tags": state["m_Tags"]["tags"],
                    "src": blend["srcBlend"], "dst": blend["destBlend"],
                    "resolved_src": floats.get(blend["srcBlend"]["name"], blend["srcBlend"]["val"]),
                    "resolved_dst": floats.get(blend["destBlend"]["name"], blend["destBlend"]["val"]),
                    "zwrite": state["zWrite"], "cull": state["culling"],
                })
            subshaders.append({"tags": sub["m_Tags"]["tags"], "passes": passes})
        materials.append({
            "file": file, "path_id": pid, "name": mat.m_Name,
            "shader": shader["m_Name"], "keywords": mat.m_ShaderKeywords,
            "floats": floats, "subshaders": subshaders,
            "exported_label_matches": {
                key: {k: value.get(k) for k in ("class", "alpha", "base", "base_color")}
                for key, value in exported["materials"].items() if value.get("label") == mat.m_Name
            },
        })
    result = {
        "level": level_name, "unitypy": UnityPy.__version__,
        "scope": "unique materials referenced by all MeshRenderers, including inactive renderers",
        "source_sha256": {Path(k).name: sha(k) for k, v in env.files.items() if hasattr(v, "objects")},
        "export_sha256": sha(exported_path),
        "export_alpha_counts": dict(collections.Counter(m.get("alpha", "opaque") for m in exported["materials"].values())),
        "materials": materials,
    }
    output = root / "out" / f"alpha-audit-{level_name}.json"
    output.write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(output, len(materials), result["export_alpha_counts"])


if __name__ == "__main__":
    main()
