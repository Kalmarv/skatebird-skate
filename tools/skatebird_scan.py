"""SkateBIRD level scan: spawn, travel points, collider bounds, lighting.

Read-only over the installed game. Emits out/skatebird-scan.json plus a summary.

Evidence rules:
- spawn = active ResetPosition with startingPosition == true (Skatebirb.Gameplay.ResetPosition.ResetToStart).
- bounds = enabled, non-trigger, active-in-hierarchy colliders; MeshCollider AABBs resolved
  through their transform; meshes whose world AABB exceeds 100 m on any axis are excluded.
- travel points = ResetPosition teleport destinations + MissionGiver objects + named landmarks.
"""
import collections
import json
import math
import os
import struct
import sys

import numpy as np
import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
OUT = r"C:\Users\kalmarv\ds-modding\out\skatebird-scan.json"

LEVELS = [
    ("level1", "bedroom 1"),
    ("level2", "rooftop"),
    ("level3", "office"),
    ("level4", "server room"),
    ("level5", "bedroom 2"),
    ("level7", "flashback"),
    ("level12", "playground"),
    ("level13", "skate heaven"),
]

MAX_MESH_SIZE = 100.0  # metres


# ---------------------------------------------------------------- math helpers
def quat_matrix(x, y, z, w):
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


def yaw_deg(m):
    f = m[:3, :3] @ np.array([0.0, 0.0, 1.0])
    return math.degrees(math.atan2(f[0], f[2]))


def euler_deg(m):
    """Unity Quaternion.Euler (ZXY) angles in degrees from a rotation matrix."""
    r = m[:3, :3]
    sx = max(-1.0, min(1.0, -r[1, 2]))
    x = math.asin(sx)
    if abs(sx) < 0.999999:
        y = math.atan2(r[0, 2], r[2, 2])
        z = math.atan2(r[1, 0], r[1, 1])
    else:
        y = math.atan2(-r[2, 0], r[0, 0])
        z = 0.0
    return [math.degrees(x), math.degrees(y), math.degrees(z)]


def vec(v):
    return [round(float(v[i]), 4) for i in range(3)]


def color(c):
    if c is None:
        return None
    return [round(float(c.r), 6), round(float(c.g), 6), round(float(c.b), 6), round(float(c.a), 6)]


def box_corners(center, half):
    c = np.asarray(center, dtype=np.float64)
    h = np.asarray(half, dtype=np.float64)
    return np.array([[c[0] + sx * h[0], c[1] + sy * h[1], c[2] + sz * h[2]]
                     for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)])


