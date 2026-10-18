# skatebird-skate

SkateBIRD (Unity 2019.4.20f1, Mono, built-in pipeline; Steam app 971030): its eight skateable levels (bedroom 1, rooftop,
office, server room, bedroom 2, flashback, playground, skate heaven) as skate. (ReSkate) maps through primase, then one
combined map. Caleb's goal: "All the Skate Bird levels as ReSkate Map(s). Verified by matching in game screenshots in
Skate Bird to ReSkate."

- `program/`: the Rust program (`skatebird-export`) that composes primase's Unity reader, passes and the skate-scene writer
  with SkateBIRD's spawn, travel points, drops and UV/variant policy.
- `tools/`: scan, census, export, Studio compile, capture and audit scripts; the BepInEx teleport/camera plugin for the
  game side (`tools/skatebird-teleport-plugin`).
- `data/`: level spawns, bounds and travel points (`skatebird-levels.json`), census and surface tables.
- `docs/`: VERIFICATION (per-level parity), QA report, COMBINED_MAP, SCALE, KNOWLEDGE (format facts), and the
  ds-modding STATE/PLAN as history, `investigations/alpha-runtime.md` (the alpha/runtime investigation record),
  `levels.md`, `recon.md`, `screenshot-runbook.md`.

All eight levels compile and load in skate.; none matches SkateBIRD yet. Paused behind GTA. The library side is unlanded:
primase-work `archive/ds-skatebird` (the six fork commits) and the Unity gate branches.

Where things stand: `STATE.md`. How the agent works: `BRIEF.md`. The audit of 2026-10-18: `docs/AUDIT-2026-10-18.md`.
