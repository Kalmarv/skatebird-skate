import sys, collections, json, os
import UnityPy

DATA = r"C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data"
GGM = os.path.join(DATA, "globalgamemanagers")

env = UnityPy.load(GGM)
for obj in env.objects:
    if obj.type.name in ("BuildSettings", "PlayerSettings", "GraphicsSettings", "ResourceManager"):
        try:
            d = obj.read()
        except Exception as e:
            print(obj.type.name, "READ FAIL", e); continue
        if obj.type.name == "BuildSettings":
            print("BuildSettings scenes:")
            for s in getattr(d, "scenes", []):
                print("  ", s)
print("objects in ggm:", len(env.objects))
print(collections.Counter(o.type.name for o in env.objects).most_common(30))
