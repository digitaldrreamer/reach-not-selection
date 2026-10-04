"""Figures for "Reach, Not Selection". Every number is read from a
registered artifact at render time; nothing an artifact can supply is
hardcoded. Grid colors use the standard ARC palette.

Run from the repository root:
    ARC_AGI_2_DIR=/path/to/ARC-AGI-2/data python papers/media/make_figures.py
ARC_AGI_2_DIR is only needed for Figure 1 (one public training task).
"""

import json
import os
from pathlib import Path

import numpy as np
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "papers" / "media"
ARC_DIR = Path(os.environ.get("ARC_AGI_2_DIR", ROOT / "arc-agi-2" / "data"))

INK, MUTED, RULE = "#1b1f24", "#5f6670", "#d0d4da"
TRAIN, EVAL = "#3b6ea8", "#b5532c"

ARC = ["#000000", "#0074D9", "#FF4136", "#2ECC40", "#FFDC00",
       "#AAAAAA", "#F012BE", "#FF851B", "#7FDBFF", "#870C25"]

plt.rcParams.update({
    "font.family": "DejaVu Sans", "font.size": 11, "text.color": INK,
    "axes.edgecolor": INK, "axes.labelcolor": INK, "axes.linewidth": 0.8,
    "xtick.color": INK, "ytick.color": INK, "svg.fonttype": "none",
})

# ---- numbers from registered artifacts -------------------------------
def load(p):
    return json.load(open(ROOT / p))

census = load("data/S2-arc2-census-001/census.json")
tier3 = load("data/S2-arc2-tier3-001/tier3.json")["summary"]
tier4_all = load("data/S2-arc2-tier4-001/tier4.json")
tier4 = tier4_all["summary"]
depth4 = load("data/S2-arc2-depth4-001/depth4.json")["summary"]
evalT = load("data/S2-b5-003/eval_T.json")["summary"]
rowsT = load("data/S2-b5-002/inject_T.json")["rows"]
rowsR = load("data/S2-b5-002/inject_R.json")["rows"]

v0 = census["summary"]["n_with_v0_fit_le3"]
v2 = tier3["v2_reached"]
v3 = tier3["v3_reached"]
v3_1m = tier4["v3_reached_at_1M"]
v4 = tier4["v4_reached_at_1M"]
assert depth4["reached_at_len_le4"] == 0 and depth4["budget_censored"] == 0
assert evalT["reached"] == 0 and evalT["censored"] == 0

# Selection bound at v4 (no reached task censored).
reached4 = [r for r in tier4_all["rows"] if r["v4_reached"]]
assert not any(r["v4_censored"] for r in reached4)
first_fit = sum(r["v4_r1_solves"] for r in reached4)
any_fit = sum(r["v4_any_solves"] for r in reached4)
oracle_gain = any_fit - first_fit

# Conversion of newly reached tasks, per test.
def newly(rows):
    nr = [r for r in rows if r["reached"] and not r["base_reached"]]
    return len(nr), sum(r["r1_solves"] for r in nr)

tests = [(tier3["N_3_newly_reached"], tier3["C_3_numerator"]),
         (tier4["N_4_newly_reached"], tier4["C_4_numerator"]),
         newly(rowsT), newly(rowsR)]
pooled_reached = sum(t[0] for t in tests)
pooled_solved = sum(t[1] for t in tests)


def clean(ax):
    for s in ("top", "right"):
        ax.spines[s].set_visible(False)
    ax.tick_params(length=3, width=0.8)


# ---- Figure 2: reach on training vs evaluation -----------------------
fig, (a, b) = plt.subplots(
    1, 2, figsize=(10, 4.6), dpi=200,
    gridspec_kw={"width_ratios": [5, 3.4], "wspace": 0.32})

labels = ["v0", "v2", "v3", "v3\n(10× budget)", "v4"]
vals = [v0, v2, v3, v3_1m, v4]
x = np.arange(len(vals))
a.bar(x, vals, width=0.6, color=TRAIN)
for xi, v in zip(x, vals):
    a.text(xi, v + 1.2, str(v), ha="center", va="bottom", fontsize=11)
