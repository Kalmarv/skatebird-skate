# Gameplay scale investigation

Caleb wants the factor that feels natural for a human skater, with an easy program-level scale option.
The old 1:1 setting is a verification baseline, not the final gameplay decision.

Run `python -X utf8 tools/measure_features.py level2` to read named collider candidates with UnityPy 1.25.4.
`out/feature-dimensions-level2.json` records 339 active, enabled, non-trigger MeshCollider/BoxCollider entries
whose hierarchy path contains rail/ledge/ramp/quarter/bowl/stair/step/gap. No collider in this named sample has
rail/ledge/stair/step/gap in its path: the candidates are ramps and their surrounding geometry.

These are transformed source local-AABB bounds, not fitted render geometry or measured heights above ground.
Do not mistake one convex decomposition piece's 0.5 m extent for the whole ramp's rise. Combining pieces by
hierarchy text gives many 3.5 m vertical ramp spans and some 7/8.5 m spans, but repeated names combine multiple
instances. Use ancestor transform IDs before treating those groups as individual ramps.

Next measurement: select identifiable rooftop ramp instances and nearby riding surfaces, then rail/ledge and
stair examples in other levels; record feature heights relative to those surfaces and gap endpoints. Compare
to stock skate geometry or coordinator-provided benchmarks, with provenance. Propose a single factor only after
those comparisons. Scaling 3.5 m to 1.75 m at 0.5 is arithmetic, not yet a gameplay recommendation.

Keep scale unchanged for the first alpha-correction experiment so a rendering change has one intended cause.
The eventual scale pass must move collision, markers, lights and authored curves consistently with render geometry;
the target camera must be scaled for meaningful source/target comparisons.
