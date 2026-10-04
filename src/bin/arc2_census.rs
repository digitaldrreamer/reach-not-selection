//! ARC-2 census (manifest S2-arc2-census-001, exploratory): load the
//! local ARC-AGI-2 training set, report surface statistics, and take
//! the v0 ambiguity taste: for every task, enumerate all v0-DSL
//! programs (len <= 3, canonical order) that FIT THE TRAIN PAIRS,
//! and record how many fit, whether the first fit solves the test,
//! and whether ANY fit solves the test. This is scaffolding-grade:
//! the v0 DSL is option (c)'s stand-up set, expected to fit only a
//! small fraction of tasks; the interesting number is the
//! fit-but-wrong rate WITHIN that fraction ([…];
//! DECISIONS 2026-07-24).
//!
//! Usage: arc2_census <task_dir> <out.json>

use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::{fits_train, load_dir, solves_test, ArcTask, GOp, G_OPS};

const MAX_LEN: usize = 3;

fn enumerate_fits(task: &ArcTask) -> Vec<Vec<GOp>> {
    let mut fits = Vec::new();
    // iterative deepening, canonical (length, op-order) order
    for len in 1..=MAX_LEN {
        let mut prog: Vec<GOp> = Vec::with_capacity(len);
        dfs(task, len, &mut prog, &mut fits);
    }
    fits
}

fn dfs(task: &ArcTask, depth_left: usize, prog: &mut Vec<GOp>, fits: &mut Vec<Vec<GOp>>) {
    for &op in G_OPS.iter() {
        prog.push(op);
        if depth_left == 1 {
            if fits_train(task, prog) {
                fits.push(prog.clone());
            }
        } else {
            dfs(task, depth_left - 1, prog, fits);
        }
        prog.pop();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: arc2_census <task_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let tasks = load_dir(Path::new(&args[0])).expect("load task dir");
    eprintln!("loaded {} tasks; running census + v0 ambiguity taste…", tasks.len());

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .map(|t| {
            let dims: Vec<(usize, usize)> = t
                .train
                .iter()
                .chain(t.test.iter())
                .flat_map(|p| {
                    [(p.input.h, p.input.w),
                     (p.output.h, p.output.w)]
                })
                .collect();
            let max_dim = dims.iter().map(|&(h, w)| h.max(w)).max().unwrap_or(0);
            let shape_preserving = t
                .train
                .iter()
                .all(|p| p.input.h == p.output.h && p.input.w == p.output.w);
            let fits = enumerate_fits(t);
            let first_fit_solves = fits.first().map(|f| solves_test(t, f));
            let any_fit_solves = fits.iter().any(|f| solves_test(t, f));
            let identity_out = t.train.iter().all(|p| p.input == p.output);
            serde_json::json!({
                "id": t.id,
                "n_train": t.train.len(), "n_test": t.test.len(),
                "max_dim": max_dim,
                "shape_preserving": shape_preserving,
                "identity_task": identity_out,
                "n_fits_le3": fits.len(),
                "first_fit_solves_test": first_fit_solves,
                "any_fit_solves_test": any_fit_solves,
                "first_fit": fits.first().map(|f| {
                    f.iter().map(|op| format!("{op:?}")).collect::<Vec<_>>()
                }),
            })
        })
        .collect();

    let n = rows.len();
    let fitted: Vec<&serde_json::Value> =
        rows.iter().filter(|r| r["n_fits_le3"].as_u64().unwrap() > 0).collect();
    let first_right = fitted
        .iter()
        .filter(|r| r["first_fit_solves_test"] == serde_json::json!(true))
        .count();
    let any_right = fitted
        .iter()
        .filter(|r| r["any_fit_solves_test"] == serde_json::json!(true))
        .count();
    let summary = serde_json::json!({
        "n_tasks": n,
        "n_with_v0_fit_le3": fitted.len(),
        "n_first_fit_solves_test": first_right,
        "n_any_fit_solves_test": any_right,
        "fit_but_first_wrong": fitted.len() - first_right,
        "mean_train_pairs":
            rows.iter().map(|r| r["n_train"].as_u64().unwrap() as f64).sum::<f64>() / n as f64,
        "n_shape_preserving":
            rows.iter().filter(|r| r["shape_preserving"] == serde_json::json!(true)).count(),
    });
    eprintln!("summary: {}", serde_json::to_string(&summary).unwrap());

    let payload = serde_json::json!({
        "manifest": "S2-arc2-census-001",
        "params": { "max_len": MAX_LEN, "n_ops": G_OPS.len(),
                    "task_dir_note": "ARC-AGI-2 training, github arcprize/ARC-AGI-2 main, downloaded 2026-07-24" },
        "summary": summary,
        "rows": rows,
    });
    let bytes = serde_json::to_vec(&payload).unwrap();
    let sha: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    std::fs::write(&args[1], &bytes).expect("write");
    println!("arc2_census complete; sha256 {sha}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpd_arc::Pair;

    #[test]
    fn census_finds_flip_task() {
        let task = ArcTask {
            id: "t".into(),
            train: vec![Pair { input: vec![vec![1, 2]], output: vec![vec![2, 1]] }],
            test: vec![Pair { input: vec![vec![3, 4]], output: vec![vec![4, 3]] }],
        };
        let fits = enumerate_fits(&task);
        assert!(!fits.is_empty());
        assert!(fits.iter().any(|f| solves_test(&task, f)));
        // canonical order: the shortest fit comes first
        assert_eq!(fits[0].len(), 1);
        let _ = tpd_arc::apply_gchain(&task.train[0].input, &fits[0]);
    }
}
