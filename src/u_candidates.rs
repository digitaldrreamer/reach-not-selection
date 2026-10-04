//! B5 candidate universe U — 40 deterministic Grid->Grid closures
//! (prereg notes/b5-prereg-2026-08-23.md, F1 SIGNED 2026-08-23; this
//! file is the F2 implementation pin, hashed in manifest S2-b5-001).
//!
//! House rules carried from F1: unresolvable/degenerate -> identity;
//! objects = 4-connected same-color components vs bg = most common
//! color; largest/smallest ties by (size, topmost-leftmost). Coding-
//! time refinements are recorded pre-discovery in DECISIONS
//! 2026-08-23i. NONE of these ops is in v0-v4; none was derived from
//! eval data or per-task probe rows.

use crate::{components, most_common_color, Grid};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UOp {
    // object-axis geometry (8)
    RotLargest90,
    RotLargest180,
    MirrorLargestH,
    MirrorLargestV,
    MirrorAllAcrossVMid,
    MirrorAllAcrossHMid,
    TransposeEachObj,
    RotEachObj90,
    // sorting / ranking (6)
    SwapPositionsBySizeLR,
    SwapPositionsBySizeTB,
    KeepTop2,
    KeepTop3,
    RemoveSmallest2,
    RankRecolorAsc,
    // counting constructs (6)
    CountObjsBar,
    CountColorsBar,
    ReplicateByCount,
    LargestSizeSquare,
    KeepObjsWithHoles,
    RemoveObjsWithHoles,
    // per-object scale / shape (4)
    ScaleLargest2x,
    StampSmallestMirrored,
    FillObjBBoxes,
    OutlineObjects,
    // color arithmetic (6)
    ColorShiftUp,
    ColorShiftDown,
    SwapMinorityMajority,
    InvertPaletteRank,
    BgSwapLeast,
    RecolorAllMajority,
    // positional / structural (10)
    MoveLargestToCenter,
    AlignObjsLeft,
    AlignObjsTop,
    SpreadRow,
    CenterColumnObjs,
    BorderFrame,
    FoldH,
    FoldV,
    DropBorderTouching,
    KeepBorderTouching,
}

pub const U_ALL: [UOp; 40] = [
    UOp::RotLargest90,
    UOp::RotLargest180,
    UOp::MirrorLargestH,
    UOp::MirrorLargestV,
    UOp::MirrorAllAcrossVMid,
    UOp::MirrorAllAcrossHMid,
    UOp::TransposeEachObj,
    UOp::RotEachObj90,
    UOp::SwapPositionsBySizeLR,
    UOp::SwapPositionsBySizeTB,
    UOp::KeepTop2,
    UOp::KeepTop3,
    UOp::RemoveSmallest2,
    UOp::RankRecolorAsc,
    UOp::CountObjsBar,
    UOp::CountColorsBar,
    UOp::ReplicateByCount,
    UOp::LargestSizeSquare,
    UOp::KeepObjsWithHoles,
    UOp::RemoveObjsWithHoles,
    UOp::ScaleLargest2x,
    UOp::StampSmallestMirrored,
    UOp::FillObjBBoxes,
    UOp::OutlineObjects,
    UOp::ColorShiftUp,
    UOp::ColorShiftDown,
    UOp::SwapMinorityMajority,
    UOp::InvertPaletteRank,
    UOp::BgSwapLeast,
    UOp::RecolorAllMajority,
    UOp::MoveLargestToCenter,
    UOp::AlignObjsLeft,
    UOp::AlignObjsTop,
    UOp::SpreadRow,
    UOp::CenterColumnObjs,
    UOp::BorderFrame,
    UOp::FoldH,
    UOp::FoldV,
    UOp::DropBorderTouching,
    UOp::KeepBorderTouching,
];

