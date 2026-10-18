"""Dump hierarchy + components for named SkateBIRD scene objects."""
import sys
import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
LEVEL = sys.argv[1]
ROOTS = sys.argv[2:]

env = UnityPy.load(DATA)
objs = [o for o in env.objects if o.assets_file.name == LEVEL]
trans = {}
go_name = {}
go_t = {}
for o in objs:
    if o.type.name in ("Transform", "RectTransform"):
        d = o.read()
        p, r, s = d.m_LocalPosition, d.m_LocalRotation, d.m_LocalScale
        trans[o.path_id] = dict(o=o, d=d, p=[p.x, p.y, p.z], r=[r.x, r.y, r.z, r.w],
                                s=[s.x, s.y, s.z],
                                f=d.m_Father.m_PathID if getattr(d, "m_Father", None) else 0)
for o in objs:
    if o.type.name == "GameObject":
        go_name[o.path_id] = o.read().m_Name
for pid, t in trans.items():
    gid = t["d"].m_GameObject.m_PathID
    go_t[gid] = pid
kids = {}
for pid, t in trans.items():
    kids.setdefault(t["f"], []).append(pid)


def comp_info(go_id):
    out = []
    go = next((o for o in objs if o.type.name == "GameObject" and o.path_id == go_id), None)
    if go is None:
        return out
    for c in go.read().m_Component:
        cp = c.component
        if not cp:
            continue
        try:
            co = cp.read()
        except Exception as e:
            out.append(("<?>", repr(e)))
            continue
        if cp.type.name == "MonoBehaviour":
            try:
                out.append(("MonoBehaviour:" + co.m_Script.read().m_ClassName, cp.path_id))
            except Exception:
                out.append(("MonoBehaviour:<unread>", cp.path_id))
        else:
            out.append((cp.type.name, cp.path_id))
    return out


def walk(tpid, depth):
    t = trans[tpid]
    gid = t["d"].m_GameObject.m_PathID
    print("  " * depth + f"{go_name.get(gid,'?')!r} local={[round(x,3) for x in t['p']]} rot={[round(x,3) for x in t['r']]}")
    if depth < 3:
        for k in kids.get(tpid, []):
            walk(k, depth + 1)


for name in ROOTS:
    for gid, n in go_name.items():
        if n == name:
            tp = go_t.get(gid)
            print("=" * 20)
            print("OBJECT", name, "go", gid, "components", comp_info(gid))
            if tp:
                walk(tp, 0)
