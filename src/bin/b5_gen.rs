//! B5 synthetic family generator (prereg notes/b5-prereg-2026-08-23.md,
//! F1 signed; manifest S2-b5-001). Seed 993850, splitmix64 PRNG
//! (self-contained: no external dependency, fully pinned).
//!
//! A family = hidden transform p (composition of length 1-3 over U,
//! length weights 0.5/0.3/0.2, ops uniform with replacement) applied
//! to random grids: dims U[6,18], bg 0, 2-4 non-bg colors, 3-7
//! objects of size 1-9 placed without 4-adjacency to other objects
//! (refinement: distinctness guarantee). Per family 60 items
//! (40 m2train / 10 m2val / 10 probe). 60 discovery + 20 held-out
//! families. Degenerate items (p(input) == input) resampled <= 100
//! tries, else the whole family is redrawn (new program).
//!
//! Usage: b5_gen <out_dir>

use std::path::Path;
use std::process::exit;

use sha2::{Digest, Sha256};
use tpd_arc::u_candidates::{apply_u_chain, u_name, UOp, U_ALL};
use tpd_arc::Grid;

const SEED: u64 = 993_850;
const N_DISCOVERY: usize = 60;
const N_HELDOUT: usize = 20;
const ITEMS: usize = 60;

struct Rng(u64);

impl Rng {
    // splitmix64 — deterministic, dependency-free
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn gen_program(rng: &mut Rng) -> Vec<UOp> {
    let r = rng.below(100);
    let len = if r < 50 { 1 } else if r < 80 { 2 } else { 3 };
    (0..len).map(|_| U_ALL[rng.below(U_ALL.len())]).collect()
}

/// Random connected blob of `size` cells; cells must not be 4-adjacent
/// to any existing non-bg cell (distinct components guaranteed).
fn place_object(g: &mut Grid, size: usize, color: u8, rng: &mut Rng)
                -> bool {
    let (h, w) = (g.h, g.w);
    'tries: for _ in 0..200 {
        let sy = rng.below(h);
        let sx = rng.below(w);
        let clear = |g: &Grid, y: usize, x: usize| {
            if g.at(y, x) != 0 {
                return false;
            }
            let nbrs = [(y.wrapping_sub(1), x), (y + 1, x),
                        (y, x.wrapping_sub(1)), (y, x + 1)];
            nbrs.into_iter()
                .all(|(ny, nx)| ny >= h || nx >= w || g.at(ny, nx) == 0)
        };
        if !clear(g, sy, sx) {
            continue;
        }
        let mut cells = vec![(sy, sx)];
        while cells.len() < size {
            // grow from a random owned cell
            let mut grew = false;
            for _ in 0..40 {
                let &(cy, cx) = &cells[rng.below(cells.len())];
                let dirs = [(1i8, 0i8), (-1, 0), (0, 1), (0, -1)];
                let (dy, dx) = dirs[rng.below(4)];
                let (ny, nx) = (cy as isize + dy as isize,
                                cx as isize + dx as isize);
                if ny < 0 || nx < 0 {
                    continue;
                }
                let (ny, nx) = (ny as usize, nx as usize);
                if ny >= h || nx >= w || cells.contains(&(ny, nx)) {
                    continue;
                }
                // adjacency check against OTHER objects only
                let nbrs = [(ny.wrapping_sub(1), nx), (ny + 1, nx),
                            (ny, nx.wrapping_sub(1)), (ny, nx + 1)];
                let ok = g.at(ny, nx) == 0
                    && nbrs.into_iter().all(|(ay, ax)| {
                        ay >= h || ax >= w || g.at(ay, ax) == 0
                    });
                if ok {
                    cells.push((ny, nx));
                    grew = true;
                    break;
                }
            }
            if !grew {
                continue 'tries;
            }
        }
        for &(y, x) in &cells {
            g.set(y, x, color);
        }
        return true;
    }
    false
}

