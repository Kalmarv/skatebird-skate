# skatebird-skate

SkateBIRD (Unity 2019.4.20f1, Mono, built-in pipeline; Steam app 971030): its eight skateable levels (bedroom 1, rooftop,
office, server room, bedroom 2, flashback, playground, skate heaven) as skate. (ReSkate) maps through primase. Caleb's
goal: "get all the Skate Bird maps into Reskate through primase with visual matching and polish."

- `program/`: the conversion. `src/recipe.rs` is the whole route for one level, in the shape of primase's Sekiro
  example: `SkatebirdReader`, a lowering pipeline, a preparation pipeline, `convert` into `MapSink`, the notices, then
  ReSkate Studio's `compile_map`. `src/choices.rs` holds what differs by level (room shells, hidden ceilings, the pause
  view, the fallback spawn) with the reasons; `src/levels.rs` the levels and their map names; `src/main.rs` the
  command line. primase is pinned by git rev in `program/Cargo.toml`.
- `tools/skatebird-tools/`: the tooling around it: `package` (mod folder with icon, manifest and trainer spots, zip),
  `views` (skate.'s console commands for a level's views), `skatebird-shots` (SkateBIRD's reference shots in the lab
  copy, through `tools/skatebird-teleport-plugin`), `sheets` (side-by-sides of a run against them).
- `tools/`: `ingame_levels.sh` (every level in one skate. launch through the shared runner), `skatebird_views.sh` and
  `skatebird_lab.sh` (the SkateBIRD side), the BepInEx plugin (exact camera views, fly mode on F6 / L3).
- `data/`: the surface table (`skatebird-surfaces.json`), the comparison views (`skatebird-views.json`).
- `docs/`: KNOWLEDGE (format and engine facts), the screenshot runbook, investigations and earlier reports.

Where things stand: `STATE.md`. How the agent works: `BRIEF.md`.

## Build and release

One Cargo workspace (`program/`, `tools/skatebird-tools/`), output in `target/`:

```
cargo run --release -p skatebird-export -- --typetrees <modding>/cache/primase/pm-core/w9/typetrees.tpk \
    --out <modding>/cache/skatebird/build --studio <modding>/tools/ReSkateStudio-20261012/reskate_cli.exe [level2 ...]
cargo run --release -p skatebird-tools -- package --out <modding>/cache/skatebird/build \
    --dest <modding>/releases/skatebird/<version> --version <version> --icons <folder of LEVEL.png>
```

Per level: `<out>/<level>/map/SkateBIRD_<Name>` (the map), `stage/Patch` (Studio's compile) and `build.log`; levels run
three at a time. The game is `$RESKATE_GAME`. Game runs only through `kit/run game --job` (see docs/KNOWLEDGE.md).
