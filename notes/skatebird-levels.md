# SkateBIRD level data (spawn, bounds, travel points, lighting)

Read-only inspection of `C:\Program Files (x86)\Steam\steamapps\common\SkateBIRD\SkateBIRD_Data`
(Unity 2019.4.20f1, Mono, built-in render pipeline). Scene files are the loose `levelN` serialized
files; meshes/materials live in `sharedassetsN`. Nothing in the install was modified.

## Method

- **spawn**: `Skatebirb.Gameplay.ResetPosition.ResetToStart` (called from `ControlBoard` on the first
  play frame) teleports to the active `ResetPosition` whose serialized `startingPosition` flag is true.
  That transform is the spawn. The flag is read from the raw MonoBehaviour tail (the player build has
  no script type trees). When no flag is set, `ResetToStart` does nothing and the serialized `Player Birb`
  placement is the effective spawn (both cases below).
- **bounds**: every enabled, non-trigger, active-in-hierarchy collider in the scene. `MeshCollider`
  bounds are the mesh's local AABB (`m_LocalAABB`) transformed by the collider's world matrix (8 corners).
  Any collider whose **world-space** AABB exceeds 100 m on any axis is excluded (the same filter is
  applied to box/sphere/capsule colliders because oversized scene-boundary volumes would otherwise
  dominate). Excluded colliders are listed in the JSON; the unfiltered min/max is also recorded.
- **travel points**: curated from the scene's reset anchors (the game's own teleport destinations)
  and `MissionGiver` objects, farthest-point sampled so they spread across the level. All positions are
  world-space metres in Unity's left-handed Y-up frame; yaw is `atan2(forward.x, forward.z)` in degrees.
- **lighting**: `Light` objects (`m_Type == 1` directional) and the scene `RenderSettings` object.
  Euler angles use Unity's ZXY `Quaternion.Euler` convention.

## Summary

| level | scene | spawn (x, y, z) | yaw | bounds min | bounds max | travel points |
|---|---|---|---|---|---|---|
| level1 | bedroom 1 | (45.40, 16.80, 55.77) | -54.82° | (-68.67, -4.70, -34.47) | (68.57, 45.00, 83.47) | 16 |
| level2 | rooftop | (-18.70, 1.19, 84.80) | 172.93° | (-121.70, -75.04, -12.00) | (53.39, 22.10, 167.00) | 16 |
| level3 | office | (-34.03, 11.98, -78.36) | 0.00° | (-69.70, -2.01, -84.71) | (124.30, 74.00, 84.00) | 16 |
| level4 | server room | (28.36, -1.00, -16.86) | -58.66° | (-38.00, -10.00, -38.00) | (38.00, 34.00, 38.30) | 16 |
| level5 | bedroom 2 | (-26.68, 15.20, 10.05) | -54.95° | (-84.00, -8.00, -31.40) | (69.08, 45.00, 99.00) | 16 |
| level7 | flashback | (0.00, 1.19, 0.00) | 172.93° | (-109.10, -18.08, -125.50) | (30.40, 78.89, 30.66) | 12 |
| level12 | playground | (-59.09, 1.10, 62.76) | 106.15° | (-66.80, -0.30, -39.80) | (73.00, 41.40, 90.00) | 16 |
| level13 | skate heaven | (30.70, 0.20, -23.60) | -14.09° | (-31.19, -0.88, -31.16) | (165.00, 146.66, 264.80) | 11 |

## Levels

### level1 — bedroom 1

