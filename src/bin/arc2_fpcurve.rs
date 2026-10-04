//! ARC-2 false-positive curve (manifest S2-arc2-fpcurve-001): the
//! expressive-tier ambiguity measurement. Per task and per k in
//! {1, 2, all}: enumerate v1 programs fitting the first k train
//! pairs, score the hazard (test-wrong share) and four selection
//! rules. Deterministic; test outputs used only for scoring.
//!
//! Usage: arc2_fpcurve <task_dir> <out.json>

use std::collections::HashMap;
use std::path::Path;
use std::process::exit;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tpd_arc::{apply_v1_chain, load_dir, v1_alphabet, v1_fits_all_train, v1_solves_test,
              ArcTask, Grid, V1Op};

const MAX_LEN: usize = 3;
const CAP: usize = 2_000;
const ENUM_BUDGET: u64 = 100_000;
const TIE_SEED: u64 = 991_800;

struct EnumOut {
    fitting_total: u64,
    collected: Vec<Vec<V1Op>>,
    cap_censored: bool,
    budget_censored: bool,
}

fn dfs(
    ops: &[V1Op],
    depth_left: usize,
    states: &[Grid],
    targets: &[Grid],
    prog: &mut Vec<V1Op>,
    budget: &mut u64,
    out: &mut EnumOut,
) -> bool {
    for &op in ops {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let ns: Vec<Grid> = states.iter().map(|g| tpd_arc::apply_v1(g, op)).collect();
        prog.push(op);
        if depth_left == 1 {
            if ns.iter().zip(targets.iter()).all(|(a, b)| a == b) {
                out.fitting_total += 1;
                if out.collected.len() < CAP {
                    out.collected.push(prog.clone());
                } else {
                    out.cap_censored = true;
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

fn enumerate_fits(task: &ArcTask, ops: &[V1Op], k: usize) -> EnumOut {
    let inputs: Vec<Grid> = task.train.iter().take(k).map(|p| p.input.clone()).collect();
    let targets: Vec<Grid> = task.train.iter().take(k).map(|p| p.output.clone()).collect();
    let mut out = EnumOut {
        fitting_total: 0,
        collected: vec![],
        cap_censored: false,
        budget_censored: false,
    };
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

/// Simple deterministic LCG for tie-breaks (avoids a rand dep here;
/// the stream is a pure function of the seed).
struct Lcg(u64);
impl Lcg {
    fn pick(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

fn pick_shortest(idx: &[usize], lens: &[usize], rng: &mut Lcg) -> usize {
    let min_len = idx.iter().map(|&i| lens[i]).min().unwrap();
    let class: Vec<usize> = idx.iter().copied().filter(|&i| lens[i] == min_len).collect();
    class[rng.pick(class.len())]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: arc2_fpcurve <task_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let tasks = load_dir(Path::new(&args[0])).expect("load task dir");
    eprintln!("loaded {} tasks; running v1 fp-curve…", tasks.len());

    let rows: Vec<serde_json::Value> = tasks
        .par_iter()
        .enumerate()
        .map(|(ti, t)| {
            let ops = v1_alphabet(t);
            // visible grids for screens: train inputs + test inputs
            let visible: Vec<&Grid> = t
                .train
                .iter()
                .map(|p| &p.input)
                .chain(t.test.iter().map(|p| &p.input))
                .collect();
            let ks: Vec<(String, usize)> = vec![
                ("1".into(), 1.min(t.train.len())),
                ("2".into(), 2.min(t.train.len())),
                ("all".into(), t.train.len()),
            ];
            let per_k: Vec<serde_json::Value> = ks
                .iter()
                .map(|(label, k)| {
                    let e = enumerate_fits(t, &ops, *k);
                    let lens: Vec<usize> = e.collected.iter().map(|p| p.len()).collect();
                    let test_ok: Vec<bool> =
                        e.collected.iter().map(|p| v1_solves_test(t, p)).collect();
                    let train_ok: Vec<bool> =
                        e.collected.iter().map(|p| v1_fits_all_train(t, p)).collect();
                    let n = e.collected.len();
                    let n_test_ok = test_ok.iter().filter(|&&b| b).count();
                    if n == 0 {
                        return serde_json::json!({
                            "k": label, "fitting_total": 0, "collected": 0,
                        });
                    }
                    let mut rng = Lcg(TIE_SEED ^ ((ti as u64) << 8) ^ *k as u64);
                    let all_idx: Vec<usize> = (0..n).collect();
                    let r1 = 0usize;
                    let r2 = pick_shortest(&all_idx, &lens, &mut rng);
                    // screens on visible grids
                    let survivors: Vec<usize> = all_idx
                        .iter()
                        .copied()
                        .filter(|&ci| {
                            let prog = &e.collected[ci];
                            let outs: Vec<Grid> =
                                visible.iter().map(|g| apply_v1_chain(g, prog)).collect();
                            let identity =
                                outs.iter().zip(visible.iter()).all(|(o, g)| &o == g);
                            let constant = outs.iter().all(|o| {
                                o.c.iter().collect::<std::collections::HashSet<_>>().len()
                                    <= 1
                            });
                            !identity && !constant
                        })
                        .collect();
                    let (r3, r3_fell_back) = if survivors.is_empty() {
                        (r2, true)
                    } else {
                        (pick_shortest(&survivors, &lens, &mut rng), false)
                    };
                    let (r4, r4_fell_back) = if survivors.is_empty() {
                        (r3, true)
                    } else {
                        let mut clusters: HashMap<Vec<Grid>, Vec<usize>> = HashMap::new();
                        for &ci in &survivors {
                            let key: Vec<Grid> = t
                                .test
                                .iter()
                                .map(|p| apply_v1_chain(&p.input, &e.collected[ci]))
                                .collect();
                            clusters.entry(key).or_default().push(ci);
                        }
                        let best = clusters
                            .values()
                            .max_by_key(|m| (m.len(), std::cmp::Reverse(m[0])))
                            .unwrap()
                            .clone();
                        (pick_shortest(&best, &lens, &mut rng), false)
                    };
                    let rule = |ci: usize| {
                        serde_json::json!({
                            "len": lens[ci], "test_ok": test_ok[ci],
                            "all_train_ok": train_ok[ci],
                        })
                    };
                    let mut r3j = rule(r3);
                    r3j["fell_back"] = serde_json::json!(r3_fell_back);
                    let mut r4j = rule(r4);
                    r4j["fell_back"] = serde_json::json!(r4_fell_back);
                    serde_json::json!({
                        "k": label,
                        "fitting_total": e.fitting_total,
                        "collected": n,
                        "cap_censored": e.cap_censored,
                        "budget_censored": e.budget_censored,
                        "n_test_ok": n_test_ok,
                        "hazard": 1.0 - n_test_ok as f64 / n as f64,
                        "rules": { "r1": rule(r1), "r2": rule(r2), "r3": r3j, "r4": r4j },
                    })
                })
                .collect();
            serde_json::json!({ "id": t.id, "n_train": t.train.len(), "per_k": per_k })
        })
        .collect();

    // aggregates per k label
    let agg: Vec<serde_json::Value> = ["1", "2", "all"]
        .iter()
        .enumerate()
        .map(|(ki, label)| {
            let cells: Vec<&serde_json::Value> = rows
                .iter()
                .map(|r| &r["per_k"][ki])
                .filter(|c| c["collected"].as_u64().unwrap_or(0) > 0)
                .collect();
            let nf = cells.len();
            let acc = |rule: &str| -> f64 {
                if nf == 0 {
                    return 0.0;
                }
                cells
                    .iter()
                    .filter(|c| c["rules"][rule]["test_ok"] == serde_json::json!(true))
                    .count() as f64
                    / nf as f64
            };
            let mean_hazard = if nf == 0 {
                0.0
            } else {
                cells.iter().map(|c| c["hazard"].as_f64().unwrap()).sum::<f64>() / nf as f64
            };
            serde_json::json!({
                "k": label,
                "n_tasks_with_fit": nf,
                "mean_hazard": mean_hazard,
                "acc": { "r1": acc("r1"), "r2": acc("r2"),
                         "r3": acc("r3"), "r4": acc("r4") },
                "n_budget_censored": rows
                    .iter()
                    .filter(|r| r["per_k"][ki]["budget_censored"] == serde_json::json!(true))
                    .count(),
            })
        })
        .collect();
    eprintln!("aggregates: {}", serde_json::to_string(&agg).unwrap());

    let payload = serde_json::json!({
        "manifest": "S2-arc2-fpcurve-001",
        "params": { "max_len": MAX_LEN, "cap": CAP, "enum_budget": ENUM_BUDGET,
                    "tie_seed": TIE_SEED, "palette_cap": 5 },
        "aggregates_per_k": agg,
        "rows": rows,
    });
    let bytes = serde_json::to_vec(&payload).unwrap();
    let sha: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    std::fs::write(&args[1], &bytes).expect("write");
    println!("arc2_fpcurve complete; sha256 {sha}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpd_arc::Pair;

    #[test]
    fn k1_admits_more_than_kall() {
        // a task where k=1 is ambiguous (mirror vs rot180 on a
        // symmetric first pair) but k=2 pins it
        let task = ArcTask {
            id: "t".into(),
            train: vec![
                Pair { input: vec![vec![1, 1], vec![2, 2]], output: vec![vec![2, 2], vec![1, 1]] },
                Pair { input: vec![vec![1, 3], vec![2, 4]], output: vec![vec![2, 4], vec![1, 3]] },
            ],
            test: vec![
                Pair { input: vec![vec![5, 6], vec![7, 8]], output: vec![vec![7, 8], vec![5, 6]] },
            ],
        };
        let ops = v1_alphabet(&task);
        let e1 = enumerate_fits(&task, &ops, 1);
        let e2 = enumerate_fits(&task, &ops, 2);
        assert!(e1.fitting_total >= e2.fitting_total);
        assert!(e2.fitting_total >= 1);
    }
}
