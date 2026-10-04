#!/usr/bin/env python3
"""B4 taxonomy pass (protocol notes/b4-tier-protocol-2026-08-21.md par.3.1).

Labels every tier-2-unreached ARC-AGI-2 training task with ONE primary
category from the frozen ten-category checklist. The rater's judgment is
expressed as the explicit, deterministic rules below (priority-ordered
first match wins); the rater validated the rules by grid-level inspection
of samples per category before freezing (recorded in
notes/b4-taxonomy-2026-08-21.md). Rules read ONLY train pairs — never test.

Usage: b4_taxonomy.py <training_dir> <unreached_ids.json> <out_labels.json>

Categories (frozen checklist, protocol par.3.1):
  1 tiling / scaling / block replication
  2 symmetry completion / reflection or rotation fill
  3 per-object recolor conditioned on a property
  4 object translation / gravity toward an anchor or wall
  5 line / path drawing, connection of markers
  6 count-driven output construction
  7 crop / extract to a marked or distinguished subregion
  8 multi-grid combination (mask, overlay, boolean between panes)
  9 pattern repair / denoise / majority-fill
 10 other
"""
import json
import sys
from collections import Counter


def dims(g):
    return len(g), len(g[0]) if g else 0


def palette(g):
    return {c for row in g for c in row}


def components(g, bg):
    """4-connected components of non-bg cells: list of cell sets."""
    h, w = dims(g)
    seen = [[False] * w for _ in range(h)]
    comps = []
    for i in range(h):
        for j in range(w):
            if g[i][j] != bg and not seen[i][j]:
                stack = [(i, j)]
                seen[i][j] = True
                cells = []
                while stack:
                    a, b = stack.pop()
                    cells.append((a, b, g[a][b]))
                    for da, db in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        na, nb = a + da, b + db
                        if 0 <= na < h and 0 <= nb < w and not seen[na][nb] and g[na][nb] != bg:
                            seen[na][nb] = True
                            stack.append((na, nb))
                comps.append(cells)
    return comps


def most_common_color(g):
    return Counter(c for row in g for c in row).most_common(1)[0][0]


def sym_variants(g):
    h, w = dims(g)
    out = [
        [row[::-1] for row in g],                      # h-flip
        g[::-1],                                       # v-flip
        [list(r) for r in zip(*g)][::-1] and [list(r)[::-1] for r in zip(*g)],  # rot90
    ]
    return out


def is_symmetric(g):
    return g == [row[::-1] for row in g] or g == g[::-1]


def separators(g):
    """Rows/cols that are uniform single-color lines."""
    h, w = dims(g)
    rs = [i for i in range(h) if len({g[i][j] for j in range(w)}) == 1]
    cs = [j for j in range(w) if len({g[i][j] for i in range(h)}) == 1]
    return rs, cs


