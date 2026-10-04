//! ARC-AGI-2 Kaggle solver, arm-extended (B5 fold-in; DECISIONS
//! 2026-08-30c). Identical selection semantics to arc2_solve (len <= 3,
//! 1M nodes, canonical shortest-first, R1 attempt_1, next-differing-fit
//! attempt_2, identity fallback, output sanitization), with the alphabet
//! extended by an arm file's UOps appended AFTER the v4 ops in the arm
//! file's order — the exact mechanism of the registered b5_inject
//! measurement. With an empty arm the submission must be byte-identical
//! to arc2_solve's (verified before use).
//!
//! Usage: arc2_solve_t <arm.json:field> <challenges.json> <submission.json> [solutions.json]

use std::collections::HashMap;
use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::u_candidates::{apply_u, u_name, UOp, U_ALL};
use tpd_arc::{apply_v4, v4_alphabet, ArcTask, Grid, Pair, V4Op};

const MAX_LEN: usize = 3;
const ENUM_BUDGET: u64 = 1_000_000;
const COLLECT_CAP: usize = 64;

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

fn apply5_chain(g: &Grid, prog: &[Op5]) -> Grid {
    prog.iter().fold(g.clone(), |acc, &op| apply5(&acc, op))
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

struct SolveTask {
    task: ArcTask,
    test_inputs: Vec<Grid>,
}

fn load_challenges(path: &Path) -> Vec<SolveTask> {
    let raw = std::fs::read_to_string(path).expect("read challenges file");
    let root: serde_json::Value = serde_json::from_str(&raw).expect("parse challenges JSON");
    let obj = root.as_object().expect("challenges root must be an object");
    let mut ids: Vec<&String> = obj.keys().collect();
    ids.sort();
    ids.iter()
        .map(|id| {
            let t = &obj[id.as_str()];
            let train: Vec<Pair> = t["train"]
                .as_array()
                .expect("train array")
                .iter()
                .map(|p| Pair {
                    input: parse_grid(&p["input"]).expect("train input grid"),
                    output: parse_grid(&p["output"]).expect("train output grid"),
                })
                .collect();
            let test_inputs: Vec<Grid> = t["test"]
                .as_array()
                .expect("test array")
                .iter()
                .map(|p| parse_grid(&p["input"]).expect("test input grid"))
                .collect();
            let task = ArcTask {
                id: (*id).clone(),
                train,
                test: test_inputs
                    .iter()
                    .map(|g| Pair { input: g.clone(), output: Grid::new(0, 0, 0) })
                    .collect(),
            };
            SolveTask { task, test_inputs }
        })
        .collect()
}

struct EnumOut {
    collected: Vec<Vec<Op5>>,
    budget_censored: bool,
}

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
            if ns.iter().zip(targets.iter()).all(|(a, b)| a == b) && out.collected.len() < COLLECT_CAP {
                out.collected.push(prog.clone());
            }
        } else if !dfs(ops, depth_left - 1, &ns, targets, prog, budget, out) {
            prog.pop();
            return false;
        }
        prog.pop();
    }
    true
}

