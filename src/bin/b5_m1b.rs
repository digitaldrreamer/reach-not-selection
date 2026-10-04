//! B5 M1b — the SECOND independent symbolic medium (fork (a); prereg
//! addendum 2026-08-24). Stochastic greedy beam search, deliberately
//! different from M1's exhaustive shortest-first enumeration:
//! beam width 8, depth <= 3, 20 restarts/family with per-restart
//! shuffled op order, mean per-item CELL agreement scoring (partial
//! credit), FIRST-FOUND exact-fit acceptance.
//!
//! Modes:
//!   b5_m1b calib <families_dir> <out.json>   (calibration gate,
//!       families named calib_*; qualification per the addendum)
//!   b5_m1b disc  <families_dir> <out.json>   (60 discovery families)
//!
//! Seeds: 993950 + family_index*100 + restart (addendum-frozen).

use std::path::Path;
use std::process::exit;

use sha2::{Digest, Sha256};
use tpd_arc::u_candidates::{apply_u, apply_u_chain, u_name, UOp, U_ALL};
use tpd_arc::Grid;

const BEAM: usize = 8;
const DEPTH: usize = 3;
const RESTARTS: u64 = 20;
const SEED_BASE: u64 = 993_950;

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn parse_grid(v: &serde_json::Value) -> Grid {
    let rows: Vec<Vec<u8>> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_array().unwrap().iter().map(|c| c.as_u64().unwrap() as u8).collect())
        .collect();
    Grid::from_rows(&rows).expect("rectangular grid")
}

/// Mean per-item cell agreement of state grids vs targets (dims
/// mismatch scores 0 for that item).
fn agreement(states: &[Grid], targets: &[Grid]) -> f64 {
    let mut total = 0.0;
    for (s, t) in states.iter().zip(targets) {
        if s.h == t.h && s.w == t.w && s.h > 0 && s.w > 0 {
            let same = s.c.iter().zip(&t.c).filter(|(a, b)| a == b).count();
            total += same as f64 / (s.h * s.w) as f64;
        }
    }
    total / targets.len() as f64
}

/// One restart of greedy beam search. Returns first exact-fit program.
fn beam_restart(probes: &[(Grid, Grid)], order: &[UOp]) -> Option<Vec<UOp>> {
    let inputs: Vec<Grid> = probes.iter().map(|(i, _)| i.clone()).collect();
    let targets: Vec<Grid> = probes.iter().map(|(_, o)| o.clone()).collect();
    // beam entries: (program, states, score)
    let mut beam: Vec<(Vec<UOp>, Vec<Grid>, f64)> =
        vec![(vec![], inputs.clone(), agreement(&inputs, &targets))];
    for _ in 0..DEPTH {
        let mut cands: Vec<(Vec<UOp>, Vec<Grid>, f64)> = Vec::new();
        for (prog, states, _) in &beam {
            for &op in order {
                let ns: Vec<Grid> = states.iter().map(|g| apply_u(g, op)).collect();
                let mut np = prog.clone();
                np.push(op);
                if ns.iter().zip(&targets).all(|(a, b)| a == b) {
                    return Some(np); // FIRST exact fit wins (frozen)
                }
                let sc = agreement(&ns, &targets);
                cands.push((np, ns, sc));
            }
        }
        // stable sort by score desc; stability preserves op-order bias
        cands.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        cands.truncate(BEAM);
        if cands.is_empty() {
            break;
        }
        beam = cands;
    }
    None
}

fn find_fit(probes: &[(Grid, Grid)], fam_idx: u64) -> Option<Vec<UOp>> {
    for r in 0..RESTARTS {
        let mut rng = SEED_BASE + fam_idx * 100 + r;
        let mut order: Vec<UOp> = U_ALL.to_vec();
        // Fisher-Yates with splitmix
        for i in (1..order.len()).rev() {
            let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        if let Some(p) = beam_restart(probes, &order) {
            return Some(p);
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 || (args[0] != "calib" && args[0] != "disc") {
        eprintln!("usage: b5_m1b <calib|disc> <families_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[2]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[2]);
        exit(1);
    }
    let prefix = if args[0] == "calib" { "calib_" } else { "family_" };
    let mut fam_paths: Vec<_> = std::fs::read_dir(&args[1])
        .expect("families dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().map(|n| n.to_string_lossy().starts_with(prefix)).unwrap_or(false)
        })
        .collect();
    fam_paths.sort();
    eprintln!("m1b {}: {} families", args[0], fam_paths.len());

    let mut per_family = Vec::new();
    let mut found_count = [0usize; 40];
    let mut n_fit = 0usize;
    let mut n_hidden_recovered = 0usize;

    for (fi, path) in fam_paths.iter().enumerate() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let fid = v["family_id"].as_str().unwrap_or("?").to_string();
        let hidden: Vec<String> = v["program"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let probes: Vec<(Grid, Grid)> = v["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|it| it["split"] == "probe")
            .map(|it| (parse_grid(&it["input"]), parse_grid(&it["output"])))
            .collect();
        let found = find_fit(&probes, fi as u64);
        let (fit, ops_named, hidden_covered) = match &found {
            Some(p) => {
                n_fit += 1;
                debug_assert!(probes.iter().all(|(i, o)| &apply_u_chain(i, p) == o));
                let names: Vec<&str> = p.iter().map(|&op| u_name(op)).collect();
                let covered = hidden.iter().all(|h| names.contains(&h.as_str()));
                if covered {
                    n_hidden_recovered += 1;
                }
                for &op in p {
                    let idx = U_ALL.iter().position(|&u| u == op).unwrap();
                    found_count[idx] += 1;
                }
                (true, names.join(","), covered)
            }
            None => (false, String::new(), false),
        };
        per_family.push(serde_json::json!({
            "family": fid, "fit": fit, "program": ops_named,
            "hidden": hidden.join(","), "hidden_covered": hidden_covered,
        }));
        eprintln!("[{}/{}] {} fit={} covered={}", fi + 1, fam_paths.len(), fid, fit,
                  hidden_covered);
    }

    let n = fam_paths.len();
    let e1b: Vec<serde_json::Value> = U_ALL
        .iter()
        .enumerate()
        .map(|(i, &op)| {
            serde_json::json!({"op": u_name(op), "families": found_count[i],
                               "e1b": found_count[i] as f64 / n as f64})
        })
        .collect();
    let qual = if args[0] == "calib" {
        let cover_rate = if n_fit > 0 { n_hidden_recovered as f64 / n_fit as f64 } else { 0.0 };
        Some(serde_json::json!({
            "fit": n_fit, "of": n, "fit_ok": n_fit >= 8,
            "hidden_cover_rate": cover_rate, "cover_ok": cover_rate >= 0.8,
            "qualified": n_fit >= 8 && cover_rate >= 0.8,
        }))
    } else {
        None
    };
    let out = serde_json::json!({
        "mode": args[0], "n_families": n, "n_fit": n_fit,
        "qualification": qual, "e1b": e1b, "per_family": per_family,
    });
    let s = serde_json::to_string_pretty(&out).unwrap();
    std::fs::write(&args[2], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("m1b {} complete; n_fit {}/{}; sha256 {:x}", args[0], n_fit, n, h.finalize());
    if let Some(q) = &out["qualification"].as_object() {
        eprintln!("QUALIFICATION: {}", serde_json::to_string(q).unwrap());
    }
}
