# Provenance

Several notes say "the git history of this file is the proof" that a
rule was frozen before the run it governs. This repository has a fresh
history, so that proof lives in the original development repository.
This file records the original commit times so the
ordering claims can still be checked against a disclosed record.

Paths: the original repository kept the solver at `crates/tpd-arc/`; here
it is the repository root (`src/`). Manifests keep their original paths
because their hashes refer to them.

## Commit times in the original repository (UTC+01:00)

Each row lists every commit that touched the file, oldest first.

| File | Commits |
|---|---|
| `notes/arc2-dsl-v1.md` | 0831200 2026-07-24 11:24; 37933ed 2026-07-24 13:53 |
| `notes/b4-tier-protocol-2026-08-21.md` | 2ab0083 2026-08-21 12:13; ef45924 2026-08-21 12:59; 8a75a02 2026-08-21 16:36 |
| `notes/b4-taxonomy-2026-08-21.md` | 911948d 2026-08-21 12:25 |
| `data/b4-taxonomy-001/labels.json` | 911948d 2026-08-21 12:25 |
| `manifest/S2-arc2-paper-rerun-001.toml` | 28d94ce 2026-08-21 12:08 |
| `data/S2-arc2-paper-rerun-001/fpcurve2.json` | 2ab0083 2026-08-21 12:13 |
| `manifest/S2-arc2-tier3-001.toml` | ca9f6a2 2026-08-21 12:26 |
| `data/S2-arc2-tier3-001/tier3.json` | 2130c1d 2026-08-21 12:54 |
| `manifest/S2-arc2-tier4-001.toml` | cf59d77 2026-08-21 13:00 |
| `data/S2-arc2-tier4-001/tier4.json` | 1edd8fe 2026-08-21 16:30 |
| `manifest/S2-arc2-depth4-001.toml` | 922d4e4 2026-08-21 16:37 |
| `data/S2-arc2-depth4-001/depth4.json` | ed82386 2026-08-30 03:42 |
| `notes/b5-design-2026-08-21.md` | 344859e 2026-08-21 16:38 |
| `notes/b5-prereg-2026-08-23.md` | 181a54c 2026-08-23 11:26; 7cc7f06 2026-08-23 18:14; 2e74ad9 2026-08-23 18:15; e384f35 2026-08-23 18:18; 026fab0 2026-08-24 10:39 |
| `manifest/S2-b5-001.toml` | 7dde135 2026-08-23 19:49 |
| `data/S2-b5-001/m1_evidence.json` | 2970839 2026-08-23 21:09 |
| `notes/b5-prereg-v2-2026-08-23.md` | 7597c22 2026-08-23 21:16 |
| `data/S2-b5-002/m1b_disc.json` | 25ba0d4 2026-08-24 10:43 |
| `data/S2-b5-002/floor_T.json` | 25ba0d4 2026-08-24 10:43 |
| `data/S2-b5-002/arm_R.json` | 25ba0d4 2026-08-24 10:43 |
| `data/S2-b5-002/inject_T.json` | 0c24bd4 2026-08-25 03:36 |
| `data/S2-b5-002/inject_R.json` | 0c24bd4 2026-08-25 03:36 |
| `data/S2-b5-003/eval_T.json` | ea9deb4 2026-08-30 10:17 |

The v0 to v2 runs (2026-07-24) were exploratory; their numbers are
cited from the preregistered rerun `S2-arc2-paper-rerun-001`. See
DECISIONS.md for what each run froze and what it found.

## Files changed for publication

All files under `data/` are byte-identical to the original repository (`data/S2-b5-002/m2/WEIGHTS.md` was written
for this release). The files below differ from their original versions
only by cuts marked […] and by
deleting internal workstream labels such as "(Lane B)":
- `notes/arc2-dsl-v1.md`
- `notes/b4-tier-protocol-2026-08-21.md`
- `notes/b5-design-2026-08-21.md`
- `notes/b5-prereg-2026-08-23.md`
- `notes/b5-prereg-v2-2026-08-23.md`
- `manifest/S2-arc2-census-001.toml`
- `manifest/S2-arc2-depth4-001.toml`
- `manifest/S2-arc2-fpcurve-001.toml`
- `manifest/S2-arc2-fpcurve-002.toml`
- `manifest/S2-arc2-paper-rerun-001.toml`
- `manifest/S2-arc2-tier3-001.toml`
- `manifest/S2-arc2-tier4-001.toml`
- `DECISIONS.md` (an excerpt; see its preamble)
- `neural/tpd_neural/model.py`, `neural/tpd_neural/data.py`,
  `neural/b5_m2_train.py`: docstrings and comments reworded; an unused sparse-autoencoder class and
  helper were deleted from `model.py`. The training code path is unchanged.
- `kaggle/tpd-arc2/`: README rewritten; the notebook is the linked
  Kaggle version 3 with its source-location sentence updated.
- `src/bin/arc2_census.rs`: one doc comment; no code change. All other
  Rust source files are byte-identical to the original repository.
