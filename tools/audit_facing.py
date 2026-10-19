"""Which way do a map object's triangles face? Winding vs stored normals, and the share facing a camera point.

  python -I tools/audit_facing.py MAPDIR MATERIAL [MATERIAL ...] [--camera X Y Z] [--json OUT]

MAPDIR is a built map (map.json + meshes/); MATERIAL a map.json material key (or a substring of one). For every
object using it, the tool reads the glTF primitive (no glTF library: the GLB's JSON chunk and buffer views), applies
the object's transform, and reports per object:

- triangles, and how many have a winding normal (counter-clockwise, glTF's front face) that agrees / disagrees with
  the mean of the stored vertex normals (dot > 0 / < 0): a mesh whose winding disagrees with its normals is lit as
  one side and culled as the other;
- with --camera (map coordinates; skate scene = Unity (x, y, -z)): how many triangles are front-facing to that point,
  i.e. survive single-sided back-face culling;
- the spread of winding normals (share pointing up / down / sideways).

Run it on a material that renders as a control (e.g. the trunk material of the same tree mesh).
"""
import argparse
import json
import struct
import sys
from pathlib import Path

import numpy as np

COMPONENT = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
WIDTH = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def read_glb(path):
    data = Path(path).read_bytes()
    magic, version, length = struct.unpack_from("<III", data, 0)
    if magic != 0x46546C67:
        raise ValueError(f"{path}: not a GLB")
    off, doc, binary = 12, None, None
    while off < length:
        clen, ctype = struct.unpack_from("<II", data, off)
        chunk = data[off + 8: off + 8 + clen]
        if ctype == 0x4E4F534A:
            doc = json.loads(chunk)
        elif ctype == 0x004E4942:
            binary = chunk
        off += 8 + clen
    return doc, binary


def accessor(doc, binary, index):
    acc = doc["accessors"][index]
    view = doc["bufferViews"][acc["bufferView"]]
    dtype = np.dtype(COMPONENT[acc["componentType"]])
    width = WIDTH[acc["type"]]
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = view.get("byteStride") or dtype.itemsize * width
    count = acc["count"]
    raw = np.frombuffer(binary, dtype=np.uint8, count=stride * (count - 1) + dtype.itemsize * width, offset=start)
    rows = np.lib.stride_tricks.as_strided(raw, shape=(count, dtype.itemsize * width), strides=(stride, 1))
    out = np.ascontiguousarray(rows).view(dtype).reshape(count, width)
    if acc.get("normalized"):
        out = out.astype(np.float32) / np.iinfo(dtype).max
    return out.astype(np.float64)


def primitives(doc):
    """Flat list of primitives in mesh order (map.json's "primitive" index)."""
    return [p for m in doc["meshes"] for p in m["primitives"]]


def audit(mapdir, obj, camera):
    doc, binary = read_glb(mapdir / obj["mesh"])
    prim = primitives(doc)[obj.get("primitive", 0)]
    pos = accessor(doc, binary, prim["attributes"]["POSITION"])
    nrm = accessor(doc, binary, prim["attributes"]["NORMAL"]) if "NORMAL" in prim["attributes"] else None
    idx = accessor(doc, binary, prim["indices"]).astype(np.int64).reshape(-1, 3) if "indices" in prim else \
        np.arange(len(pos)).reshape(-1, 3)
    t = np.array(obj.get("transform", [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]), dtype=np.float64)
    # map.json transform: 3x3 basis (rows as written, applied as columns) then translation.
    basis, origin = t[:9].reshape(3, 3), t[9:12]
    world = pos @ basis + origin
    a, b, c = world[idx[:, 0]], world[idx[:, 1]], world[idx[:, 2]]
    wn = np.cross(b - a, c - a)
    area2 = np.linalg.norm(wn, axis=1)
    ok = area2 > 1e-12
    wn = wn[ok] / area2[ok, None]
    res = {"object": obj["name"], "mesh": obj["mesh"], "primitive": obj.get("primitive", 0),
           "material": obj["material"], "triangles": int(len(idx)), "degenerate": int((~ok).sum())}
    if nrm is not None:
        sn = nrm[idx[ok]].mean(axis=1) @ basis
        d = (wn * sn).sum(axis=1)
        res["winding_vs_normals"] = {"agree": int((d > 0).sum()), "disagree": int((d < 0).sum())}
    res["winding_normal_y"] = {"up(>0.5)": int((wn[:, 1] > 0.5).sum()), "down(<-0.5)": int((wn[:, 1] < -0.5).sum()),
                               "side": int((np.abs(wn[:, 1]) <= 0.5).sum())}
    centroid = (a + b + c)[ok] / 3
    res["bounds"] = [world.min(axis=0).round(2).tolist(), world.max(axis=0).round(2).tolist()]
    if camera is not None:
        to_cam = np.asarray(camera) - centroid
        front = (wn * to_cam).sum(axis=1) > 0
        res["camera"] = list(camera)
        res["front_facing_to_camera"] = {"front": int(front.sum()), "back": int((~front).sum())}
    return res


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mapdir")
    ap.add_argument("materials", nargs="+")
    ap.add_argument("--camera", type=float, nargs=3)
    ap.add_argument("--json")
    args = ap.parse_args()
    mapdir = Path(args.mapdir)
    doc = json.loads((mapdir / "map.json").read_text(encoding="utf-8"))
    out = []
    for key in args.materials:
        objs = [o for o in doc["objects"] if key in o.get("material", "") and o.get("mesh")]
        if not objs:
            print(f"{key}: no object uses it", file=sys.stderr)
        for o in objs:
            r = audit(mapdir, o, args.camera)
            out.append(r)
            print(json.dumps(r))
    if args.json:
        Path(args.json).write_text(json.dumps(out, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
