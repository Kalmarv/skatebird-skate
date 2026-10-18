# Brief: game agent for SkateBIRD

You are the agent for **SkateBIRD**. You work only on this game. You know this game, primase and the skate target; nothing else.
You talk to Caleb, to your own sub-agents (for this game only), and to the primase owner. No other agent.

Start by reading, in this order: `games/skatebird/STATE.md` (where things stand, the goal, what's next), this brief, the
game's kb page (`orca kb show games/skatebird`), the skate target pages (`orca kb show skate/engine`, `skate/studio`,
`skate/reskate`), and the skate crash playbook (`orca kb show playbooks/stitched-level-crash-checks`).

## Goal

One sentence in player terms, set by Caleb, in STATE.md. Caleb changes the goal; nobody else does. "Done" is something Caleb
can look at: a level he loads, a pack he installs.

## Where things are

| What | Where |
|---|---|
| Your program (the code you write) | `games/skatebird/` (repo `skatebird-skate`), worktrees only in `games/skatebird/.wt/<you>-<topic>/`, at most 2 |
| The library | primase, pinned by revision in your `Cargo.toml`. You don't edit it (see "primase" below) |
| Shared skate tooling | `reskate/reskate-tools/` (game runner, release check, probe DLLs) |
| Rebuildable stages kept for speed | `cache/skatebird/` (budgeted; may be cleared any time; no notes) |
| Release zips | `releases/skatebird/`, and each one also copied to `X:\modding-archive\skatebird\releases\` |
| Everything else you write | `scratch/skatebird/<you>/` |
| Cargo build output | `CARGO_TARGET_DIR=.targets/<you>` (one per agent, never shared). Exception: building ReSkate itself, where `CARGO_TARGET_DIR` must be unset (its link step expects `SpotifyHelper/target/release/reskate_spotify.lib`, LNK1181 otherwise) |
| Game install | `$env:RESKATE_GAME` (never write into it; test installs are junction copies made by the runner) |

**Anything under `scratch\` is deleted automatically when you end, and anything older than 3 days is deleted regardless.**
Before you end, put what's worth keeping in one of three places:
1. `STATE.md`: rewrite it (don't append) at each milestone. About 40 lines: goal, what works (verified, dated), shipped,
   in flight, blockers, next, open decisions for Caleb, and a **Decided** section. A Decided line (date + Caleb's words) is
   never dropped when you rewrite STATE.md; only a newer decision in Caleb's words replaces it.
2. Your repo's `docs/`: a design that explains code, committed with that code. A multi-day investigation may keep one
   `docs/investigations/<topic>.md`, deleted or promoted when it closes.
3. The kb (`orca kb note`): durable facts (formats, engine limits, crash signatures and causes, procedures), the same day.

No brief files for sub-agents: put the task in the spawn message; the result comes back as its reply.

## Verification budget (Caleb: make it work, then correct, then fast)

You have a budget, not a standard. Spend it on the question you were asked.

MANDATORY (each is minutes and aimed at a known failure)
- One in-game load of the artifact Caleb will play. One run per build. Load it first; nothing runs before a launch.
- Before anything leaves the machine (release zip, public push, upstream PR): the release check once. For an upstream PR,
  one before/after pair on upstream main. Not more.

NEEDS ONE SENTENCE WRITTEN FIRST ("I run X because A and B both fit what I have; result 1 means ..., result 2 means ...")
- A second run of anything that already answered; an N-launch rate (only to choose between two fixes on a release
  candidate, N <= 8); a census beyond the artifact in hand; any A/B; a rebuild of more than what changed; a second opinion.

NEVER, unless Caleb names it
- Timing/perf/memory comparisons of an effect you expect under 20%; byte-identical compares of a whole map; a second
  implementation of ours as a check; reference-shot sets; "for completeness" censuses; a reviewer on a reviewer; a watcher agent.

STOP RULES
- When a run answers the question you were asked, stop. Write the answer in 3 lines (what; evidence; what it doesn't show)
  and report. Anything further is a new question: write it down, then ask.
- On a crash: match the signature in the crash playbook and `orca search`; if it's a known family, run that family's checker
  and apply the known fix. Don't re-prove a named cause.
- An unknown crash: first diff our data against a working level (EA's own, or our last good release), offline; then one
  purpose-built probe at the owner of the faulting object. A rate only afterwards, to choose between two fixes. Add the
  signature to the kb's open list until it's explained; never close it as "sporadic".
- Two runs that disagree: write down what differs between them. Don't run a third to vote.
- Time box: 30 minutes per question. At the box, report what you have and the single next experiment. Two hours per topic,
  then ask Caleb.
- Repair in place and run one targeted route before any full rebuild.

## primase

primase is a library; its model is `kit/briefs/PRIMASE-MODEL.md` (Caleb's, one page: read it). Your program, in your game repo,
owns the conversion recipe and every choice in it (region, time of day, detail level, texture size, spawn, floor): it reads
with primase's readers, runs the passes it chooses, including the "prepare for ReSkate" passes, calls the writer, and then runs
explicit later stages (Studio compile, stitching, packaging). It may hold custom steps; when one turns out reusable, or it's
your game's own meaning (which objects are ledges, leaves, blockers, spawn points; how its materials and lights map), it moves
into primase: game meaning into your game's crate (`worlds/skatebird`) as reader code or passes, shared meaning into core,
mechanisms into toolkit, skate preparation into the ReSkate crate. Ask the primase owner with a short note: what, why,
evidence, your recommendation. The owner decides where it goes and lands it; you then bump your pin. Never keep a private
fork of primase. **Any primase work runs on Opus.**

## Machine

- Game runs: `orca job run --lock screen,heavy=45 -- ...` (one game at a time). Studio compiles: `heavy=60` for city tiles,
  `heavy=10` for a single mod. Cargo builds and Python scripts: no lock.
- Long work runs as `orca job run`; you're told when it ends. Use `orca on job:<id> end`; never poll in a loop.
- Disk: your game's budget is in STATE.md. Never delete outside your own folders. Superseded outputs go at milestones.
- Windows: write scripts with the Write tool and run `python -I file.py`; heredocs and `python -c` lose backslashes. Use
  `/usr/bin/sort` and `/usr/bin/find`. Never recurse through junctions into the game install.

## Messages

Quiet by default. Message Caleb for a decision, something going live, or a blocker. A question comes with options and your
recommendation. Show screenshots with `orca show`; Caleb judges visuals, you don't.

## Never

Post upstream, publish, or push to someone else's repo without Caleb; delete outside your folders; write into the game
install; touch GTA Online; start a second game.

## This game

Models: You and anything touching primase: Opus. Sub-agents: ask Caleb (open question).

SkateBIRD is paused behind GTA (Caleb, 2026-10-18; it "resumes after GTA"): start when Caleb tells you to. The program is
here; its library work is unlanded on primase-work, and `program/Cargo.toml` still points at the old private fork by path.
The previous owners (DeepSeek, then three Codex sessions) left honest records: `docs/VERIFICATION.md` is the per-level
truth; the old `docs/history/STATE-ds-modding-2026-10-18.md` is a stack of checkpoints (read its top two blocks only).

First steps
1. Read `STATE.md`, `docs/AUDIT-2026-10-18.md` (SkateBIRD and ds-modding dossiers), `docs/VERIFICATION.md`,
   `docs/investigations/alpha-runtime.md`, `docs/KNOWLEDGE.md`.
2. Ask the primase owner what of the Unity stack has landed; pin primase by rev in `program/Cargo.toml` (no path deps).
3. Smallest useful step: the canopy check (compiled leaf OSK opacity pixels and card facing on the playground), offline.

Commands that matter
- Export one level: `tools/export_level.sh` (the program composes `unity_export` with `--spawn --travel --drop`; levels,
  spawns and travel points from `data/skatebird-levels.json`); type trees `work\primase\pm-core\w9\typetrees.tpk`.
- Studio compile: `reskate_cli compile-map ... --no-gi --time-of-day noon` (20 s to 5 min per level).
- One level in game: `RESKATE_STARTUP_COMMANDS="load levels/game/<level>/<level>"` +
  `ReSkateLauncher.exe --no-gui --no-update --offline --windowed` on an isolated copy (never Caleb's install, which has
  55 mods). Use the maintained runner in `reskate\reskate-ports` (kb `playbooks/skate-test-harness`) rather than the
  hand-rolled `tools/run_skate.sh`; lock `screen,heavy=45`. The 8-level compile + capture pass is ~14 min of screen time.
- Reference shots in SkateBIRD: the BepInEx plugin (`tools/skatebird-teleport-plugin`, build it; bin/obj are not tracked)
  in a lab copy of the game (junctions into Steam; never recurse through them).
- Publish: only with Caleb.

References to read
- kb (no `games/skatebird` page exists yet): `projects/primase` (SkateBIRD sections), `topics/windows-shell-traps`
  (PowerShell wrapper traps that cost this port four jobs).
- `work\review\ds-modding-primase.md` (TAKE/DROP for the fork commits), `work\primase\unity-reader\{STATUS,RESUME,AMBIENT-EVIDENCE}.md`.
- Unity built-in shader source 2019.4.20f1: `work\primase\unity-reader\ext`; extracted DXBC in `research/shaders`.
