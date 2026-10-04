# Decision log

This is the decision log for the measurements behind the paper "Reach,
Not Selection: Measuring What Is Missing on ARC-AGI-2". Each entry is
dated, and in the original log entries are append-only: later entries
correct earlier ones rather than editing them. Entries not about this
paper are omitted, and cuts within entries are marked […]. Manifest names
(S2-...) match the directories under data/.

## 2026-07-24: ARC-2 CENSUS LANDS (exploratory, CPU, $0): census clean (25/1000 at the v0 tier, zero selection hazard THERE)
Verdicts first, failures unsoftened; numbers from named artifacts.
[…]

1. S2-arc2-census-001 — VERDICT: PIPELINE PROVEN ON REAL ARC; THE
V0 TIER HAS NO SELECTION HAZARD. 1000 tasks loaded; mean 3.23
train pairs; 680 shape-preserving. The v0 DSL (12 parameterless
ops, len <= 3) fully fits 25 tasks — and first-fit solves the
held-back test on ALL 25 (fit_but_first_wrong = 0). Fit-count
spread (up to 50) is extensional respelling, the canonicality
fingerprint. READING: this LOCATES the ambiguity rather
than refuting it — with ~3 full-grid examples and parameterless
ops, behavior is pinned exactly […];
the fit-but-wrong hazard the selection story needs lives in the
expressive tier (parameterized/conditional ops). THE DSL UPGRADE
(published-subset swap) IS NOW THE GATE ON THE SELECTION QUESTION
[…]. Artifact:
data/S2-arc2-census-001/census.json, file-hash 1d7883e146b30dc5fe
0b9796ab727ba462b3e5b3b51eb39cd65f5dc25a889290.

[…]

## 2026-07-24: THE ARC-2 AMBIGUITY BAND IS FOUND (S2-arc2-fpcurve-001, exploratory): hazard 0.67 at k=1 falling to 0.02 at k=all; the failure decomposes into REACH vs SELECTION, and selection is near-perfect within reach from k=2
VERDICT FIRST: real few-shot ambiguity exists on ARC-2 at the v1
expressive tier […]. Mean
fit-but-test-wrong hazard: k=1: 0.671 (82 tasks with fits);
k=2: 0.226 (41); k=all(~3.2): 0.023 (34). Raw selection accuracy
at k=1 is only ~0.40 for every rule.
THE DECOMPOSITION (the load-bearing reading): at k=1 only 39/82
fitted tasks have ANY test-correct program in v1's reach — for
the other 43 the truth is outside the DSL and every fit is
coincidental; no selection rule can recover absent truth. WITHIN
the truth-reachable subset: hazard 0.309 at k=1 with rules
recovering to 0.846-0.872 (shortest-with-tie-break best); at
k=2 and k=all, R4 (agreement clustering on predicted test
outputs) selects PERFECTLY (1.000; r1-r3 at 0.971). Said plainly:
FROM TWO EXAMPLES UP, SELECTION IS ESSENTIALLY SOLVED WITHIN
REACH AT THIS TIER; the residual ARC problem is REACH (DSL
expressiveness), and reach expansion is exactly what re-inflates
spurious fits — the tension the selection-quality story lives on.
PREREGISTERABLE PREDICTION FOR TIER 2 (object-level ops), filed
now, before any tier-2 code exists: expanding the DSL will (a)
raise truth-reachable counts, (b) raise the raw k=1 hazard, and
(c) keep within-reach R4 recovery high (>= 0.9) — if (c) fails,
the validation-discipline claim dies at tier 2; that sentence is
pre-written here.
Ops notes: budget censoring at k=1 in 60/1000 tasks (100k-node
cap; recorded per task); process slip recorded honestly — the
first launch went untracked (shell background), was killed, its
partial artifact removed, and the run relaunched supervised;
write-once intact. Artifact:
data/S2-arc2-fpcurve-001/fpcurve.json, file-hash 1e59f867e6558182
8161270f02a83425a80d86846f83dc6671e98d0871d300b9.

## 2026-07-24: TIER-2 FP-CURVE LANDS (S2-arc2-fpcurve-002, exploratory): prediction clauses (a) and (b) CONFIRMED, (c) holds at k>=2; and the sharpest finding is one nobody pre-filed: R4's marginal value over first-fit is ZERO at tier 2
VERDICT against the pre-filed prediction (ledgered this morning,
before tier-2 code existed):
(a) REACH RISES: 34 -> 43 truth-reachable tasks at k=all. CONFIRMED.
(b) RAW k=1 HAZARD RISES: 0.671 -> 0.691 (fitted tasks 82 -> 101).
    CONFIRMED in direction, modest in size.
(c) WITHIN-REACH R4 >= 0.9: HOLDS at k=2 (0.957) and k=all (0.977).
    At k=1 within-reach recovery DEGRADED 0.846 -> 0.755. The
    filed clause named no k; at filing time the k=1 value was
    already 0.846 (< 0.9), so the clause cannot have meant k=1 —
    resolved against the sensible reading (k >= 2, where tier 1
    stood at 1.000) and recorded as HOLDS-WITH-DEGRADATION, not
    clean confirmation. The validation-discipline claim does NOT
    die; it also does not strengthen.
THE UNFILED FINDING (reported with equal prominence): at tier 2,
R1 = R2 = R3 = R4 in every aggregate row — agreement clustering
and the screens add NOTHING over first-fit/shortest at this tier
(tier 1 had shown small lift and within-reach perfection at k>=2).
Expressiveness also inflates the k=all hazard 0.023 -> 0.124 and
drops k=all rule accuracy 0.971 -> 0.913. Said plainly: the
canonicality prior (shortest-first) is carrying selection at both
tiers measured so far; a DISTINCTIVE validation edge has not yet
appeared on ARC-2 — if it exists, it lives in richer screens
(extensional beyond identity/constant), deeper programs,
parameter-heavy DSLs, or the two-guess submission regime, none of
which this probe tested. That is a narrower, more honest framing
than this morning's […].
Caveats: budget censoring heavy at tier 2 (87/1000 tasks at k=1;
100k-node cap) — enumeration coverage is thinner exactly where
the alphabet grew; recorded per task. Ops note (method lesson
banked): the first launch ran a STALE release binary (cargo test
does not rebuild the standalone executable) — panicked pre-write,
write-once intact, rebuilt explicitly and rerun; "rebuild
explicitly after any lib edit, before any launch" joins the
cadence. Artifact: data/S2-arc2-fpcurve-002/fpcurve2.json,
file-hash 0b208322f9349f730c235d2a8bb8041ccd28635702fc6650a35d0a
855f25dd2f.

## 2026-08-21: SCOPE CHANGE (researcher-signed): ARC Prize 2026 paper workstream opened

Researcher signed four decisions today (recorded here BEFORE any run):