# ---------------------------------------------------------------- scene model
class Scene:
    def __init__(self, env, file_name):
        self.env = env
        self.file_name = file_name
        self.objs = [o for o in env.objects if o.assets_file.name == file_name]
        self.objs_by_pid = {}
        for o in self.objs:
            self.objs_by_pid.setdefault(o.path_id, o)
        self.trans = {}
        self.go_name = {}
        self.go_active = {}
        self.go_t = {}
        self.kids = collections.defaultdict(list)
        self._read_hierarchy()
        self._memo = {}

    def _read_hierarchy(self):
        for o in self.objs:
            t = o.type.name
            if t in ("Transform", "RectTransform"):
                d = o.read()
                p, r, s = d.m_LocalPosition, d.m_LocalRotation, d.m_LocalScale
                self.trans[o.path_id] = dict(
                    o=o, d=d,
                    p=np.array([p.x, p.y, p.z], dtype=np.float64),
                    r=(r.x, r.y, r.z, r.w),
                    s=np.array([s.x, s.y, s.z], dtype=np.float64),
                    f=d.m_Father.m_PathID if getattr(d, "m_Father", None) else 0,
                )
            elif t == "GameObject":
                d = o.read()
                self.go_name[o.path_id] = d.m_Name
                self.go_active[o.path_id] = bool(d.m_IsActive)
        for pid, t in self.trans.items():
            self.go_t[t["d"].m_GameObject.m_PathID] = pid
        for pid, t in self.trans.items():
            self.kids[t["f"]].append(pid)

    def matrix(self, tpid, depth=0):
        if tpid in self._memo:
            return self._memo[tpid]
        t = self.trans.get(tpid)
        if t is None or depth > 128:
            return np.eye(4)
        local = np.eye(4)
        local[:3, :3] = quat_matrix(*t["r"]) * t["s"][None, :]
        local[:3, 3] = t["p"]
        f = t["f"]
        parent = self.matrix(f) if f and f in self.trans and f != tpid else np.eye(4)
        self._memo[tpid] = parent @ local
        return self._memo[tpid]

    def transform_of(self, go_id):
        return self.go_t.get(go_id)

    def world(self, go_id):
        tpid = self.go_t.get(go_id)
        if tpid is None:
            return None
        return self.matrix(tpid)

    def path(self, go_id):
        tpid = self.go_t.get(go_id)
        names = []
        seen = set()
        guard = 0
        while tpid and tpid in self.trans and guard < 256:
            guard += 1
            t = self.trans[tpid]
            gid = t["d"].m_GameObject.m_PathID
            names.append(self.go_name.get(gid, "?"))
            if tpid in seen:
                break
            seen.add(tpid)
            tpid = t["f"]
        return "/".join(reversed(names))

    def active_in_hierarchy(self, go_id):
        tpid = self.go_t.get(go_id)
        guard = 0
        while tpid and tpid in self.trans and guard < 256:
            guard += 1
            t = self.trans[tpid]
            gid = t["d"].m_GameObject.m_PathID
            if not self.go_active.get(gid, False):
                return False
            tpid = t["f"]
        return True

    def components(self, go_id, cls=None):
        go = self.objs_by_pid.get(go_id)
        if go is None or go.type.name != "GameObject":
            return []
        out = []
        try:
            d = go.read()
        except Exception:
            return out
        for c in d.m_Component:
            cp = c.component
            if not cp:
                continue
            if cls is None or cp.type.name == cls:
                out.append(cp)
        return out

    def mono_class(self, obj):
        try:
            h = obj.parse_monobehaviour_head()
            return h.m_Script.read().m_ClassName
        except Exception:
            return None

    def head(self, obj):
        try:
            return obj.parse_monobehaviour_head()
        except Exception:
            return None


def mesh_aabb(pptr, cache):
    """Local AABB of a Mesh via PPtr, cached; returns (center, extent, name) or None."""
    if pptr is None or getattr(pptr, "m_PathID", 0) == 0:
        return None
    # m_FileID is relative to the referencing file, so the cache key must use the
    # resolved serialized-file name; (file_id, path_id) collides across levels.
    try:
        key = (pptr.deref().assets_file.name, pptr.m_PathID)
    except Exception:
        return None
    if key in cache:
        return cache[key]
    try:
        m = pptr.read()
        a = m.m_LocalAABB
        res = (np.array([a.m_Center.x, a.m_Center.y, a.m_Center.z], dtype=np.float64),
               np.array([a.m_Extent.x, a.m_Extent.y, a.m_Extent.z], dtype=np.float64),
               getattr(m, "m_Name", ""))
    except Exception:
        res = None
    cache[key] = res
    return res


