"""List named collider feature candidates with source world bounds (not automatic gameplay heights).

Uses UnityPy and the existing source hierarchy scanner. A collider's vertical extent is not necessarily
its rise above the riding surface; confirm candidates before proposing a gameplay scale.
"""
import json
from pathlib import Path
import re
import sys

import numpy as np
import UnityPy
from skatebird_scan import Scene, box_corners, mesh_aabb, vec
import unitypy_skatebird_oracle as source


def main():
    level = sys.argv[1] if len(sys.argv) > 1 else "level2"
    source.LEVEL = level
    env, _ = source.load(source.DEFAULT_DATA)
    scene = Scene(env, level)
    cache = {}
    rows = []
    for obj in scene.objs:
        if obj.type.name not in ("MeshCollider", "BoxCollider"):
            continue
        collider = obj.read()
        gid = collider.m_GameObject.m_PathID
        path = scene.path(gid)
        if not re.search(r"rail|ledge|ramp|quarter|bowl|stair|step|gap", path, re.I):
            continue
        if not collider.m_Enabled or collider.m_IsTrigger or not scene.active_in_hierarchy(gid):
            continue
        matrix = scene.world(gid)
        if matrix is None:
            raise ValueError(f"missing transform: {path}")
        if obj.type.name == "MeshCollider":
            aabb = mesh_aabb(collider.m_Mesh, cache)
            if aabb is None:
                raise ValueError(f"missing mesh bounds: {path}")
            center, half, mesh = aabb
        else:
            center = [collider.m_Center.x, collider.m_Center.y, collider.m_Center.z]
            half = np.array([collider.m_Size.x, collider.m_Size.y, collider.m_Size.z]) / 2
            mesh = None
        corners = (matrix[:3, :3] @ box_corners(center, half).T).T + matrix[:3, 3]
        low, high = corners.min(axis=0), corners.max(axis=0)
        rows.append({"path": path, "collider_path_id": obj.path_id, "mesh": mesh,
                     "min": vec(low), "max": vec(high), "extent": vec(high-low)})
    root = Path(__file__).resolve().parents[1]
    output = root / "out" / f"feature-dimensions-{level}.json"
    output.write_text(json.dumps({"level": level, "unitypy": UnityPy.__version__,
        "method": "active non-trigger collider local AABBs transformed through source hierarchy; named candidates only",
        "features": rows}, indent=2), encoding="utf-8")
    print(output, len(rows))
    for row in rows[:35]:
        print(row["extent"], row["path"])


if __name__ == "__main__":
    main()