1. **Fork**: one ARC Prize 2026 Paper Track submission, linked to an
   ARC-AGI-2 Kaggle code entry. The entry and the paper are the same object —
   the tpd-arc census enumerator, documented in full. Paper thesis: "Reach,
   Not Selection" (25/1000 solved at tier v0 zero-FP; 43/1000 truth-reachable
   at tier 2, a lower bound under the 100k-node cap; R1=R2=R3=R4 at tier 2,
   small R4 lift at tier 1).
2. **Effort = B4 + B5**: preregistered reach-expansion tiers (op-selection
   rule, conversion-rate metric, stopping rule frozen BEFORE adding ops), plus
   the triangulated op-proposal stretch arm — extensional mode only
   […]. B5 go/no-go 2026-10-01.
3. **Git authority**: drafting and committing the W0/W1 work authorized
   (this entry, evidence preservation, LICENSE).
4. **License = MIT-0** for self-authored code (paper-track rules require a
   permissive public-domain license; plain MIT may not qualify).

This workstream makes no hidden-benchmark claim; the Kaggle entry exists
because the paper track scores Accuracy from the linked entry's leaderboard
score and Completeness from how thoroughly the paper covers that entry.
[…]

**Plan of record:** […]. Honest bands recorded there: baseline entry ~2.83/5 avg,
any-recognition 15–25%; with B4 landing above ~4% ARC-AGI-2, ~3.3–3.7 avg.

**Evidence preservation (W1):** […]

**Deadlines of record:** entry/team freeze 2026-10-26 (verified on
[…]; ARC-AGI-2 equivalent UNVERIFIED —
confirm in W0); internal code submission 2026-10-30; paper 2026-11-08
(Kaggle hard stop 2026-11-09 23:59 UTC). One submission per team (Kaggle
Hackathon rule 2.2.a).

**Not authorized by this entry:** any paid inference run;
[…]; any ARC-AGI-2 leaderboard ambition
beyond the documented entry and its preregistered reach tiers.

## 2026-08-21b: W2 executed: census + fp-curve tiers 1-2 PROMOTED by preregistered rerun (exact reproduction); B4 protocol frozen before any tier-3 work

Under `manifest/S2-arc2-paper-rerun-001.toml` (registered and committed
BEFORE the run, `exploratory = false`), all three ARC-2 instruments were
rerun on a freshly cloned, commit-pinned dataset
(github.com/arcprize/ARC-AGI-2 @ `f3283f72…`, data/training, 1000 tasks):

- **census: EXACT reproduction** — n_tasks 1000, mean_train_pairs 3.232,
  n_any_fit_solves_test 25, n_first_fit_solves_test 25,
  fit_but_first_wrong 0, n_shape_preserving 680.
- **fp-curve tier 1: aggregates identical** to S2-arc2-fpcurve-001
  (k=all: 34 with fit, r1-r3 0.9706, r4 1.0, censored 22).
- **fp-curve tier 2: aggregates identical** to S2-arc2-fpcurve-002
  (k=all: 46 with fit, r1=r2=r3=r4 0.9130, censored 38; k=1: 101 with
  fit, censored 87).

No dataset drift since 2026-07-24. **These numbers are now citable in the
paper**, always with the budget-censoring caveat and tier scoping,
and may still not feed […] any
benchmark-progress claim. Outputs force-added at
`data/S2-arc2-paper-rerun-001/`.

**B4 protocol frozen:** `notes/b4-tier-protocol-2026-08-21.md`, committed
before any unreached-task taxonomy pass or tier-3 op existed. It fixes the
selection rule (R1), instrument params, pre-assigned tie seeds, a
ten-category taxonomy checklist, the conversion metric C_k, descriptive
outcome bars with the refutation sentence pre-written, the stopping rule
(2026-10-15 / N_k < 5 twice / C_k < 0.5), and the Kaggle two-attempt rule.
Bars are explicitly descriptive, not gates: no threshold has measured
power and none pretends to.

[…]

CPU only, $0, no model call. Next: taxonomy pass over the 954
tier-2-unreached tasks (per protocol §3.1), then the Kaggle port (W3).

## 2026-08-21c: W3 started: arc2_solve built and measured; tier-2 public-eval read burned: 0/120, ZERO reach on eval

New bin `src/bin/arc2_solve.rs`: Kaggle-format challenges in,
two-attempt submission out; tier-2 (v2) alphabet, frozen enumeration
(len <= 3, 100k nodes), selection R1 per the frozen B4 protocol §2/§7;
attempt_2 = next distinct-output fit; identity fallback; optional local
self-scoring against a solutions file (test outputs read ONLY there).

- **Training self-score: 0.0430** (mean per-task fraction matched by either
  attempt, 1000 tasks); 46 tasks with >= 1 fit — exactly the promoted
  tier-2 reach; 712/1000 budget-censored; ~60 s wall on this machine.
- **Tier-2's one public-eval read (protocol §3.3) spent here: 0.0000 over
  120 tasks, ZERO tasks with any fit, 110/120 budget-censored.** The tier-2
  alphabet has no reach on evaluation-grade tasks. Consequences, recorded
  without spin: (i) expected Kaggle leaderboard score for the current entry
  is ~0%; the entry still satisfies "real, working entry". (ii) The paper's
  reach thesis sharpens: 46/1000 training reach vs 0/120 eval reach — the
  training/eval difficulty gap IS a reach gap for this solver class.
  (iii) B4's per-tier eval reads (one each, §3.3) now carry the paper's
  central open question: do taxonomy-mined ops generalize to eval reach?
  The chances assessment in […] must be
  read with Accuracy ~0 unless they do; the "1-3%" expectation there
  referred to training-grade difficulty and is superseded by this read.

Dataset: ARC-AGI-2 @ f3283f72 (training + evaluation). CPU, $0, no model
call. Next: taxonomy pass over the 954 tier-2-unreached training tasks
(protocol §3.1); tier-3 op freeze; Kaggle notebook packaging (vendored
offline cargo build or static Linux binary via VPS).

## 2026-08-21d: tier 3 run under S2-arc2-tier3-001: C_3 = 0.857 (prediction bar met); eval reach still ZERO

Order of record, each step committed before the next existed: taxonomy pass
(954 unreached tasks labeled; `notes/b4-taxonomy-2026-08-21.md`,
`scripts/b4_taxonomy.py`, `data/b4-taxonomy-001/labels.json`) → tier-3 op
freeze (`manifest/S2-arc2-tier3-001.toml`, ten op semantics) → implementation
(`V3Op` in tpd-arc lib, 18/18 tests) → measurement (`arc2_tier3` bin).

**Component-definition refinement, recorded before any outcome:** tier-3 ops
use the lib's existing 4-connected SAME-COLOR components (tier-2 precedent);
the manifest's "non-bg components" wording is resolved to that reading.

