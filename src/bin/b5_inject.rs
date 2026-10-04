//! B5 injection measurement (prereg v2 + addendum; manifest
//! S2-b5-inject-001). One arm per invocation: extends the v4 alphabet
//! with the arm's UOps (appended AFTER v4 ops, in the arm file's
//! order) and measures the B4 §4 metric on the ARC-AGI-2 training set
//! at the A1 budget: len <= 3, 1e6 nodes/task, cap 2000, R1
//! first-fit. Reports N (newly reachable vs the recorded v4
//! baseline), new solved, FP, and the regression guard.
//!
//! Usage: b5_inject <arm.json:field> <training_dir> <tier4.json> <out.json>
//!   e.g. b5_inject data/S2-b5-002/floor_T.json:T <dir> <tier4> out_T.json

use std::collections::HashMap;
use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::u_candidates::{apply_u, u_name, UOp, U_ALL};
use tpd_arc::{apply_v4, load_dir, v4_alphabet, ArcTask, Grid, V4Op};

const MAX_LEN: usize = 3;
const CAP: usize = 2_000;
const ENUM_BUDGET: u64 = 1_000_000;

#[derive(Clone, Copy, PartialEq)]
enum Op5 {
    V4(V4Op),
    U(UOp),
}

fn apply5(g: &Grid, op: Op5) -> Grid {
    match op {
        Op5::V4(o) => apply_v4(g, o),
        Op5::U(o) => apply_u(g, o),
    }
}

struct EnumOut {
    fitting_total: u64,
    collected: Vec<Vec<Op5>>,
    censored: bool,
}

// Canonical B4 §4 enumeration: shortest-first, one pass per exact
// length, fit checked ONLY at the target length (identical structure
// to arc2_fpcurve2 / arc2_depth4 — R1 semantics depend on this order).
fn dfs(
    ops: &[Op5],
    depth_left: usize,
    states: &[Grid],
    targets: &[Grid],
    prog: &mut Vec<Op5>,
    budget: &mut u64,
    out: &mut EnumOut,
) -> bool {
    for &op in ops {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let ns: Vec<Grid> = states.iter().map(|g| apply5(g, op)).collect();
        prog.push(op);
        if depth_left == 1 {
            if ns.iter().zip(targets.iter()).all(|(a, b)| a == b) {
                out.fitting_total += 1;
                if out.collected.len() < CAP {
                    out.collected.push(prog.clone());
                }
            }
        } else if !dfs(ops, depth_left - 1, &ns, targets, prog, budget, out) {
            prog.pop();
            return false;
        }
        prog.pop();
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        eprintln!("usage: b5_inject <arm.json:field> <training_dir> <tier4.json> <out.json>");
        exit(2);
    }
    if Path::new(&args[3]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[3]);
        exit(1);
    }
    let (arm_path, field) = args[0].split_once(':').expect("arm.json:field");
    let arm_v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(arm_path).unwrap()).unwrap();
    let by_name: HashMap<&str, UOp> =
        U_ALL.iter().map(|&u| (u_name(u), u)).collect();
    let arm_ops: Vec<UOp> = arm_v[field]
        .as_array()
        .expect("arm list")
        .iter()
        .map(|x| *by_name.get(x.as_str().unwrap()).expect("known op"))
        .collect();
    eprintln!("arm {} = {:?}", field,
              arm_ops.iter().map(|&u| u_name(u)).collect::<Vec<_>>());

    let base_v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let mut base: HashMap<String, (bool, bool)> = HashMap::new();
    for r in base_v["rows"].as_array().unwrap() {
        base.insert(
            r["id"].as_str().unwrap().to_string(),
            (
                r["v4_reached"].as_bool().unwrap_or(false),
                r["v4_r1_solves"].as_bool().unwrap_or(false),
            ),
        );
    }

    let tasks = load_dir(Path::new(&args[1])).expect("training dir");
    eprintln!("loaded {} tasks; arm measurement @1M…", tasks.len());
    let done = std::sync::atomic::AtomicUsize::new(0);

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .map(|t: &ArcTask| {
            let mut ops: Vec<Op5> =
                v4_alphabet(t).into_iter().map(Op5::V4).collect();
            ops.extend(arm_ops.iter().map(|&u| Op5::U(u)));
            let inputs: Vec<Grid> = t.train.iter().map(|p| p.input.clone()).collect();
            let targets: Vec<Grid> = t.train.iter().map(|p| p.output.clone()).collect();
            let mut out = EnumOut { fitting_total: 0, collected: vec![], censored: false };
            let mut budget = ENUM_BUDGET;
            let mut prog = Vec::with_capacity(MAX_LEN);
            for len in 1..=MAX_LEN {
                if !dfs(&ops, len, &inputs, &targets, &mut prog, &mut budget, &mut out) {
                    out.censored = true;
                    break;
                }
            }
            let r1 = out.collected.first().map(|p| {
                t.test.iter().all(|pair| {
                    p.iter().fold(pair.input.clone(), |acc, &op| apply5(&acc, op))
                        == pair.output
                })
            });
            let (b_reach, b_solve) = base.get(&t.id).copied().unwrap_or((false, false));
            let k = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if k % 50 == 0 {
                eprintln!("[{k}/1000]");
            }
            serde_json::json!({
                "id": t.id,
                "fitting_total": out.fitting_total,
                "reached": out.fitting_total > 0,
                "r1_solves": r1,
                "censored": out.censored,
                "base_reached": b_reach,
                "base_solved": b_solve,
            })
        })
        .collect();

    let reached = rows.iter().filter(|r| r["reached"] == true).count();
    let new_reach = rows.iter().filter(|r| r["reached"] == true && r["base_reached"] == false).count();
    let solved = rows.iter().filter(|r| r["r1_solves"] == true).count();
    let new_solved = rows.iter()
        .filter(|r| r["r1_solves"] == true && r["base_solved"] == false)
        .count();
    let regressions: Vec<&str> = rows.iter()
        .filter(|r| r["base_solved"] == true && r["r1_solves"] != true)
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    let censored = rows.iter().filter(|r| r["censored"] == true).count();

    let summary = serde_json::json!({
        "arm": field, "n_tasks": rows.len(), "reached": reached,
        "new_reach": new_reach, "solved": solved, "new_solved": new_solved,
        "regressions": regressions, "regression_count": regressions.len(),
        "censored": censored,
    });
    eprintln!("summary: {summary}");
    let out_doc = serde_json::json!({"summary": summary, "rows": rows});
    let s = serde_json::to_string(&out_doc).unwrap();
    std::fs::write(&args[3], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("b5_inject {} complete; sha256 {:x}", field, h.finalize());
}
