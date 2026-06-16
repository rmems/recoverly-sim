# recoverly-sim

**From head impact kinematics (via Ji Lab CNN or your own lightweight PINN/GNN) to brain strain metrics and simulated post-injury recovery trajectories.**

This is a research/education-oriented computational simulation platform. It is **not medical advice** and must not be used for clinical diagnosis or individual treatment decisions.

## What this crate owns
- Rust core: data model (Kinematics, StrainMetrics, InjuryDamage, RecoveryModifiers, traces)
- Synthetic kinematics + toy strain estimator (stand-in)
- Stochastic day-by-day MC recovery engine (seeded runs, modifiers for age/adherence/etc., milestones, aggregates)
- CLI (`recoverly-sim`) for estimate-strain / simulate / list-generators with pretty tables + JSON/CSV export to `outputs/`
- Basic example + tests (determinism via summary tolerance, modifier effects)
- CI, dual-remote (origin + gitlab) setup
- Scaffolding + docs for **Ji Lab CNN integration** (https://github.com/Jilab-biomechanics/CNN-estimation-of-brain-strain-distribution and CNN-brain-strains) and for training your own lightweight models (PINN / GNN)

## What this does NOT own
- The original Ji Lab pretrained weights / models (user must download per their repo README + Google Drive link; we only provide bridge stub + citation)
- The full Worcester Head Injury Model (WHIM) finite-element simulations
- Any real patient impact data or clinical outcome labels
- Production-grade training pipelines or large model artifacts (weights never committed)
- A complete connectome GNN or full voxel 3D viz (scaffolded only; see future bd issues)

## Quick start
```bash
cargo build --release
cargo run -- simulate --peak-rot 25 --mc-runs 300 --seed 42 --export outputs/
cargo run -- list-generators
cargo run --example basic
```

See `cargo run -- --help` and the simulate subcommand for age, adherence, protocol, etc.

## Ji Lab CNN Integration (GPL-3)
We aim to make it easy to feed real or lab impact kinematics into the excellent public Ji Lab surrogates for instantaneous whole-brain or regional MPS.

See their repos:
- https://github.com/Jilab-biomechanics/CNN-brain-strains (regional 95% MPS)
- https://github.com/Jilab-biomechanics/CNN-estimation-of-brain-strain-distribution (voxel-wise distrib)

Citations (include in any derived work):
- Wu et al. (2019) "Convolutional neural network for efficient estimation of regional brain strains" Scientific Reports.
- Ghazi et al. (2020) "Instantaneous Whole-brain Strain Estimation in Dynamic Head Impact" J Neurotrauma.

In this repo: `scripts/` will contain helpers (download, modern torch port stub, or subprocess bridge). For MVP the CLI falls back to synthetic but documents the exact Ji usage.

## Training your own lightweight model (PINN / GNN)
The recovery engine is deliberately decoupled: any source that can produce a `StrainMetrics` (or directly an `InjuryDamage`) can drive `run_mc_recovery`.

Scaffolds (in `julia/` or `python/`) show:
- Generate synthetic recovery traces from the engine
- Fit a tiny Neural ODE / small GNN (using SciML or torch)
- Save lightweight artifact + inference path back into the sim

See future bd issues for full end-to-end differentiable versions.

## Architecture (one line)
Rust engine + CLI (deterministic MC recovery on top of strain features) + interchange formats + thin integration layer for Ji Lab (or your PINN/GNN) + Julia/Python training viz scaffolds.

## Build & test (see CLAUDE.md for full)
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --locked
cargo run -- simulate --help
```

## License
GPL-3.0-only (chosen to be compatible with the Ji Lab tools we integrate/cite).

## Status
Early MVP scaffold. All work tracked in beads (run `bd ready`). See the plan and issues for roadmap (more profiles/metrics, real Ji bridge, proper GNN/PINN examples, better viz, sensor ingest, etc.).

Contributions that respect the narrow ownership and citation rules are welcome via the usual process.
