"""Check every number in the papers against the registered artifacts.

Each claim pairs a phrase that must appear in the paper text with the
value computed from a result file. The build fails if a phrase is missing
or a computed value disagrees. Run from the repository root:
    python papers/check_numbers.py papers/reach-not-selection-full.md papers/kaggle-writeup-paste.md
"""
import json, re, sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
J = lambda p: json.load(open(ROOT / p))

census = J("data/S2-arc2-census-001/census.json")
fp1 = J("data/S2-arc2-paper-rerun-001/fpcurve.json")["aggregates_per_k"]
fp2 = J("data/S2-arc2-paper-rerun-001/fpcurve2.json")["aggregates_per_k"]
t3 = J("data/S2-arc2-tier3-001/tier3.json")["summary"]
t4all = J("data/S2-arc2-tier4-001/tier4.json"); t4 = t4all["summary"]
d4 = J("data/S2-arc2-depth4-001/depth4.json")
injT = J("data/S2-b5-002/inject_T.json"); injR = J("data/S2-b5-002/inject_R.json")
evT = J("data/S2-b5-003/eval_T.json")["summary"]
Tops = J("data/S2-b5-002/floor_T.json")["T"]; Rops = J("data/S2-b5-002/arm_R.json")["R"]
tax = J("data/b4-taxonomy-001/labels.json")["counts"]

reached4 = [r for r in t4all["rows"] if r["v4_reached"]]
def newly(rows):
    nr = [r for r in rows if r["reached"] and not r["base_reached"]]
    return len(nr), sum(r["r1_solves"] for r in nr), {r["id"] for r in nr}
nT, sT, idsT = newly(injT["rows"]); nR, sR, idsR = newly(injR["rows"])
opsd4 = Counter(r["n_ops"] for r in d4["rows"])

# (phrase that must appear, value from artifacts, expected value in text)
checks = [
    ("v0 reached", census["summary"]["n_with_v0_fit_le3"], 25),
    ("v0 first fits solve", census["summary"]["n_first_fit_solves_test"], 25),
    ("v0 op count", census["params"]["n_ops"], 12),
    ("v1 k=all tasks", fp1[2]["n_tasks_with_fit"], 34),
    ("v2 k=all tasks", fp2[2]["n_tasks_with_fit"], 46),
    ("v2 k=all R1", round(fp2[2]["acc"]["r1"], 3), 0.913),
    ("v2 k=all rules equal", len({round(v, 6) for v in fp2[2]["acc"].values()}), 1),
    ("v1 k=all R4", round(fp1[2]["acc"]["r4"], 3), 1.0),
    ("v1 k=all R1", round(fp1[2]["acc"]["r1"], 3), 0.971),
    ("v3 reached @100k", t3["v3_reached"], 52),
    ("v3 newly reached", t3["N_3_newly_reached"], 7),
    ("v3 newly solved", t3["C_3_numerator"], 6),
    ("v3 lost reach", t3["regression_reach_lost"], 1),
    ("v3 reached @1M", t4["v3_reached_at_1M"], 56),
    ("v4 reached", t4["v4_reached_at_1M"], 58),
    ("v4 newly reached", t4["N_4_newly_reached"], 2),
    ("v4 newly solved", t4["C_4_numerator"], 2),
    ("v4 censored", t4["v4_budget_censored_at_1M"], 0),
    ("v4 first-fit solves", sum(r["v4_r1_solves"] for r in reached4), 52),
    ("v4 any-fit solves", sum(r["v4_any_solves"] for r in reached4), 54),
    ("T newly reached", nT, 6), ("T newly reached solved", sT, 5),
    ("R newly reached", nR, 3), ("R newly reached solved", sR, 2),
    ("R subset of T", idsR <= idsT, True),
    ("T new solves", injT["summary"]["new_solved"], 5),
    ("R new solves", injR["summary"]["new_solved"], 3),
    ("T/R shared ops", len(set(Tops) & set(Rops)), 3),
    ("pooled reached", 7 + 2 + nT + nR, 18), ("pooled solved", 6 + 2 + sT + sR, 15),
    ("distinct reached", 7 + 2 + len(idsT | idsR), 15), ("distinct solved", 6 + 2 + sT, 13),
    ("eval depth4 reached", d4["summary"]["reached_at_len_le4"], 0),
    ("eval depth4 censored", d4["summary"]["budget_censored"], 0),
    ("eval T reached", evT["reached"], 0), ("eval T censored", evT["censored"], 0),
    ("v4 ops max", max(opsd4), 80), ("v4 ops min", min(opsd4), 53),
    ("tasks with 80 ops", opsd4[80], 101),
    ("programs at 80 ops len<=4", 80 + 80**2 + 80**3 + 80**4, 41478480),
    ("unreached at v2", sum(tax.values()), 954),
    ("category lines", tax["5"], 242), ("category recolour", tax["3"], 91),
    ("category crop", tax["7"], 40), ("category tiling", tax["1"], 37),
    ("category repair", tax["9"], 36), ("category panes", tax["8"], 31),
    ("category residual", tax["10"], 445),
]

# Numbers whose wording must appear verbatim in the full paper.
phrases_full = ["25 tasks at v0, 46 at v2, 52 at v3, 56 at v3 with the larger budget, and 58 at v4",
                "954 training tasks", "445 tasks", "up to 50", "up to 60", "up to 70", "up to 80",
                "reach 15 and solve 13", "0.913", "0.02", "0.50", "41.5 million", "72 fitting programs",
                "first fitting program solves 52", "only 2 have any fitting program", "100 million steps"]
phrases_short = ["reach 15 and solve 13", "0 of 120", "first fitting program solves 52", "41.5 million", "up to 80 operations per task", "from 25 to 58"]

bad = [(n, got, exp) for n, got, exp in checks if got != exp]
for n, got, exp in bad:
    print(f"MISMATCH {n}: artifact={got} expected={exp}")
for path in sys.argv[1:]:
    text = Path(path).read_text()
    want = phrases_full if "full" in path else phrases_short
    for p in want:
        if p not in text:
            print(f"MISSING in {Path(path).name}: {p!r}"); bad.append(p)
    for stale in ["up to 52", "up to 62", "up to 72", "16 of 18", "100,000 steps per task for v0"]:
        if stale in text:
            print(f"STALE in {Path(path).name}: {stale!r}"); bad.append(stale)
print(f"{len(checks)} artifact checks; {'FAIL' if bad else 'all pass'}")
sys.exit(1 if bad else 0)
