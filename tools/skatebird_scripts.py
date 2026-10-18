"""Scene root list + MonoBehaviour script census + objects with marker-like scripts."""
import sys
import collections

import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
LEVEL = sys.argv[1]

env = UnityPy.load(DATA)
objs = [o for o in env.objects if o.assets_file.name == LEVEL]
trans = {}
go_name = {}
for o in objs:
    if o.type.name in ("Transform", "RectTransform"):
        d = o.read()
        trans[o.path_id] = dict(d=d,
                                f=d.m_Father.m_PathID if getattr(d, "m_Father", None) else 0)
    elif o.type.name == "GameObject":
        go_name[o.path_id] = o.read().m_Name
go_t = {t["d"].m_GameObject.m_PathID: pid for pid, t in trans.items()}
roots = [pid for pid, t in trans.items() if not t["f"] or t["f"] not in trans]
print("roots", len(roots))
for pid in roots:
    gid = trans[pid]["d"].m_GameObject.m_PathID
    print("  ", go_name.get(gid))

census = collections.Counter()
markers = []
for o in objs:
    if o.type.name != "GameObject":
        continue
    try:
        go = o.read()
    except Exception:
        continue
    for c in go.m_Component:
        cp = c.component
        if not cp:
            continue
        if cp.type.name != "MonoBehaviour":
            continue
        try:
            mb = cp.read()
            cls = mb.m_Script.read().m_ClassName
        except Exception as e:
            census["<unreadable>"] += 1
            cls = "<unreadable>"
        census[cls] += 1
        low = cls.lower()
        if any(k in low for k in ("spawn", "marker", "reset", "start", "teleport", "checkpoint")):
            markers.append((go.m_Name, cls, cp.path_id, o.path_id))
print("MonoBehaviour classes:", len(census))
for k, v in census.most_common(60):
    print(f"  {v:5d} {k}")
print("marker-ish objects:", len(markers))
for n, cls, cpid, gid in markers:
    print(f"  {n!r:40s} {cls} component={cpid}")