Scene file: `level1`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Level Start` (`Reset Positions/Level Start`)
- position: `(45.400, 16.796, 55.770)` m; yaw `-54.82°`; quaternion `(-0.000000, -0.460322, -0.000000, 0.887752)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(46.000, 17.551, 54.800)`, yaw `-64.92°`
- notes: 5 active directional lights; primary is Directional Light

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-68.667, -4.700, -34.471)` m; max `(68.571, 45.000, 83.471)` m; size `(137.238, 49.700, 117.942)` m; center `(-0.048, 20.150, 24.500)` m
- colliders used: BoxCollider 127, MeshCollider 1556, SphereCollider 5
- excluded: MeshCollider_over_100m 7
- skipped: disabled 9, failed 0, inactive 130, triggers 493
- mesh-collider-only bounds: `(-68.667, -2.900, -34.471)` .. `(68.571, 45.000, 83.471)`
- unfiltered (including >100 m colliders): `(-97.292, -45.248, -81.000)` .. `(68.571, 100.460, 83.471)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Level Start | 45.400 | 16.796 | 55.770 | -54.82° |
| reset_position | Quarter pipe set | 15.580 | 16.606 | 29.330 | -24.90° |
| reset_position | Bed | -47.600 | 15.400 | 1.300 | -24.90° |
| reset_position | Desktop | -42.330 | 16.000 | 65.550 | -24.90° |
| reset_position | On the pipe box | 53.990 | 27.000 | 69.190 | -43.50° |
| reset_position | Kitchenette | 25.320 | 20.100 | -21.400 | -24.90° |
| reset_position | The Bowl | 9.470 | 28.600 | 67.730 | 0.00° |
| reset_position | At Ramp To Bed | -28.850 | 5.300 | 40.020 | 29.40° |
| mission_giver | Munch The Leftovers Birb | 49.800 | 16.500 | 63.900 | -158.95° |
| mission_giver | Beulah | -54.890 | 5.000 | 35.050 | 84.67° |
| mission_giver | Chad Shakespeare | -4.898 | 16.500 | 42.917 | -118.89° |
| mission_giver | Chaser | -27.600 | 15.000 | 0.200 | -68.81° |
| mission_giver | High Score Birbs | -51.400 | 15.390 | 69.800 | 103.72° |
| mission_giver | Reuben | 9.420 | 28.500 | 71.332 | -60.86° |
| mission_giver | Vincey | -29.503 | 5.000 | 52.541 | 143.95° |
| mission_giver | Daniyah | -9.689 | 5.000 | 10.893 | 97.94° |

**Lighting**

- sun: `Directional Light` (`Level Elements/Bounds/Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(50.00, 113.29, 0.00)°`; forward `(0.590, -0.766, -0.254)`; quaternion `(0.232353, 0.757040, -0.353014, 0.498282)`
- other active directional lights: `Boundary Fill Light` (int 0.5, euler (-1.47, 143.80, 61.74)), `Boundary Fill Light (1)` (int 0.5, euler (-1.47, 17.71, 61.74)), `Boundary Fill Light (2)` (int 0.5, euler (-54.57, 94.77, 57.92)), `Boundary Fill Light (3)` (int 0.3, euler (2.30, -95.58, -118.30)), `Directional Light` (int 1.0, euler (50.00, 113.29, 0.00))
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Newdawn1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level2 — rooftop

Scene file: `level2`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Middle Roof` (`Start Positions/Middle Roof`)
- position: `(-18.700, 1.189, 84.800)` m; yaw `172.93°`; quaternion `(-0.000000, -0.998100, -0.000000, -0.061623)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(-10.400, 1.189, 82.300)`, yaw `172.93°`

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-121.700, -75.041, -12.000)` m; max `(53.390, 22.100, 167.000)` m; size `(175.090, 97.141, 179.000)` m; center `(-34.155, -26.470, 77.500)` m
- colliders used: BoxCollider 44, MeshCollider 439, SphereCollider 5
- excluded: MeshCollider_over_100m 4
- skipped: disabled 31, failed 0, inactive 50, triggers 530
- mesh-collider-only bounds: `(-121.636, -75.041, -11.932)` .. `(32.657, 22.100, 121.100)`
- unfiltered (including >100 m colliders): `(-121.700, -886.000, -12.000)` .. `(53.390, 22.100, 167.000)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Middle Roof | -18.700 | 1.189 | 84.800 | 172.93° |
| reset_position | Middle Roof (2) | -0.050 | 1.189 | 63.720 | -87.94° |
| reset_position | Lower Roof (1) | -109.600 | -3.000 | 89.000 | -85.63° |
| reset_position | Top Roof (1) | -5.810 | 5.880 | -5.400 | 0.00° |
| reset_position | Middle Roof (1) | 22.300 | 1.189 | 102.800 | -140.62° |
| reset_position | Lower Roof | -73.600 | -3.000 | 113.700 | -85.63° |
| reset_position | Lower Roof | -73.600 | -3.100 | 69.400 | 0.00° |
| reset_position | Top Roof (3) | 25.130 | 5.880 | 20.810 | 0.00° |
| mission_giver | Jodie | -23.233 | 1.000 | 104.724 | 143.95° |
| mission_giver | Rex Heronson | -26.922 | 6.000 | -7.828 | 161.77° |
| mission_giver | Letters Collection Birb | -117.690 | -3.400 | 63.324 | 41.04° |
| mission_giver | Reuben | 26.433 | 6.000 | 38.454 | 42.41° |
| mission_giver | Matt | -76.200 | -3.400 | 117.000 | -80.32° |
| mission_giver | Jodie (1) | -25.100 | 1.000 | 54.500 | 68.29° |
| mission_giver | Chaser | -73.860 | -3.400 | 73.532 | -78.88° |
| mission_giver | Chad Shakespeare | 18.086 | 1.000 | 101.501 | 11.18° |

