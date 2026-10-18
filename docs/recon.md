# SkateBIRD recon (map-port feasibility)

Recon date: 2026-10-18. Machine: DESKTOP-B92N5L6. Read-only inspection of the Steam install; nothing in the game
directory was modified.

Install: `C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD`
Steam appid 971030, state from `steamapps\appmanifest_971030.acf`: `StateFlags 4` (fully installed),
`buildid 11275982`, single depot `971031` manifest `6446988616964103778`, `SizeOnDisk 3396104486`,
`LastUpdated 1792303496` = 2026-10-18 06:04:56 UTC. Download completed (BytesDownloaded == BytesToDownload).

## VERIFIED FROM FILES

### Engine / runtime

| Fact | Evidence |
|---|---|
| Unity **2019.4.20f1** | ASCII `2019.4.20f1` at offset 0x19 of `SkateBIRD_Data/globalgamemanagers` |
| Same build in player binaries | `SkateBIRD.exe` FileVersion `2019.4.20.7197120`; `UnityPlayer.dll` FileVersion `2019.4.20.7197120` |
| Scripting backend **Mono** (64-bit) | `MonoBleedingEdge/EmbedRuntime/mono-2.0-bdwgc.dll`, `MonoPosixHelper.dll`, `etc/mono/{2.0,4.0,4.5}`; no `GameAssembly.dll` (no IL2CPP) |
| x86_64 Windows player | `SkateBIRD.exe` PE machine 0x8664, subsystem 2 (GUI); plugins under `SkateBIRD_Data/Plugins/x86_64` |
| Managed assemblies present | `SkateBIRD_Data/Managed/` = 104 DLLs, 26 MB, incl. `Assembly-CSharp.dll` 1,245,696 B and `Assembly-CSharp-firstpass.dll` 2,313,728 B |
| Managed profile | `mscorlib.dll` + `netstandard.dll` shipped; Unity 2019.4 Mono, .NET 4.x-era profile |
| Render pipeline | Built-in (no URP/HDRP). Shaders in `sharedassets0.assets`: `Standard` variants (`Autodesk Interactive`, `Alpha 2-Sided`, custom `Bumped Specular Voronoi`, `Dirty Glass`, `ASESampleShaders/*`) + `Hidden/PostProcessing/*` (Post Processing Stack v1, `Unity.Postprocessing.Runtime.dll`) |
| Audio middleware | Unity `AudioSource`/`AudioClip` only (813 clips in `sharedassets9`); no FMOD/Wwise DLLs |
| Steam | `Facepunch.Steamworks.Win64.dll`, `Plugins/x86_64/steam_api64.dll` |
| Input | `Rewired_Core.dll`, `Rewired_Windows.dll`, `Rewired_DirectInput.dll`; `Rewired Initializer` root object in every playable scene (0–5, 7, 12, 13); absent in credits/loading/platform scenes |
| Dev console framework present | `ReflexCLI` types in `Assembly-CSharp.dll` (13 types + `ReflexCLI.UI`) |

Artifact hashes (sha256, first 32 hex):
`globalgamemanagers` 198,156 B → `06fd2b2c53efecd87f5d770dcddd9c88`
`Managed/Assembly-CSharp.dll` 1,245,696 B → `34c5063b927b2813211d5dd2a25b41d7`
`level13` 14,243,204 B → `da784af2defc241dd2f324f2eb987f6b`

### Top-level layout

```
SkateBIRD/
  SkateBIRD.exe                    650,752 B
  UnityPlayer.dll               25,960,824 B
  UnityCrashHandler64.exe        1,095,032 B
  MonoBleedingEdge/                6.2 MB
  SkateBIRD_Data/                  3,361,965,998 B (3.13 GiB)
```

Whole install: 191 files, 3,396,104,486 B (3.16 GiB) — byte-identical to the appmanifest `SizeOnDisk`.

`SkateBIRD_Data/`:

```
app.info                 28 B   ("Glass Bottom Games\nSkateBIRD")
boot.config              79 B   (wait-for-native-debugger=0, vr-enabled=0, hdr-display-enabled=0)
globalgamemanagers      198,156 B
globalgamemanagers.assets 369,972 B
level0 … level13 (+ .resS where present)
resources.assets        703,184 B
resources.assets.resS 1,486,436 B
resources.resource        4,064 B
sharedassets0 … sharedassets13 (+ .resS / .resource)
Managed/                 104 DLLs
Plugins/x86_64/          MidiJackPlugin.dll, Rewired_DirectInput.dll, steam_api.dll, steam_api64.dll
Resources/               unity default resources (3,837,020 B), unity_builtin_extra (407,480 B)
StreamingAssets/         "put your ogg or wav music for SkateBIRD here.txt" (165 B) — no payload
```

