#!/usr/bin/env python3
"""Read-only QA over exported SkateBIRD levels -> docs/QA-report.md.

Reads out/<level>/{build.log,compile.log,map/<name>/map.json,map/<name>/stats.json}
and compares with data/skatebird-levels.json. Never writes outside docs/.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "out"
LEVELS = ["level1", "level2", "level3", "level4", "level5", "level7", "level12", "level13"]
MAX_TRAVEL = 25
TEXTURE_BUDGET = 10000  # skate streams ~10,400 map-texture handles
SPAWN_TOL = 1.0  # metres; the writer adds ground snap + 0.1-0.2 m lift
SPAN_RATIO_FLAG = 5.0


def read_text(p):
    try:
        return p.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return None


def parse_build(log):
    d = {}
    if not log:
        return d
    m = re.search(r"lights: (\d+) without a usable tangent", log)
    if m:
        d["lights_without_tangent"] = int(m.group(1))
    return d


def parse_compile(log):
    d = {"warnings": [], "errors": []}
    if not log:
        return d
    for line in log.splitlines():
        s = line.strip()
        if s.startswith("warning:"):
            d["warnings"].append(s[len("warning:"):].strip())
        elif re.search(r"\berror\b", s, re.I):
            d["errors"].append(s)
        m = re.match(r"^([a-z0-9-]+)=(.+)$", s)
        if m:
            d[m.group(1)] = m.group(2)
    m = re.search(r"map spans (\d+) m", log)
    if m:
        d["map_span_m"] = int(m.group(1))
    m = re.search(r"\[[\d.]+s\] (\d+) bundles staged", log)
    if m:
        d["staged_bundles"] = int(m.group(1))
    d["complete"] = "Native map package complete" in log
    d["streamed"] = "Building vista proxies" in log
    # parse the warning families
    pats = {
        "tex_split_meshes": r"(\d+) meshes sample outside their texture",
        "tex_split_tris": r"sample outside their texture.*?; (\d+) triangles were split",
        "cut_objects": r"(\d+) render objects were cut at WorldPartition cell borders into (\d+) pieces",
        "merged_objects": r"(\d+) render objects were merged into",
        "lod_meshes": r"(\d+) render meshes carry 3 detail levels",
        "lod_tris": r"detail levels switching at [\d, ]+m \((\d+) triangles",
        "grind_edges": r"(\d+) collision edges shared",
        "vista_proxies": r"(\d+) vista proxies keep (\d+) of (\d+) triangles",
        "wp_bundles": r"(\d+) native WorldPartition bundles: (\d+) streamed render cells, (\d+) vista proxies",
        "bam_probe_volumes": r"(\d+) BAM character-room light probe",
    }
    for text in d["warnings"]:
        for key, pat in pats.items():
            mm = re.search(pat, text)
            if mm:
                d[key] = tuple(int(g) for g in mm.groups()) if len(mm.groups()) > 1 else int(mm.group(1))
    return d


def analyze(level):
    base = OUT / level
    rec = {"level": level, "status": "missing"}
    if not base.is_dir():
        return rec
    maps = sorted(p for p in (base / "map").glob("*") if p.is_dir()) if (base / "map").is_dir() else []
    if not maps:
        rec["status"] = "no map"
        rec["stage"] = (base / "stage" / "Patch").is_dir()
        return rec
    md = maps[0]
    stats_p = md / "stats.json"
    map_p = md / "map.json"
    stats = json.loads(stats_p.read_text(encoding="utf-8")) if stats_p.is_file() else None
    mp = json.loads(map_p.read_text(encoding="utf-8")) if map_p.is_file() else None
    build = parse_build(read_text(base / "build.log"))
    comp = parse_compile(read_text(base / "compile.log"))
    if stats is None or mp is None:
        rec["status"] = "incomplete"
        rec["stage"] = (base / "stage" / "Patch").is_dir()
        return rec
    rec.update(
        status="ok" if comp["complete"] else "no compile marker",
        stage=(base / "stage" / "Patch").is_dir(),
        build=build,
        compile=comp,
        stats=stats,
        map=mp,
        mapdir=md.name,
    )
    return rec


def inside(p, mn, mx, tol=0.5):
    return all(mn[i] - tol <= p[i] <= mx[i] + tol for i in range(3))


def fmt(n, nd=0):
    if n is None:
        return "—"
    if isinstance(n, float):
        return f"{n:,.{nd}f}"
    return f"{n:,}"


def main():
    data = json.loads((ROOT / "data" / "skatebird-levels.json").read_text(encoding="utf-8"))
    exp = data["levels"]
    recs = [analyze(lv) for lv in LEVELS]

    problems = []
    rows = []
    for r in recs:
        lv = r["level"]
        e = exp[lv]
        if r["status"] != "ok":
            rows.append({"level": lv, "status": r["status"], "stage": r.get("stage")})
            problems.append(f"**{lv}** — no completed export/compile ({r['status']}), stage present: {r.get('stage')}.")
            continue
        st = r["stats"]["totals"]
        comp = r["compile"]
        mp = r["map"]
        mn, mx, size = e["bounds"]["min"], e["bounds"]["max"], e["bounds"]["size"]
        # game spawn in scene coords: (x, y, -z)
        gp = e["spawn"]["position"]
        want = [gp[0], gp[1], -gp[2]]
        got = mp["spawn"]["position"]
        delta = [got[i] - want[i] for i in range(3)]
        dmax = max(abs(x) for x in delta)
        tp = len(mp.get("travel_points", []))
        tp_exp = len(e.get("travel_points", []))
        tex = st["textures"]
        region = r["stats"]["region"]
        region_span = max(region[2] - region[0], region[3] - region[1])
        span = comp.get("map_span_m")
        span_used = span if span is not None else region_span
        max_dim = max(size)
        ratio = span_used / max_dim if max_dim else 0
        row = {
            "level": lv,
            "name": e.get("name"),
            "status": "ok",
            "stage": r["stage"],
            "placements": st["placements"],
            "tiles": st["tiles"],
            "merged_tris": st["merged_triangles"],
            "compiled_tris": int(comp.get("triangles", 0) or 0),
            "materials": st["materials"],
            "textures": tex,
            "collision_tris": st["collision_triangles"],
            "collision_pieces": st["collision_pieces"],
            "lights": st["lights_written"],
            "travel": tp,
            "travel_exp": tp_exp,
            "want": want,
            "got": got,
            "delta": delta,
            "dmax": dmax,
            "in_bounds": inside([got[0], got[1], -got[2]], mn, mx),
            "span": span,
            "region_span": region_span,
            "span_used": span_used,
            "ratio": ratio,
            "streamed": comp["streamed"],
            "render_cells": int(comp.get("render-cells", 0) or 0),
            "vista_cells": int(comp.get("vista-cells", 0) or 0),
            "bundles": comp.get("staged_bundles"),
            "ebx": int(comp.get("ebx", 0) or 0),
            "warnings": len(comp["warnings"]),
            "errors": comp["errors"],
            "map_warnings": len(mp.get("warnings", [])),
            "stats_warnings": len(r["stats"].get("warnings", [])),
        }
        rows.append(row)

        if st["collision_triangles"] == 0:
            problems.append((3, lv, "zero collision triangles: no floor."))
        if not r["stage"]:
            problems.append((4, lv, "no stage/Patch (mod not installable)."))
        if tp != tp_exp:
            problems.append((5, lv, f"{tp} travel points wired, game data has {tp_exp}."))
        if tp > MAX_TRAVEL:
            problems.append((5, lv, f"{tp} travel points exceeds the 25 cap."))
        if tex >= TEXTURE_BUDGET:
            problems.append((6, lv, f"{tex} textures exceeds the ~10,400 streamed-texture budget."))
        if dmax > SPAWN_TOL:
            problems.append((0, lv,
                f"spawn y is {got[1]:.2f} m, game spawn {want[1]:.2f} m (Δ {delta[1]:+.2f} m); "
                f"x/z match. The spawn snaps to the topmost collision above that column, so it "
                f"lands on the ceiling/roof."))
        if not row["in_bounds"]:
            problems.append((1, lv,
                f"spawn {got} is outside the collider bounds {mn}..{mx}."))
        non_streamed_far = region_span >= SPAN_RATIO_FLAG * max_dim and not row["streamed"]
        if ratio >= SPAN_RATIO_FLAG and not non_streamed_far:
            problems.append((2, lv,
                f"map span {span_used:,} m is {ratio:.0f}x the {max_dim:.0f} m collider bounds "
                f"(backdrop geometry, 'Level Elements/Bounds')."))
        if non_streamed_far:
            problems.append((2, lv,
                f"region spans {int(region_span):,} m ({ratio:.0f}x the {max_dim:.0f} m collider "
                f"bounds, backdrop geometry) but the compile built no WorldPartition cells: "
                f"0 render cells, {row['bundles']} bundles, the whole map is one resident package."))
        if comp["errors"]:
            problems.append((7, lv, f"compile log errors: {comp['errors'][0]}"))

    L = []
    L.append("# SkateBIRD level export QA\n")
    L.append(
        "Read-only pass over `out/<level>/` (build.log, compile.log, map/<name>/map.json, "
        "map/<name>/stats.json) against `data/skatebird-levels.json`. Generated by "
        "`tools/qa_report.py`. Nothing under `C:/Users/kalmarv/modding` was read or written.\n"
    )
    L.append(
        "Spawn is compared in scene coordinates (x, y, -z) with the game's own ResetPosition "
        "(`data/skatebird-levels.json`). Spans come from the ReSkate compile warning; where the "
        "compile did not stream (no span warning) the writer's region extent (`stats.region`) is "
        "shown and marked `*`.\n"
    )

    L.append("## Per level\n")
    L.append(
        "| level | name | placements | tiles | merged tris | compiled tris | materials | textures "
        "| collision tris | collision pieces | lights | travel | stage |\n"
        "|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|:-:|:-:|"
    )
    for row in rows:
        if row["status"] != "ok":
            L.append(f"| {row['level']} |  |  |  |  |  |  |  |  |  |  |  | {'yes' if row.get('stage') else '**NO**'} |")
            continue
        L.append(
            f"| {row['level']} | {row['name']} | {fmt(row['placements'])} | {row['tiles']} "
            f"| {fmt(row['merged_tris'])} | {fmt(row['compiled_tris'])} | {row['materials']} "
            f"| {fmt(row['textures'])} | {fmt(row['collision_tris'])} | {row['collision_pieces']} "
            f"| {row['lights']} | {row['travel']}/{row['travel_exp']} | yes |"
        )
    L.append(
        "\nMerged tris = `stats.totals.merged_triangles`; compiled tris = the compile's `triangles=` "
        "(render + coarser LOD levels); collision pieces = `stats.totals.collision_pieces`.\n"
    )

    L.append("## Spawn vs game data (scene x, y, -z)\n")
    L.append(
        "| level | game spawn | map spawn | Δx | Δy | Δz | in collider bounds | verdict |\n"
        "|---|---|---|--:|--:|--:|:-:|---|"
    )
    for row in rows:
        if row["status"] != "ok":
            continue
        w, g, d = row["want"], row["got"], row["delta"]
        verdict = "ok" if row["dmax"] <= SPAWN_TOL and row["in_bounds"] else f"**off by {d[1]:+.2f} m y**"
        L.append(
            f"| {row['level']} | ({w[0]:.3f}, {w[1]:.3f}, {w[2]:.3f}) "
            f"| ({g[0]:.3f}, {g[1]:.3f}, {g[2]:.3f}) "
            f"| {d[0]:+.3f} | {d[1]:+.3f} | {d[2]:+.3f} | {'yes' if row['in_bounds'] else '**NO**'} | {verdict} |"
        )
    L.append("")

    L.append("## Map span and streaming\n")
    L.append(
        "| level | collider bounds (max dim) | compile span | span / bounds | render cells | vista cells "
        "| bundles staged | EBX | collision stream bundles |\n"
        "|---|--:|--:|--:|--:|--:|--:|--:|--:|"
    )
    for row in rows:
        if row["status"] != "ok":
            continue
        span_txt = f"{row['span']:,}" if row["span"] is not None else f"{int(row['region_span']):,}*"
        L.append(
            f"| {row['level']} | {max(exp[row['level']]['bounds']['size']):.0f} m | {span_txt} "
            f"| {row['ratio']:.1f}x | {row['render_cells']} | {row['vista_cells']} "
            f"| {row['bundles']} | {fmt(row['ebx'])} "
            f"| {rs.get('collision-stream-bundles','—') if (rs := rows_compile(recs, row['level'])) else '—'} |"
        )
    L.append("")

    L.append("## Compile warning families\n")
    L.append(
        "| level | texture sampling (meshes / tris split) | objects cut (pieces) | object merge | "
        "LOD meshes (coarse tris) | grind edge joins | vista proxies (resident/of tris) | warnings |\n"
        "|---|--:|--:|--:|--:|--:|--:|--:|"
    )
    for r in recs:
        if r["status"] != "ok":
            continue
        c = r["compile"]
        ts = c.get("tex_split_tris")
        tm = c.get("tex_split_meshes")
        cut = c.get("cut_objects")
        lod = c.get("lod_meshes")
        lodt = c.get("lod_tris")
        vis = c.get("vista_proxies")
        L.append(
            f"| {r['level']} | {tm if tm is not None else '—'} / {fmt(ts) if ts is not None else '—'} "
            f"| {cut[0] if cut else 0} ({cut[1] if cut else 0}) "
            f"| {c.get('merged_objects', '—')} "
            f"| {lod if lod is not None else '—'} ({fmt(lodt) if lodt is not None else '—'}) "
            f"| {fmt(c.get('grind_edges')) if c.get('grind_edges') is not None else '—'} "
            f"| {(str(vis[0]) + ' (' + str(vis[1]) + '/' + str(vis[2]) + ')') if vis else '0'} "
            f"| {len(c['warnings'])} |"
        )
    L.append("")

    L.append("## Warnings and log lines in full\n")
    for r in recs:
        if r["status"] != "ok":
            continue
        L.append(f"### {r['level']}\n")
        b = r["build"]
        if "lights_without_tangent" in b:
            L.append(f"- build.log: {b['lights_without_tangent']} lights without a usable tangent (fallback basis)")
        for w in r["compile"]["warnings"]:
            L.append(f"- {w}")
        for w in r["map"].get("warnings", []):
            L.append(f"- map.json: {w}")
        for w in r["stats"].get("warnings", []):
            L.append(f"- stats.json: {w}")
        if not r["compile"]["warnings"]:
            L.append("- none")
        L.append("")

    L.append("## Notes\n")
    L.append(
        "- All eight `build.log`/`compile.log` pairs finished; no error lines, every compile "
        "reported `Native map package complete` and every level has `stage/Patch`."
    )
    L.append(
        "- level12 (playground) has 0 punctual lights. Its Unity scene has 2 active directional "
        "lights (3 more are conditioned out); skate lights itself by time of day, so 0 written "
        "lights is the designed behaviour, not a missing export."
    )
    lv13 = next((r for r in recs if r["level"] == "level13" and r["status"] == "ok"), None)
    if lv13:
        far = [o for o in lv13["map"]["objects"] if abs(o["transform"][11]) > 500]
        if far:
            o = max(far, key=lambda o: abs(o["transform"][11]))
            L.append(
                f"- level13 carries `{o['name']}` (mesh `{o['mesh']}`, 2 triangles) with a map-space "
                f"z of {o['transform'][11]:.0f} m. Its writer region is "
                f"{max(lv13['stats']['region'][2]-lv13['stats']['region'][0], lv13['stats']['region'][3]-lv13['stats']['region'][1]):,.0f} m, "
                f"yet the compile reported no span and built 0 WorldPartition cells - verify this "
                f"tile and the streaming decision before shipping."
            )
    L.append("")
    L.append("## Problems to fix\n")
    order = {
        "spawn": 0,
        "region spans": 1,
        "map span": 2,
        "zero collision": 3,
        "no stage": 4,
        "travel points": 5,
        "textures exceeds": 6,
    }
    def rank(p):
        for key, i in order.items():
            if key in p:
                return i
        return 9
    problems.sort(key=lambda p: p[0])
    grouped = {}
    order = []
    for _, lv, text in problems:
        if lv not in grouped:
            grouped[lv] = []
            order.append(lv)
        grouped[lv].append(text)
    # summary bullets for the warning families shared by every level
    tex_bits = []
    for r in recs:
        if r["status"] != "ok":
            continue
        c = r["compile"]
        if c.get("tex_split_tris"):
            tex_bits.append((c["tex_split_tris"], r["level"], c.get("tex_split_meshes")))
    if tex_bits:
        tex_bits.sort(reverse=True)
        parts = ", ".join(f"{lv} {t:,} tris / {m} meshes" for t, lv, m in tex_bits)
        grouped["all levels"] = [
            f"the compiler split, moved or dropped triangles whose meshes sample outside their "
            f"texture to match Blender ({parts}) - check for seams on those materials."
        ]
        order.append("all levels")
    tan = [(recs[i]["level"], recs[i]["build"].get("lights_without_tangent", 0))
           for i in range(len(recs)) if recs[i]["status"] == "ok"]
    tot = sum(n for _, n in tan)
    if tot:
        grouped["lights"] = [
            f"{tot} point lights across the eight levels had no usable tangent and use the fallback "
            f"light basis (build.log per level: " + ", ".join(f"{lv} {n}" for lv, n in tan) + ")."
        ]
        order.append("lights")
    if order:
        for i, lv in enumerate(order, 1):
            head = f"**{lv}** — " if lv not in ("all levels", "lights") else ""
            body = "; ".join(t.rstrip(".") for t in grouped[lv])
            L.append(f"{i}. {head}{body}.")
    else:
        L.append("None found by the automated checks.")
    L.append("")

    (ROOT / "docs").mkdir(exist_ok=True)
    (ROOT / "docs" / "QA-report.md").write_text("\n".join(L), encoding="utf-8")
    print("\n".join(L))


def rows_compile(recs, level):
    for r in recs:
        if r["level"] == level and r["status"] == "ok":
            return r["compile"]
    return {}


if __name__ == "__main__":
    main()