**Lighting**

- sun: `Directional Light` (`Directional Light`), intensity `0.5`, color `(0.9302, 0.9440, 0.9811)`, shadows `Soft`
  - Unity euler `(50.00, 150.00, 0.00)°`; forward `(0.321, -0.766, -0.557)`; quaternion `(0.109382, 0.875426, -0.408218, 0.234570)`
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Eerie Skybox`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level3 — office

Scene file: `level3`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Keyboard start` (`Start Positions/Keyboard start`)
- position: `(-34.026, 11.980, -78.359)` m; yaw `0.00°`; quaternion `(-0.000000, -0.000000, -0.000000, 1.000000)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(-29.020, 12.082, -77.500)`, yaw `172.93°`

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-69.701, -2.015, -84.710)` m; max `(124.300, 74.000, 84.000)` m; size `(194.001, 76.015, 168.710)` m; center `(27.300, 35.992, -0.355)` m
- colliders used: BoxCollider 43, CapsuleCollider 7, MeshCollider 2037, SphereCollider 5
- excluded: BoxCollider_over_100m 1, MeshCollider_over_100m 5
- skipped: disabled 586, failed 0, inactive 86, triggers 646
- mesh-collider-only bounds: `(-69.404, -0.500, -84.710)` .. `(124.300, 74.000, 84.000)`
- unfiltered (including >100 m colliders): `(-70.000, -2.015, -85.000)` .. `(124.300, 75.000, 85.000)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Keyboard start | -34.026 | 11.980 | -78.359 | 0.00° |
| reset_position | Halfpipe start | -13.460 | 11.980 | -79.680 | 0.00° |
| reset_position | Bowl start (1) | 36.720 | 49.120 | 33.280 | 127.08° |
| reset_position | weird ramp start (1) | -24.980 | 0.200 | 31.400 | 0.00° |
| reset_position | Snack bar start | 52.230 | 16.232 | -54.360 | 0.00° |
| reset_position | facing CEO office | 28.120 | 0.300 | -0.490 | -0.39° |
| reset_position | Top of cubicle start (1) | -21.030 | 22.030 | -32.390 | 0.00° |
| reset_position | CEO Desk | 48.300 | 0.380 | 29.430 | 70.97° |
| mission_giver | Combo Birb | 50.560 | 15.996 | -33.500 | 110.33° |
| mission_giver | Jodie | -57.274 | 0.000 | 48.928 | -8.94° |
| mission_giver | Rex Heronson | -43.965 | 12.000 | -63.707 | -123.33° |
| mission_giver | Chaser | 0.528 | 47.693 | 28.133 | 40.27° |
| mission_giver | Sam King | 25.763 | 0.000 | 11.657 | 166.22° |
| mission_giver | Vincey | -7.919 | 12.000 | -30.804 | 20.86° |
| mission_giver | Letters Collection Birb | -60.700 | 0.000 | 3.800 | -169.66° |
| mission_giver | High Score Birbs | -19.390 | 0.000 | 27.780 | -125.45° |

**Lighting**

