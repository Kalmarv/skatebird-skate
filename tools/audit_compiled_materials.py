"""Which shader programs and textures does a Studio compile bind per material key?

  python -I tools/audit_compiled_materials.py PATCH [--game DIR] [--texture SUBSTR ...] [--json OUT]

PATCH is a compiled Studio stage (`.../stage/Patch`); --game the skate. install whose Data the patch layers on
(default C:/Users/kalmarv/reskate). Reads every patch bundle with reskate-tools' readers (kit/gpl fbpatch, fbebx,
sbdfmt; ports/gta/stitch blk; nothing new parsed here): the program lookup rows (material key -> program ids) and the
material depots (key -> parameter block -> texture file GUIDs, named through the patch's texture EBX).

Prints, per program tuple, how many keys use it, and for each key whose bound texture names contain one of the
--texture substrings, its programs and textures. Earlier records: Bird's leaf key fd9936984b8d4d4c bound the stock
decal/blend programs (1444, 3612); Sledding's rendered cutout keys (825, 1891, 3096); opaque surfaces
(682, 819, 1474, 2756).
"""
import argparse
import collections
import json
import struct
import sys
from pathlib import Path

TOOLS = Path("C:/Users/kalmarv/modding/reskate/reskate-tools")
sys.path[:0] = [str(TOOLS / "kit/gpl"), str(TOOLS / "ports/gta/stitch")]
import blk  # noqa: E402
import fbebx  # noqa: E402
import fbpatch as fp  # noqa: E402
import sbdfmt  # noqa: E402

DEPOT = 0x73312045


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("patch")
    ap.add_argument("--game", default="C:/Users/kalmarv/reskate")
    ap.add_argument("--texture", nargs="*", default=[])
    ap.add_argument("--json")
    args = ap.parse_args()
    patch, game = Path(args.patch), Path(args.game)
    F, B = fp.Folder(str(patch)), fp.Folder(str(game / "Data"))

    def payload(f):
        return fp.decode_cas(fp.read_any(F, B, *f), str(game))

    textures, programs, bound = {}, {}, collections.defaultdict(set)
    for rel in F.tocs:
        toc = fp.read_toc((patch / rel).read_bytes())
        for bundle, region in toc.bundles:
            files, b, first, _ = fp.list_bundle(F, region, str(game), B)
            if b is None or first != 1 or not files[0][0].patch:
                continue
            for i, a in enumerate(b.ebx):
                if "/textures/" in a.name and files[first + i][0].patch:
                    textures[bytes(fbebx.read_file_guid(payload(files[first + i]))).hex()] = a.name
            for i, a in enumerate(b.res):
                if a.rtype not in (DEPOT, fp.PROGRAM_LOOKUP):
                    continue
                data = payload(files[first + len(b.ebx) + i])
                if a.rtype == fp.PROGRAM_LOOKUP:
                    for k, value in fp._program_rows(data, a.rmeta).items():
                        programs[f"{k:016x}"] = struct.unpack("<%dI" % (len(value) // 4), value)
                    continue
                depot = sbdfmt.read(data, a.rmeta)
                for key, bi in depot.entries:
                    params, pos, error = blk.parse(depot.blocks[bi].data)
                    if error:
                        continue
                    for name, typ, count, x, value in params:
                        if value[0] == "tex":
                            bound[f"{key:016x}"].add(value[3].hex())
    keys = sorted(set(programs) | set(bound))
    rows = {k: {"programs": list(programs.get(k, ())),
                "textures": sorted(textures.get(g, g) for g in bound.get(k, ()))} for k in keys}
    by_programs = collections.Counter(tuple(r["programs"]) for r in rows.values())
    print(f"{len(keys)} material keys, {len(textures)} patch textures")
    for t, n in by_programs.most_common():
        print(f"  programs {t}: {n} keys")
    for k, r in rows.items():
        if any(s.lower() in name.lower() for s in args.texture for name in r["textures"]):
            print(f"{k}: programs {tuple(r['programs'])} textures {r['textures']}")
    if args.json:
        Path(args.json).write_text(json.dumps(rows, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