**Measurement (data/S2-arc2-tier3-001/tier3.json):**

- **N_3 = 7** newly reached (bar >= 5 met); v3 reach 52 vs v2 46.
- **C_3 = 6/7 = 0.857** under pre-named R1 (bar >= 0.75 met — B1's
  conversion prediction HOLDS at tier 3). FP_3 = 1.
- Regression guard: **0** tasks lost R1-correctness; **1** task lost reach
  (budget pressure: censoring rose 712 → 850/1000 at the fixed 100k cap).
- Runtime 8:50 wall (both alphabets per task).

**Solver upgraded to v3 per protocol §7.** Training self-score
**0.0480** (was 0.0430). **Tier-3's one public-eval read (§3.3): 0.0000
over 120 — zero fits; the eval submission is byte-identical to tier-2's
(identity fallbacks only).** Two tiers, two eval-zeros: training-mined ops
at this granularity have so far added no evaluation-grade reach. That is
the honest generalization datum the paper reports either way; it is also
the strongest argument yet that the training/eval gap is a reach gap.

Stopping rule check (§6): N_3 >= 5 and C_3 >= 0.75 → tier 4 may proceed
(same protocol, tie seed 992100, taxonomy re-ranked on the remaining
unreached set). Strategic note for the researcher, not a protocol change:
if tier 4 also lands eval-zero, the paper's B4 section should foreground
the training-reach/eval-reach divergence rather than the raw score.

CPU only, $0, no model call.

## 2026-08-21e: tier 4 under S2-arc2-tier4-001 (A1 budget): C_4 = 2/2 = 1.0; N_4 = 2 (first miss of the N >= 5 bar); eval reach zero AND now exhaustively so

Order held: A1 amendment → taxonomy re-rank (948 v3-unreached; cat5 240,
cat3 91 — tier-3's recolor ops converted none of cat 3, recorded in the
manifest) → op freeze (scale_2x/3x, rays_from_singletons, diag_extend,
connect_diag, halo, outline_box, remove_singletons, pane_or, pane_xor) →
implementation (V4Op, 28/28 tests) → measurement.

**Measurement (data/S2-arc2-tier4-001/tier4.json), all at 1M nodes:**

- **Censoring 0/1000 for BOTH alphabets** — 1M makes len <= 3 enumeration
  EXHAUSTIVE; every reach number below is exact, not a budget lower bound.
- Budget effect isolated: v3 reach 52@100k → **56@1M** (+4 from budget
  alone). Op effect: v4 reach **58** → **N_4 = 2** (BELOW the >= 5 bar;
  first miss; per §6(b) one more sub-5 tier stops B4).
- **C_4 = 2/2 = 1.0**, FP_4 = 0; zero reach regressions, zero R1
  regressions. Cumulative tiers 3+4: 9 newly reached, 8 converted (0.889).
- Solver upgraded to v4@1M per §7 + A1. Training self-score **0.0530**
  (0.0430 → 0.0480 → 0.0530 across tiers).
- **Tier-4's one public-eval read: 0.0000 over 120, submission
  byte-identical to tiers 2 and 3 — and with zero censoring this is now an
  EXHAUSTIVE zero: no program in the entire v4 len <= 3 space fits any
  public eval task.**

**Reading, recorded for the paper:** training reach grows tier over tier
with near-perfect conversion, exactly as B1 predicts — while eval reach is
exactly zero, measured exhaustively. At this op granularity and depth, the
training/eval gap is categorical, not gradual: eval tasks are outside the
reachable space, not merely expensive to reach. The next axis is therefore
NOT more single-mechanism closures (diminishing: +7, +2) but depth
(len >= 4, exponential cost) and/or compositional/parameterized op classes
— a protocol-level design decision reserved for the researcher, and the
natural question for B5's triangulated op-proposal arm (go/no-go
2026-10-01).

CPU only, $0, no model call.

## 2026-08-21f: A2 + A3 recorded; depth-4 probe launched; B5 design opened

Researcher authorized: (1) **A2** — depth-4 eval probe (attribution:
depth vs vocabulary), manifest `S2-arc2-depth4-001` + instrument committed
before launch; run in progress on the 120 public eval tasks, v4 alphabet,
len <= 4, 1e8 nodes (exhaustive), the one extended eval read A2 allows.
(2) **A3** — B5 pulled forward to run in PARALLEL, calendar gate replaced;
substance unchanged (prereg before any run) plus a packaging-priority
guard: B5 pauses if the first real Kaggle submission has not happened by
2026-09-15. (3) Sequence of record: depth probe → packaging (parallel) →
tier 5 (closure run) → drafting; B5 parallel under the guard.

`notes/b5-design-2026-08-21.md` opened: T (triangulated ops from a frozen
candidate universe U) vs R (random-from-U, the provenance control
[…]) vs X (taxonomy ops, exploratory
context); extensional only; M2-competence fallback and kill conditions
pre-recorded. All decisional details marked TBD-PREREG; the prereg
manifest comes before any B5 run.

CPU only, $0, no model call.

## 2026-08-22: port VERIFIED on VPS; depth-4 probe died overnight, relaunched with progress logging

**Port verification (Kaggle packaging): PASS.** The x86_64-musl
`arc2_solve` binary, run on a Linux VPS against the pinned eval set
(ARC-AGI-2 @ f3283f72, 120 tasks, v4 @1M, R1, niced 4 threads),
produced `eval_sub.json` with sha256
`69a1cbf50191c27b36e749abf142a25bb55c718f7cc86bb138856502b9b6f67c` —
byte-identical to the Mac-native binary's submission. Fits: 0/120,
budget-censored: 0, matching the tier-4 exhaustive eval-zero. The
Kaggle-bound binary is behaviorally verified; packaging can proceed to
dataset upload + notebook once the researcher's Kaggle login exists.

**Depth-4 probe (S2-arc2-depth4-001): first launch died overnight**
without writing output (host sleep/restart killed the background
process; `data/S2-arc2-depth4-001/` empty, no partial artifact —
write-once discipline intact). Relaunched 2026-08-22 morning under
`caffeinate -i` after re-cloning the dataset at the pinned commit
(f3283f72 re-verified) and adding per-task stderr progress logging to
`arc2_depth4` (instrument OBSERVABILITY only: an atomic counter and an
eprintln per completed task; no measurement parameter changed — max_len,
budget, cap, alphabet, dataset, seeds all as the manifest pins them).
Manifest untouched. Run in progress.

CPU only, $0, no model call.

## 2026-08-22b: instrument optimization, byte-verified; depth-4 probe relaunched on the fast binary

