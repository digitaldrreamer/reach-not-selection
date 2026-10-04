# ARC-2 DSL v1 — the expressive tier (option (c), published subset)

Source: Hodel, arc-dsl (github.com/michaelhodel/arc-dsl, dsl.py @
main, fetched 2026-07-24; 160 typed functions; […]). v1 takes the Grid -> Grid subset relevant to
whole-grid transformation tasks, semantics reimplemented in Rust
(crates/tpd-arc), Hodel's names preserved for auditability.

## Fixed (parameterless) ops — Hodel name -> v1

| Hodel | v1 | note |
|---|---|---|
| rot90 / rot180 / rot270 | Rot90/Rot180/Rot270 | |
| hmirror / vmirror | HMirror/VMirror | horizontal/vertical mirror |
| dmirror / cmirror | DMirror/CMirror | main/anti diagonal |
| tophalf / bottomhalf / lefthalf / righthalf | TopHalf/BottomHalf/LeftHalf/RightHalf | |
| trim | Trim | strip 1-cell border |
| compress | Compress | drop uniform rows+cols (frontiers) |
| upscale (k=2,3) | Upscale2/Upscale3 | unary instantiation of Hodel's (grid,k) |
| downscale (k=2) | Downscale2 | |
| hupscale / vupscale (k=2) | HUpscale2/VUpscale2 | |
| hconcat(I, hmirror(I)) | HConcatMirror | unary closure of Hodel's binary op, self+mirror argument — a documented restriction, not a Hodel primitive |
| vconcat(I, vmirror(I)) | VConcatMirror | same restriction |

## Parameterized ops, instantiated per task

| Hodel | v1 instantiation |
|---|---|
| replace(grid, a, b) | Replace(a,b) for ordered pairs a != b drawn from the task's train-INPUT palette, capped at the 5 most frequent colors (deterministic: count desc, color asc) |
| switch(grid, a, b) | Switch(a,b) for unordered pairs from the same capped palette |

Alphabet per task <= 22 fixed + 20 replace + 10 switch = 52 ops.

## What v1 is NOT

Object-level ops (objects, paint, move, gravitate, occurrences …)
are out of v1 — they need object types and argument search, a
second tier. If the expressive-tier ambiguity is not reachable
with v1 either, that finding forces tier 2 and says something
about where ARC ambiguity actually lives; either way it is a
measurement, not a failure of the plan.

## Ambiguity protocol (S2-arc2-fpcurve-001)

k-sweep over train pairs (k = 1, 2, all): enumerate v1 programs
(len <= 3, canonical order, node-budgeted) fitting the FIRST k
train pairs; ground truth = test-correctness. Selection rules:
R1 first-fit; R2 shortest (seeded tie-break); R3 = R2 + screens
(identity-on-train-inputs, constant-output, palette-projection);
R4 = R3 + agreement clustering on the visible TEST INPUTS
(predicted-output equality; the ARC-legal analogue of fresh-input
clustering — test inputs are visible at prediction time, outputs
are not). The fp hazard = share of k-fitting programs that are
test-wrong; the recovery = rule accuracy vs k.

## Tier 2 (v2) — object-level closures (added 2026-07-24, tests the
## pre-filed prediction of DECISIONS 2026-07-24)

Objects = 4-connected same-color components against bg = most
common color (ties: smaller color id). Ten parameterless closures
over Hodel's object vocabulary (objects / argmax / argmin by size /
cover / paint / subgrid; ties broken by size then topmost-leftmost
cell — deterministic):

| v2 op | closure of |
|---|---|
| KeepLargestObj / KeepSmallestObj | paint(canvas(bg), argmax/argmin(objects, size)) |
| RemoveLargestObj / RemoveSmallestObj | cover(grid, argmax/argmin(objects, size)) |
| CropLargestObj | subgrid(argmax(objects, size), grid) |
| FillHoles | border-flood bg; unreached bg cells take the majority adjacent non-bg color (tie: smaller color) |
| GravityDown/Up/Left/Right | per-column/row slide of non-bg cells (standard ARC motif; nearest Hodel analogue: gravitate) — labeled a MOTIF closure, weakest provenance of the set |

v2 alphabet = v1 alphabet + these 10 (<= 62 ops/task).
PRE-FILED PREDICTION under test (ledger, 2026-07-24): tier 2
(a) raises reach, (b) raises raw k=1 hazard, (c) keeps within-reach
R4 recovery >= 0.9 — (c) failing kills the validation-discipline
claim at this tier.
