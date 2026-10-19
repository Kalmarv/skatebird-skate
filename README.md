# skatebird-skate

SkateBIRD (Unity 2019.4.20f1, Mono, built-in pipeline; Steam app 971030): its eight skateable levels (bedroom 1, rooftop,
office, server room, bedroom 2, flashback, playground, skate heaven) as skate. (ReSkate) maps through primase, then one
combined map. Caleb's goal: "All the Skate Bird levels as ReSkate Map(s). Verified by matching in game screenshots in
Skate Bird to ReSkate."

- `program/`: the Rust program (`skatebird-export build`) that reads each level with primase's Unity reader, runs the
  passes it chooses (`settings.rs`), writes the ReSkate map with primase's writer and compiles it with ReSkate Studio.
  primase is pinned by git rev in `program/Cargo.toml`.
- `tools/`: the level scan, the view set (`views.py`), comparison sheets, and the SkateBIRD side of the screenshots
  (BepInEx teleport/camera plugin, `tools/skatebird-teleport-plugin`).
- `data/`: level spawns, bounds and travel points (`skatebird-levels.json`), census and surface tables.
- `docs/`: VERIFICATION (per-level parity), QA report, COMBINED_MAP, SCALE, KNOWLEDGE (format facts), and the
  ds-modding STATE/PLAN as history, `investigations/alpha-runtime.md` (the alpha/runtime investigation record),
  `levels.md`, `recon.md`, `screenshot-runbook.md`.

The program runs on primase main (the Unity stack landed there); the old fork's commits are kept on primase-work
`archive/ds-skatebird`.

Where things stand: `STATE.md`. How the agent works: `BRIEF.md`. The audit of 2026-10-18: `docs/AUDIT-2026-10-18.md`.