The relaunched probe projected 15-40h (8/120 tasks at 3h15m). Profiling
attributed the cost to op-internal allocation hot spots, not hardware:
per-cell HashMaps in `symmetrize` (~900 per application), per-call
HashMaps in `most_common_color` / `SwapTopTwoColors` / `v1_alphabet`,
10x full-grid rescans in `recolor_size_freq`, and a nested visited
buffer in `components`. All five were rewritten as fixed [usize; 10]
arrays / single-pass counts / a flat visited vector — PERFORMANCE-ONLY
changes with the tie-break predicates reproduced exactly (the
symmetrize (best, tie) outcome depends only on the count multiset, not
HashMap iteration order; equivalence argued in code comments).

**Verification:** all 28 unit tests pass; old vs new `arc2_fpcurve2`
on the full 1000-task training set at the frozen 100k budget produce
BYTE-IDENTICAL output, sha256
0b208322f9349f730c235d2a8bb8041ccd28635702fc6650a35d0a855f25dd2f
(old 6:17, new 5:08 — small there because the v2/len<=3/small-grid
regime rarely hits the fixed hot spots; the v4/30x30 depth-4 regime
hits them constantly).

Old probe processes killed (no output ever written; write-once
intact); probe relaunched on the optimized binary under the SAME
manifest S2-arc2-depth4-001 (no measurement parameter changed).
Projected completion ~3-5h. mimalloc was added then removed (not
needed; keeps the musl/Kaggle build surface minimal). The flat-grid
Grid-representation rewrite was CONSIDERED and DEFERRED: remaining
cost is dominated by op-internal work, and the targeted fixes met the
researcher's ETA criterion without touching the frozen op semantics'
representation.

CPU only, $0, no model call.

## 2026-08-22e: flat-grid rewrite VERIFIED and swapped; probe resumable

**Flat-grid rewrite (researcher-authorized ETA criterion met).** `Grid`
is now a flat row-major `{h, w, c}` buffer; all ~50 ops re-derived;
degenerate-grid equality deliberately matches the old Vec<Vec<u8>>
semantics (zero-row grids canonicalise to (0,0); zero-width grids keep
row count). VERIFIED: 29 unit tests pass; full-training fpcurve2
byte-compare reproduces sha256 0b208322... exactly (third binary in a
row). Deterministic ETA computed as the researcher requested: total
work 4.491e9 nodes (exact, alphabets 53-80 ops); measured 980 CPU-sec
for dbff022c (4.148e7 nodes, 0.92% share) => ~29.5 CPU-hours => 3.5-4h
wall. Old run killed at 15/120; flat probe relaunched under caffeinate.

**Probe is now incremental/resumable** (researcher-directed): per-task
append-only journal (depth4.json.journal.jsonl), flushed per
completion; restart skips journaled tasks; final artifact assembled
write-once in deterministic task order, identical regardless of
interruption history.

[…]

CPU only, $0 paid inference.

## 2026-08-22f: FIRST ARC-AGI-2 SUBMISSION MADE (eligibility milestone; guard satisfied 3+ weeks early)

Kernel tpd-arc2-entry (old binary, sha 73372d90 — printed and verified
in the Kaggle log) completed its save-run inside the cap: 240
placeholder test tasks, 15 with >=1 fit, zero budget-censoring,
submission.json formed (sha aa0aba89...). Submitted to
arc-prize-2026-arc-agi-2 as submission 55694455 (PENDING; the
competition rerun against the hidden set now executes). This is the
eligibility anchor: paper-track linkage requires a real,
working entry — score irrelevant to eligibility. The A3
packaging-priority guard (first submission by 2026-09-15) is
satisfied on 2026-08-22. tpd-arc2-entry-fast (v2 binary) still
running; on completion it becomes the spare — submittable tomorrow
if the rerun errors, otherwise retired.

Depth-4 probe note: first live use of the resume journal succeeded
(8 tasks resumed after a battery-throttle kill+relaunch at nice 0).

CPU only, $0 paid inference.

## 2026-08-22h: RESEARCHER DECISION: B5 committed to the Nov 8 window

[…]

**B5 committed to the paper window** ("5 won't miss the window"):
B5's results land inside the Nov 8 paper, not a standalone.
Consequence: B5 prereg must be signed and runs started EARLY
September at the latest […].

CPU only, $0.

## 2026-08-23e: B5 preregistration drafted (F1); awaits researcher sign-off

notes/b5-prereg-2026-08-23.md fills every TBD-PREREG blank of the
design note: U enumerated (40 closures, six mechanism families,
none in v4, none eval-derived), generator seeded (993850), M1
load-bearing bar 0.15, M2 competence bar 0.50 + correlation-matcher
evidence bar p < 0.05/40 with spike-in gate (seed 993900), T = top
10 by frozen rule, R seeded 993800, primary outcome = newly solved
training tasks T vs R with all three readings pre-filed, one eval
read only for arms with >= 1 training solve, two-stage freeze (F2
pins implementation hashes before any medium trains; target
2026-08-27). No B5 code may run before F1 signature + F2 pin.
Kaggle status noted: the sanitized v3 submission (55709705, from the
tpd-arc2-entry-fast kernel) is the live pending entry.

## 2026-08-23e: v3 submission SCORED: 0.00 (real score, not null); bug fix CONFIRMED; eligibility fully banked; the pre-registered outcome

Submission 55709705 (sanitized v3 binary 530ca258) completed the
hidden-set rerun and received a REAL public score: 0.00. Two
conclusions, both pre-registered before the outcome:

1. **The null-score mechanism is confirmed.** Yesterday's submission
   (identical solver logic, unsanitized) got NULL; today's sanitized
   build got a number. The scorer-invalid-attempt-grids diagnosis
   (DECISIONS 2026-08-23d, an entry not included in this excerpt) is
   confirmed by intervention: sanitize -> score materializes. The fix
   is real and stays in the entry.
2. **0.00 is the calibrated expectation, on record since 2026-08-22:**
   "most likely 0.00... zero is evidence." The local eval census
   (tier-4: zero fits on 120 evaluation tasks, exhaustive at len<=3
   @1M) predicted exactly this: the hidden set is evaluation-grade,
   and the reach-limited floor scores zero there. This is the paper's
   B1 thesis measured on the hidden set — the residual ARC-AGI-2
   problem is reach, and a len<=3 v4-alphabet enumerator has no reach
   into evaluation-grade tasks. The depth-4 probe (91/120 at zero
   fits, resumed) is testing whether depth 4 changes that answer.

Paper-track consequence: the linked entry is now a real, working,
SCORED submission — eligibility requires nothing more (score
explicitly irrelevant to eligibility per the rules). The Accuracy
rubric category reads 0.00 honestly; the paper's Accuracy story is
the B4 ladder + conversion law, not this floor.

$0 (Kaggle compute).

## 2026-08-23f: B5 prereg DRAFTED (all TBD-PREREG blanks filled); awaits researcher signature

