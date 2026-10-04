# Reach, Not Selection: Measuring What Is Missing on ARC-AGI-2

**Linked entry:** [tpd-arc2-entry](https://www.kaggle.com/code/digitaldrreamer/tpd-arc2-entry) | **Code and artifacts:** [reach-not-selection](https://github.com/digitaldrreamer/reach-not-selection), MIT-0 | **Leaderboard score: 0.00, measured, and predicted in advance by our own measurements.**

## Summary

When a solver scores 0.00, what exactly failed? Only two things can: no fitting program that also gets the held-back test right was ever found (a **reach** failure), or one was found and a different fit was chosen (a **selection** failure). We measured both, with parameters committed in writing before the runs, and the answer is one-sided.

- **Reach is the wall, and it is vocabulary, not depth.** Exhaustive search over every composition of up to 4 operations from our largest, 80-operation vocabulary reaches **0 of 120** public evaluation tasks, with no search cut off by a budget.
- **Selection rarely fails inside reach.** At that vocabulary, 58 training tasks are reached with no search cut off; the first fit solves 52, and a perfect selector could add at most 2.
- **New reach converts.** Across four preregistered tests, 15 of 18 newly reached training tasks became solved.
- **Where operations come from matters.** Operations that two independent symbolic searches converged on newly solve 5 training tasks; an equal-sized random draw from the same pool solves 3.

## A solver simple enough to measure

The solver is a Rust enumerator over compositions of grid operations (rotations, reflections, recolorings, scaling, pane logic), run on the ARC-AGI-2 public training set (1,000 tasks). Programs are linear chains, so program length and composition depth coincide; length is ≤ 3 unless noted. Enumeration is shortest-first in a fixed order, so "the first fit" is well defined. Figure 1 shows one training task and the first program that fits it. Counts out of 1,000 are the public training set; out of 120, the public evaluation set; the leaderboard score comes from Kaggle's hidden set.

A program **fits** when it maps every training input to its training output exactly. A task is **reached** when at least one fitting program exists within the search budget. **Selection rule R1** takes the first fit in enumeration order; we registered three more elaborate competitors: shortest fit with a seeded tie-break, a screen rejecting degenerate fits (identity, constant outputs, palette projection), and clustering candidates by agreement of their predicted test outputs (ARC provides each test input; only its output is withheld). **Conversion** means a reached task whose selected program also solves that held-back test.

The vocabulary grows in stages v0 → v4 (80 operations at v4); v1 draws on Hodel's public arc-dsl. Stages v0 to v2 were exploratory runs, later reproduced exactly by a preregistered rerun. From v3 on, each stage's operations came from a rule committed before any unreached task was categorized, with metrics, tie-break seeds, and stopping rules fixed in advance; every result file is write-once and content-hashed. We report budgets with every number, plus **censoring**: when a search hits its budget before exhausting the program space, "no fit found" is a lower bound, not a verdict.

## Result 1: Within reach, selection rarely fails

At the base vocabulary v0, 25 of 1,000 training tasks have a fitting program within 3 operations, and all 25 first fits solve the test. At v2, 46 tasks have fits, and the four selection rules convert at an identical 0.913 on the registered aggregate metric; many v2 searches hit their budget, so that comparison runs on partial candidate lists. The cleanest test is v4, where no reached task was cut off: of 58 reached tasks the first fit solves 52, and only 2 of the other 6 have any fitting program that solves the test. A perfect selector could add at most 2 of 58. The elaborate rules helped once (clustering fixed one of 34 tasks at v1); at v2 they tie. **Whatever separates solvers from ARC-AGI-2, it is not choosing badly among candidate programs.**

## Result 2: Reach is the wall, and it is vocabulary, not depth

Result 1 licenses a falsifiable prediction: tasks a larger vocabulary newly reaches should convert at high rate. Call this the **conversion regularity**. We tested it four times:

| Preregistered test | Newly reached | Of these, solved |
|---|---|---|
| **Sequential vocabulary stages** | | |
| v3 (ops mined from the unreached-task categorization) | 7 | 6 |
| v4 | 2 | 2 |
| **Competing one-shot extensions of v4 (Result 3; alternatives, not stages)** | | |
| T (convergent proposals) | 6 | 5 |
| R (random, budget-matched) | 3 | 2 |
| **Pooled: four tests of the regularity** | **18** | **15** |

R's three newly reached tasks are all among T's six; counted as distinct tasks, the four tests reach 15 and solve 13. v4 produced no false positives and no regressions. Training reach climbs from 25 to 58 of 1,000 between v0 and v4. Stage by stage: v1 and v2 take 25 to 46; v3 adds 7, net 6 under its budget cap; raising v3's budget to 1M enumeration steps per task adds 4; v4 adds 2. Figure 2 shows the climb, and the flat zero beside it.

The public evaluation set is a different regime. The v4 vocabulary reaches 0 of 120 evaluation tasks at length ≤ 3. A preregistered follow-up separated depth from vocabulary: exhaustive enumeration at length ≤ 4, one level more than the budget used everywhere else (roughly 41.5M programs per task at 80 operations, zero censoring). Still **0 of 120**. As filed in writing before the run: the gap is missing operation classes, not composition depth. **For this solver class, the missing piece is not harder search but transformations a hand-built vocabulary does not contain.**

## Result 3: Where new operations come from changes the return

If the wall is vocabulary, where should new operations come from? Call the procedure that proposed an operation its **provenance**. Two deliberately different symbolic search procedures (exhaustive shortest-first enumeration; stochastic beam search with partial-credit scoring) searched 60 families of procedurally generated synthetic tasks, using no real ARC data. From a 40-operation candidate pool, condition T took the top 10 operations both procedures converged on; condition R drew 10 at random from the same pool under a pre-committed seed (3 operations overlap T's). The pool, seed, metric, and readings of each outcome were committed before discovery; one miscalibrated threshold was later voided, and the second procedure was specified after the first one's ranking was read. Added to v4 at identical budgets, on the same 1,000 training tasks: **T newly solved 5 tasks and R 3** (R's third gave an already-reached task its first test-correct program); neither condition regressed a solved task.

Our one preregistered evaluation-set measurement went to T: added to v4, it reaches 0 of 120 evaluation tasks, again exhaustively. **Operations that help on training do not touch the evaluation wall; the vocabulary missing there is of a different kind.** The preregistered reading of the training outcome: operations proposed by convergence beat an equal-sized random draw from the same pool, at matched cost. The claim covers two symbolic searchers only. A planned neural counterpart (one small model per task family) fit its training families but scored 0.02 mean exact match on held-out families against a preregistered threshold of 0.50, so we excluded it.

## Why we submitted an entry that scores 0.00

**Predicted score: 0.00. Measured: 0.00, for this entry and for the earlier v4-only entry.** The linked notebook runs this solver with the v4 vocabulary plus T's ten operations (same first-fit rule R1, two attempts per task). We submit because that score *is* the result: with evaluation reach exhaustively zero, no selection mechanism can produce a nonzero score. Reach is where the points are.

## What these numbers do and do not license

Every reported number comes from a run whose parameters were committed in writing before execution, or from an exact preregistered rerun of an earlier exploratory run; the submitted binary reproduces the measured runs byte-for-byte; result files are write-once and content-hashed. **What didn't work:** the neural counterpart missed its held-out threshold, one evidence threshold was voided as miscalibrated, and budget censoring at v2 and v3 was heavy. Four limits bound the numbers. Reach at a capped budget is a lower bound (the v4, length-4, and evaluation runs were censor-free). All conversion counts are small: 15 of 18 pooled. The provenance result is one comparison on one candidate pool and licenses no universality claim. Code (MIT-0), preregistration files, seeds, run logs, and the decision log are in the linked repository.

---

## Where each number comes from

All paths are in this repository. Run IDs (S2-...) match `manifest/` and `data/`.

| Result | File |
|---|---|
| v0 reach and first-fit solves (25 of 25) | `data/S2-arc2-census-001/census.json` |
| v1 selection rules (clustering fixes 1 of 34) | `data/S2-arc2-paper-rerun-001/fpcurve.json` |
| v2 selection rules (0.913 over 46) | `data/S2-arc2-fpcurve-002/fpcurve2.json`, reproduced in `data/S2-arc2-paper-rerun-001/fpcurve2.json` |
| v3 reach and conversion (7 reached, 6 solved) | `data/S2-arc2-tier3-001/tier3.json` |
| v4 reach, conversion, selection bound (58, 52, 54) | `data/S2-arc2-tier4-001/tier4.json` |
| Evaluation reach at length 4 (0 of 120) | `data/S2-arc2-depth4-001/depth4.json` |
| T and R on training (6 and 3 reached) | `data/S2-b5-002/inject_T.json`, `inject_R.json` |
| T on evaluation (0 of 120) | `data/S2-b5-003/eval_T.json` |
| Preregistrations | `notes/b4-tier-protocol-2026-08-21.md`, `notes/b5-prereg-2026-08-23.md`, `notes/b5-prereg-v2-2026-08-23.md` |
| Dated record of every run, correction and void | `DECISIONS.md` |

Figures are regenerated from these files by `papers/media/make_figures.py`.
