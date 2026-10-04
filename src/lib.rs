//! tpd-arc: the ARC-AGI-2 static arena (researcher-directed front,
//! sequencing decision DECISIONS 2026-07-24: consolidate -> ARC-2 ->
//! ARC-3 agentic). DSL is OPTION (c): a hand-rolled v0 op set to
//! stand the pipeline up; a published DSL subset replaces it for any
//! reportable run (that swap is its own ledger entry).
//!
//! Grids hold colors 0-9 (ARC convention; 0 is canonically
//! "background" but nothing here hard-codes that beyond
//! crop_to_content's most-common-color rule, which is extensional
//! and recorded).
//!
//! REPRESENTATION (2026-08-22, performance-only rewrite): `Grid` is a
//! flat row-major buffer `{h, w, c}` replacing `Vec<Vec<u8>>`. Every
//! op's OBSERVABLE semantics are unchanged; the rewrite is verified
//! byte-identical against the previous binary on the full ARC-AGI-2
//! training measurement (see DECISIONS 2026-08-22). Degenerate-grid
//! equality matches the old representation exactly: a zero-ROW grid
//! canonicalises to (0, 0) because `Vec<Vec<u8>>` could not
//! distinguish widths of row-less grids, while zero-WIDTH grids keep
//! their row count, which the old representation did distinguish.

use std::collections::HashMap;
use std::path::Path;

pub mod u_candidates;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Grid {
    pub h: usize,
    pub w: usize,
    pub c: Vec<u8>,
}

impl Grid {
    #[inline]
    pub fn new(h: usize, w: usize, fill: u8) -> Grid {
        let (h, w) = if h == 0 { (0, 0) } else { (h, w) };
        Grid { h, w, c: vec![fill; h * w] }
    }

    #[inline(always)]
    pub fn at(&self, y: usize, x: usize) -> u8 {
        self.c[y * self.w + x]
    }

    #[inline(always)]
    pub fn set(&mut self, y: usize, x: usize, v: u8) {
        let w = self.w;
        self.c[y * w + x] = v;
    }

    #[inline]
    pub fn row(&self, y: usize) -> &[u8] {
        &self.c[y * self.w..(y + 1) * self.w]
    }

    #[inline]
    pub fn is_degenerate(&self) -> bool {
        self.h == 0 || self.w == 0
    }

    /// Build with per-cell function; canonicalises h==0 to (0,0).
    pub fn build(h: usize, w: usize, mut f: impl FnMut(usize, usize) -> u8) -> Grid {
        if h == 0 || w == 0 {
            return Grid { h: if w == 0 { h } else { 0 }, w: 0, c: Vec::new() }
                .canon();
        }
        let mut c = Vec::with_capacity(h * w);
        for y in 0..h {
            for x in 0..w {
                c.push(f(y, x));
            }
        }
        Grid { h, w, c }
    }

    fn canon(mut self) -> Grid {
        // old Vec<Vec<u8>> semantics: no rows -> width unknowable
        if self.h == 0 {
            self.w = 0;
            self.c.clear();
        }
        self
    }

    /// Inclusive sub-rectangle copy.
    pub fn sub(&self, y0: usize, y1: usize, x0: usize, x1: usize) -> Grid {
        Grid::build(y1 - y0 + 1, x1 - x0 + 1, |y, x| self.at(y0 + y, x0 + x))
    }

    /// Row range [y0, y1) with all columns (old `g[a..b].to_vec()`).
    pub fn rows_range(&self, y0: usize, y1: usize) -> Grid {
        Grid { h: y1 - y0, w: self.w, c: self.c[y0 * self.w..y1 * self.w].to_vec() }
            .canon()
    }

    /// Column range [x0, x1) keeping all rows. A zero-width result
    /// keeps its row count (old per-row empty vecs did).
    pub fn cols_range(&self, x0: usize, x1: usize) -> Grid {
        if x1 == x0 {
            return Grid { h: self.h, w: 0, c: Vec::new() };
        }
        Grid::build(self.h, x1 - x0, |y, x| self.at(y, x0 + x))
    }

    pub fn from_rows(rows: &[Vec<u8>]) -> Option<Grid> {
        let h = rows.len();
        if h == 0 {
            return Some(Grid { h: 0, w: 0, c: Vec::new() });
        }
        let w = rows[0].len();
        if rows.iter().any(|r| r.len() != w) {
            return None; // ragged: not representable (ARC data never is)
        }
        let mut c = Vec::with_capacity(h * w);
        for r in rows {
            c.extend_from_slice(r);
        }
        Some(Grid { h, w, c }.canon())
    }

    pub fn to_rows(&self) -> Vec<Vec<u8>> {
        (0..self.h).map(|y| self.row(y).to_vec()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Pair {
    pub input: Grid,
    pub output: Grid,
}

#[derive(Clone, Debug)]
pub struct ArcTask {
    pub id: String,
    pub train: Vec<Pair>,
    pub test: Vec<Pair>,
}

fn parse_grid(v: &serde_json::Value) -> Option<Grid> {
    let rows: Option<Vec<Vec<u8>>> = v
        .as_array()?
        .iter()
        .map(|row| {
            row.as_array()?
                .iter()
                .map(|c| c.as_u64().map(|x| x as u8))
                .collect::<Option<Vec<u8>>>()
        })
        .collect();
    Grid::from_rows(&rows?)
}

fn parse_pairs(v: &serde_json::Value) -> Option<Vec<Pair>> {
    v.as_array()?
        .iter()
        .map(|p| {
            Some(Pair {
                input: parse_grid(&p["input"])?,
                output: parse_grid(&p["output"])?,
            })
        })
        .collect()
}

/// Load every .json task in a directory (ARC-AGI-1/2 format).
/// Returns tasks sorted by id (deterministic order).
pub fn load_dir(dir: &Path) -> std::io::Result<Vec<ArcTask>> {
    let mut tasks = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort();
    for path in entries {
        let text = std::fs::read_to_string(&path)?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let id = path.file_stem().unwrap().to_string_lossy().into_owned();
        let (train, test) = match (parse_pairs(&v["train"]), parse_pairs(&v["test"])) {
            (Some(tr), Some(te)) => (tr, te),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("malformed task {id}"),
                ))
            }
        };
        tasks.push(ArcTask { id, train, test });
    }
    Ok(tasks)
}

/// The v0 hand-rolled op set (DSL option (c)): parameterless
/// Grid -> Grid transforms, chosen for composability, NOT claimed
/// adequate for ARC — the pipeline-stand-up set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GOp {
    Rot90,
    Rot180,
    Rot270,
    FlipH,
    FlipV,
    Transpose,
    CropToContent,
    Upscale2,
    MirrorConcatH,
    MirrorConcatV,
    TrimBorder,
    SwapTopTwoColors,
}

pub const G_OPS: [GOp; 12] = [
    GOp::Rot90,
    GOp::Rot180,
    GOp::Rot270,
    GOp::FlipH,
    GOp::FlipV,
    GOp::Transpose,
    GOp::CropToContent,
    GOp::Upscale2,
    GOp::MirrorConcatH,
    GOp::MirrorConcatV,
    GOp::TrimBorder,
    GOp::SwapTopTwoColors,
];

