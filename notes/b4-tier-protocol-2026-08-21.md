# B4 reach-expansion protocol — FROZEN 2026-08-21

**Status: FROZEN before any tier-3 op was selected, classified, or
implemented.** No unreached-task taxonomy pass had been performed when this
note was committed; the git history of this file is the proof. Changes require
a dated amendment plus a `DECISIONS.md` entry. Companion:
[…], DECISIONS.md 2026-08-21 scope
entry, `manifest/S2-arc2-paper-rerun-001.toml` (the tier-1/2 baseline this
protocol extends).

## 1. Purpose and the prediction under test

B1 (measured, promoted 2026-08-21): the residual ARC-AGI-2 difficulty for a
frozen DSL-BFS solver is REACH, not SELECTION — 25/1000 solved at v0 with
zero fit-but-first-wrong; 46 tasks with any fit at tier-2 k=all, with
R1=R2=R3=R4 = 0.913; tier-1 k=all R4 = 1.0 over 34 tasks.

B4 tests B1's forward prediction: **when reach is expanded with new ops,
newly reachable tasks convert to solved tasks at high rate, because selection
(shortest-first first-fit) is already near-solved within reach.**

## 2. Frozen instrument parameters (unchanged from tiers 1-2)

- `max_len = 3`; `enum_budget_nodes = 100000`; `fitting_cap_per_task = 2000`;
  `palette_cap = 5`; canonical enumeration order; no sampling.
- **Selection rule frozen as R1** (first fit in canonical shortest-first
  order). R2-R4 are reported descriptively but R1 is the pre-named rule for
  conversion and for the Kaggle entry.
- Tie seeds pre-assigned: tier 3 = 992000, tier 4 = 992100, tier 5 = 992200.
- Dataset: ARC-AGI-2 `data/training` @ `f3283f72…` (per
  S2-arc2-paper-rerun-001). Budget censoring is reported with every reach
  number (100k-node cap; tier-2 k=1 censoring was 87/1000).

## 3. Op-selection rule (judgment contained, then frozen)

1. **Taxonomy pass (one rater, before any implementation):** every task
   unreached at tier 2 (no fitting program in the tier-2 alphabet under the
   frozen budgets, from `data/S2-arc2-paper-rerun-001/fpcurve2.json`) is
   labeled with one primary category from this FIXED checklist, written now,
   before looking at any unreached task:
   1. tiling / scaling / block replication
   2. symmetry completion / reflection or rotation fill
   3. per-object recolor conditioned on a property (size, shape, count)
   4. object translation / gravity toward an anchor or wall
   5. line / path drawing, connection of markers
   6. count-driven output construction (dims or content from counts)
   7. crop / extract to a marked or distinguished subregion
   8. multi-grid combination (mask, overlay, boolean between panes)
   9. pattern repair / denoise / majority-fill
   10. other (free text, no op may be proposed from this bucket alone)
   The per-task labels are committed as an artifact BEFORE any tier-3 op is
   implemented.
2. **Tier construction:** tier k+1 adds at most 10 new ops implementing the
   highest-count categories from the taxonomy. The exact op list (names,
   signatures, semantics) is frozen in a manifest before the tier's
   conversion run. Ops are general transformations; no op may encode a
   specific task's answer.
3. **No peeking:** test outputs of training tasks are used only as ground
   truth for scoring, never by selection logic (unchanged from census). The
   public evaluation set (120 tasks) is NEVER used for op selection; it may
   be scored at most once per tier, read-only, recorded as a generalization
   check. Kaggle's hidden sets are touched only by actual submissions.

## 4. Measurement (mechanical)

For each new tier k+1, over `data/training`:

- `N_k` = tasks with >= 1 fitting program at tier k+1 and none at tier k
  (newly reachable).
- `C_k` = fraction of `N_k` whose R1 first fit solves the held-back test
  (conversion).
- `FP_k` = fraction of `N_k` with >= 1 fit but R1 wrong on test.
- Regression guard: tasks solved at tier k must remain solved at tier k+1
  (any regression is reported, not silently absorbed).

## 5. Pre-filed outcome classification (descriptive bars, not gates)

These bars classify outcomes for honest reporting; they license nothing and
feed no verdict ([…]: no threshold here has measured power — none
pretends to).