pub fn u_name(op: UOp) -> &'static str {
    match op {
        UOp::RotLargest90 => "RotLargest90",
        UOp::RotLargest180 => "RotLargest180",
        UOp::MirrorLargestH => "MirrorLargestH",
        UOp::MirrorLargestV => "MirrorLargestV",
        UOp::MirrorAllAcrossVMid => "MirrorAllAcrossVMid",
        UOp::MirrorAllAcrossHMid => "MirrorAllAcrossHMid",
        UOp::TransposeEachObj => "TransposeEachObj",
        UOp::RotEachObj90 => "RotEachObj90",
        UOp::SwapPositionsBySizeLR => "SwapPositionsBySizeLR",
        UOp::SwapPositionsBySizeTB => "SwapPositionsBySizeTB",
        UOp::KeepTop2 => "KeepTop2",
        UOp::KeepTop3 => "KeepTop3",
        UOp::RemoveSmallest2 => "RemoveSmallest2",
        UOp::RankRecolorAsc => "RankRecolorAsc",
        UOp::CountObjsBar => "CountObjsBar",
        UOp::CountColorsBar => "CountColorsBar",
        UOp::ReplicateByCount => "ReplicateByCount",
        UOp::LargestSizeSquare => "LargestSizeSquare",
        UOp::KeepObjsWithHoles => "KeepObjsWithHoles",
        UOp::RemoveObjsWithHoles => "RemoveObjsWithHoles",
        UOp::ScaleLargest2x => "ScaleLargest2x",
        UOp::StampSmallestMirrored => "StampSmallestMirrored",
        UOp::FillObjBBoxes => "FillObjBBoxes",
        UOp::OutlineObjects => "OutlineObjects",
        UOp::ColorShiftUp => "ColorShiftUp",
        UOp::ColorShiftDown => "ColorShiftDown",
        UOp::SwapMinorityMajority => "SwapMinorityMajority",
        UOp::InvertPaletteRank => "InvertPaletteRank",
        UOp::BgSwapLeast => "BgSwapLeast",
        UOp::RecolorAllMajority => "RecolorAllMajority",
        UOp::MoveLargestToCenter => "MoveLargestToCenter",
        UOp::AlignObjsLeft => "AlignObjsLeft",
        UOp::AlignObjsTop => "AlignObjsTop",
        UOp::SpreadRow => "SpreadRow",
        UOp::CenterColumnObjs => "CenterColumnObjs",
        UOp::BorderFrame => "BorderFrame",
        UOp::FoldH => "FoldH",
        UOp::FoldV => "FoldV",
        UOp::DropBorderTouching => "DropBorderTouching",
        UOp::KeepBorderTouching => "KeepBorderTouching",
    }
}

type Obj = (u8, Vec<(usize, usize)>);

fn bbox(cells: &[(usize, usize)]) -> (usize, usize, usize, usize) {
    let y0 = cells.iter().map(|c| c.0).min().unwrap();
    let y1 = cells.iter().map(|c| c.0).max().unwrap();
    let x0 = cells.iter().map(|c| c.1).min().unwrap();
    let x1 = cells.iter().map(|c| c.1).max().unwrap();
    (y0, y1, x0, x1)
}

/// Index of largest (or smallest) object: ties by (size, then
/// topmost-leftmost first cell) — house rule.
fn arg_size(objs: &[Obj], largest: bool) -> Option<usize> {
    if objs.is_empty() {
        return None;
    }
    let mut idx: Vec<usize> = (0..objs.len()).collect();
    idx.sort_by_key(|&i| (std::cmp::Reverse(objs[i].1.len()), objs[i].1[0]));
    Some(if largest { idx[0] } else { *idx.last().unwrap() })
}

/// Objects sorted size-desc, ties topmost-leftmost (stable ranking).
fn size_ranked(objs: &[Obj]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..objs.len()).collect();
    idx.sort_by_key(|&i| (std::cmp::Reverse(objs[i].1.len()), objs[i].1[0]));
    idx
}

fn erase(g: &mut Grid, cells: &[(usize, usize)], bg: u8) {
    for &(y, x) in cells {
        g.set(y, x, bg);
    }
}

fn draw(g: &mut Grid, cells: &[(usize, usize)], color: u8) {
    for &(y, x) in cells {
        g.set(y, x, color);
    }
}