fn transpose(g: &Grid) -> Grid {
    if g.is_degenerate() {
        return g.clone();
    }
    Grid::build(g.w, g.h, |y, x| g.at(x, y))
}

fn flip_h(g: &Grid) -> Grid {
    // reverse each row
    Grid::build(g.h, g.w, |y, x| g.at(y, g.w - 1 - x))
}

fn flip_v(g: &Grid) -> Grid {
    // reverse row order
    Grid::build(g.h, g.w, |y, x| g.at(g.h - 1 - y, x))
}

fn most_common_color(g: &Grid) -> u8 {
    // fixed-array count; colors >9 counted nowhere, exactly as the
    // 0..=9 scan ignored them; ties break to the smaller color id
    let mut counts = [0usize; 10];
    for &c in &g.c {
        if (c as usize) < 10 {
            counts[c as usize] += 1;
        }
    }
    let mut best = (0u8, 0usize);
    for c in 0..=9u8 {
        let n = counts[c as usize];
        if n > best.1 {
            best = (c, n);
        }
    }
    best.0
}

pub fn apply_gop(g: &Grid, op: GOp) -> Grid {
    match op {
        GOp::Rot90 => flip_h(&transpose(g)),
        GOp::Rot180 => flip_v(&flip_h(g)),
        GOp::Rot270 => flip_v(&transpose(g)),
        GOp::FlipH => flip_h(g),
        GOp::FlipV => flip_v(g),
        GOp::Transpose => transpose(g),
        GOp::CropToContent => {
            let bg = most_common_color(g);
            let (mut y0, mut y1, mut x0, mut x1) = (usize::MAX, 0usize, usize::MAX, 0usize);
            for y in 0..g.h {
                for x in 0..g.w {
                    if g.at(y, x) != bg {
                        y0 = y0.min(y);
                        y1 = y1.max(y);
                        x0 = x0.min(x);
                        x1 = x1.max(x);
                    }
                }
            }
            if y0 == usize::MAX {
                return g.clone(); // uniform grid: nothing to crop to
            }
            g.sub(y0, y1, x0, x1)
        }
        GOp::Upscale2 => Grid::build(g.h * 2, g.w * 2, |y, x| g.at(y / 2, x / 2)),
        GOp::MirrorConcatH => Grid::build(g.h, g.w * 2, |y, x| {
            if x < g.w { g.at(y, x) } else { g.at(y, 2 * g.w - 1 - x) }
        }),
        GOp::MirrorConcatV => Grid::build(g.h * 2, g.w, |y, x| {
            if y < g.h { g.at(y, x) } else { g.at(2 * g.h - 1 - y, x) }
        }),
        GOp::TrimBorder => {
            if g.h > 2 && g.w > 2 {
                g.sub(1, g.h - 2, 1, g.w - 2)
            } else {
                g.clone()
            }
        }
        GOp::SwapTopTwoColors => {
            let mut counts = [0usize; 10];
            for &c in &g.c {
                if (c as usize) < 10 {
                    counts[c as usize] += 1;
                }
            }
            let mut by_count: Vec<(u8, usize)> =
                (0..=9u8).map(|c| (c, counts[c as usize])).collect();
            // sort by count desc, color asc: deterministic
            by_count.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
            let (a, b) = (by_count[0].0, by_count[1].0);
            Grid::build(g.h, g.w, |y, x| {
                let c = g.at(y, x);
                if c == a { b } else if c == b { a } else { c }
            })
        }
    }
}

pub fn apply_gchain(g: &Grid, ops: &[GOp]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_gop(&acc, op))
}

/// Does a program map every train input to its train output?
pub fn fits_train(task: &ArcTask, ops: &[GOp]) -> bool {
    task.train.iter().all(|p| apply_gchain(&p.input, ops) == p.output)
}

/// Does it also solve the (held-back) test pairs? The ground-truth
/// check — never available to a selection rule.
pub fn solves_test(task: &ArcTask, ops: &[GOp]) -> bool {
    task.test.iter().all(|p| apply_gchain(&p.input, ops) == p.output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(rows: &[&[u8]]) -> Grid {
        Grid::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn rotations_compose_to_identity() {
        let x = g(&[&[1, 2, 3], &[4, 5, 6]]);
        assert_eq!(apply_gchain(&x, &[GOp::Rot90, GOp::Rot90]), apply_gop(&x, GOp::Rot180));
        assert_eq!(
            apply_gchain(&x, &[GOp::Rot90, GOp::Rot90, GOp::Rot90, GOp::Rot90]),
            x
        );
        assert_eq!(apply_gchain(&x, &[GOp::FlipH, GOp::FlipH]), x);
    }

    #[test]
    fn crop_finds_content_box() {
        let x = g(&[&[0, 0, 0, 0], &[0, 3, 3, 0], &[0, 0, 3, 0], &[0, 0, 0, 0]]);
        assert_eq!(apply_gop(&x, GOp::CropToContent), g(&[&[3, 3], &[0, 3]]));
    }

    #[test]
    fn upscale_doubles_both_dims() {
        let x = g(&[&[1, 2]]);
        assert_eq!(apply_gop(&x, GOp::Upscale2), g(&[&[1, 1, 2, 2], &[1, 1, 2, 2]]));
    }

    #[test]
    fn task_parses_from_json() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"train":[{"input":[[1,2]],"output":[[2,1]]}],"test":[{"input":[[3,4]],"output":[[4,3]]}]}"#,
        )
        .unwrap();
        let train = super::parse_pairs(&v["train"]).unwrap();
        assert_eq!(train.len(), 1);
        assert_eq!(train[0].output, g(&[&[2, 1]]));
    }

    #[test]
    fn degenerate_equality_matches_old_representation() {
        // zero rows: widths unknowable -> equal
        let a = Grid::from_rows(&[]).unwrap();
        let b = Grid { h: 0, w: 5, c: vec![] }.canon();
        assert_eq!(a, b);
        // zero width: row count still distinguishes
        let c3 = Grid { h: 3, w: 0, c: vec![] };
        let c2 = Grid { h: 2, w: 0, c: vec![] };
        assert_ne!(c3, c2);
    }
}

// ===================================================================
// v1 — the expressive tier (option (c) published subset; provenance:
// Hodel arc-dsl, notes/arc2-dsl-v1.md). Fixed ops + per-task
// instantiated color ops.
// ===================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum V1Op {
    Rot90,
    Rot180,
    Rot270,
    HMirror,
    VMirror,
    DMirror,
    CMirror,
    TopHalf,
    BottomHalf,
    LeftHalf,
    RightHalf,
    Trim,
    Compress,
    Upscale2,
    Upscale3,
    Downscale2,
    HUpscale2,
    VUpscale2,
    HConcatMirror,
    VConcatMirror,
    Replace(u8, u8),
    Switch(u8, u8),
}

fn v1_transpose(g: &Grid) -> Grid {
    transpose(g)
}

fn v1_hmirror(g: &Grid) -> Grid {
    flip_v(g) // Hodel hmirror: mirror along horizontal axis (row order)
}

fn v1_vmirror(g: &Grid) -> Grid {
    flip_h(g) // per-row reversal
}