a.set_xticks(x, labels)
a.set_ylim(0, 70)
a.set_ylabel("Tasks reached (of 1,000)")
a.set_title("(a) Public training set", loc="left", fontsize=12)
clean(a)

elabels = ["v4\n≤ 3 ops", "v4\n≤ 4 ops", "v4 + agreement\nset, ≤ 3 ops"]
ex = np.arange(len(elabels))
b.bar(ex, [0, 0, 0], width=0.6, color=EVAL)
for xi in ex:
    b.text(xi, 0.4, "0", ha="center", va="bottom", fontsize=11, color=EVAL,
           fontweight="bold")
b.set_xticks(ex, elabels)
b.set_ylim(0, 8.4)
b.set_ylabel("Tasks reached (of 120)")
b.set_title("(b) Public evaluation set", loc="left", fontsize=12)
b.text(1, 4.6, "Every search finished.\nNo program fits.",
       ha="center", fontsize=10, color=MUTED)
clean(b)

fig.savefig(OUT / "fig2_reach_ladder.png", facecolor="white",
            bbox_inches="tight", pad_inches=0.15)
plt.close(fig)

# ---- Figure 1: one task and the first program that fits it ----------
task = json.load(open(ARC_DIR / "training" / "3af2c5a8.json"))
prog = next(r for r in census["rows"] if r["id"] == "3af2c5a8")["first_fit"]
assert prog == ["MirrorConcatH", "MirrorConcatV"]

def mirror_right(g): return np.hstack([g, g[:, ::-1]])
def mirror_down(g): return np.vstack([g, g[::-1, :]])

def run(g):
    steps = [np.asarray(g)]
    for f in (mirror_right, mirror_down):
        steps.append(f(steps[-1]))
    return steps

for p in task["train"]:
    assert np.array_equal(run(p["input"])[-1], np.array(p["output"]))
test = task["test"][0]
assert np.array_equal(run(test["input"])[-1], np.array(test["output"]))

MAXW, MAXH = 8, 6

def draw_grid(ax, g):
    h, w = g.shape
    x0, y0 = (MAXW - w) / 2, (MAXH - h) / 2
    for (r, c), v in np.ndenumerate(g):
        ax.add_patch(plt.Rectangle((x0 + c, y0 + h - 1 - r), 1, 1,
                     fc=ARC[int(v)], ec="#8a8f96", lw=0.5))
    ax.add_patch(plt.Rectangle((x0, y0), w, h, fill=False, ec=INK, lw=0.8))
    ax.set_xlim(-0.3, MAXW + 0.3)
    ax.set_ylim(-0.3, MAXH + 0.3)
    ax.set_aspect("equal")
    ax.axis("off")

fig = plt.figure(figsize=(10, 5.4), dpi=200)
gs = fig.add_gridspec(2, 3, left=0.16, right=0.98, top=0.86, bottom=0.03,
                      wspace=0.12, hspace=0.12)
cols = ["Input", "Step 1: mirror right", "Step 2: mirror down\n(= expected output)"]
rows = [("Training\nexample", task["train"][0]["input"]),
        ("Test\n(output held back)", test["input"])]
for ri, (rlab, g) in enumerate(rows):
    steps = run(g)
    for ci in range(3):
        ax = fig.add_subplot(gs[ri, ci])
        draw_grid(ax, steps[ci])
        if ri == 0:
            ax.set_title(cols[ci], fontsize=11.5, pad=4)
    pos = ax.get_position()
    fig.text(0.02, (pos.y0 + pos.y1) / 2, rlab, fontsize=11.5, va="center")
fig.savefig(OUT / "fig1_task_walkthrough.png", facecolor="white")
plt.close(fig)

# ---- Cover -----------------------------------------------------------
fig = plt.figure(figsize=(12, 6.75), dpi=200)
fig.text(0.06, 0.74, "Reach, Not Selection", fontsize=38,
         fontweight="bold", color=INK)
fig.text(0.06, 0.65, "Measuring what is missing on ARC-AGI-2",
         fontsize=18, color=MUTED)
