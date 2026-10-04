//! Measured ETA for the depth-4 probe on specific tasks: samples
//! random length-4 chains (uniform over ops — matching the exhaustive
//! enumeration's per-level op distribution) and times the op
//! applications to estimate per-node cost, then extrapolates to the
//! full n + n^2 + n^3 + n^4 node count. Diagnostic only; touches no
//! registered artifact.
//!
//! Usage: depth4_eta <eval_task_dir> <task_id> [n_samples]

use std::path::Path;
use std::process::exit;
use std::time::Instant;

use tpd_arc::{apply_v4, load_dir, v4_alphabet, Grid};

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: depth4_eta <eval_task_dir> <task_id> [n_samples]");
        exit(2);
    }
    let n_samples: u64 = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(5_000);
    let tasks = load_dir(Path::new(&args[0])).expect("load dir");
    let t = tasks.iter().find(|t| t.id == args[1]).expect("task id");
    let ops = v4_alphabet(t);
    let n = ops.len() as u64;
    let inputs: Vec<Grid> = t.train.iter().map(|p| p.input.clone()).collect();

    // Leaf-focused measurement: the enumeration's cost is dominated
    // by the n^4 depth-4 applications, whose input states are grids
    // already transformed by a 3-op prefix (often grown by scale
    // ops). Sample random 3-prefixes, then time applying every op at
    // depth 4 from that state — this is the per-leaf cost directly,
    // not a chain average that dilutes it with cheap shallow levels.
    let mut rng: u64 = 0xC0FFEE;
    let mut leaf_applied: u64 = 0;
    let mut sink: u64 = 0;
    let mut leaf_secs = 0.0f64;
    for _ in 0..n_samples {
        let mut states = inputs.clone();
        for _ in 0..3 {
            let op = ops[(splitmix(&mut rng) % n) as usize];
            states = states.iter().map(|g| apply_v4(g, op)).collect();
        }
        let start = Instant::now();
        for &op in &ops {
            let ns: Vec<Grid> = states.iter().map(|g| apply_v4(g, op)).collect();
            sink ^= ns.iter().map(|g| g.c.len() as u64).sum::<u64>();
            leaf_applied += 1;
        }
        leaf_secs += start.elapsed().as_secs_f64();
    }
    let per_leaf_us = leaf_secs / leaf_applied as f64 * 1e6;
    let leaves = n * n * n * n;
    let inner = n + n * n + n * n * n;
    // inner nodes cost <= leaf cost per node is not guaranteed, but
    // they are 1.3% of nodes; bound them at 2x leaf cost.
    let core_secs = (leaves as f64 + 2.0 * inner as f64) * per_leaf_us / 1e6;
    println!(
        "task {} n_ops {} leaves {} | {:.1} us/leaf ({} sampled, sink {}) | \
         single-core {:.1} h | /9 cores {:.2} h | /8 cores {:.2} h",
        t.id, n, leaves, per_leaf_us, leaf_applied, sink & 1,
        core_secs / 3600.0, core_secs / 9.0 / 3600.0, core_secs / 8.0 / 3600.0
    );
}