def scan_level(env, level, display, mesh_cache):
    sc = Scene(env, level)
    res = dict(scene=level, name=display)
    notes = []

    # ---- lights + render settings
    lights = []
    for o in sc.objs:
        if o.type.name != "Light":
            continue
        try:
            d = o.read()
        except Exception:
            continue
        go_id = d.m_GameObject.m_PathID
        m = sc.world(go_id)
        lights.append(dict(
            name=sc.go_name.get(go_id, ""), path=sc.path(go_id),
            type={0: "Spot", 1: "Directional", 2: "Point", 3: "Rectangle"}.get(d.m_Type, str(d.m_Type)),
            type_id=d.m_Type,
            position=vec(m[:3, 3]) if m is not None else None,
            rotation_quaternion=[round(float(q), 6) for q in sc.trans[sc.transform_of(go_id)]["r"]]
            if sc.transform_of(go_id) else None,
            euler_deg=[round(v, 3) for v in euler_deg(m)] if m is not None else None,
            forward=vec(m[:3, :3] @ np.array([0.0, 0.0, 1.0])) if m is not None else None,
            color=color(d.m_Color), intensity=round(float(d.m_Intensity), 6),
            shadows={0: "None", 1: "Hard", 2: "Soft"}.get(d.m_Shadows.m_Type, str(d.m_Shadows.m_Type)),
            shadow_strength=round(float(d.m_Shadows.m_Strength), 4),
            enabled=bool(d.m_Enabled),
            active_in_hierarchy=sc.active_in_hierarchy(go_id),
        ))
    res["lights"] = lights
    directional = [l for l in lights if l["type"] == "Directional"]
    # the scene sun: active, enabled, literally named "Directional Light", brightest
    sun = max(directional, key=lambda l: (
        l["active_in_hierarchy"], l["enabled"],
        l["name"].strip().lower() == "directional light",
        l["intensity"], -l["path"].count("/"),
    )) if directional else None
    res["directional_light"] = sun
    res["directional_lights"] = directional
    if len([l for l in directional if l["active_in_hierarchy"] and l["enabled"]]) > 1:
        notes.append(f"{len([l for l in directional if l['active_in_hierarchy'] and l['enabled']])} "
                     f"active directional lights; primary is {sun['name'] if sun else None}")

    rs = None
    for o in sc.objs:
        if o.type.name == "RenderSettings":
            try:
                rs = o.read()
            except Exception:
                pass
            break
    lighting = {"directional": sun}
    if rs is not None:
        sky_name = None
        try:
            if rs.m_SkyboxMaterial and rs.m_SkyboxMaterial.m_PathID:
                sky_name = rs.m_SkyboxMaterial.read().m_Name
        except Exception:
            sky_name = None
        sun_name = None
        try:
            if rs.m_Sun and rs.m_Sun.m_PathID:
                sun_name = rs.m_Sun.read().m_GameObject.read().m_Name
        except Exception:
            sun_name = None
        lighting["render_settings"] = dict(
            ambient_mode={0: "Skybox", 1: "Trilight", 2: "Flat", 3: "Custom"}.get(rs.m_AmbientMode, str(rs.m_AmbientMode)),
            ambient_mode_id=rs.m_AmbientMode,
            ambient_sky_color=color(rs.m_AmbientSkyColor),
            ambient_equator_color=color(rs.m_AmbientEquatorColor),
            ambient_ground_color=color(rs.m_AmbientGroundColor),
            ambient_light=color(rs.m_AmbientLight) if rs.m_AmbientLight else None,
            ambient_intensity=round(float(rs.m_AmbientIntensity), 6),
            skybox_material=sky_name,
            skybox_material_ptr=[rs.m_SkyboxMaterial.m_FileID, rs.m_SkyboxMaterial.m_PathID]
            if rs.m_SkyboxMaterial else None,
            sun_in_settings=sun_name,
            fog=bool(rs.m_Fog),
            fog_color=color(rs.m_FogColor),
            fog_mode={0: "Linear", 1: "Exponential", 2: "ExponentialSquared", 3: "ExponentialSquared(V3)"}.get(
                rs.m_FogMode, str(rs.m_FogMode)),
            fog_density=round(float(rs.m_FogDensity), 6),
            linear_fog_start=round(float(rs.m_LinearFogStart), 4),
            linear_fog_end=round(float(rs.m_LinearFogEnd), 4),
            default_reflection_mode={0: "Skybox", 1: "Custom"}.get(rs.m_DefaultReflectionMode, str(rs.m_DefaultReflectionMode)),
            reflection_intensity=round(float(rs.m_ReflectionIntensity), 6),
            subtractive_shadow_color=color(rs.m_SubtractiveShadowColor),
            flare_strength=round(float(rs.m_FlareStrength), 4),
            halo_strength=round(float(rs.m_HaloStrength), 4),
        )
    else:
        notes.append("no RenderSettings object in scene")
    res["lighting"] = lighting

    # ---- collect MonoBehaviours once
    monos = []
    for o in sc.objs:
        if o.type.name != "MonoBehaviour":
            continue
        h = sc.head(o)
        if h is None:
            continue
        cls = None
        try:
            cls = h.m_Script.read().m_ClassName
        except Exception:
            cls = None
        monos.append((o, h, cls))

    # ---- spawn / reset positions
    reset_positions = []
    for o, h, cls in monos:
        if cls != "ResetPosition":
            continue
        go_id = h.m_GameObject.m_PathID
        raw = o.get_raw_data()
        name = h.m_Name or ""
        off = 12 + 4 + 12 + 4 + len(name.encode("utf-8"))
        off = (off + 3) // 4 * 4
        flag = False
        if off + 4 <= len(raw):
            flag = struct.unpack_from("<I", raw, off)[0] != 0
        m = sc.world(go_id)
        reset_positions.append(dict(
            game_object=sc.go_name.get(go_id, ""), path=sc.path(go_id),
            starting_position=flag,
            position=vec(m[:3, 3]) if m is not None else None,
            yaw_deg=round(yaw_deg(m), 4) if m is not None else None,
            quaternion=[round(float(q), 6) for q in sc.trans[sc.transform_of(go_id)]["r"]]
            if sc.transform_of(go_id) else None,
            active_in_hierarchy=sc.active_in_hierarchy(go_id),
            enabled=bool(h.m_Enabled),
        ))
    res["reset_positions"] = reset_positions

    player_pos = None
    for go_id, n in sc.go_name.items():
        if n == "Player Birb":
            m = sc.world(go_id)
            if m is not None:
                player_pos = dict(
                    game_object=n, path=sc.path(go_id),
                    position=vec(m[:3, 3]), yaw_deg=round(yaw_deg(m), 4),
                    quaternion=[round(float(q), 6) for q in sc.trans[sc.transform_of(go_id)]["r"]],
                )
                break
    res["player_birb_instance"] = player_pos

    spawn_candidates = [r for r in reset_positions if r["starting_position"]]
    active_spawn = [r for r in spawn_candidates if r["active_in_hierarchy"]]
    spawn = active_spawn[0] if active_spawn else (spawn_candidates[0] if spawn_candidates else None)
    if spawn:
        res["spawn"] = dict(
            source="ResetPosition(startingPosition=true)",
            game_object=spawn["game_object"], path=spawn["path"],
            position=spawn["position"], yaw_deg=spawn["yaw_deg"], quaternion=spawn["quaternion"],
            active_in_hierarchy=spawn["active_in_hierarchy"],
        )
    elif player_pos:
        res["spawn"] = dict(source="Player Birb instance (no starting ResetPosition)",
                            game_object=player_pos["game_object"], path=player_pos["path"],
                            position=player_pos["position"], yaw_deg=player_pos["yaw_deg"],
                            quaternion=player_pos["quaternion"], active_in_hierarchy=True)
        notes.append("no startingPosition ResetPosition; used Player Birb placement")
    else:
        res["spawn"] = None
        notes.append("no spawn found")

    # ---- mission givers
    mission_givers = []
    for o, h, cls in monos:
        if cls != "MissionGiver":
            continue
        go_id = h.m_GameObject.m_PathID
        m = sc.world(go_id)
        go_name = sc.go_name.get(go_id, "")
        path = sc.path(go_id)
        label = go_name
        if go_name in ("Mission Giver", "MissionGiver") and "/" in path:
            seg = path.split("/")
            # .../<Mission>/Mission Giver -> mission name
            if len(seg) >= 2:
                label = seg[-2]
        mission_givers.append(dict(
            name=go_name, label=label, path=path,
            position=vec(m[:3, 3]) if m is not None else None,
            yaw_deg=round(yaw_deg(m), 4) if m is not None else None,
            active_in_hierarchy=sc.active_in_hierarchy(go_id),
        ))
    res["mission_givers"] = mission_givers

    # ---- named landmarks (level element roots and their named descendants)
    landmark_roots = []
    for go_id, n in sc.go_name.items():
        if n in ("Level Elements", "Level Contents", "Core Elements", "Non-Static Level Elements",
                 "Partially Static Level Elements", "Partially Dynamic Level Elements",
                 "Holiday Enabled Level Elements", "Level", "Ground Stuff", "Main Tower",
                 "Brick Bedroom"):
            landmark_roots.append(go_id)
    landmarks = []
    seen_names = set()

    def geometry_world_bounds(gid):
        """Union of world AABBs of renderer/collider components on a GameObject."""
        blo = np.array([np.inf] * 3)
        bhi = np.array([-np.inf] * 3)
        m = sc.world(gid)
        if m is None:
            return None
        for cp in sc.components(gid):
            try:
                if cp.type.name == "MeshRenderer":
                    d = cp.read()
                    a = mesh_aabb(d.m_Mesh, mesh_cache)
                    if a is None:
                        continue
                    wp = (m[:3, :3] @ box_corners(a[0], a[1]).T).T + m[:3, 3]
                    blo = np.minimum(blo, wp.min(axis=0))
                    bhi = np.maximum(bhi, wp.max(axis=0))
                elif cp.type.name == "BoxCollider":
                    d = cp.read()
                    wp = (m[:3, :3] @ box_corners(
                        [d.m_Center.x, d.m_Center.y, d.m_Center.z],
                        [d.m_Size.x / 2, d.m_Size.y / 2, d.m_Size.z / 2]).T).T + m[:3, 3]
                    blo = np.minimum(blo, wp.min(axis=0))
                    bhi = np.maximum(bhi, wp.max(axis=0))
            except Exception:
                continue
        if not np.isfinite(blo).all():
            return None
        return blo, bhi

    for root in landmark_roots:
        tpid = sc.go_t.get(root)
        if tpid is None:
            continue
        # BFS: the first named node with geometry on a branch becomes its landmark,
        # so a building is one landmark instead of every bolt under it.
        frontier = [(k, 1) for k in sc.kids.get(tpid, [])]
        while frontier and len(landmarks) < 60:
            kid, depth = frontier.pop(0)
            if depth > 4:
                continue
            gid = sc.trans[kid]["d"].m_GameObject.m_PathID
            nm = sc.go_name.get(gid, "")
            qualified = False
            if nm and not nm.startswith(("pb_Mesh", "Combined Mesh", "Generated_", "ProBuilder")):
                if nm not in seen_names and nm not in ("Bounds", "Dev Zone Trigger"):
                    comps = sc.components(gid)
                    if any(c.type.name in ("MeshRenderer", "MeshCollider", "BoxCollider",
                                           "SkinnedMeshRenderer") for c in comps):
                        m = sc.matrix(kid)
                        pos = vec(m[:3, 3])
                        if abs(pos[0]) < 1e-6 and abs(pos[1]) < 1e-6 and abs(pos[2]) < 1e-6:
                            gb = geometry_world_bounds(gid)
                            if gb:
                                pos = vec((gb[0] + gb[1]) / 2)
                        seen_names.add(nm)
                        landmarks.append(dict(
                            name=nm, path=sc.path(gid), position=pos,
                            active_in_hierarchy=sc.active_in_hierarchy(gid),
                        ))
                        qualified = True
            if not qualified and depth < 4:
                frontier.extend((k, depth + 1) for k in sc.kids.get(kid, []))
    res["landmarks"] = landmarks

    # ---- collider bounds
    lo = np.array([np.inf] * 3)
    hi = np.array([-np.inf] * 3)
    lo_mesh = np.array([np.inf] * 3)
    hi_mesh = np.array([-np.inf] * 3)
    lo_all = np.array([np.inf] * 3)
    hi_all = np.array([-np.inf] * 3)
    counts = collections.Counter()
    excluded = collections.Counter()
    oversized = []
    colliders_all = collections.Counter()
    triggers = 0
    disabled = 0
    inactive = 0
    failed = 0
    for o in sc.objs:
        t = o.type.name
        if t not in ("MeshCollider", "BoxCollider", "SphereCollider", "CapsuleCollider"):
            continue
        colliders_all[t] += 1
        try:
            d = o.read()
        except Exception:
            failed += 1
            continue
        go_id = d.m_GameObject.m_PathID
        if getattr(d, "m_IsTrigger", False):
            triggers += 1
            continue
        if not getattr(d, "m_Enabled", True):
            disabled += 1
            continue
        if not sc.active_in_hierarchy(go_id):
            inactive += 1
            continue
        m = sc.world(go_id)
        if m is None:
            failed += 1
            continue
        if t == "MeshCollider":
            aabb = mesh_aabb(d.m_Mesh, mesh_cache)
            if aabb is None:
                failed += 1
                continue
            c, e, mname = aabb
            wp = (m[:3, :3] @ box_corners(c, e).T).T + m[:3, 3]
        elif t == "BoxCollider":
            wp = (m[:3, :3] @ box_corners(
                np.array([d.m_Center.x, d.m_Center.y, d.m_Center.z]),
                np.array([abs(d.m_Size.x), abs(d.m_Size.y), abs(d.m_Size.z)]) / 2.0).T).T + m[:3, 3]
        elif t == "SphereCollider":
            r = float(d.m_Radius)
            wp = (m[:3, :3] @ box_corners(
                np.array([d.m_Center.x, d.m_Center.y, d.m_Center.z]),
                np.array([r, r, r])).T).T + m[:3, 3]
        else:  # CapsuleCollider
            r = float(d.m_Radius)
            h = max(float(d.m_Height) / 2.0, r)
            ax = int(d.m_Direction)
            half = np.array([r, r, r], dtype=np.float64)
            half[ax] = h
            wp = (m[:3, :3] @ box_corners(
                np.array([d.m_Center.x, d.m_Center.y, d.m_Center.z]), half).T).T + m[:3, 3]
        if not np.isfinite(wp).all():
            failed += 1
            continue
        cmin, cmax = wp.min(axis=0), wp.max(axis=0)
        lo_all = np.minimum(lo_all, cmin)
        hi_all = np.maximum(hi_all, cmax)
        size = cmax - cmin
        if float(size.max()) > MAX_MESH_SIZE:
            excluded[f"{t}_over_{MAX_MESH_SIZE:g}m"] += 1
            if len(oversized) < 25:
                oversized.append(dict(
                    type=t, name=sc.go_name.get(go_id, ""), path=sc.path(go_id),
                    min=vec(cmin), max=vec(cmax), size=vec(size),
                ))
            continue
        counts[t] += 1
        lo = np.minimum(lo, cmin)
        hi = np.maximum(hi, cmax)
        if t == "MeshCollider":
            lo_mesh = np.minimum(lo_mesh, cmin)
            hi_mesh = np.maximum(hi_mesh, cmax)

    if not np.isfinite(lo).all():
        bounds = None
        notes.append("no usable colliders")
    else:
        bounds = dict(
            min=vec(lo), max=vec(hi), size=vec(hi - lo), center=vec((hi + lo) / 2),
            colliders_used=dict(counts),
            colliders_total=dict(colliders_all),
            excluded=dict(excluded),
            excluded_colliders=oversized,
            skipped=dict(triggers=triggers, disabled=disabled, inactive=inactive, failed=failed),
            rule="enabled, non-trigger, active-in-hierarchy colliders; MeshCollider AABBs resolved through "
                 f"the collider transform; every collider whose world-space AABB exceeds {MAX_MESH_SIZE:g} m "
                 "on any axis is excluded (bad static-batch/backdrop/oversized boundary volumes)",
            unfiltered_min=vec(lo_all) if np.isfinite(lo_all).all() else None,
            unfiltered_max=vec(hi_all) if np.isfinite(hi_all).all() else None,
            mesh_collider_min=vec(lo_mesh) if np.isfinite(lo_mesh).all() else None,
            mesh_collider_max=vec(hi_mesh) if np.isfinite(hi_mesh).all() else None,
        )
    res["bounds"] = bounds

    # ---- travel points (curated: reset anchors + mission givers + landmarks)
    def spread(pool, n):
        """Farthest-point sample positions to keep travel points distributed."""
        if len(pool) <= n:
            return list(pool)
        chosen = [pool[0]]
        while len(chosen) < n:
            best, bestd = None, -1.0
            for cand in pool:
                if cand in chosen or cand.get("position") is None:
                    continue
                d = min(sum((a - b) ** 2 for a, b in zip(cand["position"], c["position"]))
                        for c in chosen if c.get("position") is not None)
                if d > bestd:
                    bestd, best = d, cand
            if best is None:
                break
            chosen.append(best)
        return chosen

    def inside(pos, margin=10.0):
        if pos is None or bounds is None:
            return True
        return all(bounds["min"][i] - margin <= pos[i] <= bounds["max"][i] + margin for i in range(3))

    travel = []
    reset_pool = [r for r in reset_positions if r["active_in_hierarchy"] and inside(r["position"])]
    if not reset_pool:
        reset_pool = [r for r in reset_positions if r["active_in_hierarchy"]] or reset_positions
    by_label = {}
    for g in mission_givers:
        key = g["label"]
        if key not in by_label and inside(g["position"]):
            by_label[key] = g
    giver_pool = [g for g in by_label.values() if g["active_in_hierarchy"]] or list(by_label.values())

    # no mission structure (flashback): more reset anchors instead
    reset_cap = 8 if giver_pool else 12
    starters = [r for r in reset_pool if r["starting_position"]]
    rest = [r for r in reset_pool if not r["starting_position"]]
    chosen_resets = (starters[:1] + spread(rest, reset_cap - min(1, len(starters))))[:reset_cap]
    for r in chosen_resets:
        travel.append(dict(name=r["game_object"], kind="reset_position", path=r["path"],
                           position=r["position"], yaw_deg=r["yaw_deg"],
                           active_in_hierarchy=r["active_in_hierarchy"],
                           starting_position=r["starting_position"]))
    for g in spread(giver_pool, 8):
        travel.append(dict(name=g["label"], kind="mission_giver", path=g["path"],
                           position=g["position"], yaw_deg=g["yaw_deg"],
                           active_in_hierarchy=g["active_in_hierarchy"]))
    if len(travel) < 3:
        lowmark_pool = [l for l in landmarks
                        if l["active_in_hierarchy"] and inside(l["position"])] or \
                       [l for l in landmarks if inside(l["position"])]
        for l in spread(lowmark_pool, 3 - len(travel)):
            travel.append(dict(name=l["name"], kind="landmark", path=l["path"],
                               position=l["position"], yaw_deg=None,
                               active_in_hierarchy=l["active_in_hierarchy"]))
    res["travel_points"] = travel

    res["notes"] = notes
    return res


def main():
    env = UnityPy.load(DATA)
    mesh_cache = {}
    out = {"source": DATA, "levels": {}}
    for level, display in LEVELS:
        res = scan_level(env, level, display, mesh_cache)
        out["levels"][level] = res
        sp = res["spawn"]
        b = res["bounds"]
        print(f"{level:8s} {display:13s} spawn={sp['position'] if sp else None} "
              f"yaw={sp['yaw_deg'] if sp else None} "
              f"bounds={b['min'] if b else None}..{b['max'] if b else None} "
              f"travel={len(res['travel_points'])} resets={len(res['reset_positions'])} "
              f"givers={len(res['mission_givers'])} landmarks={len(res['landmarks'])} "
              f"sun={'yes' if res['directional_light'] else 'NO'} notes={res['notes']}")
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
    print("wrote", OUT)
    return out


if __name__ == "__main__":
    main()
