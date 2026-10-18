"""Decode-level oracle for SkateBIRD level2 (rooftop) with UnityPy.

Writes data/skatebird-oracle-level2.json: sampled meshes (name, vertex count, the first eight
positions in Unity's own frame), materials used by the level's MeshRenderers (name, shader,
_Color, the _MainTex/_BumpMap/_MetallicGlossMap/_OcclusionMap texture names and the matching
floats), textures those materials name (dimensions, format id, mip count), and every
MeshRenderer/MeshCollider/MeshFilter path id of level2.

worlds/unity/tests/skatebird_decode.rs parses the same objects with primase and compares.

Usage:
    python tools/unitypy_skatebird_oracle.py [DATA_DIR] [OUT_JSON]
"""

from __future__ import annotations

import json
import os
import struct
import sys
import tempfile

import UnityPy

DEFAULT_DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
DEFAULT_OUT = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "data",
    "skatebird-oracle-level2.json",
)
LEVEL = "level2"
SLOTS = ("_MainTex", "_BumpMap", "_MetallicGlossMap", "_OcclusionMap")
FLOATS = ("_Metallic", "_Glossiness", "_BumpScale", "_OcclusionStrength")
SAMPLE = 25

# Unity VertexFormat component sizes.
COMPONENT_SIZE = {
    0: 4,   # Float32
    1: 2,   # Float16
    2: 1,   # UNorm8
    3: 1,   # SNorm8
    4: 2,   # UNorm16
    5: 2,   # SNorm16
    6: 1,   # UInt8
    7: 1,   # SInt8
    8: 2,   # UInt16
    9: 2,   # SInt16
    10: 4,  # UInt32
    11: 4,  # SInt32
}


def basename(path: str) -> str:
    return os.path.basename(path).lower()


def load(data_dir: str):
    """level2 and the serialized files it names (dependency loading is explicit in UnityPy 1.25)."""
    env = UnityPy.load(os.path.join(data_dir, LEVEL))
    level = next(f for f in env.files.values() if basename(f.name) == LEVEL)
    for external in level.externals:
        path = os.path.join(data_dir, external.path.replace("/", os.sep))
        if os.path.isfile(path):
            env.load_file(path, is_dependency=True)
    return env, level


def align16(n: int) -> int:
    return (n + 15) & ~15


def first_positions(mesh, data_dir: str) -> list[list[float]]:
    """The first eight vertex positions as Unity stores them (x, y, z f32)."""
    vd = mesh.m_VertexData
    if len(mesh.m_CompressedMesh.m_Vertices.m_Data) > 0:
        raise SystemExit(f"mesh {mesh.m_Name!r}: compressed, not decoded by this oracle")
    count = vd.m_VertexCount
    channel = vd.m_Channels[0]
    if channel.format != 0 or channel.dimension != 3:
        raise SystemExit(
            f"mesh {mesh.m_Name!r}: position channel is format {channel.format}, "
            f"dimension {channel.dimension}"
        )
    def dim(c):
        # Unity packs flags above the low nibble into `dimension` on some channels
        return c.dimension & 0xF

    streams = sorted(
        {c.stream for c in vd.m_Channels if c.stream >= 0 and dim(c) > 0}
    )
    base = 0
    for stream in streams:
        if stream == channel.stream:
            break
        stride = sum(
            COMPONENT_SIZE[c.format] * dim(c)
            for c in vd.m_Channels
            if c.stream == stream and dim(c) > 0
        )
        base = align16(base + count * stride)
    stride = sum(
        COMPONENT_SIZE[c.format] * dim(c)
        for c in vd.m_Channels
        if c.stream == channel.stream and dim(c) > 0
    )
    offset = base + channel.offset
    data = vd.m_DataSize
    if not data:
        stream = getattr(mesh, "m_StreamData", None)
        if stream is None or not stream.size:
            raise SystemExit(f"mesh {mesh.m_Name!r}: no vertex data and no stream")
        with open(os.path.join(data_dir, stream.path.replace("/", os.sep)), "rb") as fh:
            fh.seek(stream.offset)
            data = fh.read(stream.size)
    return [
        list(struct.unpack_from("<fff", data, offset + i * stride))
        for i in range(min(8, count))
    ]


def shader_name(material) -> str | None:
    """The material's shader name: `m_ParsedForm.m_Name`, else the Shader object's `m_Name`."""
    if material.m_Shader is None or material.m_Shader.m_PathID == 0:
        return None
    shader = material.m_Shader.deref()
    if shader is None:
        return None
    data = shader.read()
    form = getattr(data, "m_ParsedForm", None)
    if form is not None and getattr(form, "m_Name", ""):
        return form.m_Name
    return getattr(data, "m_Name", "") or None