notes/b5-prereg-2026-08-23.md: U frozen at 36 enumerated closures
(none in v0-v4, none eval-derived); generator seeded (993800, 60
families, 48/12 split); M1 bar = top-20 by minimal-solution usage;
M2 = toy neural machinery […] with a
15-point-over-copy competence bar (2026-09-30 fallback fork); M2
correspondence = a correlation matcher […] REUSED at a new operating point with its own spike-in gate;
T = top-10 survivors by rank product; R = seeded uniform draw from U,
overlap allowed (conservative); primary statistic DeltaSolved with
pre-filed readings including the honest negative and the
uninformative-universe case; eval untouched by the decision. Drafted
BEFORE the depth-4 probe's final outcome was observed (114/120 at the
time) — B5's design is provably independent of it. Runs start only on
signature.

## 2026-08-23g: PROCESS ERROR caught and reverted: agent overwrote the standing B5 prereg F1 draft with a re-derived duplicate

The agent drafted a fresh B5 prereg without checking that
notes/b5-prereg-2026-08-23.md already existed (F1 draft, U=40,
two-stage F1/F2 freeze, commit 181a54c, awaiting signature). The
overwrite (7cc7f06) is REVERTED; the F1 draft stands as the document
of record. Neither version was signed, so no frozen document was
altered — but the class of error (silent replacement of a standing
prereg) is the exact failure this repo's discipline targets, and it
is recorded here rather than smoothed over.

Substantive differences in the discarded duplicate, surfaced for the
researcher to consider AT SIGN-OFF as possible F1 amendments (not
adopted now):
1. The duplicate declared one additional pre-filed reading the F1
   draft lacks: the UNINFORMATIVE-UNIVERSE case (both arms add ~0 ->
   provenance comparison uninformative, distinct from the negative).
2. The duplicate pinned per-arm tie seeds and a probe-battery seed
   explicitly; F1's F2 stage would pin these at implementation anyway.
3. Seed values differ throughout (F1's are authoritative).
Everything else is convergent re-derivation of the same design.

The duplicate remains recoverable at commit 7cc7f06 if wanted; the
working tree now carries F1 verbatim.

## 2026-08-23h: B5 F1 SIGNED; uninformative-universe reading promoted; F2 begins

Researcher signed the B5 prereg F1 and directed one at-signature
amendment (licensed: pre-implementation, discovery unobserved): the
uninformative-universe case is promoted to pre-filed reading 3 with
an explicit boundary (T+R <= 1 new solve -> statement about U, not a
triangulation negative; richer-U rerun = fresh prereg, not a rescue).
F2 (implementation pin) starts now: U's 40 closures + the family
generator, committed with hashes recorded in manifest S2-b5-001
before any generation runs. Per F1, semantic refinements made while
coding are recorded pre-discovery in DECISIONS (follow-up entry).

## 2026-08-23i: B5 F2: U implemented (40 closures) + generator; pinned in S2-b5-001; pre-discovery refinements

src/u_candidates.rs (40 UOps, apply_u, tests) and
src/bin/b5_gen.rs (seeded splitmix64 generator)
committed; sha256 of both recorded in manifest/S2-b5-001.toml (the F2
pin). 39/39 crate tests pass. Coding-time refinements, all recorded
BEFORE any medium trains or discovery output exists (F1's
pre-discovery amendment clause):

R1 multi-object draw order = component discovery order; later wins on
   collision. R2 per-object clipped transform -> that object
   unchanged; single-object ops -> whole-op identity (per F1). R3
   object "has holes" iff >= 1 border-unreachable bg cell is
   4-adjacent to it. R4 Fold outputs include the middle line (dims
   ceil/2); near-half cell wins when both non-bg. R5 RankRecolorAsc
   cycles colors when objects outnumber colors. R6 SwapPositionsBySize
   anchors = bbox top-left sorted (x,y)/(y,x); any clipped placement
   -> whole-op identity. R7 CountColorsBar bar color = majority
   non-bg. R8 bars/squares guard n in 1..=30 else identity. R9
   generator objects placed with no 4-adjacency to other objects
   (guarantees distinct components). R10 ColorShift skip-bg exactly
   per F1. R11 keep/drop family: empty kept set -> identity.

Next (F3 milestones): M1 enumeration driver, M2 training, evidence
bars — each under its own manifest, per the signed prereg.

## 2026-08-23j: B5 family generation complete under S2-b5-001

80/80 families generated (60 discovery + 20 held-out), seed 993850,
corpus sha256 523eb4bb20e4b5365aae31382227291bf2186ebadfbdc419d8e66b
697729a617 (recorded at data/S2-b5-001/corpus_sha256.txt, force-added;
the corpus itself is deterministically regenerable from the pinned
code + seed). Observed: 39/40 ops appear in hidden programs;
LargestSizeSquare sampled zero times (uniform draw over 141 slots) —
a substrate property, not a design defect: U membership, not program
usage, is what T/R draw from. F2 CLOSED. Next: F3 — M1 enumeration
driver over the probe splits, then M2 training, each under its own
manifest per the signed prereg.

## 2026-08-23j: B5 M1 observed: 0/40 pass; the F1 M1 bar is VOID-BY-MISCALIBRATION; successor prereg required

M1 discovery ran under S2-b5-001 (all 60 families fit; minimal-fit
intersections computed; artifact data/S2-b5-001/m1_evidence.json,
sha 9a7abaa5, force-added). ZERO of 40 ops clear the frozen e1 >= 0.15
bar. Post-hoc-independent diagnosis: the bar was unreachable IN
EXPECTATION under F1's own frozen generator — uniform op sampling,
mean program length 1.7, so E[load-bearing families per op] ~= 2.6
vs a bar of 9 (~Poisson tail; expected passes ~= 0.03). This is an
internal inconsistency of the signed prereg, derivable from its text
alone — the same failure class as an earlier refused threshold
proposal ("thresholds with no measured power"), recommitted by the
agent despite the in-repo precedent […].

**Handling (strict):**
1. The signed prereg CANNOT be amended post-observation. Its literal
   consequence is T = survivor floor = empty -> v4+T == v4, both arms
   zero by construction. Reading 3 ("uninformative universe") would
   fire literally but would MISATTRIBUTE the outcome to U; the honest
   classification is **VOID-BY-MISCALIBRATION of the M1 bar** — a
   statement about the prereg, not about U or about triangulation.
   No provenance conclusion of any kind is licensed by this run.
2. Successor prereg (B5-v2, manifests S2-b5-002*) drafted for
   researcher signature: sole change = the M1 evidence rule reverts
   to the PRE-SIGNATURE draft's rank-based form (top-20 by e1) whose
   provenance predates all discovery data (git history), plus a
   mandatory sizing appendix (the check that was skipped).
3. Contamination statement, on the record: before drafting v2 the
   agent observed the per-family stderr tail and the aggregate
   "0/40" — NOT the e1 ranking table, which remains unread until v2
   is signed. The rank-based rule references no observed value.
4. M2 training proceeds regardless (needed identically under v2;
   competence bar and fallback dates unchanged).

$0, CPU.

## 2026-08-23k: B5 v2 SIGNED; e1 table opened under the rank rule

Researcher signed prereg v2. The e1 ranking (committed unread since
2026-08-23j) is now opened and the frozen top-20-by-e1 rule applied.

Result: top-20 locked (data/S2-b5-001/m1_pass_v2.json). Boundary note,
recorded for honesty: ranks 13-22+ tie at e1 = 0.050 (3 families);
the frozen name-ascending tie-break decides the cut (KeepBorderTouching
and KeepObjsWithHoles in; KeepTop2 and SwapPositionsBySizeLR out).
Arbitrary but preregistered and value-free; noted as a limitation in
any writeup. Leader: RemoveObjsWithHoles (7 families, e1 = 0.117) —
consistent with the v1 sizing analysis (expectation ~2.6 families).
M2 stage begins: per-family toy models, competence on the 20 held-out
families, then the correlation-matcher evidence read under its
spike-in gate.

## 2026-08-23l: B5 M2 implementation refinements, frozen BEFORE any competence observation (F2 precedent)

The prereg's M2 clause ("one model per discovery family batch,
~1M params, grid-pair in/out") is implemented as follows, recorded
pre-observation:
- Item encoding: [BOS] grid_tokens(input) [SEP] grid_tokens(output)
  [EOS] (tpd_neural.data conventions, ROW separators); loss =
  next-token CE on the output segment only.
- One TinyTransformer per family (d_model 128, n_layer 4, default
  heads — the toy defaults […], ~1M
  params), trained on the family's 40 m2train items; deterministic
  seed 993804 + family index (discovery families indexed 0-59,
  held-out 100-119).
- Optimizer AdamW lr 3e-3, cosine to 3e-4, 400 epochs, full-batch;
  set_determinism per house rule.
- Exact-match (competence + any readout) = greedy decode of the
  output segment (teacher-free) equals the target exactly, measured
  on the family's 10 m2val items. Competence statistic = mean over
  the 20 HELD-OUT families' models (bar >= 0.50, fallback fork
  otherwise, dates unchanged).