### Content packaging: loose serialized Unity files, no bundles

- **No AssetBundles and no Addressables.** Full-tree search for `*.bundle`, `*.unity3d`, `*.ab`, `*catalog*`
  returns nothing. `StreamingAssets/` holds only a 165-byte readme text file. `Managed/` contains no
  `Unity.Addressables.dll` / `Unity.ResourceManager.dll`; `Assembly-CSharp.dll` contains 0 occurrences of the
  UTF-8 string `AssetBundle`, 0 of `StreamingAssets`, 0 of `Mods`/`mods`.
- All content is standard player serialized files: `levelN` + paired `sharedassetsN.assets[.resS|.resource]`,
  `resources.assets`, `globalgamemanagers.assets`.
- `globalgamemanagers` contains 2 UTF-8 occurrences of `AssetBundle` (metadata/class-name text only, no bundle files).

### Size distribution

| Group | Size |
|---|---|
| All `levelN` + `levelN.resS` | 0.26 GB |
| All `sharedassetsN.assets` + `.resS` + `sharedassets9.resource` | 2.84 GB |
| `resources.assets` + `.resS` | 2.1 MB |
| `globalgamemanagers(.assets)` | 555 KB |

Largest single files: `sharedassets9.resource` 857,493,792 B (audio), `sharedassets0.assets.resS` 744,555,316 B,
`sharedassets1.assets.resS` 364,676,240 B, `sharedassets3.assets.resS` 225,691,840 B.

Per-file listing (bytes):

```
level0   2,394,356   resS  3,005,236
level1   9,758,532   resS 27,954,496
level2  11,714,148   resS 11,834,616
level3   8,668,708   resS 30,305,428
level4   6,577,020   resS 13,751,380
level5  12,010,348   resS 20,654,836
level6     935,996
level7   5,293,372   resS 21,271,732
level8      13,644   resS    131,232
level9     703,184   resS    672,456
level10      4,716
level11      4,680
level12 12,963,748   resS 29,175,800
level13 14,243,204   resS 30,982,536
sharedassets0.assets    43,165,508   resS 744,555,316
sharedassets1.assets     4,360,888   resS 364,676,240
sharedassets2.assets     5,313,788   resS 139,896,744
sharedassets3.assets    10,437,604   resS 225,691,840
sharedassets4.assets     2,918,448   resS  54,380,376
sharedassets5.assets     5,547,312   resS 180,438,496
sharedassets6.assets        90,620   resS 145,417,248
sharedassets7.assets     3,602,096   resS 213,461,112
sharedassets8.assets        20,656   resS     131,072
sharedassets9.assets       147,416   .resource 857,493,792
sharedassets10.assets        4,157
sharedassets11.assets        4,157
sharedassets12.assets      850,528   resS  21,236,780
sharedassets13.assets      637,184   resS  28,362,808
```

### Levels / scenes

Build-settings scene list extracted from the `globalgamemanagers` scene table (order == level index):

| index | scene path | scene |
|---|---|---|
| 0 | `Assets/Skatebirb/Scenes/Game Levels/title screen.unity` | title screen |
| 1 | `.../Game Levels/bedroom 1.unity` | bedroom 1 |
| 2 | `.../Game Levels/rooftop.unity` | rooftop |
| 3 | `.../Game Levels/office.unity` | office |
| 4 | `.../Game Levels/server room.unity` | server room |
| 5 | `.../Game Levels/bedroom 2.unity` | bedroom 2 |
| 6 | `.../Game Levels/credits.unity` | credits |
| 7 | `.../Game Levels/flashback.unity` | flashback |
| 8 | `.../Background Loaded/loading.unity` | loading |
| 9 | `.../Background Loaded/MasterAudio.unity` | MasterAudio |
| 10 | `.../Platform Airgaps/Win10 Platform.unity` | Win10 Manager |
| 11 | `.../Platform Airgaps/Steam Store Platform.unity` | Steam Store Manager |
| 12 | `.../Game Levels/playground.unity` | playground |
| 13 | `.../Game Levels/skate heaven.unity` | skate heaven |

`Assembly-CSharp.dll` enum `GameLevels` confirms: `title, bedroom_1, rooftop, office, server_room, bedroom_2,
credits, playground, skate_heaven, NumLevels, Invalid`. `flashback`, `loading`, `MasterAudio`, platform scenes
are not in the enum.