pub fn apply_v1(g: &Grid, op: V1Op) -> Grid {
    use V1Op::*;
    match op {
        Rot90 => v1_vmirror(&v1_transpose(g)),
        Rot180 => v1_hmirror(&v1_vmirror(g)),
        Rot270 => v1_hmirror(&v1_transpose(g)),
        HMirror => v1_hmirror(g),
        VMirror => v1_vmirror(g),
        DMirror => v1_transpose(g),
        CMirror => v1_hmirror(&v1_vmirror(&v1_transpose(g))),
        TopHalf => g.rows_range(0, g.h / 2),
        BottomHalf => g.rows_range(g.h - g.h / 2, g.h),
        LeftHalf => g.cols_range(0, g.w / 2),
        RightHalf => g.cols_range(g.w - g.w / 2, g.w),
        Trim => {
            if g.h > 2 && g.w > 2 {
                g.sub(1, g.h - 2, 1, g.w - 2)
            } else {
                g.clone()
            }
        }
        Compress => {
            // uniform rows/cols removed; vacuously uniform when the
            // other dimension is <= 1 (windows(2) had no pairs)
            let kept_rows: Vec<usize> = (0..g.h)
                .filter(|&y| !(0..g.w.saturating_sub(1))
                    .all(|x| g.at(y, x) == g.at(y, x + 1)))
                .collect();
            let kept_cols: Vec<usize> = (0..g.w)
                .filter(|&x| !(0..g.h.saturating_sub(1))
                    .all(|y| g.at(y, x) == g.at(y + 1, x)))
                .collect();
            if kept_rows.is_empty() || kept_cols.is_empty() {
                g.clone()
            } else {
                Grid::build(kept_rows.len(), kept_cols.len(), |y, x| {
                    g.at(kept_rows[y], kept_cols[x])
                })
            }
        }
        Upscale2 => upscale_k(g, 2),
        Upscale3 => upscale_k(g, 3),
        Downscale2 => {
            if g.h >= 2 && g.w >= 2 {
                Grid::build(g.h.div_ceil(2), g.w.div_ceil(2), |y, x| g.at(2 * y, 2 * x))
            } else {
                g.clone()
            }
        }
        HUpscale2 => Grid::build(g.h, g.w * 2, |y, x| g.at(y, x / 2)),
        VUpscale2 => Grid::build(g.h * 2, g.w, |y, x| g.at(y / 2, x)),
        HConcatMirror => Grid::build(g.h, g.w * 2, |y, x| {
            if x < g.w { g.at(y, x) } else { g.at(y, 2 * g.w - 1 - x) }
        }),
        VConcatMirror => Grid::build(g.h * 2, g.w, |y, x| {
            if y < g.h { g.at(y, x) } else { g.at(2 * g.h - 1 - y, x) }
        }),
        Replace(a, b) => Grid::build(g.h, g.w, |y, x| {
            let c = g.at(y, x);
            if c == a { b } else { c }
        }),
        Switch(a, b) => Grid::build(g.h, g.w, |y, x| {
            let c = g.at(y, x);
            if c == a { b } else if c == b { a } else { c }
        }),
    }
}

fn upscale_k(g: &Grid, k: usize) -> Grid {
    Grid::build(g.h * k, g.w * k, |y, x| g.at(y / k, x / k))
}

pub fn apply_v1_chain(g: &Grid, ops: &[V1Op]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_v1(&acc, op))
}

pub const V1_FIXED: [V1Op; 20] = [
    V1Op::Rot90,
    V1Op::Rot180,
    V1Op::Rot270,
    V1Op::HMirror,
    V1Op::VMirror,
    V1Op::DMirror,
    V1Op::CMirror,
    V1Op::TopHalf,
    V1Op::BottomHalf,
    V1Op::LeftHalf,
    V1Op::RightHalf,
    V1Op::Trim,
    V1Op::Compress,
    V1Op::Upscale2,
    V1Op::Upscale3,
    V1Op::Downscale2,
    V1Op::HUpscale2,
    V1Op::VUpscale2,
    V1Op::HConcatMirror,
    V1Op::VConcatMirror,
];

/// Per-task v1 alphabet: fixed ops + Replace/Switch over the <=5
/// most frequent train-INPUT colors (count desc, color asc —
/// deterministic; notes/arc2-dsl-v1.md).
pub fn v1_alphabet(task: &ArcTask) -> Vec<V1Op> {
    let mut counts = [0usize; 10];
    for p in &task.train {
        for &c in &p.input.c {
            if (c as usize) < 10 {
                counts[c as usize] += 1;
            }
        }
    }
    let mut by_count: Vec<(u8, usize)> =
        (0..=9u8).map(|c| (c, counts[c as usize])).collect();
    by_count.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
    let palette: Vec<u8> =
        by_count.iter().filter(|&&(_, n)| n > 0).take(5).map(|&(c, _)| c).collect();
    let mut ops: Vec<V1Op> = V1_FIXED.to_vec();
    for &a in &palette {
        for &b in &palette {
            if a != b {
                ops.push(V1Op::Replace(a, b));
            }
        }
    }
    for i in 0..palette.len() {
        for j in (i + 1)..palette.len() {
            ops.push(V1Op::Switch(palette[i], palette[j]));
        }
    }
    ops
}

pub fn v1_fits_train_k(task: &ArcTask, ops: &[V1Op], k: usize) -> bool {
    task.train.iter().take(k).all(|p| apply_v1_chain(&p.input, ops) == p.output)
}

pub fn v1_fits_all_train(task: &ArcTask, ops: &[V1Op]) -> bool {
    task.train.iter().all(|p| apply_v1_chain(&p.input, ops) == p.output)
}

pub fn v1_solves_test(task: &ArcTask, ops: &[V1Op]) -> bool {
    task.test.iter().all(|p| apply_v1_chain(&p.input, ops) == p.output)
}

// ===================================================================
// Tier 2 (v2) — object-level closures (notes/arc2-dsl-v1.md tier-2
// table). Objects: 4-connected same-color components vs bg = most
// common color; ties by (size, topmost-leftmost), deterministic.
// ===================================================================

fn v2_bg(g: &Grid) -> u8 {
    most_common_color(g)
}

/// (color, cells) components, 4-connected, same color, excluding bg.
fn components(g: &Grid, bg: u8) -> Vec<(u8, Vec<(usize, usize)>)> {
    let h = g.h;
    let w = g.w;
    let mut seen = vec![false; w * h];
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if seen[y * w + x] || g.at(y, x) == bg {
                continue;
            }
            let color = g.at(y, x);
            let mut cells = Vec::new();
            let mut stack = vec![(y, x)];
            seen[y * w + x] = true;
            while let Some((cy, cx)) = stack.pop() {
                cells.push((cy, cx));
                let nbrs = [(cy.wrapping_sub(1), cx), (cy + 1, cx),
                            (cy, cx.wrapping_sub(1)), (cy, cx + 1)];
                for (ny, nx) in nbrs {
                    if ny < h && nx < w && !seen[ny * w + nx] && g.at(ny, nx) == color {
                        seen[ny * w + nx] = true;
                        stack.push((ny, nx));
                    }
                }
            }
            cells.sort_unstable();
            out.push((color, cells));
        }
    }
    out
}