- The 60 discovery models' hidden states feed the evidence read; the
  evidence read's own operating details (pooling, bases, spike-in)
  are v2-frozen already.

## 2026-08-24: B5 M2 COMPETENCE MISS (0.020 vs 0.50): the pre-registered fallback fork fires; researcher decision required

All 80 family models trained (60 discovery + 20 held-out; sharded,
survived one full host power-loss via per-family resumability; torch
2.13 noted vs requirements' 2.3 pin). Held-out mean exact-match =
0.020 (18/20 families at 0.0; max 0.3) vs the frozen bar 0.50 —
a decisive MISS, not a near-miss. Diagnostic from family_000
(2026-08-23): near-zero train loss with immediate width-drift on
val decode — memorization without generalization at 800k params /
40 examples […].

Per the signed prereg, the fork is RESEARCHER-RESERVED:
 (a) second INDEPENDENT symbolic medium (different search basis);
     cross-medium claim explicitly weakened to CROSS-INSTRUMENT;
     T-vs-R stays decisional; ~2-4 days build;
 (b) kill arm T; record the boundary (cross-medium triangulation
     infeasible at $0 toy scale on this substrate); B5's paper
     section becomes the boundary finding + M1 evidence alone.
No work proceeds on either branch until the choice is logged.
Competence artifact: data/S2-b5-002/m2/competence.json (committed).

## 2026-08-24b: fork (a) LOGGED; M1b frozen (addendum); claim class now cross-instrument

Researcher chose (a). M1b design + qualification gate frozen in the
prereg addendum before any M1b execution; value-free w.r.t. the
opened M1 ranking (mechanism + seeds only). Build + calibration next;
discovery read only if the gate passes.

## 2026-08-24c: M1b qualified (10/10, 100% recovery); floor = 17; arms T and R locked pre-measurement

M1b calibration PASSED decisively (10/10 fresh families fitted,
hidden program covered in all — bars 8/10, 80%); discovery fit 56/60.
Cross-instrument floor = top-20(M1) ∩ top-20(M1b) = 17 ops. Frozen
rank-product rule -> T = {RemoveObjsWithHoles, FoldV, SpreadRow,
StampSmallestMirrored, AlignObjsLeft, BgSwapLeast, RotEachObj90,
ColorShiftUp, OutlineObjects, SwapPositionsBySizeTB}. R drawn by
frozen seed 993800 (house splitmix64), first draw accepted; overlap
with T = 3 ops (allowed; dilutes toward null, conservative). All
artifacts committed BEFORE any measurement run. Next: v4+T and v4+R
injection arms under the frozen A1 budget; primary readout
DeltaSolved(T) vs DeltaSolved(R); readings 1-3 as pre-filed.

## 2026-08-24d: b5_inject nonconforming enumeration caught BY THE REGRESSION GUARD; instrument corrected; both arms re-running

First arm-T run reported +5 new solves but 1 regression (c909285e).
Diagnosis before accepting: b5_inject enumerated in prefix-DFS order
(fit checked at every prefix), while the frozen B4 §4 metric — and
every canonical instrument (arc2_fpcurve2, arc2_depth4) — enumerates
SHORTEST-FIRST (per-length passes). R1 = "first fit" is
order-defined, so the instrument was nonconforming and its
regressions/solve counts untrustworthy. The guard fired exactly as
designed and caught it. Handling: T output quarantined
(inject_T.json.NONCONFORMING, uncommitted), R run killed mid-flight,
dfs corrected to the canonical per-length structure (correction
TOWARD the frozen metric — licensed and required), both arms
re-running from scratch. If the guard fires again under conforming
order, the violation is real and the frozen consequence (arm
invalid) applies with no further appeal.

## 2026-08-25: B5 DECISIONAL RESULT: reading 1 fires; triangulated provenance BEATS random at matched budget (cross-instrument scope)

Both corrected arms complete under the canonical B4 enumeration
(conforming instrument, commit ae476cd), zero regressions, zero
censoring, both valid:

  T (triangulated, 10 ops): new_reach 6, NEW_SOLVED 5
  R (random,       10 ops): new_reach 3, NEW_SOLVED 3

Frozen reading 1 (prereg v2, signed 2026-08-23) FIRES: **T > R ->
provenance-positive at this surface.** Counts reported exactly per
the prereg's own rule — no significance claim at these magnitudes.
Scope carried with the sentence: CROSS-INSTRUMENT (two independent
symbolic searchers, fork (a)), one universe instance (40 closures),
one budget (len<=3 @1M), extensional injection only
[…]. The nonconforming first T run
remains quarantined (.NONCONFORMING); its regression was the
instrument's, not T's.

Paper sentence (draft grade): "Ops selected by cross-instrument
agreement solved 5 new evaluation-grade training tasks where an
equal-sized random draw from the same candidate pool solved 3 —
direction as predicted by the provenance hypothesis, magnitude
small and reported exactly."

Eval-read decision (per prereg: one read, post-recording) is
researcher-reserved; not spent here.

$0.

## 2026-08-25c: depth-4 probe parallelized (prefix-parallel, byte-verified); final two tasks relaunched

The two remaining eval tasks (0934a4d8, 981571dc — max grids, max
pairs, max alphabets) exceeded a day of serial compute each across
repeated interruptions. The enumeration was restructured to
prefix-parallel ((op1,op2) units merged in lexicographic order;
semantics identical by construction) and BYTE-VERIFIED against the
preserved serial binary on three fits-bearing training tasks
(0c786b71/19bb5feb/1c786137: per-length counts incl. 681/371/1347
depth-4 fits and R1 identities all exactly equal; VERDICT
BYTE-IDENTICAL). Serial binary preserved locally. Journal (118/120)
untouched; relaunch covers only the missing two.

## 2026-08-29: depth-4 probe: unit-level checkpointing (researcher-directed)

Post-mortem of the stalled relaunch: pmset logs show the Mac in
lid-close sleep for most of the prior run's 7+ h wall clock
(caffeinate -i does not survive lid close), and a leaf-focused cost
measurement (new diagnostic bin depth4_eta, samples random 3-prefix
states then times all depth-4 op applications) puts the two remaining
tasks (0934a4d8, 981571dc) at ~9.3-9.8 ms/leaf x 40.96M leaves =
~110-115 single-core hours EACH — ~3x the other 118 tasks combined,
and far above the earlier chain-average estimate (which diluted leaf
cost with cheap shallow levels). Researcher, unable to guarantee a
full day of lid-open time, directed: kill the in-flight run (57
core-hours forfeited from process memory) and adopt unit-level
checkpointing.

Change to arc2_depth4: each (op1,op2) prefix unit's result (counts,
programs as op indices, nodes, censor flag) is appended to a per-task
side journal on completion and skipped on restart; side journal
deleted once the task row reaches the main journal; progress line
every 320 units. Interruption cost drops from whole-task (days) to
in-flight units (minutes). Semantics unchanged: units are
deterministic and merged by idx, so assembled output is byte-identical
regardless of interruption history.