fn gen_input(rng: &mut Rng) -> Option<Grid> {
    let h = 6 + rng.below(13);
    let w = 6 + rng.below(13);
    let mut g = Grid::new(h, w, 0);
    let n_colors = 2 + rng.below(3);
    let mut palette: Vec<u8> = Vec::new();
    while palette.len() < n_colors {
        let c = 1 + rng.below(9) as u8;
        if !palette.contains(&c) {
            palette.push(c);
        }
    }
    let n_objects = 3 + rng.below(5);
    for _ in 0..n_objects {
        let size = 1 + rng.below(9);
        let color = palette[rng.below(palette.len())];
        if !place_object(&mut g, size, color, rng) {
            return None;
        }
    }
    Some(g)
}

fn grid_json(g: &Grid) -> serde_json::Value {
    serde_json::json!(g.to_rows())
}

fn gen_family(rng: &mut Rng, id: &str) -> serde_json::Value {
    'family: loop {
        let program = gen_program(rng);
        let mut items = Vec::with_capacity(ITEMS);
        for i in 0..ITEMS {
            let mut ok = None;
            for _ in 0..100 {
                let Some(inp) = gen_input(rng) else { continue };
                let out = apply_u_chain(&inp, &program);
                if out != inp {
                    ok = Some((inp, out));
                    break;
                }
            }
            let Some((inp, out)) = ok else {
                continue 'family; // degenerate family: redraw program
            };
            let split = if i < 40 { "m2train" }
                        else if i < 50 { "m2val" } else { "probe" };
            items.push(serde_json::json!({
                "split": split,
                "input": grid_json(&inp),
                "output": grid_json(&out),
            }));
        }
        return serde_json::json!({
            "family_id": id,
            "program": program.iter().map(|&o| u_name(o)).collect::<Vec<_>>(),
            "items": items,
        });
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.len() > 2 || (args.len() == 2 && args[1] != "calib") {
        eprintln!("usage: b5_gen <out_dir> [calib]");
        exit(2);
    }
    let out = Path::new(&args[0]);
    std::fs::create_dir_all(out.join("families")).expect("mkdir");
    // calibration mode (prereg addendum 2026-08-24): 10 fresh families,
    // seed 993960, prefix calib_, disjoint from discovery/held-out
    if args.len() == 2 && args[1] == "calib" {
        let mut rng = Rng(993_960);
        let mut corpus = Sha256::new();
        for k in 0..10 {
            let id = format!("calib_{k:03}");
            let fam = gen_family(&mut rng, &id);
            let path = out.join("families").join(format!("{id}.json"));
            if path.exists() {
                eprintln!("REFUSING to overwrite {} (write-once)", path.display());
                exit(1);
            }
            let s = serde_json::to_string(&fam).unwrap();
            corpus.update(s.as_bytes());
            std::fs::write(&path, s).expect("write family");
        }
        eprintln!("calib corpus sha256 {:x}", corpus.finalize());
        return;
    }
    let mut rng = Rng(SEED);
    let mut corpus = Sha256::new();
    let mut program_counts = std::collections::BTreeMap::new();
    for k in 0..(N_DISCOVERY + N_HELDOUT) {
        let (prefix, idx) = if k < N_DISCOVERY {
            ("family", k)
        } else {
            ("heldout", k - N_DISCOVERY)
        };
        let id = format!("{prefix}_{idx:03}");
        let fam = gen_family(&mut rng, &id);
        for op in fam["program"].as_array().unwrap() {
            *program_counts.entry(op.as_str().unwrap().to_string())
                .or_insert(0usize) += 1;
        }
        let path = out.join("families").join(format!("{id}.json"));
        if path.exists() {
            eprintln!("REFUSING to overwrite {} (write-once)", path.display());
            exit(1);
        }
        let s = serde_json::to_string(&fam).unwrap();
        corpus.update(s.as_bytes());
        std::fs::write(&path, s).expect("write family");
        if (k + 1) % 10 == 0 {
            eprintln!("generated {}/{}", k + 1, N_DISCOVERY + N_HELDOUT);
        }
    }
    let digest = format!("{:x}", corpus.finalize());
    eprintln!("op usage across all programs: {program_counts:?}");
    println!("corpus sha256 {digest}");
    std::fs::write(out.join("corpus_sha256.txt"), &digest).expect("write");
}
