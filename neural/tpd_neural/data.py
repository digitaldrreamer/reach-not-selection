"""Task encoding for the B5 neural arm (M2).

Grids serialize to token sequences ARC-style. The tokenizer is defined
here and only here — the neural arm shares
nothing with the symbolic solver beyond the task encoding itself (no DSL code, no
shared vocabulary beyond these tokens).

Sequence layout per example:
    BOS  <input cells, ROW after each row>  SEP  <output ...>  EOS
"""

from dataclasses import dataclass

import numpy as np
import pyarrow.parquet as pq

# Vocabulary: colors 0..9 map to token ids 0..9.
ROW = 10  # end of grid row
SEP = 11  # input/output separator
BOS = 12
EOS = 13
PAD = 14
EX = 15  # episode example separator
VOCAB_SIZE = 16


@dataclass
class Example:
    task_id: int
    input_w: int
    input_h: int
    input_cells: bytes
    output_w: int
    output_h: int
    output_cells: bytes
    planted_ids: tuple[int, ...]
    composition_depth: int


def grid_tokens(w: int, h: int, cells: bytes) -> list[int]:
    toks: list[int] = []
    for y in range(h):
        toks.extend(cells[y * w : (y + 1) * w])
        toks.append(ROW)
    return toks


def encode_example(ex: Example) -> list[int]:
    return (
        [BOS]
        + grid_tokens(ex.input_w, ex.input_h, ex.input_cells)
        + [SEP]
        + grid_tokens(ex.output_w, ex.output_h, ex.output_cells)
        + [EOS]
    )


def decode_output_grid(tokens: list[int]) -> tuple[int, int, bytes] | None:
    """Inverse of the output portion: tokens after SEP up to EOS back to
    (w, h, cells). Returns None if malformed (non-rectangular or empty)."""
    if SEP not in tokens:
        return None
    out = tokens[tokens.index(SEP) + 1 :]
    if EOS in out:
        out = out[: out.index(EOS)]
    rows: list[list[int]] = []
    row: list[int] = []
    for t in out:
        if t == ROW:
            rows.append(row)
            row = []
        elif 0 <= t <= 9:
            row.append(t)
        else:
            return None
    if row or not rows:
        return None  # trailing partial row, or nothing
    w = len(rows[0])
    if w == 0 or any(len(r) != w for r in rows):
        return None
    return w, len(rows), bytes(t for r in rows for t in r)


def encode_pair(input_w, input_h, input_cells, output_w, output_h, output_cells) -> list[int]:
    return (
        grid_tokens(input_w, input_h, input_cells)
        + [SEP]
        + grid_tokens(output_w, output_h, output_cells)
    )


def encode_episode(support: list[Example], query: Example) -> tuple[list[int], int]:
    """MLC-style episode: support pairs then the query pair, EX-separated.
    Returns (tokens, query_out_start) where query_out_start is the index
    of the first token after the query's SEP — the segment the model is
    graded on."""
    toks = [BOS]
    for ex in support:
        toks += encode_pair(ex.input_w, ex.input_h, ex.input_cells,
                            ex.output_w, ex.output_h, ex.output_cells)
        toks.append(EX)
    toks += grid_tokens(query.input_w, query.input_h, query.input_cells)
    toks.append(SEP)
    query_out_start = len(toks)
    toks += grid_tokens(query.output_w, query.output_h, query.output_cells)
    toks.append(EOS)
    return toks, query_out_start


def task_episodes(task_examples: list[Example]) -> list[tuple[list[int], int]]:
    """All rotations: each example serves once as query, others as support."""
    out = []
    for qi in range(len(task_examples)):
        support = [e for i, e in enumerate(task_examples) if i != qi]
        out.append(encode_episode(support, task_examples[qi]))
    return out


def load_examples(tasks_parquet: str) -> list[Example]:
    table = pq.read_table(tasks_parquet)
    cols = {name: table.column(name).to_pylist() for name in table.column_names}
    n = table.num_rows
    return [
        Example(
            task_id=cols["task_id"][i],
            input_w=cols["input_w"][i],
            input_h=cols["input_h"][i],
            input_cells=cols["input_cells"][i],
            output_w=cols["output_w"][i],
            output_h=cols["output_h"][i],
            output_cells=cols["output_cells"][i],
            planted_ids=tuple(
                int(x) for x in cols["planted_ids"][i].split(",") if x != ""
            ),
            composition_depth=cols["composition_depth"][i],
        )
        for i in range(n)
    ]


def batch_sequences(seqs: list[list[int]]) -> np.ndarray:
    """Right-pad with PAD to the longest sequence; returns (B, L) int64."""
    max_len = max(len(s) for s in seqs)
    out = np.full((len(seqs), max_len), PAD, dtype=np.int64)
    for i, s in enumerate(seqs):
        out[i, : len(s)] = s
    return out


def split_by_task(
    examples: list[Example], holdout_fraction: float, seed: int
) -> tuple[list[Example], list[Example]]:
    """Deterministic train/holdout split at task granularity, so no task
    leaks examples across the split."""
    task_ids = sorted({ex.task_id for ex in examples})
    rng = np.random.default_rng(seed)
    rng.shuffle(task_ids)
    n_holdout = max(1, int(len(task_ids) * holdout_fraction))
    holdout = set(task_ids[:n_holdout])
    train = [ex for ex in examples if ex.task_id not in holdout]
    held = [ex for ex in examples if ex.task_id in holdout]
    return train, held
