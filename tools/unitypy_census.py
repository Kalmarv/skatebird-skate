"""Census of SkateBIRD's SerializedFiles with UnityPy.

Writes, for every SerializedFile in the player's data folder, the container version, the
Unity version string, the object count and the full object table (path id, class id, data
offset, data size), plus totals. This is an independent-reader oracle for primase_unity's
SerializedFile: worlds/unity/tests/skatebird_census.rs parses the same files and compares.

Usage:
    python tools/unitypy_census.py [DATA_DIR] [OUT_JSON]

Defaults are the installed SkateBIRD player and data/skatebird-census.json.
"""

from __future__ import annotations

import json
import os
import re
import sys
import tempfile

import UnityPy

DEFAULT_DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
DEFAULT_OUT = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "data",
    "skatebird-census.json",
)

LEVEL = re.compile(r"^level(\d+)$")
SHARED = re.compile(r"^sharedassets(\d+)\.assets$")
NAMED = ["globalgamemanagers", "globalgamemanagers.assets", "resources.assets"]


def serialized_files(data_dir: str) -> list[str]:
    """The data folder's SerializedFiles, in a stable order: levels, sharedassets, then the named ones."""
    names = os.listdir(data_dir)
    levels = sorted(
        (n for n in names if LEVEL.match(n)), key=lambda n: int(LEVEL.match(n).group(1))
    )
    shared = sorted(
        (n for n in names if SHARED.match(n)), key=lambda n: int(SHARED.match(n).group(1))
    )
    named = [n for n in NAMED if n in names]
    missing = [n for n in NAMED if n not in names]
    if missing:
        raise SystemExit(f"missing SerializedFiles in {data_dir}: {', '.join(missing)}")
    return levels + shared + named


def census_one(data_dir: str, name: str) -> dict:
    path = os.path.join(data_dir, name)
    env = UnityPy.load(path)
    matches = [f for f in env.files.values() if os.path.basename(f.name) == name]
    if len(matches) != 1:
        raise SystemExit(
            f"{name}: UnityPy loaded {len(matches)} matching SerializedFiles "
            f"({', '.join(os.path.basename(f.name) for f in env.files.values())})"
        )
    f = matches[0]
    objects = sorted(
        ([o.path_id, o.class_id, o.byte_start, o.byte_size] for o in f.objects.values()),
        key=lambda entry: entry[0],
    )
    return {
        "size": os.path.getsize(path),
        "version": f.header.version,
        "unity_version": f.unity_version,
        "object_count": len(objects),
        "objects": objects,
    }


def main() -> None:
    data_dir = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DATA
    out = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_OUT
    if not os.path.isdir(data_dir):
        raise SystemExit(f"not a directory: {data_dir}")

    files = {}
    for name in serialized_files(data_dir):
        files[name] = census_one(data_dir, name)
        entry = files[name]
        print(
            f"{name}: {entry['size']} bytes, version {entry['version']}, "
            f"unity {entry['unity_version']}, {entry['object_count']} objects"
        )

    totals = {
        "files": len(files),
        "objects": sum(e["object_count"] for e in files.values()),
        "file_bytes": sum(e["size"] for e in files.values()),
        "object_bytes": sum(o[3] for e in files.values() for o in e["objects"]),
    }
    document = {
        "data_dir": os.path.abspath(data_dir),
        "reader": {"library": "UnityPy", "version": UnityPy.__version__},
        "files": files,
        "totals": totals,
    }

    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=os.path.dirname(os.path.abspath(out)), suffix=".json")
    with os.fdopen(fd, "w", encoding="utf-8") as fh:
        json.dump(document, fh, indent=1)
        fh.write("\n")
    os.replace(tmp, out)
    print(
        f"wrote {out}: {totals['files']} files, {totals['objects']} objects, "
        f"{totals['file_bytes']} file bytes, {totals['object_bytes']} object bytes"
    )


if __name__ == "__main__":
    main()
