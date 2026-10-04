# Reach, Not Selection: Measuring What Is Missing on ARC-AGI-2

Code, data, preregistrations and the decision log behind the ARC Prize
2026 paper-track writeup of the same name
([papers/reach-not-selection-writeup.md](papers/reach-not-selection-writeup.md))
and its linked Kaggle entry ([kaggle/tpd-arc2](kaggle/tpd-arc2)).
License: MIT-0.

## What is measured

A program-search solver can fail a task in two ways. It can fail to
find any program that fits the training pairs and also gets the test
right (a reach failure), or it can find one and choose a different fit
(a selection failure). The paper measures both on the ARC-AGI-2 public
training and evaluation sets, with preregistered runs whose result
files are written once and content-hashed. In short: selection rarely
fails, newly reachable tasks mostly convert, and the evaluation set is
unreachable even by exhaustive search up to four operations.

## Layout

- `src/`        the Rust solver (`lib.rs`) and every measurement binary (`bin/`)
- `data/`       result files of every registered run, by run ID
- `manifest/`   run registrations (parameters fixed before each run)
- `notes/`      the vocabulary definition, the preregistrations and the
                unreached-task categorization behind v3
- `scripts/`    the categorization helper
- `neural/`     training script for the neural arm that was excluded
                after missing its preregistered threshold
- `kaggle/`     the linked Kaggle notebook and how to build its binary
- `papers/`     the Kaggle writeup, the full paper (`reach-not-selection-full.pdf`),
                the figures and the scripts that draw them and check every number
- `DECISIONS.md` dated record of every run, correction and voided bar
- `PROVENANCE.md` original commit times that back the "frozen before" claims,
                and the list of files edited for publication

## Reproduce

Requires Rust (stable) and the public ARC-AGI-2 data
(github.com/arcprize/ARC-AGI-2; the runs used commit `f3283f72`).

    cargo build --release
    target/release/arc2_solve_t data/S2-b5-002/floor_T.json:T <challenges.json> <out.json>

Each measurement binary (`arc2_census`, `arc2_fpcurve`, `arc2_fpcurve2`,
`arc2_tier3`, `arc2_tier4`, `arc2_depth4`, `b5_*`) prints the sha256 of
its output; compare with `DECISIONS.md`. Enumeration is deterministic,
so identical inputs give identical bytes.

Check every number in the papers against the result files:

    python3 papers/check_numbers.py papers/reach-not-selection-full.md papers/kaggle-writeup-paste.md

Figures:

    ARC_AGI_2_DIR=<path to ARC-AGI-2/data> python papers/media/make_figures.py

## About this repository

The notes and decision log were lightly edited for publication. Every
edited file is listed in `PROVENANCE.md`. The 80 neural-arm checkpoints
are omitted and can be regenerated; see `data/S2-b5-002/m2/WEIGHTS.md`.