/// Index of the argmax/argmin object by (size, then topmost-leftmost
/// for determinism). None if no objects.
fn arg_obj(objs: &[(u8, Vec<(usize, usize)>)], largest: bool) -> Option<usize> {
    if objs.is_empty() {
        return None;
    }
    let key = |i: usize| (objs[i].1.len(), std::cmp::Reverse(objs[i].1[0]));
    let idx = (0..objs.len()).max_by_key(|&i| {
        if largest { key(i) } else { (usize::MAX - key(i).0, key(i).1) }
    });
    idx
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum V2Op {
    V1(V1Op),
    KeepLargestObj,
    KeepSmallestObj,
    RemoveLargestObj,
    RemoveSmallestObj,
    CropLargestObj,
    FillHoles,
    GravityDown,
    GravityUp,
    GravityLeft,
    GravityRight,
}

pub const V2_OBJECT_OPS: [V2Op; 10] = [
    V2Op::KeepLargestObj,
    V2Op::KeepSmallestObj,
    V2Op::RemoveLargestObj,
    V2Op::RemoveSmallestObj,
    V2Op::CropLargestObj,
    V2Op::FillHoles,
    V2Op::GravityDown,
    V2Op::GravityUp,
    V2Op::GravityLeft,
    V2Op::GravityRight,
];

fn gravity(g: &Grid, bg: u8, dir: (i8, i8)) -> Grid {
    let h = g.h;
    let w = g.w;
    let mut out = Grid::new(h, w, bg);
    match dir {
        (1, 0) | (-1, 0) => {
            for x in 0..w {
                let col: Vec<u8> =
                    (0..h).map(|y| g.at(y, x)).filter(|&c| c != bg).collect();
                if dir.0 == 1 {
                    for (i, &c) in col.iter().enumerate() {
                        out.set(h - col.len() + i, x, c);
                    }
                } else {
                    for (i, &c) in col.iter().enumerate() {
                        out.set(i, x, c);
                    }
                }
            }
        }
        _ => {
            for y in 0..h {
                let row: Vec<u8> =
                    g.row(y).iter().copied().filter(|&c| c != bg).collect();
                if dir.1 == 1 {
                    for (i, &c) in row.iter().enumerate() {
                        out.set(y, w - row.len() + i, c);
                    }
                } else {
                    for (i, &c) in row.iter().enumerate() {
                        out.set(y, i, c);
                    }
                }
            }
        }
    }
    out
}

pub fn apply_v2(g: &Grid, op: V2Op) -> Grid {
    // degenerate grids (zero rows or zero-width rows) pass through:
    // object machinery indexes rows and borders
    // old representation returned the grid unchanged for EVERY op on
    // degenerate grids (zero rows / zero-width rows), including
    // wrapped lower-tier ops — replicate exactly
    if g.is_degenerate() {
        return g.clone();
    }
    match op {
        V2Op::V1(v1) => apply_v1(g, v1),
        V2Op::KeepLargestObj | V2Op::KeepSmallestObj => {
            let bg = v2_bg(g);
            let objs = components(g, bg);
            match arg_obj(&objs, matches!(op, V2Op::KeepLargestObj)) {
                None => g.clone(),
                Some(i) => {
                    let mut out = Grid::new(g.h, g.w, bg);
                    for &(y, x) in &objs[i].1 {
                        out.set(y, x, objs[i].0);
                    }
                    out
                }
            }
        }
        V2Op::RemoveLargestObj | V2Op::RemoveSmallestObj => {
            let bg = v2_bg(g);
            let objs = components(g, bg);
            match arg_obj(&objs, matches!(op, V2Op::RemoveLargestObj)) {
                None => g.clone(),
                Some(i) => {
                    let mut out = g.clone();
                    for &(y, x) in &objs[i].1 {
                        out.set(y, x, bg);
                    }
                    out
                }
            }
        }
        V2Op::CropLargestObj => {
            let bg = v2_bg(g);
            let objs = components(g, bg);
            match arg_obj(&objs, true) {
                None => g.clone(),
                Some(i) => {
                    let cells = &objs[i].1;
                    let (y0, y1) = (cells.iter().map(|c| c.0).min().unwrap(),
                                    cells.iter().map(|c| c.0).max().unwrap());
                    let (x0, x1) = (cells.iter().map(|c| c.1).min().unwrap(),
                                    cells.iter().map(|c| c.1).max().unwrap());
                    g.sub(y0, y1, x0, x1)
                }
            }
        }
        V2Op::FillHoles => {
            let bg = v2_bg(g);
            let h = g.h;
            let w = g.w;
            // flood bg from the border; unreached bg cells are holes
            let mut open = vec![false; w * h];
            let mut stack = Vec::new();
            for y in 0..h {
                for x in [0, w.saturating_sub(1)] {
                    if g.at(y, x) == bg && !open[y * w + x] {
                        open[y * w + x] = true;
                        stack.push((y, x));
                    }
                }
            }
            for x in 0..w {
                for y in [0, h.saturating_sub(1)] {
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
            let mut out = g.clone();
            for y in 0..h {
                for x in 0..w {
                    if g.at(y, x) == bg && !open[y * w + x] {
                        // majority adjacent non-bg color, tie -> smaller
                        let mut counts = [0usize; 10];
                        let nbrs = [(y.wrapping_sub(1), x), (y + 1, x),
                                    (y, x.wrapping_sub(1)), (y, x + 1)];
                        for (ny, nx) in nbrs {
                            if ny < h && nx < w && g.at(ny, nx) != bg {
                                counts[g.at(ny, nx) as usize] += 1;
                            }
                        }
                        if let Some(c) = (0..10u8)
                            .filter(|&c| counts[c as usize] > 0)
                            .max_by_key(|&c| (counts[c as usize], std::cmp::Reverse(c)))
                        {
                            out.set(y, x, c);
                        }
                    }
                }
            }
            out
        }
        V2Op::GravityDown => gravity(g, v2_bg(g), (1, 0)),
        V2Op::GravityUp => gravity(g, v2_bg(g), (-1, 0)),
        V2Op::GravityLeft => gravity(g, v2_bg(g), (0, -1)),
        V2Op::GravityRight => gravity(g, v2_bg(g), (0, 1)),
    }
}

pub fn apply_v2_chain(g: &Grid, ops: &[V2Op]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_v2(&acc, op))
}

pub fn v2_alphabet(task: &ArcTask) -> Vec<V2Op> {
    let mut ops: Vec<V2Op> = v1_alphabet(task).into_iter().map(V2Op::V1).collect();
    ops.extend(V2_OBJECT_OPS);
    ops
}

pub fn v2_fits_all_train(task: &ArcTask, ops: &[V2Op]) -> bool {
    task.train.iter().all(|p| apply_v2_chain(&p.input, ops) == p.output)
}

pub fn v2_solves_test(task: &ArcTask, ops: &[V2Op]) -> bool {
    task.test.iter().all(|p| apply_v2_chain(&p.input, ops) == p.output)
}

// ===================================================================
// Tier 3 (v3) — ten frozen closures (manifest S2-arc2-tier3-001;
// taxonomy notes/b4-taxonomy-2026-08-21.md). Objects: the lib's
// 4-connected same-color components vs bg = most common color (tier-2
// precedent; refinement of the manifest's "non-bg components" wording,
// recorded in DECISIONS before any outcome was observed). Unresolvable
// cases return the input unchanged.
// ===================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum V3Op {
    V2(V2Op),
    ConnectPairs,
    ExtendSegments,
    RecolorSizeFreq,
    RecolorToLargest,
    Tile2x2,
    TileMirror2x2,
    StampSelf,
    CropUnique,
    PaneAnd,
    Symmetrize,
}

pub const V3_NEW_OPS: [V3Op; 10] = [
    V3Op::ConnectPairs,
    V3Op::ExtendSegments,
    V3Op::RecolorSizeFreq,
    V3Op::RecolorToLargest,
    V3Op::Tile2x2,
    V3Op::TileMirror2x2,
    V3Op::StampSelf,
    V3Op::CropUnique,
    V3Op::PaneAnd,
    V3Op::Symmetrize,
];

fn connect_pairs(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    let mut out = g.clone();
    // row pass reads g; column pass reads the row-pass result
    for y in 0..h {
        let nonbg: Vec<usize> = (0..w).filter(|&x| g.at(y, x) != bg).collect();
        for pair in nonbg.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b > a + 1 && g.at(y, a) == g.at(y, b) {
                for x in (a + 1)..b {
                    out.set(y, x, g.at(y, a));
                }
            }
        }
    }
    let mid = out.clone();
    for x in 0..w {
        let nonbg: Vec<usize> = (0..h).filter(|&y| mid.at(y, x) != bg).collect();
        for pair in nonbg.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b > a + 1 && mid.at(a, x) == mid.at(b, x) {
                for y in (a + 1)..b {
                    out.set(y, x, mid.at(a, x));
                }
            }
        }
    }
    out
}

