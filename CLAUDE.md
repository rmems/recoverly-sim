# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:7510c1e2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->


## Build & Test

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --locked
cargo run -- simulate --peak-rot 23 --mc-runs 120 --seed 7 --export /tmp/outs/
cargo run --example basic
# (after Ji setup) cargo run -- simulate --from-ji ...
```

### GitLab CI testing (on feature branches / current branch)
```bash
# After pushing the branch to gitlab remote
glab pipeline list --branch feat/add-gitlab-ci   # or current branch name
glab pipeline status   # latest on current branch
# Visit the pipeline in browser: https://gitlab.com/rmems/recoverly-sim/-/pipelines
# To follow logs: glab ci trace <job-id>  (or use web)
```
Note: .gitlab-ci.yml runs fmt/clippy/test on MRs, main, and feat/* branches for easy testing of current work.

## Architecture Overview

**One line:** Rust lib+CLI for kinematics → (Ji Lab CNN bridge or synthetic/PINN/GNN) strain metrics → stochastic MC recovery trajectories (modifiers, setbacks, milestones) + exports; thin scripts + Julia/Python scaffolds for custom lightweight model training + viz. GPL-3, dual GH+GL remotes, beads-tracked.

See README for Ji citations, owns/does-not-owns, and how custom models plug in.

## Conventions & Patterns

- Use `bd` for **all** task tracking (create before code, claim, close in batches).
- Follow sibling patterns (xai-dissect, gaming-telemetry, Surrogate_Viz.jl): focused ownership, determinism + seeded tests, structured exports, detailed README with citations.
- Non-interactive shell ops.
- Session close requires quality gates + `git pull --rebase && git push` + clean `git status` "up to date".
- Never commit large weights or real patient data.