fig.add_artist(plt.Line2D([0.06, 0.94], [0.56, 0.56], color=RULE, lw=1))
stats = [
    ("0 of 120", "evaluation tasks reachable,\nsearching every program\nup to 4 operations", EVAL),
    (f"{pooled_solved} of {pooled_reached}", "newly reachable training\ntasks became solved", TRAIN),
    (f"{oracle_gain} of {len(reached4)}", "reached tasks a perfect selection\nrule could add, at most", INK),
]
for i, (big, small, col) in enumerate(stats):
    xs = 0.06 + i * 0.31
    fig.text(xs, 0.36, big, fontsize=30, fontweight="bold", color=col)
    fig.text(xs, 0.31, small, fontsize=13, color=MUTED, va="top",
             linespacing=1.35)
fig.savefig(OUT / "cover.png", facecolor="white")
plt.close(fig)

# ---- Card and thumbnail (2:1 card; the square thumbnail is its centre) ---
fig = plt.figure(figsize=(11.2, 5.6), dpi=100)
fig.text(0.5, 0.62, "Reach,\nNot Selection", fontsize=40, fontweight="bold",
         color=INK, ha="center", va="center", linespacing=1.1)
fig.text(0.5, 0.33, "What is missing\non ARC-AGI-2", fontsize=19,
         color=MUTED, ha="center", va="center", linespacing=1.25)
fig.add_artist(plt.Line2D([0.36, 0.64], [0.45, 0.45], color=RULE, lw=1.2))
fig.savefig(OUT / "card.png", facecolor="white")
plt.close(fig)

# ---- Extra figures for the full paper --------------------------------
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch

def box(ax, x, y, w, h, text, fc="white", ec=INK, size=11, weight="normal"):
    ax.add_patch(FancyBboxPatch((x - w / 2, y - h / 2), w, h,
                 boxstyle="round,pad=0.02,rounding_size=0.06",
                 fc=fc, ec=ec, lw=1.0))
    ax.text(x, y, text, ha="center", va="center", fontsize=size,
            fontweight=weight, color=INK, linespacing=1.3)

def arrow(ax, x0, y0, x1, y1, label=None, lx=0, ly=0):
    ax.add_patch(FancyArrowPatch((x0, y0), (x1, y1), arrowstyle="-|>",
                 mutation_scale=12, lw=1.0, color=INK))
    if label:
        ax.text((x0 + x1) / 2 + lx, (y0 + y1) / 2 + ly, label, fontsize=10,
                color=MUTED, ha="center", va="center")

# Flow: how a task ends up solved, a reach failure or a selection failure.
fig, ax = plt.subplots(figsize=(10, 3.6), dpi=200)
ax.set_xlim(0, 10); ax.set_ylim(0, 3.6); ax.axis("off")
box(ax, 1.15, 2.5, 2.0, 1.0, "Search every\nprogram within\nthe budget")
box(ax, 3.9, 2.5, 2.2, 1.0, "Does any program\nfit all example\npairs?")
box(ax, 6.75, 2.5, 2.2, 1.0, "Is the chosen\nprogram right on\nthe test input?")
box(ax, 9.05, 2.5, 1.4, 0.8, "Solved", fc="#e1eaf5", ec=TRAIN, weight="bold")
box(ax, 3.9, 0.55, 2.2, 0.8, "Reach failure", fc="#f6e3dc", ec=EVAL, weight="bold")
box(ax, 6.75, 0.55, 2.2, 0.8, "Selection failure", fc="#f6e3dc", ec=EVAL, weight="bold")
arrow(ax, 2.15, 2.5, 2.8, 2.5)
arrow(ax, 5.0, 2.5, 5.65, 2.5, "yes", ly=0.18)
arrow(ax, 7.85, 2.5, 8.35, 2.5, "yes", ly=0.18)
arrow(ax, 3.9, 2.0, 3.9, 0.95, "no", lx=0.25)
arrow(ax, 6.75, 2.0, 6.75, 0.95, "no", lx=0.25)
fig.savefig(OUT / "fig_flow.png", facecolor="white", bbox_inches="tight", pad_inches=0.1)
plt.close(fig)

