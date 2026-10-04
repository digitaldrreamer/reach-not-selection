//! ARC-2 depth-4 eval probe (manifest S2-arc2-depth4-001; B4
//! Amendment A2). v4 alphabet, len <= 4, 1e8 nodes (exhaustive for the
//! <= ~80-op alphabet), public evaluation set only. Reports per-task
//! reach at each depth 1..4 and whether R1 solves the held-back test.
//! Attribution question: is the len<=3 exhaustive eval-zero a depth
//! limit or a vocabulary limit?
//!
//! Usage: arc2_depth4 <eval_task_dir> <out.json>

use std::path::Path;
use std::process::exit;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::{apply_v4, apply_v4_chain, load_dir, v4_alphabet, ArcTask, Grid, V4Op};

const MAX_LEN: usize = 4;
const CAP: usize = 2_000;
const ENUM_BUDGET: u64 = 100_000_000;

struct EnumOut {
    fitting_by_len: [u64; MAX_LEN],
    collected: Vec<Vec<V4Op>>,
    budget_censored: bool,
}

fn dfs(
    ops: &[V4Op],
    depth: usize,
    depth_left: usize,
    states: &[Grid],
    targets: &[Grid],
    prog: &mut Vec<V4Op>,
    budget: &mut u64,
    out: &mut EnumOut,
) -> bool {
    for &op in ops {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let ns: Vec<Grid> = states.iter().map(|g| apply_v4(g, op)).collect();
        prog.push(op);
        if depth_left == 1 {
            if ns.iter().zip(targets.iter()).all(|(a, b)| a == b) {
                out.fitting_by_len[depth - 1] += 1;
                if out.collected.len() < CAP {
                    out.collected.push(prog.clone());
                }
            }
        } else if !dfs(ops, depth, depth_left - 1, &ns, targets, prog, budget, out) {
            prog.pop();
            return false;
        }
        prog.pop();
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: arc2_depth4 <eval_task_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let tasks = load_dir(Path::new(&args[0])).expect("load eval task dir");
    eprintln!("loaded {} eval tasks; depth-4 probe @1e8…", tasks.len());
    let n_total = tasks.len();
    let done = AtomicUsize::new(0);

    // incremental journal: one JSON row per completed task, appended
    // and flushed as it finishes. On restart, completed tasks are
    // loaded from the journal and skipped — a kill costs only the
    // in-flight tasks. Rows are deterministic per task, so the final
    // assembled output is identical regardless of interruption
    // history.
    let journal_path = format!("{}.journal.jsonl", &args[1]);
    let mut prior: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    if let Ok(text) = std::fs::read_to_string(&journal_path) {
        for line in text.lines() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(id) = v["id"].as_str() {
                    prior.insert(id.to_string(), v);
                }
            }
        }
        eprintln!("journal: {} tasks already complete, resuming", prior.len());
        done.store(prior.len(), Ordering::Relaxed);
    }
    let journal = Mutex::new(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&journal_path)
            .expect("open journal"),
    );

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .map(|t| {
            if let Some(v) = prior.get(&t.id) {
                return v.clone();
            }
            let ops = v4_alphabet(t);
            let inputs: Vec<Grid> = t.train.iter().map(|p| p.input.clone()).collect();
            let targets: Vec<Grid> = t.train.iter().map(|p| p.output.clone()).collect();
            // Prefix-parallel enumeration (2026-08-25): identical
            // semantics to the serial per-length DFS — units are the
            // (op1, op2) prefixes, merged in lexicographic prefix
            // order, so per-length counts and the canonical first
            // fit are reproduced exactly (byte-verified against the
            // serial binary on fits-bearing training tasks before
            // adoption). Budget bookkeeping: node total is summed
            // across units and compared to ENUM_BUDGET post hoc;
            // the budget never binds at these alphabet sizes
            // (n_ops^4 << 1e8), which the census confirms.
            let n_ops = ops.len();
            let mut out = EnumOut {
                fitting_by_len: [0; MAX_LEN],
                collected: vec![],
                budget_censored: false,
            };
            let mut nodes_total: u64 = 0;
            // Per-unit checkpointing (2026-08-29): each (op1, op2)
            // prefix unit's result is appended to a per-task side
            // journal the moment it completes, and completed units
            // are skipped on restart — an interruption costs only
            // the units in flight, not the whole task. Deterministic
            // per unit and merged by idx, so the assembled output is
            // byte-identical regardless of interruption history; the
            // side journal is deleted once the task row reaches the
            // main journal. Programs are stored as indices into the
            // task's op vector (fixed order from v4_alphabet).
            let units_dir = format!("{}.units", &args[1]);
            let _ = std::fs::create_dir_all(&units_dir);
            let units_path = format!("{}/{}.jsonl", units_dir, t.id);
            let mut prior_units: std::collections::HashMap<usize, serde_json::Value> =
                std::collections::HashMap::new();
            if let Ok(text) = std::fs::read_to_string(&units_path) {
                for line in text.lines() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(idx) = v["idx"].as_u64() {
                            prior_units.insert(idx as usize, v);
                        }
                    }
                }
                eprintln!("{}: {} units already complete, resuming", t.id, prior_units.len());
            }
            let unit_journal = Mutex::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&units_path)
                    .expect("open unit journal"),
            );
            let units_done = AtomicUsize::new(prior_units.len());
            let op_index = |op: V4Op| ops.iter().position(|&o| o == op).unwrap();
            // length 1: serial, canonical order
            {
                let mut prog = Vec::with_capacity(MAX_LEN);
                let mut budget = ENUM_BUDGET;
                if !dfs(&ops, 1, 1, &inputs, &targets, &mut prog, &mut budget, &mut out) {
                    out.budget_censored = true;
                }
                nodes_total += ENUM_BUDGET - budget;
            }
            // lengths 2..=MAX_LEN under parallel (op1, op2) prefixes
            let mut units: Vec<(usize, EnumOut, u64)> = (0..n_ops * n_ops)
                .into_par_iter()
                .map(|idx| {
                    if let Some(v) = prior_units.get(&idx) {
                        let f = v["f"].as_array().unwrap();
                        let mut fitting_by_len = [0u64; MAX_LEN];
                        for (k, x) in f.iter().enumerate() {
                            fitting_by_len[k] = x.as_u64().unwrap();
                        }
                        let collected: Vec<Vec<V4Op>> = v["p"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|prog| {
                                prog.as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|o| ops[o.as_u64().unwrap() as usize])
                                    .collect()
                            })
                            .collect();
                        let local = EnumOut {
                            fitting_by_len,
                            collected,
                            budget_censored: v["c"].as_bool().unwrap_or(false),
                        };
                        return (idx, local, v["n"].as_u64().unwrap());
                    }
                    let (i, j) = (idx / n_ops, idx % n_ops);
                    let mut local = EnumOut {
                        fitting_by_len: [0; MAX_LEN],
                        collected: vec![],
                        budget_censored: false,
                    };
                    let mut budget = ENUM_BUDGET;
                    budget -= 1;
                    let s1: Vec<Grid> =
                        inputs.iter().map(|g| apply_v4(g, ops[i])).collect();
                    budget -= 1;
                    let s2: Vec<Grid> =
                        s1.iter().map(|g| apply_v4(g, ops[j])).collect();
                    if s2.iter().zip(targets.iter()).all(|(a, b)| a == b) {
                        local.fitting_by_len[1] += 1;
                        local.collected.push(vec![ops[i], ops[j]]);
                    }
                    for len in 3..=MAX_LEN {
                        let mut prog = vec![ops[i], ops[j]];
                        if !dfs(&ops, len, len - 2, &s2, &targets, &mut prog,
                                &mut budget, &mut local) {
                            local.budget_censored = true;
                            break;
                        }
                    }
                    let nodes = ENUM_BUDGET - budget;
                    let row = serde_json::json!({
                        "idx": idx,
                        "f": local.fitting_by_len.to_vec(),
                        "p": local
                            .collected
                            .iter()
                            .map(|prog| prog.iter().map(|&o| op_index(o)).collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        "n": nodes,
                        "c": local.budget_censored,
                    });
                    {
                        let mut f = unit_journal.lock().unwrap();
                        let _ = writeln!(f, "{row}");
                        let _ = f.flush();
                    }
                    let u = units_done.fetch_add(1, Ordering::Relaxed) + 1;
                    if u % 320 == 0 {
                        eprintln!("{}: {}/{} units", t.id, u, n_ops * n_ops);
                    }
                    (idx, local, nodes)
                })
                .collect();
            units.sort_by_key(|(idx, _, _)| *idx);
            // merge in prefix order: counts sum; collected merged by
            // (length, prefix order) to reproduce canonical order
            for len_idx in 1..MAX_LEN {
                for (_, u, _) in &units {
                    out.fitting_by_len[len_idx] += u.fitting_by_len[len_idx];
                }
            }
            for len_idx in 1..MAX_LEN {
                for (_, u, _) in &units {
                    for p in &u.collected {
                        if p.len() == len_idx + 1 && out.collected.len() < CAP {
                            out.collected.push(p.clone());
                        }
                    }
                }
            }
            // canonical global order: length ascending, prefix order
            // within length — length-1 fits (already in out.collected
            // from the serial pass) must precede all longer ones
            out.collected.sort_by_key(|p| p.len());
            for (_, u, n) in &units {
                nodes_total += n;
                if u.budget_censored {
                    out.budget_censored = true;
                }
            }
            if nodes_total > ENUM_BUDGET {
                out.budget_censored = true;
            }
            let r1_solves = out.collected.first().map(|p| {
                t.test.iter().all(|pair| apply_v4_chain(&pair.input, p) == pair.output)
            });
            let total: u64 = out.fitting_by_len.iter().sum();
            let k = done.fetch_add(1, Ordering::Relaxed) + 1;
            eprintln!("[{k}/{n_total}] {} fits={total} censored={}", t.id, out.budget_censored);
            let row = serde_json::json!({
                "id": t.id,
                "n_ops": ops.len(),
                "fitting_by_len": out.fitting_by_len.to_vec(),
                "fitting_total": total,
                "reached": total > 0,
                "r1_solves": r1_solves,
                "budget_censored": out.budget_censored,
            });
            {
                let mut f = journal.lock().unwrap();
                let _ = writeln!(f, "{row}");
                let _ = f.flush();
            }
            let _ = std::fs::remove_file(&units_path);
            row
        })
        .collect();

    let reached = rows.iter().filter(|r| r["reached"].as_bool() == Some(true)).count();
    let solved = rows.iter().filter(|r| r["r1_solves"].as_bool() == Some(true)).count();
    let censored =
        rows.iter().filter(|r| r["budget_censored"].as_bool() == Some(true)).count();
    let reached_len4_only = rows
        .iter()
        .filter(|r| {
            let f = r["fitting_by_len"].as_array().unwrap();
            r["reached"].as_bool() == Some(true)
                && f[0].as_u64() == Some(0)
                && f[1].as_u64() == Some(0)
                && f[2].as_u64() == Some(0)
        })
        .count();

    let summary = serde_json::json!({
        "manifest": "S2-arc2-depth4-001",
        "n_tasks": rows.len(),
        "reached_at_len_le4": reached,
        "reached_only_at_len4": reached_len4_only,
        "r1_solved": solved,
        "budget_censored": censored,
    });
    eprintln!("summary: {summary}");

    let out = serde_json::json!({ "summary": summary, "rows": rows });
    let s = serde_json::to_string(&out).unwrap();
    std::fs::write(&args[1], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("arc2_depth4 complete; sha256 {:x}", h.finalize());
}