fn extend_segments(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    let mut out = g.clone();
    for y in 0..h {
        let mut x = 0;
        while x < w {
            if g.at(y, x) != bg {
                let c = g.at(y, x);
                let start = x;
                while x < w && g.at(y, x) == c {
                    x += 1;
                }
                if x - start >= 2 {
                    for xx in 0..start {
                        if out.at(y, xx) == bg {
                            out.set(y, xx, c);
                        }
                    }
                    for xx in x..w {
                        if out.at(y, xx) == bg {
                            out.set(y, xx, c);
                        }
                    }
                }
            } else {
                x += 1;
            }
        }
    }
    for x in 0..w {
        let mut y = 0;
        while y < h {
            if g.at(y, x) != bg {
                let c = g.at(y, x);
                let start = y;
                while y < h && g.at(y, x) == c {
                    y += 1;
                }
                if y - start >= 2 {
                    for yy in 0..start {
                        if out.at(yy, x) == bg {
                            out.set(yy, x, c);
                        }
                    }
                    for yy in y..h {
                        if out.at(yy, x) == bg {
                            out.set(yy, x, c);
                        }
                    }
                }
            } else {
                y += 1;
            }
        }
    }
    out
}

fn recolor_size_freq(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let mut objs = components(g, bg);
    if objs.is_empty() {
        return g.clone();
    }
    objs.sort_by_key(|(_, cells)| (std::cmp::Reverse(cells.len()), cells[0]));
    // single-pass recount of the original per-color grid scans
    let mut cell_counts = [0usize; 10];
    for &c in &g.c {
        if (c as usize) < 10 {
            cell_counts[c as usize] += 1;
        }
    }
    let mut freq: Vec<(usize, u8)> = (0..=9u8)
        .filter(|&c| c != bg)
        .map(|c| (cell_counts[c as usize], c))
        .filter(|&(n, _)| n > 0)
        .collect();
    freq.sort_by_key(|&(n, c)| (std::cmp::Reverse(n), c));
    let mut out = g.clone();
    for (i, (_, cells)) in objs.iter().enumerate() {
        if let Some(&(_, c)) = freq.get(i) {
            for &(y, x) in cells {
                out.set(y, x, c);
            }
        }
    }
    out
}

fn recolor_to_largest(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let objs = components(g, bg);
    let Some(idx) = arg_obj(&objs, true) else {
        return g.clone();
    };
    let target = objs[idx].0;
    let mut out = g.clone();
    for (_, cells) in &objs {
        for &(y, x) in cells {
            out.set(y, x, target);
        }
    }
    out
}

fn tile_2x2(g: &Grid) -> Grid {
    Grid::build(2 * g.h, 2 * g.w, |y, x| g.at(y % g.h, x % g.w))
}

fn tile_mirror_2x2(g: &Grid) -> Grid {
    Grid::build(2 * g.h, 2 * g.w, |y, x| {
        let sy = if y < g.h { y } else { 2 * g.h - 1 - y };
        let sx = if x < g.w { x } else { 2 * g.w - 1 - x };
        g.at(sy, sx)
    })
}

fn stamp_self(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    if h * h > 900 || w * w > 900 {
        return g.clone(); // ARC grids cap at 30x30; larger stamps are unrepresentable
    }
    let mut out = Grid::new(h * h, w * w, bg);
    for i in 0..h {
        for j in 0..w {
            if g.at(i, j) != bg {
                for a in 0..h {
                    for b in 0..w {
                        out.set(i * h + a, j * w + b, g.at(a, b));
                    }
                }
            }
        }
    }
    out
}

fn crop_unique(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let objs = components(g, bg);
    if objs.len() < 2 {
        return g.clone();
    }
    let norm = |&(color, ref cells): &(u8, Vec<(usize, usize)>)| -> Vec<(usize, usize, u8)> {
        let my = cells.iter().map(|&(y, _)| y).min().unwrap();
        let mx = cells.iter().map(|&(_, x)| x).min().unwrap();
        let mut v: Vec<(usize, usize, u8)> =
            cells.iter().map(|&(y, x)| (y - my, x - mx, color)).collect();
        v.sort_unstable();
        v
    };
    let forms: Vec<Vec<(usize, usize, u8)>> = objs.iter().map(norm).collect();
    let mut counts: HashMap<&Vec<(usize, usize, u8)>, usize> = HashMap::new();
    for f in &forms {
        *counts.entry(f).or_default() += 1;
    }
    let singles: Vec<usize> =
        (0..objs.len()).filter(|&i| counts[&forms[i]] == 1).collect();
    let has_repeat = counts.values().any(|&n| n >= 2);
    if singles.len() != 1 || !has_repeat {
        return g.clone();
    }
    let cells = &objs[singles[0]].1;
    let (y0, y1) = (
        cells.iter().map(|&(y, _)| y).min().unwrap(),
        cells.iter().map(|&(y, _)| y).max().unwrap(),
    );
    let (x0, x1) = (
        cells.iter().map(|&(_, x)| x).min().unwrap(),
        cells.iter().map(|&(_, x)| x).max().unwrap(),
    );
    g.sub(y0, y1, x0, x1)
}

fn pane_and(g: &Grid) -> Grid {
    let Some((p1, p2)) = pane_split(g) else {
        return g.clone();
    };
    let bg1 = most_common_color(&p1);
    let bg2 = most_common_color(&p2);
    Grid::build(p1.h, p1.w, |y, x| {
        let (a, b) = (p1.at(y, x), p2.at(y, x));
        if a != bg1 && b != bg2 { a } else { bg1 }
    })
}