def texture_ref(material, name: str):
    """A slot's texture as [file, path id, name], or None (no texture set)."""
    envs = dict(material.m_SavedProperties.m_TexEnvs)
    slot = envs.get(name)
    if slot is None or slot.m_Texture is None or slot.m_Texture.m_PathID == 0:
        return None
    reader = slot.m_Texture.deref()
    if reader is None:
        return None
    texture = reader.read()
    return {
        "file": basename(reader.assets_file.name),
        "path_id": reader.path_id,
        "name": texture.m_Name,
    }


def named_property_sheet(material):
    return material.m_SavedProperties


def main() -> None:
    data_dir = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DATA
    out = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_OUT
    if not os.path.isdir(data_dir):
        raise SystemExit(f"not a directory: {data_dir}")

    env, level = load(data_dir)
    objects = list(level.objects.values())

    def pids(class_name: str) -> list[int]:
        return sorted(o.path_id for o in objects if o.type.name == class_name)

    mesh_readers = sorted(
        (o for o in objects if o.type.name == "Mesh"), key=lambda o: o.path_id
    )[:SAMPLE]
    meshes = []
    for reader in mesh_readers:
        mesh = reader.read()
        meshes.append(
            {
                "file": basename(reader.assets_file.name),
                "path_id": reader.path_id,
                "name": mesh.m_Name,
                "vertex_count": mesh.m_VertexData.m_VertexCount,
                "positions": first_positions(mesh, data_dir),
            }
        )

    # Materials the level's MeshRenderers draw, unique and in a stable order.
    references = {}
    for reader in objects:
        if reader.type.name != "MeshRenderer":
            continue
        for ptr in reader.read().m_Materials:
            if ptr.m_PathID == 0:
                continue
            target = ptr.deref()
            if target is not None:
                references[(basename(target.assets_file.name), target.path_id)] = target
    material_readers = [references[k] for k in sorted(references)[:SAMPLE]]
    materials = []
    for reader in material_readers:
        material = reader.read()
        sheet = named_property_sheet(material)
        colors = dict(sheet.m_Colors)
        floats = dict(sheet.m_Floats)
        color = colors.get("_Color")
        materials.append(
            {
                "file": basename(reader.assets_file.name),
                "path_id": reader.path_id,
                "name": material.m_Name,
                "shader": shader_name(material),
                "color": [color.r, color.g, color.b, color.a] if color else None,
                "textures": {slot: texture_ref(material, slot) for slot in SLOTS},
                "floats": {k: floats.get(k) for k in FLOATS},
            }
        )

    # Textures those materials name, unique and stable; sampled across every level material.
    texture_refs = {}
    for reader in references.values():
        material = reader.read()
        for name, slot in dict(named_property_sheet(material).m_TexEnvs).items():
            if name not in SLOTS or slot.m_Texture is None or slot.m_Texture.m_PathID == 0:
                continue
            target = slot.m_Texture.deref()
            if target is not None:
                texture_refs[(basename(target.assets_file.name), target.path_id)] = target
    texture_readers = [texture_refs[k] for k in sorted(texture_refs)[:SAMPLE]]
    textures = []
    for reader in texture_readers:
        texture = reader.read()
        textures.append(
            {
                "file": basename(reader.assets_file.name),
                "path_id": reader.path_id,
                "name": texture.m_Name,
                "width": texture.m_Width,
                "height": texture.m_Height,
                "format": texture.m_TextureFormat,
                "mips": texture.m_MipCount,
            }
        )

    document = {
        "level": LEVEL,
        "data_dir": os.path.abspath(data_dir),
        "reader": {"library": "UnityPy", "version": UnityPy.__version__},
        "counts": {
            "meshes_in_level": len([o for o in objects if o.type.name == "Mesh"]),
            "materials_used": len(references),
            "textures_used": len(texture_refs),
        },
        "meshes": meshes,
        "materials": materials,
        "textures": textures,
        "renderers": pids("MeshRenderer"),
        "colliders": pids("MeshCollider"),
        "filters": pids("MeshFilter"),
    }

    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=os.path.dirname(os.path.abspath(out)), suffix=".json")
    with os.fdopen(fd, "w", encoding="utf-8") as fh:
        json.dump(document, fh, indent=1, allow_nan=False)
        fh.write("\n")
    os.replace(tmp, out)
    print(
        f"wrote {out}: {len(meshes)} meshes, {len(materials)} materials, {len(textures)} textures, "
        f"{len(document['renderers'])} renderers, {len(document['colliders'])} colliders, "
        f"{len(document['filters'])} filters"
    )


if __name__ == "__main__":
    main()
