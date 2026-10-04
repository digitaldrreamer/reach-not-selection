# Reach, Not Selection: Measuring What Is Missing on ARC-AGI-2

**Linked entry:** [tpd-arc2-entry](https://www.kaggle.com/code/digitaldrreamer/tpd-arc2-entry) (submission 55910824, leaderboard score 0.00) | **Code, data and preregistrations:** [reach-not-selection](https://github.com/digitaldrreamer/reach-not-selection), MIT-0

## Abstract

We take a simple program-search solver for ARC-AGI-2 and measure why it fails. A failure has one of two causes: the solver's set of operations cannot express a correct program (a **reach** failure), or it can, and the solver picks the wrong program (a **selection** failure). With parameters fixed in writing before each run, we find that reach is the limit. Searching every program of up to 4 operations from an 80-operation vocabulary reaches 0 of the 120 public evaluation tasks, and our entry scores 0.00 on the leaderboard, as these measurements predicted.

## 1. Introduction

Each ARC-AGI-2 task shows a few example pairs of coloured grids, an input and its output. A solver must infer the rule and apply it to a new test input. The tasks are easy for people and hard for current AI.

One family of solvers searches for programs. It combines grid operations from a fixed list (its **vocabulary**, such as "rotate" or "mirror") and keeps any program that turns every example input into its output. When such a solver scores 0.00, the cause matters because the fixes differ: a reach failure needs new operations, and a selection failure needs a better way to choose among programs. We measured both, so that effort goes to the right one.

## 2. Prior work

The 2020 Kaggle ARC winner (icecuber) searched compositions of grid transformations up to a fixed depth. Hodel's arc-dsl is a public library of ARC operations; part of our vocabulary reimplements a subset of it. Among the 2025 paper awards, SOAR fine-tunes a language model on its own program-search traces and needs no hand-written vocabulary, and Pang's evolutionary search builds a library of reusable program pieces as it goes. NVARC, the 2025 top Kaggle score (24%), relied on large amounts of synthetic data. TRM and CompressARC solve tasks with very small neural networks.

Our work is closest to the library-building approaches, because it asks where new operations should come from. Its contribution is a measurement: it locates where a vocabulary-based search fails, and it tests what happens when operations are added, with each step committed in advance.

## 3. Approach

**Solver.** A Rust program enumerates chains of grid operations (rotations, reflections, recolourings, scaling, combining grid halves), shortest first in a fixed order, up to 3 operations unless noted. A program **fits** a task when it reproduces every example output exactly. A task is **reached** when at least one fitting program exists within the search budget. The default selection rule, **R1**, submits the first fitting program found. Figure 1 shows a real task and its first fit.

![Figure 1](media/fig1_task_walkthrough.png)

*Figure 1. One task and the first program that fits it. Two steps, mirror right then mirror down, reproduce the example output and also produce the held-back test output.*

**Vocabulary stages.** The vocabulary grows in stages v0 to v4, reaching 80 operations at v4; v1 adds a subset of arc-dsl. Stages v0 to v2 were first run as exploration and later reproduced exactly under a preregistered rerun. From v3 on, new operations came from a rule written down before we looked at any unsolved task: sort the tasks that no program reaches into categories, then add operations for the largest categories.

**Budgets.** Each search has a step budget of 100,000 or 1 million steps per task. A search that runs out of budget only shows that no fit was found yet, so we report every such case. All runs on the evaluation set searched the whole program space.

**Where operations come from.** To test whether the source of an operation matters, two different search procedures (exhaustive enumeration, and a randomised beam search) were run on 60 families of generated practice tasks, with no real ARC data. From a pool of 40 candidate operations, condition **T** took the 10 that both procedures relied on most; condition **R** took 10 at random from the same pool, using a seed fixed in advance.

## 4. Results: leaderboard and public evaluation set

**Leaderboard: 0.00**, the score we predicted before submitting. The linked entry runs the v4 vocabulary plus T's 10 operations, with rule R1 and two attempts per task.

**Public evaluation set: 0 of 120 tasks reached.** At v4, no program of up to 3 operations fits any evaluation task. To check whether longer programs would help, a preregistered run searched every program of up to 4 operations: about 41.5 million programs per task, with no search cut short. It also reached 0 of 120. Adding T's 10 operations gave 0 of 120 again, also searched completely.

With no correct program anywhere in the search space, no selection rule can score above zero, so the leaderboard result follows from reach alone. The note we filed before the length-4 run stated what a zero would mean: the evaluation tasks need **kinds of operations this vocabulary lacks**, and longer combinations of the existing ones do not supply them.

![Figure 2](media/fig2_reach_ladder.png)

*Figure 2. Tasks reached as the vocabulary grows. (a) Public training set, out of 1,000: reach climbs from 25 to 58. (b) Public evaluation set, out of 120: every search ran to completion and found no fitting program.*

## 5. Analysis on the training set

The evaluation set gives nothing to analyse, since no program fits any of its tasks. To study the mechanism, we ran the same solver on the 1,000 public training tasks. We report these numbers as measurements of how the solver behaves.

**Selection rarely fails.** At v0, 25 tasks are reached and the first fit solves all 25. At v4, where no search was cut short, 58 tasks are reached and the first fit solves 52. Of the other 6, only 2 have any fitting program that solves the test, so a perfect selection rule could add at most 2. Three more elaborate rules (shortest program, filtering out trivial programs, and voting among candidates) tied with R1 at v2; voting helped on one task at v1.

**Newly reached tasks mostly become solved.** If selection works, adding operations should turn new reach into new solves. We tested this four times, each under its own preregistration:

| Test | Newly reached | Of these, solved |
|---|---|---|
| v3 (operations from the categories) | 7 | 6 |
| v4 | 2 | 2 |
| T added to v4 | 6 | 5 |
| R added to v4 | 3 | 2 |
| **All four** | **18** | **15** |

R's 3 newly reached tasks are also among T's 6, so across distinct tasks the four tests reach 15 and solve 13. No previously solved task was lost at v4, with T, or with R.

**The source of operations matters.** At equal budgets, T newly solved 5 training tasks and R solved 3. One of R's three was a task the solver already reached, for which R supplied the first correct program. The preregistered reading is that operations chosen by agreement between two search procedures beat an equal-sized random draw from the same pool. The two conditions share 3 operations.

## 6. Conclusion

For this solver, ARC-AGI-2 difficulty is a reach problem. Selection is close to its ceiling, new reach mostly converts to solves, and the evaluation tasks need kinds of operations that neither our hand-built stages nor our proposal method supplied. Progress for this family of solvers depends on methods that acquire new kinds of operations.

**What did not work.** A planned neural version of the proposal method learned its practice tasks but scored 0.02 on held-out families, below its preregistered bar of 0.50, so we excluded it. One preregistered threshold turned out to be impossible to meet with the fixed task generator; it was replaced by a ranking rule before the rankings were read. The second search procedure was specified after the first one's results were seen.

**Limits.** Searches that hit their budget give lower bounds. The counts are small. The T-versus-R result is one comparison on one pool of operations. Every number comes from a run whose settings were fixed in writing beforehand, or from an exact preregistered rerun of an exploratory run, and the submitted binary reproduces the measured runs byte for byte. The repository holds the code, data, preregistrations and a dated log of every run and correction.

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
