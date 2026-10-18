"""Quick probe of one SkateBIRD scene: roots, spawn-ish names, lights, RenderSettings."""
import os
import sys
import collections

import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"


def main(level):
    env = UnityPy.load(DATA)
    print("loaded", len(env.objects), "objects")
    objs = [o for o in env.objects if o.assets_file.name == level]
    print(level, "objects", len(objs))
    print(collections.Counter(o.type.name for o in objs).most_common(25))

    # render settings
    for o in objs:
        if o.type.name == "RenderSettings":
            d = o.read()
            print("RenderSettings:", d.__dict__ if hasattr(d, "__dict__") else d)
    # lights
    for o in objs:
        if o.type.name == "Light":
            d = o.read()
            print("Light", d.m_GameObject.read().m_Name, "type", d.m_Type,
                  "color", d.m_Color, "int", d.m_Intensity, "shadow", d.m_Shadows)
    # names matching spawn-ish
    pats = ("spawn", "start", "reset", "player", "respawn", "checkpoint", "teleport")
    hits = []
    for o in objs:
        if o.type.name != "GameObject":
            continue
        try:
            d = o.read()
        except Exception as e:
            continue
        n = d.m_Name
        if any(p in n.lower() for p in pats):
            hits.append((o.path_id, n, len(d.m_Component)))
    print("spawn-ish GOs:", len(hits))
    for pid, n, c in hits[:200]:
        print(f"  {pid:>12} {c:3d} {n!r}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "level2")
