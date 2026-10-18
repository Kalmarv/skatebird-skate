import sys, collections, os, json
import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
level = sys.argv[1]
path = os.path.join(DATA, level)
env = UnityPy.load(path)
c = collections.Counter(o.type.name for o in env.objects)
print(level, "objects:", len(env.objects))
for k, v in c.most_common(40):
    print(f"  {k:28s} {v}")

read_fail = collections.Counter()
shaders = collections.Counter()
meshes = 0
verts = 0
tris = 0
renderers = collections.Counter()
colliders = collections.Counter()
lights = 0
textures = collections.Counter()
for obj in env.objects:
    t = obj.type.name
    try:
        if t == "Mesh":
            d = obj.read()
            meshes += 1
            verts += len(d.m_Vertices) // 3 if hasattr(d, "m_Vertices") else 0
            if hasattr(d, "m_IndexBuffer"):
                tris += len(d.m_IndexBuffer) // 6
        elif t == "Material":
            d = obj.read()
            s = getattr(d, "m_Shader", None)
            if s is not None:
                try:
                    shaders[s.read().m_Name] += 1
                except Exception:
                    shaders["<unreadable>"] += 1
        elif t in ("MeshRenderer", "SkinnedMeshRenderer", "SpriteRenderer", "ParticleSystemRenderer", "Terrain"):
            d = obj.read()
            renderers[t] += 1
            if t == "MeshRenderer" and not getattr(d, "m_Enabled", True):
                renderers[t + " (disabled)"] += 1
        elif t in ("MeshCollider", "BoxCollider", "SphereCollider", "CapsuleCollider", "TerrainCollider", "WheelCollider"):
            colliders[t] += 1
        elif t == "Light":
            lights += 1
        elif t in ("Texture2D", "Cubemap", "RenderTexture"):
            textures[t] += 1
    except Exception as e:
        read_fail[t] += 1

print("read failures:", dict(read_fail))
print("meshes", meshes, "verts", verts, "tris", tris)
print("renderers", dict(renderers))
print("colliders", dict(colliders), "lights", lights, "textures", dict(textures))
print("shaders (top 40):")
for k, v in shaders.most_common(40):
    print(f"  {v:5d}  {k}")
print("distinct shaders:", len(shaders))