# What happens to the reached tasks at v4 (all searches complete).
n_first = first_fit
n_fixable = oracle_gain
n_nocorrect = len(reached4) - any_fit
parts = [(n_first, "solved by the first fitting program", TRAIN),
         (n_fixable, "a correct fitting program existed, but another was chosen", "#d9a441"),
         (n_nocorrect, "no fitting program is right on the test", EVAL)]
fig, ax = plt.subplots(figsize=(10, 2.4), dpi=200)
left = 0
for n, lab, col in parts:
    ax.barh(0, n, left=left, color=col, height=0.55, edgecolor="white", lw=1)
    left += n
ax.set_xlim(0, len(reached4)); ax.set_ylim(-0.4, 0.4); ax.axis("off")
ax.set_title(f"The {len(reached4)} training tasks reached at v4 (every search complete)", loc="left", fontsize=12)
from matplotlib.patches import Patch
ax.legend(handles=[Patch(color=c, label=f"{n}  {lab}") for n, lab, c in parts],
          loc="upper left", bbox_to_anchor=(0, -0.05), frameon=False, fontsize=10.5, ncol=1)
fig.savefig(OUT / "fig_v4_breakdown.png", facecolor="white", bbox_inches="tight", pad_inches=0.15)
plt.close(fig)

# Selection rules by number of example pairs used (v1 and v2).
f1 = load("data/S2-arc2-paper-rerun-001/fpcurve.json")["aggregates_per_k"]
f2 = load("data/S2-arc2-paper-rerun-001/fpcurve2.json")["aggregates_per_k"]
fig, axes = plt.subplots(1, 2, figsize=(10, 3.8), dpi=200, sharey=True)
names = {"r1": "first fit", "r2": "shortest", "r3": "filter", "r4": "vote"}
styles = {"r1": ("-", "o"), "r2": ("--", "s"), "r3": (":", "^"), "r4": ("-.", "D")}
for ax, aggs, title in [(axes[0], f1, "(a) v1"), (axes[1], f2, "(b) v2")]:
    xs = range(len(aggs))
    for r in ["r1", "r2", "r3", "r4"]:
        ls, mk = styles[r]
        ax.plot(xs, [a["acc"][r] for a in aggs], ls, marker=mk, color=TRAIN if r == "r1" else MUTED,
                lw=1.2, ms=5, label=names[r])
    ax.set_xticks(list(xs), [f"first 1\n({aggs[0]['n_tasks_with_fit']} tasks)",
                             f"first 2\n({aggs[1]['n_tasks_with_fit']} tasks)",
                             f"all\n({aggs[2]['n_tasks_with_fit']} tasks)"])
    ax.set_title(title, loc="left", fontsize=12); clean(ax); ax.set_ylim(0.3, 1.05)
axes[0].set_ylabel("Share solved by the chosen program")
axes[1].legend(frameon=False, fontsize=9.5, loc="lower right")
fig.supxlabel("Number of example pairs a program must fit", fontsize=11, y=-0.08)
fig.savefig(OUT / "fig_selection_rules.png", facecolor="white", bbox_inches="tight", pad_inches=0.15)
plt.close(fig)

# New reach and new solves per test.
labels_t = ["v3", "v4", "agreement set\nadded to v4", "random set\nadded to v4"]
fig, ax = plt.subplots(figsize=(8, 3.8), dpi=200)
x = np.arange(len(tests)); w = 0.36
ax.bar(x - w / 2, [t[0] for t in tests], w, color=MUTED, label="newly reached")
ax.bar(x + w / 2, [t[1] for t in tests], w, color=TRAIN, label="of these, solved")
for i, (r, sv) in enumerate(tests):
    ax.text(i - w / 2, r + 0.12, str(r), ha="center", fontsize=11)
    ax.text(i + w / 2, sv + 0.12, str(sv), ha="center", fontsize=11)
