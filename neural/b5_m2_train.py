"""B5 M2 training: one small transformer per family (manifest
S2-b5-002; prereg v2; refinements DECISIONS 2026-08-23l).

Trains 60 discovery + 20 held-out family models; writes per-family
model.pt + metrics; prints the competence statistic (mean held-out
family exact-match) against the frozen 0.50 bar at the end.

Usage:
  python b5_m2_train.py --families ../data/S2-b5-001/families \
      --out ../data/S2-b5-002/m2
"""

import argparse
import json
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn

from tpd_neural.data import BOS, EOS, PAD, SEP, VOCAB_SIZE, grid_tokens
from tpd_neural.model import TinyTransformer, count_params, set_determinism

BASE_SEED = 993804
EPOCHS = 400
LR = 3e-3
LR_MIN = 3e-4


def item_tokens(item):
    gi, go = item["input"], item["output"]
    wi, hi = len(gi[0]), len(gi)
    wo, ho = len(go[0]), len(go)
    ci = bytes(c for row in gi for c in row)
    co = bytes(c for row in go for c in row)
    toks = [BOS] + grid_tokens(wi, hi, ci) + [SEP]
    q_start = len(toks)
    toks += grid_tokens(wo, ho, co) + [EOS]
    return toks, q_start


def pad_batch(seqs):
    n = max(len(s) for s in seqs)
    arr = np.full((len(seqs), n), PAD, dtype=np.int64)
    for i, s in enumerate(seqs):
        arr[i, : len(s)] = s
    return torch.from_numpy(arr)


def train_family(fam, seed, device):
    set_determinism(seed)
    train = [it for it in fam["items"] if it["split"] == "m2train"]
    val = [it for it in fam["items"] if it["split"] == "m2val"]
    enc = [item_tokens(it) for it in train]
    toks = pad_batch([t for t, _ in enc]).to(device)
    starts = [q for _, q in enc]
    tgt_mask = torch.zeros_like(toks, dtype=torch.bool)
    for i, (t, q) in enumerate(enc):
        tgt_mask[i, q : len(t)] = True

    model = TinyTransformer().to(device)
    opt = torch.optim.AdamW(model.parameters(), lr=LR)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(
        opt, T_max=EPOCHS, eta_min=LR_MIN)
    inp, tgt = toks[:, :-1], toks[:, 1:]
    mask = tgt_mask[:, 1:]
    for _ in range(EPOCHS):
        model.train()
        opt.zero_grad()
        logits = model(inp)
        loss = nn.functional.cross_entropy(
            logits[mask], tgt[mask])
        loss.backward()
        opt.step()
        sched.step()

    # greedy teacher-free decode on val items
    model.eval()
    correct = 0
    with torch.no_grad():
        for it in val:
            toks_v, q = item_tokens(it)
            prefix = torch.tensor([toks_v[:q]], dtype=torch.long,
                                  device=device)
            target = toks_v[q:]
            ok = True
            cur = prefix
            for want in target:
                logits = model(cur)
                nxt = int(logits[0, -1].argmax())
                if nxt != want:
                    ok = False
                    break
                cur = torch.cat(
                    [cur, torch.tensor([[nxt]], device=device)], dim=1)
            correct += int(ok)
    return model, float(loss.item()), correct / len(val)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--families", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--shard", default="0/1",
                    help="k/n: process families with index %% n == k "
                         "(wall-time sharding only; per-family seeds "
                         "are index-derived and unchanged)")
    args = ap.parse_args()
    shard_k, shard_n = (int(x) for x in args.shard.split("/"))
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    device = "cpu"

    fam_files = sorted(Path(args.families).glob("family_*.json")) + \
        sorted(Path(args.families).glob("heldout_*.json"))
    results = []
    for fi, fp in enumerate(fam_files):
        if fi % shard_n != shard_k:
            continue
        fam = json.loads(fp.read_text())
        name = fp.stem
        mp = out / f"{name}.pt"
        rp = out / f"{name}.json"
        if rp.exists():
            results.append(json.loads(rp.read_text()))
            continue
        idx = int(name.split("_")[1]) + (100 if name.startswith("heldout") else 0)
        model, final_loss, em = train_family(fam, BASE_SEED + idx, device)
        torch.save(model.state_dict(), mp)
        rec = {"family": name, "held_out": name.startswith("heldout"),
               "final_loss": final_loss, "val_exact_match": em,
               "params": count_params(model)}
        rp.write_text(json.dumps(rec, indent=1))
        results.append(rec)
        print(json.dumps(rec), flush=True)

    held = [r["val_exact_match"] for r in results if r["held_out"]]
    comp = sum(held) / len(held) if held else 0.0
    summary = {"competence_mean_heldout_em": comp,
               "bar": 0.50, "passes": comp >= 0.50,
               "n_models": len(results)}
    (out / "summary.json").write_text(json.dumps(summary, indent=1))
    print("COMPETENCE:", json.dumps(summary))


if __name__ == "__main__":
    main()