Index order is corroborated by per-scene root labels: level1 `Missions - Bedroom 1`, level2 `Missions - Rooftop`,
level5 `Missions - Bedroom 2`, level12 `Missions - Playground`, level13 `Missions - Skate Heaven` (the level3
label is a stale copy — see caveat below).

Serialized externals prove each level links the shared library:
`level12` → `globalgamemanagers.assets, sharedassets0/1/2/3/4/5/7/12, resources.assets, Library/unity default resources`;
`level13` additionally links `sharedassets13` (and 12); `level1` links only `sharedassets0/1` + resources.

### Scene structure (map-port template)

Top-level `Transform` roots:

- playground (level12, 18 roots): `Rewired Initializer, Reflection Probes, Grass plane, REMEMBER TO RE-ENABLE CAMERA
  OVERLAY, Level Elements, Missions - Playground, Non-Static Level Elements, Skatebirb Dialogue Manager,
  Directional Light, Reset Positions, Player Birb, Cinemachine SkateCam 2.0, Skateboard For Skating, Main Camera,
  Skatebirb Default Postprocessing, Birb Factory, Important Stuff`
- skate heaven (level13, 15 roots): `Rewired Initializer, Reset Limiters, Mission - Skate Heaven, Skatebirb Dialogue
  Manager, Level Contents, Core Elements, Reset Points, Player Birb, ...`
- rooftop (level2): adds `Start Positions`, `Partially Static Level Elements`, `Unlocks`
- bedroom 1 (level1): adds `Sticky Notes`, `Partially Dynamic Level Elements`

Every playable scene is self-contained (player bird, skateboard, cameras, post-processing, dialogue manager,
mission set) — a new map is a full scene, not just geometry.

### Asset census

`sharedassetsN`: `Mesh, Texture2D, Material, AnimationClip, AudioClip, Shader` counts (UnityPy):

```
sa0   objs 5495  mesh 335  tex 447  mat 380  anim 49  shader 58
sa1   objs 1320  mesh 653  tex 163  mat 123
sa2   objs 1659  mesh 1305 tex  60  mat  54
sa3   objs 2472  mesh 2028 tex 108  mat 102
sa4   objs 1555  mesh 1166 tex  21  mat 201
sa5   objs 1720  mesh 1198 tex  85  mat  77  anim 6
sa6   objs  121  mesh   1  tex  52  mat  53
sa7   objs  361  mesh  94  tex 116  mat 114  anim 18
sa8   objs   16
sa9   objs 1118  audio 813
sa12  objs  331  mesh  19  tex  10  mat  11
sa13  objs  257  mesh  22  tex  22  mat  32
```

Level files (UnityPy object counts, types 27–30 each):

```
level12 playground   86,103 objects: GO 32,277, Transform 31,256, MonoBehaviour 8,186, MeshCollider 4,133,
                                 MeshRenderer/MeshFilter 2,655, BoxCollider 1,338, Mesh 95, ReflectionProbe 9
level13 skate heaven 80,168 objects: GO 27,354, Transform 26,333, MonoBehaviour 7,778, MeshCollider 5,206,
                                 MeshRenderer/MeshFilter 4,248, BoxCollider 1,388, Mesh 133, ReflectionProbe 7
level1  bedroom 1    44,305 objects, Mesh 59, ReflectionProbe 6
level2  rooftop      69,136 objects, Mesh 61, ReflectionProbe 5
```

Geometry characteristics:
- Playable-level meshes are ProBuilder output: names `pb_Mesh294342`, `pb_Mesh295110`, … plus
  `Combined Mesh (root: scene) 10` — static geometry already combined into large meshes.
- `level12`: 95 meshes, 85 `m_IsReadable=true`, 10 false (vertex data still in `.resS`, extractable).
- Collision is real geometry via `MeshCollider` (4,133 in playground) plus `BoxCollider`; no terrain
  (`Terrain`/`TerrainData` = 0), no `LODGroup` anywhere, no `NavMeshData`.
- Lighting: `ReflectionProbe`s per level (0/7 = 1, 2/5 = 5, 1/4 = 6, 13 = 7, 12 = 9, **office = 23**),
  `LightmapSettings` present but `m_LightingDataAsset = None` (no baked lightmaps shipped, `LightmapsMode 1`);
  no lightmap textures found in `sharedassets0/12/13`.
- Texture formats (sharedassets0): `DXT1` 207, `DXT5` 146, `Alpha8` 82, `RGBA32` 10, `RGB24` 2 — no Crunch.
- Level files contain no `Texture2D`/`Material`/`Shader`/`AudioClip` in any of the 14 scenes (verified per file);
  renderers reference the shared assets files.
