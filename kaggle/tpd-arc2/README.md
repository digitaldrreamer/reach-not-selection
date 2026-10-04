# Kaggle entry: tpd-arc2-entry

`tpd-arc2-entry.ipynb` is the code of the linked Kaggle notebook,
version 3, which produced submission 55910824 (public score 0.00).
The only difference from the Kaggle copy is the sentence naming where
the source lives, which now points to this repository. Indentation was
restored from the rendered notebook.

The notebook needs two inputs: the ARC Prize 2026 ARC-AGI-2
competition data, and a Kaggle dataset holding the `arc2_solve_t`
static binary and `data/S2-b5-002/floor_T.json` (arm T's ten
operations).

Build the binary on x86_64 Linux from the repository root:

    cargo build --release --target x86_64-unknown-linux-musl --bin arc2_solve_t

The binary is deterministic. The entry's binary has sha256
3c1b0530e2fcab89106fdd730bb3a21aef0ce463c6e366209710155864c3ddd0;
on the 120 public evaluation tasks it reproduces the measured folded
submission byte for byte (see DECISIONS.md, 2026-08-31a). License:
MIT-0.