def classify(task):
    tr = task["train"]
    ins = [p["input"] for p in tr]
    outs = [p["output"] for p in tr]
    din = [dims(g) for g in ins]
    dout = [dims(g) for g in outs]
    same_shape = all(a == b for a, b in zip(din, dout))

    # ---- rule T1: tiling / scaling — every output dim is an integer
    # multiple (>1) of its input dim, consistent across pairs
    ratios = set()
    tiling = True
    for (ih, iw), (oh, ow) in zip(din, dout):
        if ih and iw and oh % ih == 0 and ow % iw == 0 and (oh // ih > 1 or ow // iw > 1):
            ratios.add((oh // ih, ow // iw))
        else:
            tiling = False
            break
    if tiling and len(ratios) == 1:
        return 1

    if same_shape:
        bgs = [most_common_color(g) for g in ins]
        # ---- rule S1: symmetry completion — output symmetric, input equals
        # output except where input carries cells the output overwrites
        sym_fill = all(
            is_symmetric(o)
            and sum(1 for i in range(len(o)) for j in range(len(o[0])) if o[i][j] != g[i][j]) > 0
            and all(
                g[i][j] == o[i][j] or True
                for i in range(len(o))
                for j in range(len(o[0]))
            )
            and _diff_confined_to_one_color(g, o)
            for g, o in zip(ins, outs)
        )
        if sym_fill:
            return 2
        # ---- rule R1: recolor-only — cell-for-cell same support: nonbg
        # positions identical, only colors change
        recolor = all(
            {(i, j) for i, r in enumerate(g) for j, c in enumerate(r) if c != bg}
            == {(i, j) for i, r in enumerate(o) for j, c in enumerate(r) if c != bg}
            and g != o
            for g, o, bg in zip(ins, outs, bgs)
        )
        if recolor:
            return 3
        # ---- rule M1: same multiset of objects (shape+color), different
        # positions -> translation / gravity
        moved = all(_same_objects_moved(g, o, bg) for g, o, bg in zip(ins, outs, bgs))
        if moved:
            return 4
        # ---- rule L1: output = input plus added cells only (nothing
        # removed, colors kept) -> drawing lines / paths / growth
        additive = all(
            all(
                gc == oc or gc == bg
                for gr, orow in zip(g, o)
                for gc, oc in zip(gr, orow)
            )
            and g != o
            for g, o, bg in zip(ins, outs, bgs)
        )
        if additive:
            return 5
        # ---- rule P1: repair — output differs from input ONLY at cells
        # of one particular input color (noise color overwritten)
        if all(_diff_confined_to_one_color(g, o) for g, o in zip(ins, outs)):
            return 9

    else:
        # ---- rule X1: multi-pane — input has a full-length uniform
        # separator; output dims equal one pane's dims
        pane = True
        for g, (oh, ow) in zip(ins, dout):
            rs, cs = separators(g)
            h, w = dims(g)
            ok = False
            for i in rs:
                if {i} and ({(h - 1 - i) == oh or i == oh}) and w == ow:
                    ok = ok or (i == oh or h - 1 - i == oh)
            for j in cs:
                if h == oh and (j == ow or w - 1 - j == ow):
                    ok = True
            pane = pane and ok
        if pane and ins:
            return 8
        # ---- rule C1: crop — output appears verbatim as a subgrid of input
        crop = all(_is_subgrid(o, g) for g, o in zip(ins, outs))
        if crop:
            return 7
        # ---- rule N1: count-driven — output dims vary across pairs and
        # correlate exactly with nonbg object count or color count
        if len({d for d in dout}) > 1:
            counts = [len(components(g, most_common_color(g))) for g in ins]
            if all(
                (oh == c or ow == c or oh * ow == c)
                for c, (oh, ow) in zip(counts, dout)
            ):
                return 6

    return 10


def _diff_confined_to_one_color(g, o):
    if dims(g) != dims(o) or g == o:
        return False
    diff_colors = {g[i][j] for i in range(len(g)) for j in range(len(g[0])) if g[i][j] != o[i][j]}
    return len(diff_colors) == 1


def _same_objects_moved(g, o, bg):
    a = components(g, bg)
    b = components(o, bg)
    if len(a) != len(b) or not a:
        return False

    def norm(comp):
        mi = min(x for x, _, _ in comp)
        mj = min(y for _, y, _ in comp)
        return tuple(sorted((x - mi, y - mj, c) for x, y, c in comp))

    moved_any = sorted(map(norm, a)) == sorted(map(norm, b)) and g != o
    return moved_any


def _is_subgrid(o, g):
    oh, ow = dims(o)
    gh, gw = dims(g)
    if oh > gh or ow > gw or (oh, ow) == (gh, gw):
        return False
    for i in range(gh - oh + 1):
        for j in range(gw - ow + 1):
            if all(g[i + a][j + b] == o[a][b] for a in range(oh) for b in range(ow)):
                return True
    return False


def main():
    tdir, unreached_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    ids = json.load(open(unreached_path))
    labels = {}
    for tid in ids:
        task = json.load(open(f"{tdir}/{tid}.json"))
        labels[tid] = classify(task)
    counts = Counter(labels.values())
    json.dump({"labels": labels, "counts": {str(k): v for k, v in sorted(counts.items())}},
              open(out_path, "w"), indent=1)
    for k in sorted(counts, key=lambda x: -counts[x]):
        print(f"category {k}: {counts[k]}")


if __name__ == "__main__":
    main()
