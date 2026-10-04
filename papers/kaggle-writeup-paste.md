**Linked entry:** [tpd-arc2-entry](https://www.kaggle.com/code/digitaldrreamer/tpd-arc2-entry) (submission 55910824, leaderboard score 0.00) | **Code, data and plans:** [reach-not-selection](https://github.com/digitaldrreamer/reach-not-selection), MIT-0 | **Full paper:** [PDF](https://github.com/digitaldrreamer/reach-not-selection/blob/main/papers/reach-not-selection-full.pdf)

## Abstract

We study a simple ARC-AGI-2 solver that searches for programs, and we measure why it fails. A failed task has one of two causes. In a **reach failure**, no program the solver can build fits the task. In a **selection failure**, a correct program exists, but the solver picks a different one. We wrote down the plans in advance. The conclusion lists the exceptions. Reach is what limits this solver. We searched every program of up to 4 operations, with up to 80 operations per task. No program fits any of the 120 public evaluation tasks. Our entry scored 0.00, as we predicted before submitting.

## 1. Introduction

An ARC-AGI-2 task shows a few example pairs of coloured grids. Each pair has an input and an output. A solver must find the rule and apply it to a new test input. People find these tasks easy. Current AI systems find them hard.

One kind of solver searches for programs. It builds programs from a fixed list of grid operations, such as "rotate" or "mirror". We call this list its **vocabulary**. The solver keeps every program that turns each example input into its example output.

When this kind of solver scores 0.00, the cause matters, because each cause has a different fix. A reach failure needs new operations. A selection failure needs a better way to choose among programs.

[INSERT fig_flow.png]

*Figure 1. How a task ends. Every failed task is either a reach failure or a selection failure.*

## 2. Prior work

icecuber won the 2020 Kaggle ARC competition by searching chains of grid operations up to a fixed length. Hodel's arc-dsl is a public library of ARC operations. Part of our vocabulary is rewritten from it. Among the 2025 papers, SOAR trains a language model on records of its own program searches, so it needs no hand-written vocabulary. Pang's evolutionary method builds a library of reusable program parts while it searches. NVARC had the top Kaggle score in 2025 (24%) and used large amounts of generated data.

Our work is closest to the library-building methods, because it asks where new operations should come from. Our main contribution is a measurement of where a vocabulary-based search fails.

## 3. Approach

**Solver.** A Rust program lists chains of grid operations, such as rotations, mirror images, recolourings and scaling. It tries shorter chains first, in a fixed order, up to 3 operations unless we say otherwise. A program **fits** a task when it gives every example output exactly. A task is **reached** when at least one fitting program is found. Our solver submits the first fitting program it finds. Figure 2 shows a real task.

[INSERT fig1_task_walkthrough.png]

*Figure 2. One task and the first program that fits it. Mirror right, then mirror down, gives the example output. It also gives the correct output for the test input, which the solver does not see.*

**Vocabulary.** The vocabulary grows in five stages, v0 to v4. At v4 it has up to 80 operations per task. Stage v1 replaces v0 with operations taken from arc-dsl. We first ran v0 to v2 as exploration, then ran them again under a plan written in advance and got the same results. For v3 and v4, we wrote the rule for adding operations before looking at any unsolved task. The rule sorts the unreached tasks into categories and adds operations for the largest ones.

**Budgets.** Each search has a limit on how much work it may do. A search that hits the limit only shows that no fit was found yet. We report every such case. All v4 runs on the evaluation set checked every program up to the length limit.

**Where operations come from.** We had a pool of 40 candidate operations and 60 families of generated practice tasks. A family is a group of tasks made by the same rule. No real ARC data was used. Two search methods solved the practice tasks. One tries every program. The other is a beam search, which keeps only the most promising unfinished programs at each step. The **agreement set** is the 10 operations both methods relied on most. The **random set** is 10 operations drawn at random from the same pool, using a random seed fixed in advance.

## 4. Results: leaderboard and public evaluation set

**Leaderboard: 0.00.** We predicted this before submitting. The entry uses the v4 vocabulary plus the agreement set, and two attempts per task.

**Public evaluation set: 0 of 120 tasks reached.** At v4, no program of up to 3 operations fits any evaluation task. A planned run then checked every program of up to 4 operations: up to about 41.5 million programs per task. It also reached 0 of 120. Adding the agreement set gave 0 of 120 again.

No searched program is correct on any evaluation task, so no choosing rule can help. Before the length-4 run, we filed a note saying what a zero would mean: the evaluation tasks need **kinds of operations this vocabulary does not have**. Programs of up to 4 existing operations do not reach these tasks.

[INSERT fig2_reach_ladder.png]

*Figure 3. Tasks reached as the vocabulary grows. (a) Public training set, out of 1,000: reach grows from 25 to 58. (b) Public evaluation set, out of 120: every search finished, and no program fits.*

## 5. Analysis on the training set

No program fits any evaluation task, so there is nothing more to study there. To see how the solver succeeds and fails, we ran it on the 1,000 public training tasks. These numbers describe the solver's behaviour.

**Selection rarely fails.** At v4 every search finished. The solver reaches 58 tasks, and the first fitting program solves 52. Of the other 6, only 2 have any fitting program that solves the test. So a perfect choosing rule could add at most 2 tasks. Three other choosing rules, described in the full paper, tied with the first-fit rule at v2 when programs had to fit all example pairs.

**Most newly reached tasks are solved.** If choosing works well, most tasks that new operations reach for the first time should be solved. We tested this four times, each with its own plan:

| Test | Newly reached | Of these, solved |
|---|---|---|
| v3 | 7 | 6 |
| v4 | 2 | 2 |
| agreement set added to v4 | 6 | 5 |
| random set added to v4 | 3 | 2 |
| **All four** | **18** | **15** |

The random set's 3 newly reached tasks are also among the agreement set's 6. Counting each task once, the four tests reach 15 and solve 13.

**The way operations are chosen made a difference.** With the same budget, the agreement set newly solved 5 training tasks and the random set newly solved 3. One of the random set's 3 was a task v4 already reached, for which it supplied the first correct program. Our plan said in advance what this result means: operations that two search methods both rely on beat operations drawn at random from the same pool. This rests on one comparison with small counts. The two sets share 3 operations, so a difference is harder to find.

## 6. Conclusion

For this solver, ARC-AGI-2 is hard because of a lack of reach. A better choosing rule would add little. The evaluation tasks need kinds of operations that our hand-built stages did not supply, and the agreement set did not either. This kind of solver needs methods that add new kinds of operations.

**What did not work.** We planned a neural way to choose operations. On new examples from 20 practice families, it gave exactly the right output 0.02 of the time. That is below the 0.50 bar we set in advance, so we did not use it. Our first planned threshold, for how often an operation appears in the search methods' solutions, was far too strict: almost no candidate could pass it, and none did. We replaced it with a ranking rule before looking at the rankings. We designed the second search method after seeing the first method's results. One run used the wrong search order. A check caught it, and we ran it again.

**Limits.** Searches that hit their limit may undercount. The counts are small. The comparison of the two sets was run once, on one pool of operations. The leaderboard uses a hidden set we cannot inspect. The solver we ran on Kaggle gives the same output as our measured run on all 120 evaluation tasks, byte for byte.
