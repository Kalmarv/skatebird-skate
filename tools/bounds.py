import sys, os, collections
import UnityPy
import numpy as np

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"


def quat_to_mat(x, y, z, w):
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


def main(level):
    env = UnityPy.load(os.path.join(DATA, level))
    world = {}
    # pass 1: read transforms
    for obj in env.objects:
        if obj.type.name not in ("Transform", "RectTransform"):
            continue
        d = obj.read()
        p = d.m_LocalPosition
        r = d.m_LocalRotation
        s = d.m_LocalScale
        father = None
        if getattr(d, "m_Father", None):
            father = d.m_Father.m_PathID if hasattr(d.m_Father, "m_PathID") else None
        world[obj.path_id] = dict(
            pos=np.array([p.x, p.y, p.z], dtype=np.float64),
            rot=quat_to_mat(r.x, r.y, r.z, r.w),
            scale=np.array([s.x, s.y, s.z], dtype=np.float64),
            father=father,
            go=d.m_GameObject.m_PathID if getattr(d, "m_GameObject", None) else None,
            name="",
        )
    # pass 2: names of game objects
    names = {}
    for obj in env.objects:
        if obj.type.name == "GameObject":
            try:
                d = obj.read()
                names[obj.path_id] = d.m_Name
            except Exception:
                pass
    for t in world.values():
        t["name"] = names.get(t["go"], "")
    # resolve world matrices (memoized, iterative)
    def mat(pid, depth=0):
        t = world.get(pid)
        if t is None:
            return np.eye(4)
        if "m" in t:
            return t["m"]
        if depth > 64:
            return np.eye(4)
        local = np.eye(4)
        local[:3, :3] = t["rot"] * t["scale"][None, :]
        local[:3, 3] = t["pos"]
        parent = mat(t["father"], depth + 1) if t["father"] in world and t["father"] != pid else np.eye(4)
        t["m"] = parent @ local
        return t["m"]

    # renderer bounds: transform mesh AABB corners
    lo = np.array([1e18] * 3)
    hi = np.array([-1e18] * 3)
    n = 0
    by_root = collections.Counter()
    for obj in env.objects:
        if obj.type.name not in ("MeshRenderer", "SkinnedMeshRenderer"):
            continue
        try:
            d = obj.read()
        except Exception:
            continue
        if obj.type.name == "MeshRenderer":
            mf = getattr(d, "m_Mesh", None)
            mid = mf.m_PathID if mf else None
            if not mid:
                continue
            try:
                mesh = env.objects_by_path_id[int(mid)] if False else None
            except Exception:
                mesh = None
            # read the mesh through environment
            mesh_obj = None
            for o in env.objects:
                if o.path_id == mid and o.type.name == "Mesh":
                    mesh_obj = o
                    break
            if mesh_obj is None:
                continue
            try:
                m = mesh_obj.read()
                aabb = m.m_LocalAABB
                c = np.array([aabb.m_Center.x, aabb.m_Center.y, aabb.m_Center.z])
                e = np.array([aabb.m_Extent.x, aabb.m_Extent.y, aabb.m_Extent.z])
            except Exception:
                continue
            if e.max() > 1e7:
                continue
        else:
            # skinned: use stored bounds if any
            try:
                aabb = d.m_AABB
                c = np.array([aabb.m_Center.x, aabb.m_Center.y, aabb.m_Center.z])
                e = np.array([aabb.m_Extent.x, aabb.m_Extent.y, aabb.m_Extent.z])
            except Exception:
                continue
        tpid = d.m_GameObject.m_PathID if getattr(d, "m_GameObject", None) else None
        # transform's game object == renderer's game object
        tp = None
        for pid, t in world.items():
            if t["go"] == tpid:
                tp = pid
                break
        if tp is None:
            continue
        M = mat(tp)
        corners = []
        for sx in (-1, 1):
            for sy in (-1, 1):
                for sz in (-1, 1):
                    corners.append(c + np.array([sx * e[0], sy * e[1], sz * e[2]]))
        corners = np.stack(corners)
        wc = (M[:3, :3] @ corners.T).T + M[:3, 3]
        lo = np.minimum(lo, wc.min(axis=0))
        hi = np.maximum(hi, wc.max(axis=0))
        n += 1
    print(level, "renderers", n)
    print("  bounds min", np.round(lo, 2), "max", np.round(hi, 2))
    print("  size", np.round(hi - lo, 2))


if __name__ == "__main__":
    main(sys.argv[1])