fn symmetrize(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    let mut out = g.clone();
    for y in 0..h {
        for x in 0..w {
            let orbit = [(y, x), (y, w - 1 - x), (h - 1 - y, x), (h - 1 - y, w - 1 - x)];
            // fixed-array recount of the original per-cell HashMap. The
            // original's (best, tie) outcome depended only on the
            // multiset of counts: winner = max-count color, tie iff the
            // max is achieved by >= 2 colors. This scan computes
            // exactly that predicate.
            let mut counts = [0usize; 10];
            for &(oy, ox) in &orbit {
                let v = g.at(oy, ox);
                if v != bg && (v as usize) < 10 {
                    counts[v as usize] += 1;
                }
            }
            let mut best: Option<(usize, u8)> = None;
            let mut tie = false;
            for v in 0..10u8 {
                let n = counts[v as usize];
                if n == 0 {
                    continue;
                }
                match best {
                    None => best = Some((n, v)),
                    Some((bn, _)) if n > bn => {
                        best = Some((n, v));
                        tie = false;
                    }
                    Some((bn, _)) if n == bn => tie = true,
                    _ => {}
                }
            }
            if let Some((n, v)) = best {
                if n >= 2 && !tie && g.at(y, x) != v {
                    out.set(y, x, v);
                }
            }
        }
    }
    out
}

pub fn apply_v3(g: &Grid, op: V3Op) -> Grid {
    // old representation returned the grid unchanged for EVERY op on
    // degenerate grids (zero rows / zero-width rows), including
    // wrapped lower-tier ops — replicate exactly
    if g.is_degenerate() {
        return g.clone();
    }
    match op {
        V3Op::V2(v2) => apply_v2(g, v2),
        V3Op::ConnectPairs => connect_pairs(g),
        V3Op::ExtendSegments => extend_segments(g),
        V3Op::RecolorSizeFreq => recolor_size_freq(g),
        V3Op::RecolorToLargest => recolor_to_largest(g),
        V3Op::Tile2x2 => tile_2x2(g),
        V3Op::TileMirror2x2 => tile_mirror_2x2(g),
        V3Op::StampSelf => stamp_self(g),
        V3Op::CropUnique => crop_unique(g),
        V3Op::PaneAnd => pane_and(g),
        V3Op::Symmetrize => symmetrize(g),
    }
}

pub fn apply_v3_chain(g: &Grid, ops: &[V3Op]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_v3(&acc, op))
}

pub fn v3_alphabet(task: &ArcTask) -> Vec<V3Op> {
    let mut ops: Vec<V3Op> = v2_alphabet(task).into_iter().map(V3Op::V2).collect();
    ops.extend(V3_NEW_OPS);
    ops
}

pub fn v3_fits_all_train(task: &ArcTask, ops: &[V3Op]) -> bool {
    task.train.iter().all(|p| apply_v3_chain(&p.input, ops) == p.output)
}

pub fn v3_solves_test(task: &ArcTask, ops: &[V3Op]) -> bool {
    task.test.iter().all(|p| apply_v3_chain(&p.input, ops) == p.output)
}

// ===================================================================
// Tier 4 (v4) — ten frozen closures (manifest S2-arc2-tier4-001;
// Amendment A1 budget). Objects: 4-connected same-color components vs
// bg = most common color. Unresolvable cases return the input.
// ===================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum V4Op {
    V3(V3Op),
    Scale2x,
    Scale3x,
    RaysFromSingletons,
    DiagExtend,
    ConnectDiag,
    Halo,
    OutlineBox,
    RemoveSingletons,
    PaneOr,
    PaneXor,
}

pub const V4_NEW_OPS: [V4Op; 10] = [
    V4Op::Scale2x,
    V4Op::Scale3x,
    V4Op::RaysFromSingletons,
    V4Op::DiagExtend,
    V4Op::ConnectDiag,
    V4Op::Halo,
    V4Op::OutlineBox,
    V4Op::RemoveSingletons,
    V4Op::PaneOr,
    V4Op::PaneXor,
];

fn scale_n(g: &Grid, n: usize) -> Grid {
    if g.h * n > 30 || g.w * n > 30 {
        return g.clone(); // ARC grids cap at 30x30
    }
    Grid::build(g.h * n, g.w * n, |y, x| g.at(y / n, x / n))
}

fn rays_from_singletons(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    let singles: Vec<(usize, usize, u8)> = components(g, bg)
        .into_iter()
        .filter(|(_, cells)| cells.len() == 1)
        .map(|(c, cells)| (cells[0].0, cells[0].1, c))
        .collect();
    let mut out = g.clone();
    for (y, x, c) in singles {
        for yy in 0..h {
            if out.at(yy, x) == bg {
                out.set(yy, x, c);
            }
        }
        for xx in 0..w {
            if out.at(y, xx) == bg {
                out.set(y, xx, c);
            }
        }
    }
    out
}

fn diag_extend(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h as isize;
    let w = g.w as isize;
    let mut out = g.clone();
    let get = |y: isize, x: isize| g.at(y as usize, x as usize);
    for &(dy, dx) in &[(1isize, 1isize), (1, -1)] {
        for sy in 0..h {
            for sx in 0..w {
                // start of a maximal run only
                let py = sy - dy;
                let px = sx - dx;
                let c = get(sy, sx);
                if c == bg {
                    continue;
                }
                if py >= 0 && py < h && px >= 0 && px < w && get(py, px) == c {
                    continue;
                }
                let mut len = 1isize;
                while sy + len * dy >= 0
                    && sy + len * dy < h
                    && sx + len * dx >= 0
                    && sx + len * dx < w
                    && get(sy + len * dy, sx + len * dx) == c
                {
                    len += 1;
                }
                if len >= 2 {
                    let mut t = 1isize;
                    loop {
                        let (yy, xx) = (sy - t * dy, sx - t * dx);
                        if yy < 0 || yy >= h || xx < 0 || xx >= w {
                            break;
                        }
                        if out.at(yy as usize, xx as usize) == bg {
                            out.set(yy as usize, xx as usize, c);
                        }
                        t += 1;
                    }
                    let mut t = len;
                    loop {
                        let (yy, xx) = (sy + t * dy, sx + t * dx);
                        if yy < 0 || yy >= h || xx < 0 || xx >= w {
                            break;
                        }
                        if out.at(yy as usize, xx as usize) == bg {
                            out.set(yy as usize, xx as usize, c);
                        }
                        t += 1;
                    }
                }
            }
        }
    }
    out
}

fn connect_diag(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h as isize;
    let w = g.w as isize;
    let mut out = g.clone();
    for &(dy, dx) in &[(1isize, 1isize), (1, -1)] {
        let src = out.clone();
        let at = |y: isize, x: isize| src.at(y as usize, x as usize);
        // walk every diagonal line
        let starts: Vec<(isize, isize)> = if dx == 1 {
            (0..h).map(|y| (y, 0)).chain((1..w).map(|x| (0, x))).collect()
        } else {
            (0..h).map(|y| (y, w - 1)).chain((0..w - 1).map(|x| (0, x))).collect()
        };
        for (sy, sx) in starts {
            let mut cells: Vec<(isize, isize)> = Vec::new();
            let (mut y, mut x) = (sy, sx);
            while y >= 0 && y < h && x >= 0 && x < w {
                cells.push((y, x));
                y += dy;
                x += dx;
            }
            let nonbg: Vec<usize> =
                (0..cells.len()).filter(|&i| at(cells[i].0, cells[i].1) != bg).collect();
            for pair in nonbg.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let ca = at(cells[a].0, cells[a].1);
                if b > a + 1 && ca == at(cells[b].0, cells[b].1) {
                    for i in (a + 1)..b {
                        out.set(cells[i].0 as usize, cells[i].1 as usize, ca);
                    }
                }
            }
        }
    }
    out
}