Verification (3-task fits-bearing training set): (1) uninterrupted
checkpointed binary vs preserved parallel-v1 binary: sha256 8fa141a7…
EQUAL; (2) kill -9 at 45 s (7,865 units on disk, 0 journal rows) then
resume: same sha256 8fa141a7… — VERDICT BYTE-IDENTICAL under
interruption.

Hazard caught by the verification gate and recorded as doctrine: the
workspace's stitch_core/lambdas deps enable serde_json
preserve_order; with resolver v2 feature unification this is baked in
only when building FROM THE WORKSPACE ROOT. A build from inside
the solver crate drops it and flips JSON key order (alphabetical vs
insertion), silently breaking byte-consistency with every registered
artifact. ALL artifact-producing binaries must be built from the
workspace root.

Run relaunched under the checkpointed binary; journal 118/120 intact;
remaining work ~220 core-hours, safe to interrupt at will.

## 2026-08-30: S2-arc2-depth4-001 COMPLETE: A2 adjudicated, gap is op-class

Run complete 03:42 under the checkpointed binary (120/120; final two
tasks 0934a4d8 and 981571dc finished overnight, fits=0
censored=false each). Aggregate: reached_at_len_le4 = 0,
reached_only_at_len4 = 0, r1_solved = 0, budget_censored = 0 across
all 120 public evaluation tasks; every row's fitting_by_len is
[0,0,0,0] (sanity-checked). Output sha256
f8957c89b9148bd1693dadd878bf5a807fc4161e2d595965831a24a1a1019ade.

Amendment A2's pre-filed outcome fires verbatim: "zero fits at depth
4 → the gap is op-class, not depth (B5's motivation sharpens)." The
v4 alphabet reaches ZERO eval tasks even with exhaustive length-4
composition — one level deeper than the tier-4 len<=3 zero — so the
eval frontier is attributable to the op vocabulary, not composition
depth. Per A2, no op selection derives from per-task results; the
aggregate finding is the sole licensed read. This closes the last
open measurement of the experimental campaign; remaining work is
writing.

## 2026-08-30b: housekeeping: b5_m1.rs source provenance gap

- b5_m1.rs source pinned to the repo only now; the binary it built ran
  the registered S2-b5-001/-002 discovery with hashes recorded in the
  manifest at F2, but the source file itself was missed at commit
  time. Recorded as the provenance gap it is; no measurement changes.
- […]

## 2026-08-30c: B5 licensed eval read AUTHORIZED (researcher): arm T, parameters fixed pre-run

Researcher directs the prereg's ONE licensed eval read (arm T qualifies:
5 new training solves ≥ 1) and folding T's ops into the Kaggle entry.
Read parameters, fixed before execution: instrument b5_inject (registered
arm semantics: v4+T alphabet, T appended after v4 ops in floor_T.json
order; len ≤ 3; 1e6 nodes/task; cap 2000; R1), target = the 120 public
evaluation tasks (pinned dataset); baseline = v4 eval reach 0/120
(exhaustive at len ≤ 4 per S2-arc2-depth4-001, so every eval task enters
with base_reached = base_solved = false). One run, write-once to
data/S2-b5-003/eval_T.json. This is the single eval read the prereg
licenses; its result stands regardless of direction. Entry fold-in
proceeds after the read is recorded; the Kaggle leaderboard scores the
HIDDEN set, so the read predicts public-eval reach only.

## 2026-08-30d: B5 eval read RESULT: arm T adds zero eval reach

The one licensed eval read (parameters fixed in 2026-08-30c) completed:
v4+T on the 120 public evaluation tasks at len <= 3, 1e6 nodes/task —
reached 0, new_reach 0, new_solved 0, censored 0. With 90 ops the
len<=3 space (~737k programs/task) sits under budget, so this is an
exhaustive zero, not a truncated search. Reading: T's operations,
provenance-positive on training (+6 reach, +5 solved), add nothing to
evaluation reach. This sharpens the depth-4 op-class finding: the eval
vocabulary gap is not closed by more operations of this kind, however
well-proposed; it is qualitatively elsewhere. Artifact:
data/S2-b5-003/eval_T.json (sha256 56b2dfc0…, write-once).
Entry fold-in proceeds as researcher-directed: arc2_solve_t with arm T
is a strict alphabet superset, byte-verified at empty arm; on the
public eval its submission is predicted identical to v4's (no fits →
identity fallback both). Predicted leaderboard score of the folded
entry remains 0.00; hidden-set reach cannot be measured locally.

