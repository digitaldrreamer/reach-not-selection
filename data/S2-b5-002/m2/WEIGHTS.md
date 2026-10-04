# Model weights omitted

The 80 per-family model checkpoints (~247 MB) are omitted from this
repository. They are fully regenerable: neural/b5_m2_train.py with the
seeds frozen in notes/b5-prereg-2026-08-23.md (base seed 993804,
per-family offsets in the script) reproduces them deterministically on
CPU. Per-family metrics (final loss, held-out exact-match) are kept
here as the *.json files, and the competence summary in summary.json.