fn halo(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let h = g.h;
    let w = g.w;
    let mut out = g.clone();
    for (color, cells) in components(g, bg) {
        for &(y, x) in &cells {
            let nbrs = [(y.wrapping_sub(1), x), (y + 1, x), (y, x.wrapping_sub(1)), (y, x + 1)];
            for (ny, nx) in nbrs {
                if ny < h && nx < w && g.at(ny, nx) == bg && out.at(ny, nx) == bg {
                    out.set(ny, nx, color);
                }
            }
        }
    }
    out
}

fn outline_box(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let mut out = g.clone();
    for (color, cells) in components(g, bg) {
        let (y0, y1) = (
            cells.iter().map(|&(y, _)| y).min().unwrap(),
            cells.iter().map(|&(y, _)| y).max().unwrap(),
        );
        let (x0, x1) = (
            cells.iter().map(|&(_, x)| x).min().unwrap(),
            cells.iter().map(|&(_, x)| x).max().unwrap(),
        );
        for y in y0..=y1 {
            for x in x0..=x1 {
                let border = y == y0 || y == y1 || x == x0 || x == x1;
                if border && g.at(y, x) == bg && out.at(y, x) == bg {
                    out.set(y, x, color);
                }
            }
        }
    }
    out
}

fn remove_singletons(g: &Grid) -> Grid {
    let bg = v2_bg(g);
    let mut out = g.clone();
    for (_, cells) in components(g, bg) {
        if cells.len() == 1 {
            let (y, x) = cells[0];
            out.set(y, x, bg);
        }
    }
    out
}

fn pane_split(g: &Grid) -> Option<(Grid, Grid)> {
    let h = g.h;
    let w = g.w;
    let sep_rows: Vec<usize> = (0..h)
        .filter(|&y| g.row(y).iter().all(|&c| c == g.at(y, 0)))
        .collect();
    let sep_cols: Vec<usize> = (0..w)
        .filter(|&x| (0..h).all(|y| g.at(y, x) == g.at(0, x)))
        .collect();
    if sep_rows.len() == 1 && sep_cols.is_empty() && sep_rows[0] * 2 + 1 == h {
        let r = sep_rows[0];
        Some((g.rows_range(0, r), g.rows_range(r + 1, h)))
    } else if sep_cols.len() == 1 && sep_rows.is_empty() && sep_cols[0] * 2 + 1 == w {
        let c = sep_cols[0];
        Some((g.cols_range(0, c), g.cols_range(c + 1, w)))
    } else {
        None
    }
}

fn pane_or(g: &Grid) -> Grid {
    let Some((p1, p2)) = pane_split(g) else {
        return g.clone();
    };
    let bg1 = most_common_color(&p1);
    let bg2 = most_common_color(&p2);
    Grid::build(p1.h, p1.w, |y, x| {
        let (a, b) = (p1.at(y, x), p2.at(y, x));
        if a != bg1 { a } else if b != bg2 { b } else { bg1 }
    })
}

fn pane_xor(g: &Grid) -> Grid {
    let Some((p1, p2)) = pane_split(g) else {
        return g.clone();
    };
    let bg1 = most_common_color(&p1);
    let bg2 = most_common_color(&p2);
    Grid::build(p1.h, p1.w, |y, x| {
        let (a, b) = (p1.at(y, x), p2.at(y, x));
        match (a != bg1, b != bg2) {
            (true, false) => a,
            (false, true) => b,
            _ => bg1,
        }
    })
}

pub fn apply_v4(g: &Grid, op: V4Op) -> Grid {
    // old representation returned the grid unchanged for EVERY op on
    // degenerate grids (zero rows / zero-width rows), including
    // wrapped lower-tier ops — replicate exactly
    if g.is_degenerate() {
        return g.clone();
    }
    match op {
        V4Op::V3(v3) => apply_v3(g, v3),
        V4Op::Scale2x => scale_n(g, 2),
        V4Op::Scale3x => scale_n(g, 3),
        V4Op::RaysFromSingletons => rays_from_singletons(g),
        V4Op::DiagExtend => diag_extend(g),
        V4Op::ConnectDiag => connect_diag(g),
        V4Op::Halo => halo(g),
        V4Op::OutlineBox => outline_box(g),
        V4Op::RemoveSingletons => remove_singletons(g),
        V4Op::PaneOr => pane_or(g),
        V4Op::PaneXor => pane_xor(g),
    }
}

pub fn apply_v4_chain(g: &Grid, ops: &[V4Op]) -> Grid {
    ops.iter().fold(g.clone(), |acc, &op| apply_v4(&acc, op))
}

pub fn v4_alphabet(task: &ArcTask) -> Vec<V4Op> {
    let mut ops: Vec<V4Op> = v3_alphabet(task).into_iter().map(V4Op::V3).collect();
    ops.extend(V4_NEW_OPS);
    ops
}

pub fn v4_fits_all_train(task: &ArcTask, ops: &[V4Op]) -> bool {
    task.train.iter().all(|p| apply_v4_chain(&p.input, ops) == p.output)
}

pub fn v4_solves_test(task: &ArcTask, ops: &[V4Op]) -> bool {
    task.test.iter().all(|p| apply_v4_chain(&p.input, ops) == p.output)
}

#[cfg(test)]
mod v4_tests {
    use super::*;

    fn g(rows: &[&[u8]]) -> Grid {
        Grid::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn scale_2x_doubles_pixels() {
        let x = g(&[&[1, 2]]);
        assert_eq!(apply_v4(&x, V4Op::Scale2x), g(&[&[1, 1, 2, 2], &[1, 1, 2, 2]]));
    }

    #[test]
    fn rays_cross_from_singleton() {
        let x = g(&[&[0, 0, 0], &[0, 7, 0], &[0, 0, 0]]);
        let out = apply_v4(&x, V4Op::RaysFromSingletons);
        assert_eq!(out, g(&[&[0, 7, 0], &[7, 7, 7], &[0, 7, 0]]));
    }

    #[test]
    fn diag_extend_reaches_corners() {
        let x = g(&[
            &[0, 0, 0, 0],
            &[0, 6, 0, 0],
            &[0, 0, 6, 0],
            &[0, 0, 0, 0],
        ]);
        let out = apply_v4(&x, V4Op::DiagExtend);
        assert_eq!(out.at(0, 0), 6);
        assert_eq!(out.at(3, 3), 6);
    }

    #[test]
    fn connect_diag_fills_gap() {
        let x = g(&[
            &[4, 0, 0],
            &[0, 0, 0],
            &[0, 0, 4],
        ]);
        let out = apply_v4(&x, V4Op::ConnectDiag);
        assert_eq!(out.at(1, 1), 4);
    }

    #[test]
    fn halo_dilates_once() {
        let x = g(&[&[0, 0, 0], &[0, 3, 0], &[0, 0, 0]]);
        let out = apply_v4(&x, V4Op::Halo);
        assert_eq!(out, g(&[&[0, 3, 0], &[3, 3, 3], &[0, 3, 0]]));
    }

    #[test]
    fn outline_box_draws_border() {
        let x = g(&[
            &[0, 0, 0, 0],
            &[0, 5, 0, 0],
            &[0, 5, 5, 0],
            &[0, 0, 0, 0],
        ]);
        let out = apply_v4(&x, V4Op::OutlineBox);
        assert_eq!(out.at(1, 2), 5); // bbox corner filled on the border
        assert_eq!(out.at(0, 0), 0);
    }

    #[test]
    fn remove_singletons_clears_noise() {
        let x = g(&[&[8, 0, 0], &[0, 0, 0], &[0, 3, 3]]);
        let out = apply_v4(&x, V4Op::RemoveSingletons);
        assert_eq!(out.at(0, 0), 0);
        assert_eq!(out.at(2, 1), 3);
    }

    #[test]
    fn pane_or_unions() {
        let x = g(&[
            &[0, 4, 5, 0, 0],
            &[4, 0, 5, 0, 2],
            &[0, 0, 5, 2, 0],
        ]);
        let out = apply_v4(&x, V4Op::PaneOr);
        assert_eq!(out, g(&[&[0, 4], &[4, 2], &[2, 0]]));
    }

    #[test]
    fn pane_xor_excludes_overlap() {
        let x = g(&[
            &[0, 4, 5, 0, 4],
            &[4, 0, 5, 0, 2],
            &[0, 0, 5, 2, 0],
        ]);
        let out = apply_v4(&x, V4Op::PaneXor);
        assert_eq!(out, g(&[&[0, 0], &[4, 2], &[2, 0]]));
    }

    #[test]
    fn v4_chain_composes() {
        let x = g(&[&[0, 1], &[0, 0]]);
        let out = apply_v4_chain(&x, &[V4Op::Scale2x, V4Op::V3(V3Op::Tile2x2)]);
        assert_eq!(out.h, 8);
    }
}

#[cfg(test)]
mod v3_tests {
    use super::*;

