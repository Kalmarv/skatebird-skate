"""World-space bounds and triangle counts per SkateBIRD level, resolving meshes across serialized files.

UnityPy is the independent oracle here (not primase).
"""
import os
import sys
import collections
import numpy as np
import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
LEVELS = ["level1", "level2", "level3", "level4", "level5", "level7", "level12", "level13"]


def q2m(x, y, z, w):
    n = x * x + y * y + z * z + w * w
    if n < 1e-12:
        return np.eye(3)
    s = 2.0 / n
    xx, yy, zz = x * x * s, y * y * s, z * z * s
    xy, xz, yz = x * y * s, x * z * s, y * z * s
    wx, wy, wz = w * x * s, w * y * s, w * z * s
    return np.array([
        [1 - (yy + zz), xy - wz, xz + wy],
        [xy + wz, 1 - (xx + zz), yz - wx],
        [xz - wy, yz + wx, 1 - (xx + yy)],
    ])


env = UnityPy.load(DATA)
print("loaded", len(env.objects), "objects")

by_file = collections.defaultdict(list)
for o in env.objects:
    by_file[o.assets_file.name].append(o)

for level in LEVELS:
    if level not in by_file:
        print(level, "missing")
        continue
    objs = by_file[level]
    trans = {}
    go_to_t = {}
    for o in objs:
        if o.type.name in ("Transform", "RectTransform"):
            d = o.read()
            p, r, s = d.m_LocalPosition, d.m_LocalRotation, d.m_LocalScale
            trans[o.path_id] = dict(
                pos=np.array([p.x, p.y, p.z], dtype=np.float64),
                rot=q2m(r.x, r.y, r.z, r.w),
                scale=np.array([s.x, s.y, s.z], dtype=np.float64),
                father=d.m_Father.m_PathID if getattr(d, "m_Father", None) else None,
            )
            go_to_t[d.m_GameObject.m_PathID] = o.path_id
    memo = {}

    def mat(pid, depth=0):
        if pid in memo:
            return memo[pid]
        t = trans.get(pid)
        if t is None or depth > 64:
            return np.eye(4)
        local = np.eye(4)
        local[:3, :3] = t["rot"] * t["scale"][None, :]
        local[:3, 3] = t["pos"]
        parent = mat(t["father"]) if t["father"] in trans and t["father"] != pid else np.eye(4)
        memo[pid] = parent @ local
        return memo[pid]

    lo = np.array([1e18] * 3)
    hi = np.array([-1e18] * 3)
    tris = 0
    nren = 0
    ncollide = 0
    skipped = 0
    for o in objs:
        if o.type.name not in ("MeshRenderer", "SkinnedMeshRenderer"):
            continue
        try:
            d = o.read()
        except Exception:
            skipped += 1
            continue
        try:
            mesh = d.m_Mesh.read() if o.type.name == "MeshRenderer" else None
        except Exception:
            mesh = None
        if mesh is None:
            skipped += 1
            continue
        m = mesh
        try:
            aabb = m.m_LocalAABB
            c = np.array([aabb.m_Center.x, aabb.m_Center.y, aabb.m_Center.z])
            e = np.array([aabb.m_Extent.x, aabb.m_Extent.y, aabb.m_Extent.z])
        except Exception:
            skipped += 1
            continue
        if e.max() > 1e7 or not np.isfinite(e).all():
            skipped += 1
            continue
        tp = go_to_t.get(d.m_GameObject.m_PathID) if getattr(d, "m_GameObject", None) else None
        if tp is None:
            skipped += 1
            continue
        M = mat(tp)
        corners = np.array([[c[0] + sx * e[0], c[1] + sy * e[1], c[2] + sz * e[2]]
                            for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)])
        wc = (M[:3, :3] @ corners.T).T + M[:3, 3]
        if not np.isfinite(wc).all():
            skipped += 1
            continue
        lo = np.minimum(lo, wc.min(axis=0))
        hi = np.maximum(hi, wc.max(axis=0))
        nren += 1
        try:
            idx = m.m_IndexBuffer or []
            tris += len(idx) // 6
        except Exception:
            pass
    print(f"{level}: renderers {nren} tris {tris} skipped {skipped} size {np.round(hi-lo,1)} centre {np.round((hi+lo)/2,1)}")