ax.set_xticks(x, labels_t); ax.set_ylabel("Training tasks"); ax.set_ylim(0, 8.2)
ax.legend(frameon=False, fontsize=10, loc="upper right"); clean(ax)
ax.set_title(f"Newly reached tasks and how many were solved ({pooled_solved} of {pooled_reached} in total)", loc="left", fontsize=12)
fig.savefig(OUT / "fig_conversion.png", facecolor="white", bbox_inches="tight", pad_inches=0.15)
plt.close(fig)

# Categories of the tasks v2 could not reach.
lab = load("data/b4-taxonomy-001/labels.json")["counts"]
catnames = {"1": "tiling or scaling", "2": "symmetry completion", "3": "recolour objects by a property",
            "4": "move objects", "5": "draw lines or paths", "6": "build output from counts",
            "7": "crop to a marked region", "8": "combine sub-grids", "9": "repair a pattern",
            "10": "none of the above (no operations taken from here)"}
items = sorted(((int(v), catnames[str(k)]) for k, v in lab.items()), reverse=True)
fig, ax = plt.subplots(figsize=(9, 4.2), dpi=200)
ys = np.arange(len(items))[::-1]
ax.barh(ys, [c for c, _ in items], color=[MUTED if n.startswith("none") else TRAIN for _, n in items], height=0.6)
for y, (c, n) in zip(ys, items):
    ax.text(c + 5, y, str(c), va="center", fontsize=10)
ax.set_yticks(ys, [n for _, n in items]); ax.set_xlabel("Training tasks no v2 program reaches")
ax.set_title(f"Categories of the {sum(c for c, _ in items)} unreached training tasks", loc="left", fontsize=12)
clean(ax)
fig.savefig(OUT / "fig_categories.png", facecolor="white", bbox_inches="tight", pad_inches=0.15)
plt.close(fig)

# Design of the T vs R comparison.
fig, ax = plt.subplots(figsize=(10, 4.2), dpi=200)
ax.set_xlim(0, 10); ax.set_ylim(0, 4.2); ax.axis("off")
box(ax, 1.3, 2.1, 2.1, 1.1, "40 candidate\noperations\n+ 60 practice\ntask families", size=10)
box(ax, 4.0, 3.2, 2.2, 0.8, "Search method 1\n(every program)", size=10)
box(ax, 4.0, 2.1, 2.2, 0.8, "Search method 2\n(randomised beam)", size=10)
box(ax, 4.0, 0.7, 2.2, 0.8, "Random draw\n(seed fixed in advance)", size=10)
box(ax, 6.7, 2.65, 1.9, 0.9, "Agreement set:\ntop 10 by\nboth methods", fc="#e1eaf5", ec=TRAIN, size=10, weight="bold")
box(ax, 6.7, 0.7, 1.9, 0.8, "Random set:\n10 at random", fc="#eeeeee", ec=MUTED, size=10, weight="bold")
box(ax, 9.0, 1.7, 1.7, 1.4, "Add each set\nto v4, same\nbudget; count\nnew solves", size=10)
arrow(ax, 2.35, 2.4, 2.9, 3.1); arrow(ax, 2.35, 2.1, 2.9, 2.1); arrow(ax, 2.35, 1.8, 2.9, 0.85)
arrow(ax, 5.1, 3.15, 5.75, 2.85); arrow(ax, 5.1, 2.15, 5.75, 2.5); arrow(ax, 5.1, 0.7, 5.75, 0.7)
arrow(ax, 7.65, 2.55, 8.15, 2.05); arrow(ax, 7.65, 0.8, 8.15, 1.35)
ax.text(5.0, 4.05, "No real ARC tasks are used to choose operations.", ha="center", fontsize=10.5, color=MUTED)
fig.savefig(OUT / "fig_t_vs_r_design.png", facecolor="white", bbox_inches="tight", pad_inches=0.1)
plt.close(fig)

print("pooled", pooled_solved, "/", pooled_reached,
      "| v4 first-fit", first_fit, "any-fit", any_fit, "of", len(reached4))
print("figures written:", sorted(p.name for p in OUT.glob("*.png")))