## 2026-08-30e: fold-in verification chain complete

arc2_solve_t with arm T on the 120 public evaluation tasks produced a
submission byte-identical to arc2_solve's v4 submission (cmp clean;
120 tasks; both all-identity fallbacks, as the eval read's 0/120
predicted). Chain: empty-arm byte-identity, licensed eval read 0/120
censor-free, folded public-eval submission identical. The T-folded
entry is therefore measurably indistinguishable from the scored v4
entry everywhere we can measure; folding in changes hidden-set
optionality only. Eval pack preserved locally.

## 2026-08-31a: Linux port verification passed; folded entry submitted

Port verification (post-hoc per the 2026-08-30 "submit now" decision):
arc2_solve_t built on x86_64 Linux (musl static-pie, VPS build path,
binary sha 3c1b0530) ran the folded arm on the 120 public evaluation
tasks and produced sub_folded_linux.json with sha256 69a1cbf5…, byte-
identical to the registered Mac-built folded submission. 0 fits, 0
censored, matching the eval read. The port criterion (byte identity)
is met; the Kaggle binary is verified equivalent to the measured one.

Kaggle resubmission: kernel tpd-arc2-entry version 3 (dataset
tpd-arc2-solver v2, binary 3c1b0530) save-run COMPLETE with 0
budget-censored tasks; submitted to arc-prize-2026-arc-agi-2 as
submission 55910824 (2026-08-31 08:02 UTC), status PENDING. Predicted
score 0.00. Note (unregistered observation, no claim): the save-run
placeholder test file contains 240 tasks, on which the folded solver
found fits for 16; the placeholder is not the hidden set and this
number feeds no paper claim.

## 2026-08-31b: ledger adjudications surfaced by drafting (reach-ladder accounting; B5 candidate-pool size)

1. Reach-ladder accounting: tier3.json records N_3_newly_reached 7 AND
regression_reach_lost 1 (a v2-reached task falls out of reach under
v3's capped budget; larger vocabulary, larger branching factor, same
cap). Stage sums must therefore read net: 46 +6 (7 gross, 1 lost) +4
(1M raise) +2 = 58, matching tier4.json v4_reached_at_1M. The writeup
previously listed gross increments that sum to 59 against the 58
endpoint; corrected to "v3 adds 7, net 6 under its budget cap"
(writeup now 1,469 words incl. byline). Citations of the ladder use
net accounting with the tier3.json trace.
2. […]
3. B5 candidate-pool size: the signed prereg of record
(notes/b5-prereg-2026-08-23.md:24) froze U at 40 closures; the "36"
in DECISIONS 2026-08-23f is a stale draft-stage figure superseded by
the signed prereg. 40 is the number of record.

## 2026-10-04: paper-track facts verified on Kaggle; writeup v4.1 corrects the pooled conversion count

Checked on Kaggle today (logged-in session, read-only; nothing changed):

- **Deadlines.** ARC-AGI-2 overview: entry (rules acceptance) and team
  merger 2026-10-26, final submission 2026-11-02, all 23:59 UTC. This
  clears the "ARC-AGI-2 equivalent UNVERIFIED" flag in 2026-08-21.
  Paper track (arc-prize-2026-paper-track): final submission
  2026-11-09 23:59 UTC; no separate entry deadline listed; draft
  Writeups are not judged; ties go to the paper entered first.
- **Paper track state.** Joined; one-member team, matching the
  ARC-AGI-2 team. An unsubmitted draft Writeup exists.
- **Scores.** tpd-arc2-entry v3 (the T-folded entry, 55910824):
  Succeeded, public score 0.00, as predicted (2026-08-31a). The byline's
  "0.00, measured" is now verified. tpd-arc2-entry-fast v2 (55709705):
  0.00, as recorded. tpd-arc2-entry v2 (55694455) now displays
  "Notebook Timeout"; 2026-08-23e recorded that submission as a NULL
  score. Recorded as observed; no claim depends on it. 0 of 2 final
  submissions are selected (Kaggle auto-selects).

**Writeup v4.1** (papers/reach-not-selection-writeup.md), after a red
team and a defense brief, with the load-bearing claims of both
spot-verified against the artifacts:

1. Pooled conversion is **15/18 across the four tests, 13/15 distinct
   tasks**, not 16/18. The v4 table mixed B5's registered "newly
   solved" metric into a conversion column: R converts 2 of its 3 newly
   reached tasks (fcb5c309 fails in both arms), and R's 3 newly reached
   are a subset of T's 6 (data/S2-b5-002/inject_{T,R}.json). R's 3 new
   solves stand under the B5 metric: its third, aabf363d, was reached at
   v4 with 72 fits and no test-correct fit (tier4.json row,
   uncensored), so it is a reach gain to a correct program, not a
   selection change. T > R holds under both metrics.
2. Result 1 restated as an oracle bound: at v4 (uncensored), 58
   reached, 52 first-fit solves, 54 with any test-correct fit; true
   selection failures are 5614dbcf and d9fac9be only (tier4.json).
3. Preregistration wording scoped: v0–v2 exploratory then exactly
   reproduced (2026-08-21b); v3+ under the B4 protocol; B5's second
   searcher specified after the first ranking was opened (2026-08-23k,
   2026-08-24b); T and R share 3 operations.

The 16/18 figure is superseded wherever it appears
[…]. No artifact changes.

## 2026-10-04f: corrections found while writing the full paper

Two errors in earlier records, found by an audit of the full paper
against the result files and source code:

1. **v3 lost one solved task.** Task 72ca375d was solved at v2
   (tier3.json row: v2_r1_solves true) and not reached at v3 under the
   100,000-step budget (v3_reached false, v3_censored true). The tier-3
   summary key regression_r1_broken is empty because the counter only
   checks tasks whose v3 first-fit result is false, and this one is
   null (src/bin/arc2_tier3.rs). The 2026-08-21d entry's "0 tasks lost
   R1-correctness" is therefore wrong. The v3 rerun at 1 million steps
   and v4 both solve the task again (tier4.json row). No later stage,
   and neither the agreement set nor the random set, lost a solved task.
2. **The filter rule (R3) has two screens, not three.** The code
   removes programs that leave every visible grid unchanged and
   programs whose outputs are all single-coloured
   (src/bin/arc2_fpcurve2.rs). The palette-projection screen named in
   notes/arc2-dsl-v1.md was never implemented. Results are unchanged;
   only the description was wrong.

The papers now state both correctly. papers/check_numbers.py recomputes
48 values from the result files and fails the build if the text
disagrees.