fn enumerate_fits(task: &ArcTask, ops: &[Op5]) -> EnumOut {
    let inputs: Vec<Grid> = task.train.iter().map(|p| p.input.clone()).collect();
    let targets: Vec<Grid> = task.train.iter().map(|p| p.output.clone()).collect();
    let mut out = EnumOut { collected: vec![], budget_censored: false };
    let mut budget = ENUM_BUDGET;
    let mut prog = Vec::with_capacity(MAX_LEN);
    for len in 1..=MAX_LEN {
        if !dfs(ops, len, &inputs, &targets, &mut prog, &mut budget, &mut out) {
            out.budget_censored = true;
            break;
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!("usage: arc2_solve_t <arm.json:field> <challenges.json> <submission.json> [solutions.json]");
        exit(2);
    }
    let (arm_path, field) = args[0].split_once(':').expect("arm.json:field");
    let arm_v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(arm_path).unwrap()).unwrap();
    let by_name: HashMap<&str, UOp> = U_ALL.iter().map(|&u| (u_name(u), u)).collect();
    let arm_ops: Vec<UOp> = arm_v[field]
        .as_array()
        .expect("arm list")
        .iter()
        .map(|x| *by_name.get(x.as_str().unwrap()).expect("known op"))
        .collect();
    eprintln!(
        "arm {} = {:?}",
        field,
        arm_ops.iter().map(|&u| u_name(u)).collect::<Vec<_>>()
    );

    let tasks = load_challenges(Path::new(&args[1]));
    eprintln!("loaded {} tasks; solving (v4+{} alphabet @1M, R1)…", tasks.len(), field);

    let solved: Vec<(String, Vec<(Grid, Grid)>, bool, bool)> = tasks
        .par_iter()
        .map(|st| {
            let mut ops: Vec<Op5> = v4_alphabet(&st.task).into_iter().map(Op5::V4).collect();
            ops.extend(arm_ops.iter().map(|&u| Op5::U(u)));
            let en = enumerate_fits(&st.task, &ops);
            let attempts: Vec<(Grid, Grid)> = st
                .test_inputs
                .iter()
                .map(|inp| {
                    let valid = |g: &Grid| g.h >= 1 && g.h <= 30 && g.w >= 1 && g.w <= 30;
                    let sane = |g: Grid, inp: &Grid| if valid(&g) { g } else { inp.clone() };
                    match en.collected.first() {
                        None => (inp.clone(), inp.clone()),
                        Some(first) => {
                            let a1 = apply5_chain(inp, first);
                            let a2 = en.collected[1..]
                                .iter()
                                .map(|p| apply5_chain(inp, p))
                                .find(|g| *g != a1)
                                .unwrap_or_else(|| a1.clone());
                            (sane(a1, inp), sane(a2, inp))
                        }
                    }
                })
                .collect();
            (st.task.id.clone(), attempts, !en.collected.is_empty(), en.budget_censored)
        })
        .collect();

    let n_with_fit = solved.iter().filter(|r| r.2).count();
    let n_censored = solved.iter().filter(|r| r.3).count();
    eprintln!("tasks with >=1 fit: {n_with_fit}; budget-censored: {n_censored}");

    let mut sub = serde_json::Map::new();
    for (id, attempts, _, _) in &solved {
        let entries: Vec<serde_json::Value> = attempts
            .iter()
            .map(|(a1, a2)| {
                serde_json::json!({ "attempt_1": a1.to_rows(), "attempt_2": a2.to_rows() })
            })
            .collect();
        sub.insert(id.clone(), serde_json::Value::Array(entries));
    }
    let out_str = serde_json::to_string(&serde_json::Value::Object(sub)).unwrap();
    std::fs::write(&args[2], &out_str).expect("write submission");
    let mut h = Sha256::new();
    h.update(out_str.as_bytes());
    eprintln!("submission written; sha256 {:x}", h.finalize());

    if let Some(sol_path) = args.get(3) {
        let raw = std::fs::read_to_string(sol_path).expect("read solutions");
        let sols: serde_json::Value = serde_json::from_str(&raw).expect("parse solutions");
        let mut task_points = 0f64;
        let mut n_tasks = 0usize;
        for (id, attempts, _, _) in &solved {
            let truth = &sols[id.as_str()];
            let truths: Vec<Grid> = truth
                .as_array()
                .expect("solutions per task must be an array of grids")
                .iter()
                .map(|g| parse_grid(g).expect("solution grid"))
                .collect();
            n_tasks += 1;
            let mut correct = 0usize;
            for (i, t) in truths.iter().enumerate() {
                if let Some((a1, a2)) = attempts.get(i) {
                    if a1 == t || a2 == t {
                        correct += 1;
                    }
                }
            }
            task_points += correct as f64 / truths.len().max(1) as f64;
        }
        eprintln!(
            "LOCAL SCORE (mean per-task fraction of test inputs matched by either attempt): {:.4} over {} tasks",
            task_points / n_tasks.max(1) as f64,
            n_tasks
        );
    }
}