    fn g(rows: &[&[u8]]) -> Grid {
        Grid::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn connect_pairs_fills_between() {
        let x = g(&[&[0, 3, 0, 0, 3], &[0, 0, 0, 0, 0]]);
        let out = apply_v3(&x, V3Op::ConnectPairs);
        assert_eq!(out.row(0), &[0, 3, 3, 3, 3]);
    }

    #[test]
    fn connect_pairs_blocked_by_other_color() {
        let x = g(&[&[3, 5, 0, 3], &[0, 0, 0, 0]]);
        let out = apply_v3(&x, V3Op::ConnectPairs);
        assert_eq!(out.row(0), &[3, 5, 0, 3]);
    }

    #[test]
    fn extend_segments_reaches_edges() {
        let x = g(&[&[0, 0, 0, 0, 0], &[0, 4, 4, 0, 0], &[0, 0, 0, 0, 0]]);
        let out = apply_v3(&x, V3Op::ExtendSegments);
        assert_eq!(out.row(1), &[4, 4, 4, 4, 4]);
    }

    #[test]
    fn tile_2x2_replicates() {
        let x = g(&[&[1, 2]]);
        assert_eq!(apply_v3(&x, V3Op::Tile2x2), g(&[&[1, 2, 1, 2], &[1, 2, 1, 2]]));
    }

    #[test]
    fn tile_mirror_quadrants() {
        let x = g(&[&[1, 2]]);
        assert_eq!(
            apply_v3(&x, V3Op::TileMirror2x2),
            g(&[&[1, 2, 2, 1], &[1, 2, 2, 1]])
        );
    }

    #[test]
    fn stamp_self_fractal() {
        let x = g(&[&[5, 0], &[0, 0]]);
        let out = apply_v3(&x, V3Op::StampSelf);
        assert_eq!(out.h, 4);
        assert_eq!(out.at(0, 0), 5);
        assert_eq!(out.at(2, 2), 0);
    }

    #[test]
    fn crop_unique_picks_odd_one_out() {
        let x = g(&[
            &[8, 0, 8, 0, 3],
            &[0, 0, 0, 0, 3],
        ]);
        let out = apply_v3(&x, V3Op::CropUnique);
        assert_eq!(out, g(&[&[3], &[3]]));
    }

    #[test]
    fn pane_and_intersects() {
        // single uniform separator col at x=2; both panes have bg 0
        let x = g(&[
            &[0, 4, 5, 0, 4],
            &[4, 0, 5, 4, 0],
            &[0, 0, 5, 0, 3],
        ]);
        let out = apply_v3(&x, V3Op::PaneAnd);
        // non-bg in BOTH panes: (0,1) and (1,0); output takes pane-1 color
        assert_eq!(out, g(&[&[0, 4], &[4, 0], &[0, 0]]));
    }

    #[test]
    fn symmetrize_restores_majority() {
        let x = g(&[
            &[7, 0, 7],
            &[0, 0, 0],
            &[7, 0, 7],
        ]);
        let mut broken = x.clone();
        broken.set(0, 0, 0);
        let out = apply_v3(&broken, V3Op::Symmetrize);
        assert_eq!(out, x);
    }

    #[test]
    fn v3_chain_composes_with_v2() {
        let x = g(&[&[0, 1, 0], &[0, 0, 0]]);
        let out = apply_v3_chain(&x, &[V3Op::Tile2x2, V3Op::V2(V2Op::GravityDown)]);
        assert_eq!(out.h, 4);
    }
}

#[cfg(test)]
mod v2_tests {
    use super::*;

    fn g(rows: &[&[u8]]) -> Grid {
        Grid::from_rows(&rows.iter().map(|r| r.to_vec()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn keep_largest_isolates_biggest_component() {
        let x = g(&[&[0, 0, 0, 0], &[3, 3, 0, 5], &[3, 3, 0, 0], &[0, 0, 0, 0]]);
        let out = apply_v2(&x, V2Op::KeepLargestObj);
        assert_eq!(out, g(&[&[0, 0, 0, 0], &[3, 3, 0, 0], &[3, 3, 0, 0], &[0, 0, 0, 0]]));
        let out2 = apply_v2(&x, V2Op::RemoveLargestObj);
        assert_eq!(out2, g(&[&[0, 0, 0, 0], &[0, 0, 0, 5], &[0, 0, 0, 0], &[0, 0, 0, 0]]));
    }

    #[test]
    fn gravity_down_slides_cells() {
        let x = g(&[&[7, 0], &[0, 0], &[0, 4]]);
        let out = apply_v2(&x, V2Op::GravityDown);
        assert_eq!(out, g(&[&[0, 0], &[0, 0], &[7, 4]]));
    }

    #[test]
    fn fill_holes_fills_enclosed_bg() {
        let x = g(&[&[0, 0, 0, 0, 0],
                    &[0, 6, 6, 6, 0],
                    &[0, 6, 0, 6, 0],
                    &[0, 6, 6, 6, 0],
                    &[0, 0, 0, 0, 0]]);
        let out = apply_v2(&x, V2Op::FillHoles);
        assert_eq!(out.at(2, 2), 6);
        assert_eq!(out.at(0, 0), 0); // border bg untouched
    }

    #[test]
    fn crop_largest_takes_bbox() {
        let x = g(&[&[0, 0, 0], &[0, 8, 8], &[0, 8, 0]]);
        let out = apply_v2(&x, V2Op::CropLargestObj);
        assert_eq!(out, g(&[&[8, 8], &[8, 0]]));
    }
}