/// Non-bg color frequency table (present colors only).
fn color_freqs(g: &Grid, bg: u8) -> Vec<(u8, usize)> {
    let mut counts = [0usize; 10];
    for &c in &g.c {
        if c != bg && (c as usize) < 10 {
            counts[c as usize] += 1;
        }
    }
    (0..=9u8).filter(|&c| counts[c as usize] > 0)
        .map(|c| (c, counts[c as usize])).collect()
}

/// Most frequent non-bg color (ties -> smaller id); None if no non-bg.
fn majority_color(g: &Grid, bg: u8) -> Option<u8> {
    color_freqs(g, bg).into_iter()
        .max_by_key(|&(c, n)| (n, std::cmp::Reverse(c))).map(|(c, _)| c)
}

/// Least frequent non-bg color (ties -> smaller id).
fn minority_color(g: &Grid, bg: u8) -> Option<u8> {
    color_freqs(g, bg).into_iter()
        .min_by_key(|&(c, n)| (n, c)).map(|(c, _)| c)
}

/// Border-unreachable bg cells (FillHoles-style global flood).
fn hole_mask(g: &Grid, bg: u8) -> Vec<bool> {
    let (h, w) = (g.h, g.w);
    let mut open = vec![false; h * w];
    let mut stack = Vec::new();
    for y in 0..h {
        for x in [0, w - 1] {
            if g.at(y, x) == bg && !open[y * w + x] {
                open[y * w + x] = true;
                stack.push((y, x));
            }
        }
    }
    for x in 0..w {
        for y in [0, h - 1] {
            if g.at(y, x) == bg && !open[y * w + x] {
                open[y * w + x] = true;
                stack.push((y, x));
            }
        }
    }
    while let Some((cy, cx)) = stack.pop() {
        let nbrs = [(cy.wrapping_sub(1), cx), (cy + 1, cx),
                    (cy, cx.wrapping_sub(1)), (cy, cx + 1)];
        for (ny, nx) in nbrs {
            if ny < h && nx < w && g.at(ny, nx) == bg && !open[ny * w + nx] {
                open[ny * w + nx] = true;
                stack.push((ny, nx));
            }
        }
    }
    (0..h * w).map(|i| g.c[i] == bg && !open[i]).collect()
}

/// Does object have >= 1 hole cell 4-adjacent to it (refinement R3)?
fn has_hole(g: &Grid, obj: &Obj, holes: &[bool]) -> bool {
    let (h, w) = (g.h, g.w);
    obj.1.iter().any(|&(y, x)| {
        [(y.wrapping_sub(1), x), (y + 1, x),
         (y, x.wrapping_sub(1)), (y, x + 1)]
            .into_iter()
            .any(|(ny, nx)| ny < h && nx < w && holes[ny * w + nx])
    })
}

/// Keep only the objects at `keep` indices (others -> bg).
fn keep_only(g: &Grid, objs: &[Obj], keep: &[usize], bg: u8) -> Grid {
    let mut out = g.clone();
    for (i, obj) in objs.iter().enumerate() {
        if !keep.contains(&i) {
            erase(&mut out, &obj.1, bg);
        }
    }
    out
}

/// Translate object cells by (dy, dx); None if any cell leaves grid.
fn translated(obj: &Obj, dy: isize, dx: isize, h: usize, w: usize)
              -> Option<Vec<(usize, usize)>> {
    obj.1.iter().map(|&(y, x)| {
        let (ny, nx) = (y as isize + dy, x as isize + dx);
        if ny >= 0 && (ny as usize) < h && nx >= 0 && (nx as usize) < w {
            Some((ny as usize, nx as usize))
        } else {
            None
        }
    }).collect()
}

