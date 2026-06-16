# scripts/

Helpers for Ji Lab CNN integration and training scaffolds.

## Ji integration (MVP)
- `ji_predict.py` (placeholder): will either call the original TF1/MATLAB demo or a modern torch re-export.
- Download the distrib model weights from the Ji Lab Google Drive link in their README (do not commit here).
- For now the CLI uses synthetic + documents exact usage of https://github.com/Jilab-biomechanics/CNN-estimation-of-brain-strain-distribution

## Training lightweight (PINN/GNN)
See `julia/` (or `python/`) for example that:
1. Runs the Rust engine to generate recovery traces from varied strain inputs.
2. Fits a tiny model.
3. Exports weights/inference stub that can feed back into recoverly-sim.

Run from repo root after `cargo build`.

Citations required when using Ji outputs or reimplementations.