- Editor label caveat: root names are stale copies in places — the `office` scene (index 3) has a root literally
  called `Missions - Server Room`, and `flashback` has `Missions - TEST`. Object contents confirm index 3 is the
  office (`Cubical side wall`, `Snack Bar`, `wine_bottle_a_01`). Do not derive level identity from root names.
- `title screen` (level0) is itself a playable diorama scene: roots include `Player Birb`, `NPC Birds`,
  `Player Prison`, `Cinemachine SkateCam 2.0`.

### Assembly-CSharp.dll

- 947 type definitions, not obfuscated.
- Namespaces: `<global>` 386, `I2.Loc` 79, `Skatebirb.Menus` 74, `GBG_Utils` 50, `Skatebirb.Util` 44,
  `Rewired.UI.ControlMapper` 28, `TMPro.Examples` 27, `Skatebirb.Gameplay` 26, `Skatebirb` 25,
  `Skatebirb.Managers` 16, `ReflexCLI` 13, `Technie.PhysicsCreator*` 18, `Skatebirb.FX` 11,
  `Skatebirb.Camera` 9, `Skatebirb.Platforms` 5 (+`.Win10` 3), `Skatebirb.Birbs` 4, `Skatebirb.SkateControl` 3,
  `Skatebirb.Timeline` 4, `Skatebirb.Animation` 3, `Skatebirb.VideoTapeScenes`.
- Gameplay primitives: `Skatebirb.GrindableRail`, `GrindableElementProgression`, `RampZone`, `VertZone`, `FanZone`,
  `AreaDenialZone`, `BasicGapGate`, `GapGateZone`, `TrickGapGate`, `LippableNubbin`, `SkateboardGateway`,
  `TrickableElement`, generated scene markers `Generated_Rail_Node*`, `Generated_Ramp_Zone*`, `Generated_Vert_Zone*`
  appear as GameObjects across the gameplay levels (track authoring is procedural markers in the scene).
- Progression: `Skatebirb.Menus.MissionDef`, `MissionGoalDef`, `MissionGoalTypes`, `Skatebirb.Managers.MissionManager`,
  `SaveToDiskManager`, `SpawnPools`, `Skatebirb.Gameplay.SkateCollectible`, `PermanentUnlockingCollectible`.
- Scene flow helpers: `Skatebirb.Util.LoadSceneImmediately`, `LoadSceneOnTriggerEnter`, `SpawnLocationMarker`;
  `Skatebirb.Platforms.GameState`.
- Third-party: Rewired, I2 Localization, DOTween, Cinemachine, TextMeshPro, ProBuilder/ProGrids, Odin Inspector
  attributes, Facepunch.Steamworks, SimpleJSON, DynamicBone, MidiJack (plugin DLL).

### Anti-cheat / DRM

- No EasyAntiCheat, BattlEye, or `*.sys` files anywhere in the install.
- `SkateBIRD.exe` import table is `UnityPlayer.dll` + `KERNEL32.dll` only — no Steam DRM wrapper stub;
  Steam integration is dynamic via `steam_api64.dll`.
- No integrity check assemblies present.

## WEB ONLY (not verified against the install)

| Claim | Source | Date |
|---|---|---|
| Developer/publisher Glass Bottom Games; released Sep 16, 2021; Windows/macOS/Linux | Steam store API `https://store.steampowered.com/api/appdetails?appids=971030` | accessed 2026-10-18 |
| Engine: Unity; "Technologies: Unity Engine" tag | SteamDB app page `https://steamdb.info/app/971030/info/` (search-index snippet; direct fetch returns HTTP 403 to this agent) | accessed 2026-10-18 |
| Engine: Unity, build **2018.4.12.56772**, reference user Suicide_machine dated 2019-12-12; page last modified 2025-10-09 | PCGamingWiki `https://www.pcgamingwiki.com/wiki/SkateBIRD` (wikitext) | page 2025-10-09 |
| Config at `HKCU\Software\Glass Bottom Games\SkateBIRD\`; saves at `%USERPROFILE%\AppData\LocalLow\Glass Bottom Games\SkateBIRD\saved_*.sav`; no Steam Cloud, Xbox cloud true | PCGamingWiki (same page) | 2025-10-09 |
| DRM-free when launched directly (verified on Linux by a wiki contributor, 2025-10-09) | PCGamingWiki (same page) | 2025-10-09 |
| AssetStudio supports Unity 3.4–2022.1 | `https://github.com/Perfare/AssetStudio` README | accessed 2026-10-18 |
| AssetRipper supports Unity 3.5.0–6000.4.X | `https://github.com/AssetRipper/AssetRipper` README / repo description | accessed 2026-10-18 |
| BepInEx 5 is the Mono-oriented Unity loader (Unity 3.x–2020.x per Nexus listing) | `https://github.com/BepInExTeam/BepInEx`, `https://www.nexusmods.com/site/mods/115` | accessed 2026-10-18 |

