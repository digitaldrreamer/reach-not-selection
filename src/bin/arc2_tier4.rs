//! ARC-2 tier-4 conversion measurement (manifest S2-arc2-tier4-001;
//! B4 protocol §4 + Amendment A1). Per task: enumerate v3 and v4 fits
//! at k=all (len <= 3, 1M nodes each, cap 2000, canonical order);
//! R1 = first collected fit. Reports N_4 (v4@1M minus v3@1M), C_4,
//! FP_4, regression guards, and v3@1M reach (for the delta against the
//! recorded v3@100k = 52). Test outputs are used only for scoring.
//!
//! Usage: arc2_tier4 <task_dir> <out.json>

use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::{
    apply_v3, apply_v3_chain, apply_v4, apply_v4_chain, load_dir, v3_alphabet, v4_alphabet,
    ArcTask, Grid, V3Op, V4Op,
};

const MAX_LEN: usize = 3;
const CAP: usize = 2_000;
const ENUM_BUDGET: u64 = 1_000_000;

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
        eprintln!("usage: arc2_tier4 <task_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let tasks = load_dir(Path::new(&args[0])).expect("load task dir");
    eprintln!("loaded {} tasks; tier-4 measurement @1M…", tasks.len());

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .map(|t| {
            let ops3 = v3_alphabet(t);
            let ops4 = v4_alphabet(t);
            let e3 = enumerate(t, &ops3, &|g, op: V3Op| apply_v3(g, op));
            let e4 = enumerate(t, &ops4, &|g, op: V4Op| apply_v4(g, op));
            let r1_v3_solves = e3.collected.first().map(|p| {
                t.test.iter().all(|pair| apply_v3_chain(&pair.input, p) == pair.output)
            });
            let r1_v4_solves = e4.collected.first().map(|p| {
                t.test.iter().all(|pair| apply_v4_chain(&pair.input, p) == pair.output)
            });
            let any_v4_solves = e4
                .collected
                .iter()
                .any(|p| t.test.iter().all(|pair| apply_v4_chain(&pair.input, p) == pair.output));
            serde_json::json!({
                "id": t.id,
                "v3_reached": e3.fitting_total > 0,
                "v4_reached": e4.fitting_total > 0,
                "v3_r1_solves": r1_v3_solves,
                "v4_r1_solves": r1_v4_solves,
                "v4_any_solves": any_v4_solves,
                "v3_censored": e3.budget_censored,
                "v4_censored": e4.budget_censored,
                "v4_fitting_total": e4.fitting_total,
            })
        })
        .collect();

    let b = |v: &serde_json::Value, k: &str| v[k].as_bool().unwrap_or(false);
    let n_v3 = rows.iter().filter(|r| b(r, "v3_reached")).count();
    let n_v4 = rows.iter().filter(|r| b(r, "v4_reached")).count();
    let newly: Vec<&serde_json::Value> =
        rows.iter().filter(|r| b(r, "v4_reached") && !b(r, "v3_reached")).collect();
    let n4 = newly.len();
    let c4_num = newly.iter().filter(|r| b(r, "v4_r1_solves")).count();
    let fp4 = newly.iter().filter(|r| !b(r, "v4_r1_solves")).count();
    let regress_reach =
        rows.iter().filter(|r| b(r, "v3_reached") && !b(r, "v4_reached")).count();
    let regress_r1: Vec<&str> = rows
        .iter()
        .filter(|r| {
            r["v3_r1_solves"].as_bool() == Some(true) && r["v4_r1_solves"].as_bool() == Some(false)
        })
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    let censored4 = rows.iter().filter(|r| b(r, "v4_censored")).count();
    let censored3 = rows.iter().filter(|r| b(r, "v3_censored")).count();

    let summary = serde_json::json!({
        "manifest": "S2-arc2-tier4-001",
        "n_tasks": rows.len(),
        "v3_reached_at_1M": n_v3,
        "v3_reached_at_100k_recorded": 52,
        "v4_reached_at_1M": n_v4,
        "N_4_newly_reached": n4,
        "C_4_r1_conversion": if n4 > 0 { c4_num as f64 / n4 as f64 } else { 0.0 },
        "C_4_numerator": c4_num,
        "FP_4": fp4,
        "regression_reach_lost": regress_reach,
        "regression_r1_broken": regress_r1,
        "v3_budget_censored_at_1M": censored3,
        "v4_budget_censored_at_1M": censored4,
    });
    eprintln!("summary: {summary}");

    let out = serde_json::json!({ "summary": summary, "rows": rows });
    let s = serde_json::to_string(&out).unwrap();
    std::fs::write(&args[1], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("arc2_tier4 complete; sha256 {:x}", h.finalize());
}