- **P1 (reach):** each tier is expected to add `N_k >= 5`.
- **P2 (conversion, the thesis bar):** `C_k >= 0.75` reads as supporting
  B1's prediction (tier-1/2 precedent: 0.91-1.0). `0.5 <= C_k < 0.75` reads
  as weakening. **Refutation sentence, pre-written:** if `C_k < 0.5`, the
  claim "selection is near-solved within reach" does not extend to tier k+1's
  op families, and the paper reports that boundary as a finding.
- The paper reports measured `C_k` whatever it is; no bar may be moved after
  a run.

## 6. Stopping rule

Stop adding tiers at the EARLIEST of:

- (a) calendar stop 2026-10-15 (paper draft needs the numbers);
- (b) two consecutive tiers with `N_k < 5`;
- (c) any tier with `C_k < 0.5` — stop, report the boundary; continuing
  requires a new `DECISIONS.md` entry.

## 7. Kaggle entry rule

The entry submits, per test input: attempt_1 = R1 first fit; attempt_2 =
next distinct fitting program in canonical order (if none, attempt_1
repeated). The entry's alphabet is the highest tier frozen by (6) at
submission time. No eval-derived tuning of any kind.

## 8. B5 stub (design-only until go/no-go 2026-10-01)

Comparison arms, preregistered separately before any B5 run: triangulated
ops (proposed by two-media convergence on synthetic task families,
extensional mode only; […]) vs
taxonomy ops from §3, at matched op-count budget, measured with §4's metric.
If B5 does not go, it is recorded as future work; nothing in B4 depends on it.

---

## Amendment A1 — 2026-08-21 (researcher-directed): node budget 100k → 1M

Authorized by the researcher before any tier-4 artifact existed. Grounds:
at 100k the tier-3 run censored 850/1000 tasks and cost one v2-reached task
its reach; the cap, not the alphabet, was becoming the binding constraint.

- `enum_budget_nodes = 1_000_000` for all runs from tier 4 onward, applied
  UNIFORMLY to every alphabet inside a comparison (N_k and C_k are defined
  only between alphabets measured at the same budget).
- Recorded tier-2/3 numbers at 100k stand as measured under their manifests;
  they are not restated. The tier-4 manifest measures its v3 baseline fresh
  at 1M, and reports the v3@1M vs v3@100k reach delta so the budget effect
  is isolated from the op effect.
- Per-tier eval reads from tier 4 onward also run at 1M.
- All other parameters unchanged (len <= 3, cap 2000, R1, seeds as
  pre-assigned: tier 4 = 992100).

---

## Amendment A2 — 2026-08-21 (researcher-authorized): depth-4 eval probe

A one-off attribution measurement, registered before any depth-4 code or
run existed. Question: is the exhaustive eval-zero at len <= 3 (tier-4
finding) attributable to composition DEPTH or to the OP VOCABULARY?

- Instrument: v4 alphabet, len <= 4, node budget 100,000,000 (exhaustive
  for the <= ~80-op v4 alphabet: 80^4 ≈ 4.1e7 < 1e8), cap 2000, canonical
  order, R1. Public evaluation set (120 tasks) ONLY — this is a
  measurement about the eval reach frontier, not training op-mining, so
  §3.3's once-per-tier eval-read allowance is extended by this amendment
  to cover exactly this ONE probe run.
- Outcomes, pre-filed: zero fits at depth 4 → the gap is op-class, not
  depth (B5's motivation sharpens). Nonzero fits → depth was binding; the
  entry's search depth becomes a live design question (separate
  amendment required before changing the entry).
- Manifest: S2-arc2-depth4-001. No training-set measurement, no op
  selection may derive from this probe's per-task results beyond the
  aggregate finding.

## Amendment A3 — 2026-08-21 (researcher-directed): B5 pulled forward, parallel

§8's go/no-go date (2026-10-01) is replaced by an immediate go, in
PARALLEL with the depth probe, tier 5, and Kaggle packaging, under two
conditions:

1. B5 still requires its own preregistration (design note first, prereg
   manifest before any B5 run) — §8's substance is unchanged, only its
   calendar gate.
2. Packaging-priority guard: if the first real Kaggle submission has not
   happened by 2026-09-15, B5 PAUSES until it has. The eligibility path
   outranks the research arm.
