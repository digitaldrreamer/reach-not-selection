//! B5 M1 discovery: symbolic evidence per candidate op (manifest
//! S2-b5-001; prereg notes/b5-prereg-2026-08-23.md, F1 signed,
//! F2 pinned). For each of the 60 discovery families: exhaustively
//! enumerate programs over U (len <= 3, canonical order = U_ALL index
//! order) fitting all 10 probe items; at the minimal fitting length,
//! evidence ops = the INTERSECTION of op-sets over ALL minimal-length
//! fits (per F1: u in the canonical minimal program AND no
//! equal-length fitting program avoids u — equivalent to membership
//! in every minimal fit). Aggregate e1(u) = share of the 60 families
//! where u is load-bearing; frozen M1 bar: e1 >= 0.15.
//!
//! Usage: b5_m1 <families_dir> <out.json>

use std::collections::BTreeSet;
use std::path::Path;
use std::process::exit;

use sha2::{Digest, Sha256};
use tpd_arc::u_candidates::{apply_u_chain, UOp, U_ALL};
use tpd_arc::Grid;

fn parse_grid(v: &serde_json::Value) -> Grid {
    let rows: Vec<Vec<u8>> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_array().unwrap().iter().map(|c| c.as_u64().unwrap() as u8).collect())
        .collect();
    Grid::from_rows(&rows).expect("rectangular grid")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: b5_m1 <families_dir> <out.json>");
        exit(2);
    }
    if Path::new(&args[1]).exists() {
        eprintln!("REFUSING to overwrite existing {} (write-once)", args[1]);
        exit(1);
    }
    let mut fam_paths: Vec<_> = std::fs::read_dir(&args[0])
        .expect("families dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().starts_with("family_"))
                .unwrap_or(false)
        })
        .collect();
    fam_paths.sort();
    assert_eq!(fam_paths.len(), 60, "expected 60 discovery families");

    let mut per_family = Vec::new();
    let mut load_bearing_count = [0usize; 40];

    for path in &fam_paths {
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
        assert_eq!(probes.len(), 10, "family {fid}: expected 10 probe items");

        let fits = |ops: &[UOp]| -> bool {
            probes.iter().all(|(i, o)| &apply_u_chain(i, ops) == o)
        };

        // enumerate lengths 1..=3 in canonical order; stop at first
        // length with >= 1 fit; collect ALL fits at that length
        let mut minimal_fits: Vec<Vec<UOp>> = Vec::new();
        let mut min_len = 0usize;
        'outer: for len in 1..=3usize {
            let mut found = Vec::new();
            match len {
                1 => {
                    for &a in &U_ALL {
                        if fits(&[a]) {
                            found.push(vec![a]);
                        }
                    }
                }
                2 => {
                    for &a in &U_ALL {
                        for &b in &U_ALL {
                            if fits(&[a, b]) {
                                found.push(vec![a, b]);
                            }
                        }
                    }
                }
                _ => {
                    for &a in &U_ALL {
                        for &b in &U_ALL {
                            for &c in &U_ALL {
                                if fits(&[a, b, c]) {
                                    found.push(vec![a, b, c]);
                                }
                            }
                        }
                    }
                }
            }
            if !found.is_empty() {
                min_len = len;
                minimal_fits = found;
                break 'outer;
            }
        }

        let (canonical, evidence_ops): (Vec<String>, Vec<String>) =
            if minimal_fits.is_empty() {
                (vec![], vec![])
            } else {
                let canonical: Vec<String> =
                    minimal_fits[0].iter().map(|o| format!("{o:?}")).collect();
                let mut inter: BTreeSet<UOp> =
                    minimal_fits[0].iter().copied().collect();
                for f in &minimal_fits[1..] {
                    let s: BTreeSet<UOp> = f.iter().copied().collect();
                    inter = inter.intersection(&s).copied().collect();
                }
                for &u in &inter {
                    let idx = U_ALL.iter().position(|&x| x == u).unwrap();
                    load_bearing_count[idx] += 1;
                }
                (canonical, inter.iter().map(|o| format!("{o:?}")).collect())
            };

        eprintln!(
            "{fid}: min_len={min_len} fits={} canonical={:?} load-bearing={:?}",
            minimal_fits.len(),
            canonical,
            evidence_ops
        );
        per_family.push(serde_json::json!({
            "family_id": fid,
            "hidden_program": hidden,
            "min_fit_len": min_len,
            "n_minimal_fits": minimal_fits.len(),
            "canonical_fit": canonical,
            "load_bearing": evidence_ops,
            "unfit": minimal_fits.is_empty(),
        }));
    }

    let e1: Vec<serde_json::Value> = U_ALL
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let share = load_bearing_count[i] as f64 / 60.0;
            serde_json::json!({
                "op": format!("{u:?}"),
                "families": load_bearing_count[i],
                "e1": share,
                "passes_bar": share >= 0.15,
            })
        })
        .collect();
    let n_pass = e1.iter().filter(|r| r["passes_bar"].as_bool().unwrap()).count();
    let unfit = per_family.iter().filter(|f| f["unfit"].as_bool().unwrap()).count();

    let out = serde_json::json!({
        "manifest": "S2-b5-001",
        "stage": "M1 discovery",
        "bar": 0.15,
        "n_families": 60,
        "families_unfit": unfit,
        "n_ops_passing": n_pass,
        "e1": e1,
        "per_family": per_family,
    });
    let s = serde_json::to_string_pretty(&out).unwrap();
    std::fs::write(&args[1], &s).expect("write out");
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    eprintln!("M1 pass list: {n_pass}/40 ops clear the 0.15 bar; {unfit} families unfit");
    eprintln!("b5_m1 complete; sha256 {:x}", h.finalize());
}