- sun: `Directional Light` (`Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(50.00, 180.00, 0.00)°`; forward `(0.000, -0.766, -0.643)`; quaternion `(0.000000, 0.906308, -0.422618, 0.000000)`
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Newdawn1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level4 — server room

Scene file: `level4`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Level Start` (`Main Tower/First Level/Reset Points/Level Start`)
- position: `(28.360, -1.000, -16.860)` m; yaw `-58.66°`; quaternion `(-0.000000, -0.489852, -0.000000, 0.871806)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(2.600, -0.585, -4.500)`, yaw `172.93°`

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-38.000, -10.000, -38.000)` m; max `(38.000, 34.000, 38.300)` m; size `(76.000, 44.000, 76.300)` m; center `(-0.000, 12.000, 0.150)` m
- colliders used: BoxCollider 23, CapsuleCollider 7, MeshCollider 946, SphereCollider 5
- excluded: none
- skipped: disabled 17, failed 0, inactive 54, triggers 597
- mesh-collider-only bounds: `(-38.000, -10.000, -38.000)` .. `(38.000, 34.000, 38.000)`
- unfiltered (including >100 m colliders): `(-38.000, -10.000, -38.000)` .. `(38.000, 34.000, 38.300)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Level Start | 28.360 | -1.000 | -16.860 | -58.66° |
| reset_position | Wire Shelves | -33.406 | 24.362 | 2.749 | -90.00° |
| reset_position | Shelf Path To Third Level | 29.200 | 7.000 | 34.800 | 180.00° |
| reset_position | Tight Ramps | 18.200 | -1.000 | -4.000 | 1.38° |
| reset_position | Ramp To Third Level | -13.900 | 6.000 | -19.200 | -90.00° |
| reset_position | First Level Of Tower | -11.900 | -1.000 | 11.900 | 0.00° |
| reset_position | Third Level Of Tower Upper | -27.700 | 19.000 | 27.000 | 97.73° |
| reset_position | Third Level Of Tower | -2.300 | 16.000 | -2.400 | 0.00° |
| mission_giver | Dangling Buddy The Spider | -33.100 | -1.000 | -19.172 | 143.95° |
| mission_giver | Letters Collection Birb | 31.182 | 7.004 | 35.803 | -124.27° |
| mission_giver | Vincey | -14.498 | 19.001 | 22.436 | -155.95° |
| mission_giver | High Score Birbs | 13.913 | -1.010 | -5.203 | 87.06° |
| mission_giver | Combo Birb | 4.770 | 24.134 | -1.550 | -35.58° |
| mission_giver | Rex Heronson | -11.378 | -1.000 | -4.472 | -143.14° |
| mission_giver | Daniyah | -28.000 | 24.117 | 3.000 | 15.00° |
| mission_giver | Sharisse | 2.828 | 5.987 | 17.409 | -70.60° |

**Lighting**

- sun: `Directional Light` (`Missions - Server Room/Lights Full Power/Contents/Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(50.00, -30.00, -0.00)°`; forward `(-0.321, -0.766, 0.557)`; quaternion `(0.408218, -0.234570, 0.109382, 0.875426)`
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Doomsday1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.548`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level5 — bedroom 2

Scene file: `level5`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Bed` (`Reset Positions/Bed`)
- position: `(-26.680, 15.200, 10.050)` m; yaw `-54.95°`; quaternion `(-0.000000, 0.461339, -0.000000, -0.887224)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(0.000, 5.300, 0.000)`, yaw `172.93°`
- notes: 5 active directional lights; primary is Directional Light

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-84.000, -8.000, -31.400)` m; max `(69.078, 45.000, 99.000)` m; size `(153.078, 53.000, 130.400)` m; center `(-7.461, 18.500, 33.800)` m
- colliders used: BoxCollider 76, MeshCollider 2409, SphereCollider 8
- excluded: MeshCollider_over_100m 6
- skipped: disabled 302, failed 0, inactive 387, triggers 792
- mesh-collider-only bounds: `(-68.667, -8.000, -31.400)` .. `(65.061, 45.000, 80.000)`
- unfiltered (including >100 m colliders): `(-84.000, -8.000, -81.000)` .. `(69.078, 47.000, 99.000)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Bed | -26.680 | 15.200 | 10.050 | -54.95° |
| reset_position | Couch Bridge | -36.690 | 16.940 | 33.410 | 2.53° |
| reset_position | Bathroom Shower | 52.287 | 5.200 | -20.048 | -24.90° |
| reset_position | Top Of The World | 28.150 | 37.600 | 71.470 | -87.20° |
| reset_position | Stove | 1.470 | 18.000 | -22.590 | -24.90° |
| reset_position | By Fridge | 25.630 | 5.000 | 23.840 | -24.90° |
| reset_position | By Record Player | -9.392 | 20.998 | 71.296 | -121.43° |
| reset_position | Under Bed | -35.670 | 5.100 | -13.590 | -24.90° |
| mission_giver | Letters Collection Birb | 55.902 | 5.000 | 33.888 | -138.67° |
| mission_giver | Rex Heronson | -55.000 | 18.100 | 54.550 | 9.76° |
| mission_giver | Beulah | 14.044 | 17.700 | -23.071 | 167.29° |
| mission_giver | Daniyah | 7.500 | 30.700 | 67.900 | -15.04° |
| mission_giver | Reuben | -35.300 | 15.000 | -9.800 | -144.39° |
| mission_giver | Sam King | 12.913 | 5.000 | 15.739 | -25.35° |
| mission_giver | Craig | 43.100 | 5.000 | -6.010 | -87.66° |
| mission_giver | Combo Birb | 26.756 | 5.200 | 72.836 | 174.80° |

**Lighting**

- sun: `Directional Light` (`Bounds/Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(50.00, 113.29, 0.00)°`; forward `(0.590, -0.766, -0.254)`; quaternion `(0.232353, 0.757040, -0.353014, 0.498282)`
- other active directional lights: `Boundary Fill Light (1)` (int 0.5, euler (-1.47, 17.71, 61.74)), `Directional Light` (int 1.0, euler (50.00, 113.29, 0.00)), `Boundary Fill Light (2)` (int 0.5, euler (-54.57, 94.77, 57.92)), `Boundary Fill Light` (int 0.5, euler (-1.47, 143.80, 61.74)), `Boundary Fill Light (3)` (int 0.3, euler (2.30, -95.58, -118.30))
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Newdawn1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level7 — flashback

Scene file: `level7`.

**Spawn**

- source: `Player Birb instance (no starting ResetPosition)`
- GameObject: `Player Birb` (`Player Birb`)
- position: `(0.000, 1.189, 0.000)` m; yaw `172.93°`; quaternion `(-0.000000, -0.998100, 0.000000, -0.061623)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(0.000, 1.189, 0.000)`, yaw `172.93°`
- notes: no startingPosition ResetPosition; used Player Birb placement

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-109.096, -18.080, -125.500)` m; max `(30.402, 78.895, 30.659)` m; size `(139.498, 96.975, 156.159)` m; center `(-39.347, 30.407, -47.420)` m
- colliders used: BoxCollider 185, MeshCollider 623, SphereCollider 6
- excluded: MeshCollider_over_100m 1
- skipped: disabled 3, failed 0, inactive 35, triggers 156
- mesh-collider-only bounds: `(-109.096, -16.000, -125.500)` .. `(30.402, 78.895, 30.659)`
- unfiltered (including >100 m colliders): `(-109.096, -20.500, -130.000)` .. `(35.000, 78.895, 35.000)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | On the pipe box | 24.180 | 10.500 | 11.010 | -43.50° |
| reset_position | Around The Room | -86.800 | -0.390 | -105.000 | 94.64° |
| reset_position | Around The Room | 12.100 | -4.370 | -91.090 | 94.64° |
| reset_position | Around The Room | -70.500 | 3.210 | 17.200 | 94.64° |
| reset_position | Around The Room | -31.630 | -15.190 | -36.980 | 94.64° |
| reset_position | Around The Room | -86.360 | -0.400 | -36.240 | 94.64° |
| reset_position | Half moon grind | -24.940 | 0.106 | 10.370 | -24.90° |
| reset_position | Around The Room | -51.900 | -15.190 | -116.000 | 94.64° |
| reset_position | Around The Room | -59.670 | -2.300 | -65.120 | 94.64° |
| reset_position | Around The Room | 15.040 | -15.190 | -57.310 | 94.64° |
| reset_position | The two C's | -0.300 | 0.296 | -13.180 | 94.64° |
| reset_position | Around The Room | -9.080 | -15.190 | -88.400 | 94.64° |

**Lighting**

- sun: `Directional light` (`Directional light`), intensity `0.8`, color `(1.0000, 1.0000, 1.0000)`, shadows `Soft`
  - Unity euler `(50.00, -30.00, -0.00)°`; forward `(-0.321, -0.766, 0.557)`; quaternion `(0.408218, -0.234570, 0.109382, 0.875426)`
- ambient: `Trilight`, sky `(0.5621, 0.5986, 0.6765)`, equator `(0.7206, 0.5195, 0.4716)`, ground `(0.5441, 0.5441, 0.5441)`, intensity `1.0`
- skybox material: `Newdawn1 Sky`
- fog: `False`, color `(0.6627, 0.7176, 0.8118)`, mode `ExponentialSquared(V3)`, density `0.06`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.0`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level12 — playground

Scene file: `level12`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Start` (`Reset Positions/Start`)
- position: `(-59.090, 1.100, 62.760)` m; yaw `106.15°`; quaternion `(-0.000000, 0.799407, -0.000000, 0.600789)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(0.000, 5.053, 0.000)`, yaw `172.93°`
- notes: 2 active directional lights; primary is Directional Light

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-66.800, -0.300, -39.800)` m; max `(73.000, 41.400, 90.000)` m; size `(139.800, 41.700, 129.800)` m; center `(3.100, 20.550, 25.100)` m
- colliders used: BoxCollider 23, MeshCollider 3371, SphereCollider 5
- excluded: BoxCollider_over_100m 2, MeshCollider_over_100m 2
- skipped: disabled 31, failed 0, inactive 760, triggers 1526
- mesh-collider-only bounds: `(-66.800, -0.300, -39.800)` .. `(73.000, 41.400, 90.000)`
- unfiltered (including >100 m colliders): `(-2000.000, -1.000, -2000.000)` .. `(2000.000, 41.400, 2000.000)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Start | -59.090 | 1.100 | 62.760 | 106.15° |
| reset_position | Underpass (6) | -37.200 | 1.100 | -31.000 | 27.35° |
| reset_position | Underpass (1) | 64.300 | 1.100 | 82.200 | -93.66° |
| reset_position | Edges (1) | -57.100 | 1.100 | 83.200 | 159.49° |
| reset_position | Underpass (4) | 64.400 | 1.100 | -29.100 | -96.47° |
| reset_position | Third floor | 26.300 | 39.000 | 33.800 | -177.23° |
| reset_position | Ramp | -53.300 | 27.200 | 31.200 | -177.23° |
| reset_position | Edges | -9.600 | 1.100 | 82.400 | -160.83° |
| mission_giver | Jodie | 9.500 | 1.000 | 45.400 | -130.51° |
| mission_giver | Sharisse | -48.800 | 1.000 | -4.500 | -112.01° |
| mission_giver | Firenze | 49.057 | 12.500 | -8.629 | -164.06° |
| mission_giver | Beulah | -54.700 | 1.000 | 59.700 | -82.85° |
| mission_giver | Combo Birb | -7.500 | 38.400 | -3.500 | -77.13° |
| mission_giver | Vincey | 56.600 | 12.500 | 69.000 | 21.01° |
| mission_giver | Chaser | -52.100 | 27.000 | 30.900 | -106.84° |
| mission_giver | Reuben | -32.390 | 1.000 | 21.277 | -22.38° |

**Lighting**

- sun: `Directional Light` (`Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(50.00, 113.29, 0.00)°`; forward `(0.590, -0.766, -0.254)`; quaternion `(0.232353, 0.757040, -0.353014, 0.498282)`
- other active directional lights: `Directional Light` (int 1.0, euler (50.00, 113.29, 0.00)), `Directional Light` (int 1.0, euler (50.00, -30.00, -0.00))
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Newdawn1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

### level13 — skate heaven

Scene file: `level13`.

**Spawn**

- source: `ResetPosition(startingPosition=true)`
- GameObject: `Level Start` (`Reset Points/Level Start`)
- position: `(30.700, 0.200, -23.600)` m; yaw `-14.09°`; quaternion `(-0.000000, -0.122661, -0.000000, 0.992449)` (x, y, z, w)
- active in hierarchy: `True`
- serialized Player Birb instance: `(32.440, 1.189, -26.130)`, yaw `-20.93°`

**Bounds** (colliders, world-space, colliders > 100 m excluded)

- min `(-31.192, -0.876, -31.155)` m; max `(165.000, 146.663, 264.800)` m; size `(196.192, 147.539, 295.955)` m; center `(66.904, 72.894, 116.823)` m
- colliders used: BoxCollider 119, MeshCollider 3741, SphereCollider 5
- excluded: BoxCollider_over_100m 7, MeshCollider_over_100m 2
- skipped: disabled 1030, failed 0, inactive 510, triggers 1316
- mesh-collider-only bounds: `(-31.100, -0.876, -31.000)` .. `(165.000, 146.663, 264.800)`
- unfiltered (including >100 m colliders): `(-31.192, -360.000, -31.155)` .. `(165.100, 146.663, 11063.595)`

**Travel points**

| kind | name | x | y | z | yaw |
|---|---|---|---|---|---|
| reset_position | Level Start | 30.700 | 0.200 | -23.600 | -14.09° |
| reset_position | Third Level By Ramp | 59.000 | 20.000 | 99.000 | -14.07° |
| reset_position | Book Pipe | 20.200 | 117.100 | 246.800 | 101.21° |
| reset_position | Middle Building Mega Ramp Maze Exterior | 106.600 | 10.100 | 16.000 | -8.94° |
| reset_position | Third Level Corner | -23.500 | 20.000 | 122.700 | 78.40° |
| reset_position | Third Level Wigglies | 32.400 | 20.000 | 140.900 | -171.30° |
| reset_position | Third Level Corner | 83.300 | 20.000 | 141.200 | -136.50° |
| reset_position | Third Level By Pizza | 21.200 | 20.000 | 96.500 | -73.23° |
| mission_giver | Hawkogram | 22.150 | 113.900 | 231.780 | -142.65° |
| mission_giver | Anthony Hawk | 60.500 | 0.000 | -4.700 | 151.62° |
| mission_giver | Seagull | 24.427 | 0.000 | 5.679 | 163.10° |

**Lighting**

- sun: `Directional Light` (`Core Elements/Directional Light`), intensity `1.0`, color `(1.0000, 0.9569, 0.8392)`, shadows `Soft`
  - Unity euler `(30.00, 39.00, -0.00)°`; forward `(0.545, -0.500, 0.673)`; quaternion `(0.243974, 0.322433, -0.086396, 0.910522)`
- ambient: `Trilight`, sky `(0.5647, 0.5972, 0.6795)`, equator `(0.7835, 0.7835, 0.7835)`, ground `(0.5457, 0.5457, 0.5457)`, intensity `1.0`
- skybox material: `Nostalgia 1 Sky`
- fog: `False`, color `(0.5000, 0.5000, 0.5000)`, mode `ExponentialSquared(V3)`, density `0.01`, linear `0.0`..`300.0`
- reflections: `Skybox`, intensity `0.344`, subtractive shadow color `(0.4200, 0.4780, 0.6270)`

## Caveats

- `level7` (flashback) has **no** `startingPosition` reset flag; `ResetToStart` is a no-op there and the
  serialized `Player Birb` transform `(0, 1.189, 0)` is the effective spawn. Nearest named reset anchor:
  `The Alley` `(4.75, 0.296, 5.29)`.
- `level12` (playground) and `level5` (bedroom 2) place the `Player Birb` instance at the origin; the game
  teleports it to the flagged ResetPosition at scene start, so the flagged reset is the real spawn.
- Bounds are collider coverage, not render coverage. Some real level structure (e.g. 130 x 160 m room
  floors/walls in bedroom 1) exceeds the 100 m filter and is excluded from the bounds but still present
  in the level. The JSON keeps the unfiltered range and the list of excluded oversized colliders.
- Scene root names are stale in places (the office scene root says `Missions - Server Room`); level
  identity above comes from the scene file, not the root names.
