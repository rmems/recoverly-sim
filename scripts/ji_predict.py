#!/usr/bin/env python3
"""
ji_predict.py - placeholder / bridge for Ji Lab CNN brain strain estimation.

In MVP this prints a message and exits. Replace with:
- subprocess call to the original demo_evaluation.py + preprocessed .mat, or
- a torch reimplementation that loads the pretrained weights (after download).

See https://github.com/Jilab-biomechanics/CNN-estimation-of-brain-strain-distribution
and the companion regional one.

Usage (planned):
  python scripts/ji_predict.py --input Input.mat --output Output.mat

Then recoverly-sim can read the .mat or converted json for StrainMetrics.
"""
import sys
print("ji_predict.py (MVP stub): Ji Lab integration not wired yet.")
print("Follow the Ji Lab repos for preprocessing + prediction, then feed")
print("the resulting MPS values into recoverly-sim via JSON or future .mat loader.")
print("Required citations in any work using their models:")
print("  Wu et al. Sci Rep 2019; Ghazi et al. J Neurotrauma 2020.")
sys.exit(0)