pub fn apply_u(g: &Grid, op: UOp) -> Grid {
    if g.h == 0 || g.w == 0 {
        return g.clone();
    }
    let bg = most_common_color(g);
    let (h, w) = (g.h, g.w);
    match op {
        // ---------------- object-axis geometry ----------------
        UOp::RotLargest90 => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let (y0, y1, x0, x1) = bbox(&objs[i].1);
            let (bh, bw) = (y1 - y0 + 1, x1 - x0 + 1);
            // 90° cw about bbox anchor: (dy,dx) -> (dx, bh-1-dy);
            // rotated bbox is bw x bh at the same anchor
            if y0 + bw > h || x0 + bh > w {
                return g.clone(); // clipped -> identity (F1)
            }
            let mut out = g.clone();
            erase(&mut out, &objs[i].1, bg);
            let cells: Vec<(usize, usize)> = objs[i].1.iter()
                .map(|&(y, x)| (y0 + (x - x0), x0 + (bh - 1 - (y - y0))))
                .collect();
            draw(&mut out, &cells, objs[i].0);
            out
        }
        UOp::RotLargest180 => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let (y0, y1, x0, x1) = bbox(&objs[i].1);
            let mut out = g.clone();
            erase(&mut out, &objs[i].1, bg);
            let cells: Vec<(usize, usize)> = objs[i].1.iter()
                .map(|&(y, x)| (y0 + (y1 - y), x0 + (x1 - x)))
                .collect();
            draw(&mut out, &cells, objs[i].0);
            out
        }
        UOp::MirrorLargestH | UOp::MirrorLargestV => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let (y0, y1, x0, x1) = bbox(&objs[i].1);
            let mut out = g.clone();
            erase(&mut out, &objs[i].1, bg);
            let horiz = matches!(op, UOp::MirrorLargestH);
            let cells: Vec<(usize, usize)> = objs[i].1.iter()
                .map(|&(y, x)| if horiz {
                    (y, x0 + (x1 - x))
                } else {
                    (y0 + (y1 - y), x)
                })
                .collect();
            draw(&mut out, &cells, objs[i].0);
            out
        }
        UOp::MirrorAllAcrossVMid | UOp::MirrorAllAcrossHMid => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let mut out = g.clone();
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            let vert = matches!(op, UOp::MirrorAllAcrossVMid);
            for obj in &objs {
                let cells: Vec<(usize, usize)> = obj.1.iter()
                    .map(|&(y, x)| if vert {
                        (y, w - 1 - x)
                    } else {
                        (h - 1 - y, x)
                    })
                    .collect();
                draw(&mut out, &cells, obj.0); // discovery order; later wins (R1)
            }
            out
        }
        UOp::TransposeEachObj | UOp::RotEachObj90 => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let mut out = g.clone();
            let rot = matches!(op, UOp::RotEachObj90);
            // erase all first, then redraw transformed (skip clipped: R2)
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            for obj in &objs {
                let (y0, y1, x0, x1) = bbox(&obj.1);
                let (bh, bw) = (y1 - y0 + 1, x1 - x0 + 1);
                let fits = y0 + bw <= h && x0 + bh <= w;
                let cells: Vec<(usize, usize)> = if fits {
                    obj.1.iter().map(|&(y, x)| {
                        let (dy, dx) = (y - y0, x - x0);
                        if rot {
                            (y0 + dx, x0 + (bh - 1 - dy))
                        } else {
                            (y0 + dx, x0 + dy)
                        }
                    }).collect()
                } else {
                    obj.1.clone() // clipped -> object unchanged (R2)
                };
                draw(&mut out, &cells, obj.0);
            }
            out
        }
        // ---------------- sorting / ranking ----------------
        UOp::SwapPositionsBySizeLR | UOp::SwapPositionsBySizeTB => {
            let objs = components(g, bg);
            if objs.len() < 2 {
                return g.clone();
            }
            let lr = matches!(op, UOp::SwapPositionsBySizeLR);
            // anchors = bbox top-left, sorted by x then y (LR) or y then x (TB)
            let mut anchors: Vec<(usize, usize)> = objs.iter().map(|o| {
                let (y0, _, x0, _) = bbox(&o.1);
                (y0, x0)
            }).collect();
            let mut order: Vec<usize> = (0..objs.len()).collect();
            if lr {
                order.sort_by_key(|&i| (anchors[i].1, anchors[i].0));
            } else {
                order.sort_by_key(|&i| (anchors[i].0, anchors[i].1));
            }
            anchors.sort_by_key(|&(y, x)| if lr { (x, y) } else { (y, x) });
            let ranked = size_ranked(&objs);
            // largest object -> first anchor, etc.; clipped -> identity (R6)
            let mut placed: Vec<(Vec<(usize, usize)>, u8)> = Vec::new();
            for (k, &oi) in ranked.iter().enumerate() {
                let (y0, _, x0, _) = bbox(&objs[oi].1);
                let (ty, tx) = anchors[k];
                let (dy, dx) = (ty as isize - y0 as isize,
                                tx as isize - x0 as isize);
                match translated(&objs[oi], dy, dx, h, w) {
                    Some(cells) => placed.push((cells, objs[oi].0)),
                    None => return g.clone(),
                }
            }
            let mut out = g.clone();
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            for (cells, color) in placed {
                draw(&mut out, &cells, color);
            }
            out
        }
        UOp::KeepTop2 | UOp::KeepTop3 => {
            let objs = components(g, bg);
            let k = if matches!(op, UOp::KeepTop2) { 2 } else { 3 };
            if objs.len() <= k {
                return g.clone();
            }
            let keep: Vec<usize> = size_ranked(&objs)[..k].to_vec();
            keep_only(g, &objs, &keep, bg)
        }
        UOp::RemoveSmallest2 => {
            let objs = components(g, bg);
            if objs.len() <= 2 {
                return g.clone();
            }
            let ranked = size_ranked(&objs);
            let keep: Vec<usize> = ranked[..ranked.len() - 2].to_vec();
            keep_only(g, &objs, &keep, bg)
        }
        UOp::RankRecolorAsc => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let mut colors: Vec<u8> =
                color_freqs(g, bg).into_iter().map(|(c, _)| c).collect();
            colors.sort_unstable();
            let mut out = g.clone();
            for (k, &oi) in size_ranked(&objs).iter().enumerate() {
                let c = colors[k % colors.len()]; // cycle (R5)
                draw(&mut out, &objs[oi].1, c);
            }
            out
        }
        // ---------------- counting constructs ----------------
        UOp::CountObjsBar => {
            let objs = components(g, bg);
            let n = objs.len();
            let Some(mc) = majority_color(g, bg) else { return g.clone() };
            if n == 0 || n > 30 {
                return g.clone();
            }
            Grid::build(1, n, |_, _| mc)
        }
        UOp::CountColorsBar => {
            let k = color_freqs(g, bg).len();
            let Some(mc) = majority_color(g, bg) else { return g.clone() };
            if k == 0 || k > 30 {
                return g.clone();
            }
            Grid::build(1, k, |_, _| mc)
        }
        UOp::ReplicateByCount => {
            let n = components(g, bg).len();
            if n == 0 || n * w > 30 {
                return g.clone();
            }
            Grid::build(h, w * n, |y, x| g.at(y, x % w))
        }
        UOp::LargestSizeSquare => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let k = objs[i].1.len();
            if k > 30 {
                return g.clone();
            }
            let c = objs[i].0;
            Grid::build(k, k, |_, _| c)
        }
        UOp::KeepObjsWithHoles | UOp::RemoveObjsWithHoles => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let holes = hole_mask(g, bg);
            let want = matches!(op, UOp::KeepObjsWithHoles);
            let keep: Vec<usize> = (0..objs.len())
                .filter(|&i| has_hole(g, &objs[i], &holes) == want)
                .collect();
            if keep.is_empty() || keep.len() == objs.len() {
                return g.clone(); // R11: empty kept set -> identity
            }
            keep_only(g, &objs, &keep, bg)
        }
        // ---------------- per-object scale / shape ----------------
        UOp::ScaleLargest2x => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let (y0, y1, x0, x1) = bbox(&objs[i].1);
            if y0 + 2 * (y1 - y0 + 1) > h || x0 + 2 * (x1 - x0 + 1) > w {
                return g.clone(); // clipped -> identity (F1)
            }
            let mut out = g.clone();
            erase(&mut out, &objs[i].1, bg);
            for &(y, x) in &objs[i].1 {
                let (dy, dx) = (y - y0, x - x0);
                for a in 0..2 {
                    for b in 0..2 {
                        out.set(y0 + 2 * dy + a, x0 + 2 * dx + b, objs[i].0);
                    }
                }
            }
            out
        }
        UOp::StampSmallestMirrored => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, false) else { return g.clone() };
            let mut out = g.clone();
            let cells: Vec<(usize, usize)> = objs[i].1.iter()
                .map(|&(y, x)| (h - 1 - y, w - 1 - x)).collect();
            draw(&mut out, &cells, objs[i].0); // copy wins (F1)
            out
        }
        UOp::FillObjBBoxes => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let mut out = g.clone();
            for obj in &objs {
                let (y0, y1, x0, x1) = bbox(&obj.1);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        out.set(y, x, obj.0); // discovery order; later wins
                    }
                }
            }
            out
        }
        UOp::OutlineObjects => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let mut out = g.clone();
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            for obj in &objs {
                let (y0, y1, x0, x1) = bbox(&obj.1);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if y == y0 || y == y1 || x == x0 || x == x1 {
                            out.set(y, x, obj.0);
                        }
                    }
                }
            }
            out
        }
        // ---------------- color arithmetic ----------------
        UOp::ColorShiftUp | UOp::ColorShiftDown => {
            let up = matches!(op, UOp::ColorShiftUp);
            let step = |c: u8| -> u8 {
                let mut n = if up { (c + 1) % 10 } else { (c + 9) % 10 };
                if n == bg {
                    n = if up { (n + 1) % 10 } else { (n + 9) % 10 };
                }
                n
            };
            Grid::build(h, w, |y, x| {
                let c = g.at(y, x);
                if c == bg { c } else { step(c) }
            })
        }
        UOp::SwapMinorityMajority => {
            let (Some(a), Some(b)) = (minority_color(g, bg),
                                      majority_color(g, bg)) else {
                return g.clone();
            };
            if a == b {
                return g.clone();
            }
            Grid::build(h, w, |y, x| {
                let c = g.at(y, x);
                if c == a { b } else if c == b { a } else { c }
            })
        }
        UOp::InvertPaletteRank => {
            let mut freqs = color_freqs(g, bg);
            if freqs.len() < 2 {
                return g.clone();
            }
            // rank by (freq desc, color asc)
            freqs.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
            let k = freqs.len();
            let mut map = [0u8; 10];
            for c in 0..10u8 {
                map[c as usize] = c;
            }
            for i in 0..k {
                map[freqs[i].0 as usize] = freqs[k - 1 - i].0;
            }
            Grid::build(h, w, |y, x| {
                let c = g.at(y, x);
                if c == bg { c } else { map[c as usize] }
            })
        }
        UOp::BgSwapLeast => {
            let Some(a) = minority_color(g, bg) else { return g.clone() };
            Grid::build(h, w, |y, x| {
                let c = g.at(y, x);
                if c == bg { a } else if c == a { bg } else { c }
            })
        }
        UOp::RecolorAllMajority => {
            let Some(mc) = majority_color(g, bg) else { return g.clone() };
            Grid::build(h, w, |y, x| {
                let c = g.at(y, x);
                if c == bg { c } else { mc }
            })
        }
        // ---------------- positional / structural ----------------
        UOp::MoveLargestToCenter => {
            let objs = components(g, bg);
            let Some(i) = arg_size(&objs, true) else { return g.clone() };
            let (y0, y1, x0, x1) = bbox(&objs[i].1);
            let (cy, cx) = ((y0 + y1) / 2, (x0 + x1) / 2); // rounded up-left
            let (ty, tx) = ((h - 1) / 2, (w - 1) / 2);
            let (dy, dx) = (ty as isize - cy as isize,
                            tx as isize - cx as isize);
            match translated(&objs[i], dy, dx, h, w) {
                None => g.clone(),
                Some(cells) => {
                    let mut out = g.clone();
                    erase(&mut out, &objs[i].1, bg);
                    draw(&mut out, &cells, objs[i].0);
                    out
                }
            }
        }
        UOp::AlignObjsLeft | UOp::AlignObjsTop => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let left = matches!(op, UOp::AlignObjsLeft);
            let mut out = g.clone();
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            for obj in &objs {
                let (y0, _, x0, _) = bbox(&obj.1);
                let (dy, dx) = if left {
                    (0isize, -(x0 as isize))
                } else {
                    (-(y0 as isize), 0isize)
                };
                let cells = translated(obj, dy, dx, h, w).unwrap();
                draw(&mut out, &cells, obj.0); // discovery order; later wins
            }
            out
        }
        UOp::SpreadRow => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            // discovery order, bbox tops at row 0, 1-cell gaps from col 0
            let widths: Vec<usize> = objs.iter().map(|o| {
                let (_, _, x0, x1) = bbox(&o.1);
                x1 - x0 + 1
            }).collect();
            let total: usize = widths.iter().sum::<usize>() + objs.len() - 1;
            if total > w {
                return g.clone();
            }
            let mut out = Grid::new(h, w, bg);
            let mut cx = 0usize;
            for (k, obj) in objs.iter().enumerate() {
                let (y0, _, x0, _) = bbox(&obj.1);
                for &(y, x) in &obj.1 {
                    let (ny, nx) = (y - y0, cx + (x - x0));
                    if ny < h && nx < w {
                        out.set(ny, nx, obj.0);
                    }
                }
                cx += widths[k] + 1;
            }
            out
        }
        UOp::CenterColumnObjs => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let target = (w - 1) / 2;
            let mut out = g.clone();
            for obj in &objs {
                erase(&mut out, &obj.1, bg);
            }
            for obj in &objs {
                let (_, _, x0, x1) = bbox(&obj.1);
                let c = (x0 + x1) / 2;
                let dx = target as isize - c as isize;
                match translated(obj, 0, dx, h, w) {
                    Some(cells) => draw(&mut out, &cells, obj.0),
                    None => draw(&mut out, &obj.1, obj.0), // skip (R2)
                }
            }
            out
        }
        UOp::BorderFrame => {
            let Some(mc) = majority_color(g, bg) else { return g.clone() };
            let mut out = g.clone();
            for y in 0..h {
                for x in 0..w {
                    if y == 0 || y == h - 1 || x == 0 || x == w - 1 {
                        out.set(y, x, mc);
                    }
                }
            }
            out
        }
        UOp::FoldH => {
            // right half mirrored onto left; odd width keeps middle;
            // output = left half incl. middle (R4); non-bg wins, left first
            let ow = w.div_ceil(2);
            if w < 2 {
                return g.clone();
            }
            Grid::build(h, ow, |y, x| {
                let l = g.at(y, x);
                let r = g.at(y, w - 1 - x);
                if l != bg { l } else if r != bg { r } else { bg }
            })
        }
        UOp::FoldV => {
            let oh = h.div_ceil(2);
            if h < 2 {
                return g.clone();
            }
            Grid::build(oh, w, |y, x| {
                let t = g.at(y, x);
                let b = g.at(h - 1 - y, x);
                if t != bg { t } else if b != bg { b } else { bg }
            })
        }
        UOp::DropBorderTouching | UOp::KeepBorderTouching => {
            let objs = components(g, bg);
            if objs.is_empty() {
                return g.clone();
            }
            let touching = |o: &Obj| o.1.iter().any(|&(y, x)| {
                y == 0 || y == h - 1 || x == 0 || x == w - 1
            });
            let want_touch = matches!(op, UOp::KeepBorderTouching);
            let keep: Vec<usize> = (0..objs.len())
                .filter(|&i| touching(&objs[i]) == want_touch)
                .collect();
            if keep.is_empty() || keep.len() == objs.len() {
                return g.clone(); // R11 / no-op
            }
            keep_only(g, &objs, &keep, bg)
        }
    }
}

