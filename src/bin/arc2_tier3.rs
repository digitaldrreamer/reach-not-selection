//! ARC-2 tier-3 conversion measurement (manifest S2-arc2-tier3-001;
//! B4 protocol §4). Per task: enumerate v2 and v3 fits at k=all
//! (len <= 3, 100k nodes each, cap 2000, canonical order); R1 = first
//! collected fit. Reports N_3 (newly reachable at v3), C_3 (share of
//! N_3 whose R1 solves the held-back test), FP_3, and the regression
//! guard (v2-reached tasks losing reach or R1-correctness at v3).
//! Test outputs are used only for scoring, never by selection.
//!
//! Usage: arc2_tier3 <task_dir> <out.json>

use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::{
    apply_v2, apply_v2_chain, apply_v3, apply_v3_chain, load_dir, v2_alphabet, v3_alphabet,
    ArcTask, Grid, V2Op, V3Op,
};

const MAX_LEN: usize = 3;
const CAP: usize = 2_000;
const ENUM_BUDGET: u64 = 100_000;

struct EnumOut<T> {
    fitting_total: u64,
    collected: Vec<Vec<T>>,
    budget_censored: bool,
}

fn dfs<T: Copy>(
    ops: &[T],
    apply: &dyn Fn(&Grid, T) -> Grid,
    depth_left: usize,
    states: &[Grid],
    targets: &[Grid],
    prog: &mut Vec<T>,
    budget: &mut u64,
    out: &mut EnumOut<T>,
) -> bool {
    for &op in ops {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let ns: Vec<Grid> = states.iter().map(|g| apply(g, op)).collect();
        prog.push(op);
        if depth_left == 1 {
            if ns.iter().zip(targets.iter()).all(|(a, b)| a == b) {
                out.fitting_total += 1;
                if out.collected.len() < CAP {
                    out.collected.push(prog.clone());
                }
            }
        } else if !dfs(ops, apply, depth_left - 1, &ns, targets, prog, budget, out) {
            prog.pop();
            return false;
        }
        prog.pop();
    }
    true
}

fn enumerate<T: Copy>(
    task: &ArcTask,
    ops: &[T],
    apply: &dyn Fn(&Grid, T) -> Grid,
) -> EnumOut<T> {
    let inputs: Vec<Grid> = task.train.iter().map(|p| p.input.clone()).collect();
    let targets: Vec<Grid> = task.train.iter().map(|p| p.output.clone()).collect();
    let mut out = EnumOut { fitting_total: 0, collected: vec![], budget_censored: false };
    let mut budget = ENUM_BUDGET;
    let mut prog = Vec::with_capacity(MAX_LEN);
    for len in 1..=MAX_LEN {
        if !dfs(ops, apply, len, &inputs, &targets, &mut prog, &mut budget, &mut out) {
            out.budget_censored = true;
            break;
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: arc2_tier3 <task_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let tasks = load_dir(Path::new(&args[0])).expect("load task dir");
    eprintln!("loaded {} tasks; tier-3 measurement…", tasks.len());

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .map(|t| {
            let ops2 = v2_alphabet(t);
            let ops3 = v3_alphabet(t);
            let e2 = enumerate(t, &ops2, &|g, op: V2Op| apply_v2(g, op));
            let e3 = enumerate(t, &ops3, &|g, op: V3Op| apply_v3(g, op));
            let r1_v2_solves = e2.collected.first().map(|p| {
                t.test.iter().all(|pair| apply_v2_chain(&pair.input, p) == pair.output)
            });
            let r1_v3_solves = e3.collected.first().map(|p| {
                t.test.iter().all(|pair| apply_v3_chain(&pair.input, p) == pair.output)
            });
            let any_v3_solves = e3
                .collected
                .iter()
                .any(|p| t.test.iter().all(|pair| apply_v3_chain(&pair.input, p) == pair.output));
            serde_json::json!({
                "id": t.id,
                "v2_reached": e2.fitting_total > 0,
                "v3_reached": e3.fitting_total > 0,
                "v2_r1_solves": r1_v2_solves,
                "v3_r1_solves": r1_v3_solves,
                "v3_any_solves": any_v3_solves,
                "v2_censored": e2.budget_censored,
                "v3_censored": e3.budget_censored,
                "v3_fitting_total": e3.fitting_total,
            })
        })
        .collect();

    let b = |v: &serde_json::Value, k: &str| v[k].as_bool().unwrap_or(false);
    let n_v2 = rows.iter().filter(|r| b(r, "v2_reached")).count();
    let n_v3 = rows.iter().filter(|r| b(r, "v3_reached")).count();
    let newly: Vec<&serde_json::Value> =
        rows.iter().filter(|r| b(r, "v3_reached") && !b(r, "v2_reached")).collect();
    let n3 = newly.len();
    let c3_num = newly.iter().filter(|r| b(r, "v3_r1_solves")).count();
    let fp3 = newly.iter().filter(|r| !b(r, "v3_r1_solves")).count();
    let regress_reach =
        rows.iter().filter(|r| b(r, "v2_reached") && !b(r, "v3_reached")).count();
    let regress_r1: Vec<&str> = rows
        .iter()
        .filter(|r| {
            r["v2_r1_solves"].as_bool() == Some(true) && r["v3_r1_solves"].as_bool() == Some(false)
        })
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    let censored3 = rows.iter().filter(|r| b(r, "v3_censored")).count();

    let summary = serde_json::json!({
        "manifest": "S2-arc2-tier3-001",
        "n_tasks": rows.len(),
        "v2_reached": n_v2,
        "v3_reached": n_v3,
        "N_3_newly_reached": n3,
        "C_3_r1_conversion": if n3 > 0 { c3_num as f64 / n3 as f64 } else { 0.0 },
        "C_3_numerator": c3_num,
        "FP_3": fp3,
        "regression_reach_lost": regress_reach,
        "regression_r1_broken": regress_r1,
        "v3_budget_censored": censored3,
    });
    eprintln!("summary: {summary}");

    let out = serde_json::json!({ "summary": summary, "rows": rows });
    let s = serde_json::to_string(&out).unwrap();
    std::fs::write(&args[1], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("arc2_tier3 complete; sha256 {:x}", h.finalize());
}