**Discrepancy:** PCGamingWiki's Unity build (2018.4.12.56772) is stale — it dates from the 2019 pre-release demo.
The shipped build is 2019.4.20f1. Treat the wiki engine entry as correct only at the "Unity" level.

### Existing modding scene (web)

- **No dedicated mod platform.** `https://thunderstore.io/c/skatebird/` → HTTP 404. Nexus Mods game page
  (`https://www.nexusmods.com/games/skatebird`) → HTTP 403 to this agent and no indexed SkateBIRD mods found.
- **Steam Community**: 7 guides total, categories Secrets/Achievements/Walkthroughs — none about modding.
  `https://steamcommunity.com/app/971030/guides/`
- **GitHub search** (`api.github.com/search/repositories?q=SkateBIRD`, 2026-10-18): 11 repos.
  - `crypto33x/skatebird` (2021-06-24) — no description.
  - 10 × `SIGFAI/skatebird-*` repos (updated 2026-10-05 … 2026-10-09): AI-generated "takeover" mods built on
    **BepInEx 5**; inspected `SIGFAI/skatebird-portal-2-takeover` — its source imports only `Sigf.Kit`, `UnityEngine`,
    no game-specific `Skatebirb` types, so it is weak evidence of a working game-API mod, but it does assert
    BepInEx 5 installability.
- No public extraction notes, no level editor, no custom-map reports found.

## Map-port implications

1. **Loading custom content is possible without touching game files.** Unity 2019.4 + Mono means BepInEx 5
   (x64, `BepInEx_win_x64_5.4.x`) can host a plugin; the plugin can `AssetBundle.LoadFromFile` and/or
   `SceneManager.LoadScene(..., LoadSceneMode.Additive)`. The shipping build has no bundle-loading code paths
   of its own (`AssetBundle` never referenced in `Assembly-CSharp.dll`), so the loader must be injected.
2. **Authoring version must match 2019.4.20f1** for serialized asset compatibility (bundles built with the same
   2019.4.x line). Shaders/materials can be reused from `sharedassets0.assets`.
3. **A ported map is a full gameplay scene**, not just geometry: clone the root scaffold of `level12`/`level13`
   (`Player Birb`, `Skateboard For Skating`, `Main Camera`, `Cinemachine SkateCam 2.0`, `Rewired Initializer`,
   `Skatebirb Dialogue Manager`, `Birb Factory`, `Important Stuff`, `Reset Positions`, post-processing) and add
   mission definitions + collectibles if progression is wanted.
4. **Geometry pipeline is straightforward**: ProBuilder/combined meshes + `MeshCollider`, no LODs, no baked
   lightmaps, no terrain, no navmesh. Collision can be rebuilt from render meshes; lighting is realtime +
   reflection probes.
5. **Track markers matter**: `Generated_Rail_Node*`, `Generated_Ramp_Zone*`, `Generated_Vert_Zone*` GameObjects carry
   grind/ramp/vert volumes via `GrindableRail`/`RampZone`/`VertZone` MonoBehaviours — a ported map needs these
   authored or the level will render but not be skatable.
6. **No anti-cheat, no DRM stub** — runtime injection and file swaps (in a copy of the install) are unimpeded.
7. `ReflexCLI` is compiled into the game; a dev console may be reachable at runtime (not verified — no launch was
   performed in this task).

## Method / reproducibility

- Files inspected read-only with `find`/`stat`, Python 3.13 + `UnityPy 1.25.4`, `dnfile`, `pefile`
  (installed under `C:/Users/kalmarv/ds-modding/tools/pylib`).
- Unity version: raw bytes at 0x19 of `globalgamemanagers`; PE `VersionInfo` for both binaries.
- Scene table: regex over `globalgamemanagers` for `Assets/...*.unity`; order == build index.
- Package check: filesystem search for bundle/catalog extensions; metadata string scan of `Assembly-CSharp.dll`.
- No game launch; no writes to the game install or to any other project. Analysis packages were pip-installed into
  `C:/Users/kalmarv/ds-modding/tools/pylib` (pip may also have populated its default user cache).