pub fn apply_u_chain(g: &Grid, ops: &[UOp]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_u(&acc, op))
}

#[cfg(test)]
mod u_tests {
    use super::*;

    fn g(rows: &[&[u8]]) -> Grid {
        Grid::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>())
            .unwrap()
    }

    #[test]
    fn count_objs_bar_counts_components() {
        let x = g(&[&[1, 0, 2], &[0, 0, 0], &[3, 0, 0]]);
        let out = apply_u(&x, UOp::CountObjsBar);
        assert_eq!((out.h, out.w), (1, 3));
        assert_eq!(out.at(0, 0), 1); // majority = smallest id on tie
    }

    #[test]
    fn fold_v_overlays_halves() {
        let x = g(&[&[4, 0], &[0, 0], &[0, 7]]);
        let out = apply_u(&x, UOp::FoldV);
        assert_eq!((out.h, out.w), (2, 2));
        assert_eq!(out.at(0, 0), 4);
        assert_eq!(out.at(0, 1), 7); // bottom mirrored up
    }

    #[test]
    fn color_shift_up_skips_bg() {
        let x = g(&[&[0, 9, 3], &[0, 0, 0]]);
        let out = apply_u(&x, UOp::ColorShiftUp);
        assert_eq!(out.at(0, 1), 1); // 9 -> 0 == bg -> 1
        assert_eq!(out.at(0, 2), 4);
        assert_eq!(out.at(0, 0), 0);
    }

    #[test]
    fn keep_top2_keeps_largest_two() {
        let x = g(&[&[1, 1, 0, 2, 2, 0, 3]]);
        let out = apply_u(&x, UOp::KeepTop2);
        assert_eq!(out.at(0, 6), 0);
        assert_eq!(out.at(0, 0), 1);
        assert_eq!(out.at(0, 3), 2);
    }

    #[test]
    fn replicate_by_count_tiles() {
        let x = g(&[&[5, 0, 6]]);
        let out = apply_u(&x, UOp::ReplicateByCount);
        assert_eq!((out.h, out.w), (1, 6)); // 2 objects -> 2 tiles
        assert_eq!(out.at(0, 3), 5);
    }

    #[test]
    fn rot_largest_180_flips_in_bbox() {
        let x = g(&[&[7, 7, 0], &[7, 0, 0], &[0, 0, 0]]);
        let out = apply_u(&x, UOp::RotLargest180);
        assert_eq!(out.at(1, 1), 7);
        assert_eq!(out.at(0, 0), 0);
        assert_eq!(out.at(1, 0), 7);
    }

    #[test]
    fn mirror_all_across_vmid_moves_objects() {
        let x = g(&[&[4, 0, 0, 0]]);
        let out = apply_u(&x, UOp::MirrorAllAcrossVMid);
        assert_eq!(out.at(0, 0), 0);
        assert_eq!(out.at(0, 3), 4);
    }

    #[test]
    fn stamp_smallest_mirrored_stamps_copy() {
        let x = g(&[&[8, 8, 0], &[0, 0, 0], &[0, 0, 5]]);
        let out = apply_u(&x, UOp::StampSmallestMirrored);
        assert_eq!(out.at(0, 0), 5); // 5's point reflection of (2,2)
        assert_eq!(out.at(2, 2), 5); // source kept
    }

    #[test]
    fn bg_swap_least_swaps() {
        let x = g(&[&[0, 0, 1, 1, 2]]);
        let out = apply_u(&x, UOp::BgSwapLeast);
        assert_eq!(out.at(0, 0), 2);
        assert_eq!(out.at(0, 4), 0);
        assert_eq!(out.at(0, 2), 1);
    }

    #[test]
    fn chain_is_deterministic() {
        let x = g(&[&[1, 0, 2], &[0, 3, 0], &[0, 0, 0]]);
        let a = apply_u_chain(&x, &[UOp::FillObjBBoxes, UOp::FoldH]);
        let b = apply_u_chain(&x, &[UOp::FillObjBBoxes, UOp::FoldH]);
        assert_eq!(a, b);
    }
}
