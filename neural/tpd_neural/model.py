"""Small decoder-only transformer used as the B5 neural arm (M2).
Written from scratch on torch primitives: no pretrained weights, no
external model code."""

import torch
import torch.nn as nn

from .data import BOS, EX, PAD, ROW, SEP, VOCAB_SIZE

# 2D positional structure ([…]):
# row/col indices are grid coordinates; NULL_POS is the dedicated slot for
# tokens that sit outside a grid cell (BOS/EOS/SEP/EX/PAD; ROW keeps the
# row it terminates but a null column). Grid dims are <= 9, so 16 leaves
# headroom without letting the table grow silently.
NULL_POS = 16
MAX_EX = 4  # example-index slots: 2 support + query + tail


def grid_positions(
    tokens: torch.Tensor,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    """Structural position of every token, derived from the tokens alone:
    (row, col, segment, example_index), each shaped like `tokens`.
    segment is 0 for input grids, 1 for output grids; example_index counts
    EX-terminated pairs. Vectorized cumsum/gather, no Python loop."""
    is_color = tokens <= 9
    is_row = tokens == ROW
    is_sep = tokens == SEP
    is_ex = tokens == EX
    is_bos = tokens == BOS

    def excl(mask: torch.Tensor) -> torch.Tensor:
        c = mask.long().cumsum(dim=1)
        return c - mask.long()

    def last_reset(mask: torch.Tensor) -> torch.Tensor:
        idx = torch.arange(tokens.shape[1], device=tokens.device)
        return (idx[None, :] * mask.long()).cummax(dim=1).values

    grid_start = is_bos | is_sep | is_ex  # a new grid begins after these
    r_excl = excl(is_row)
    c_excl = excl(is_color)
    row = r_excl - r_excl.gather(1, last_reset(grid_start))
    col = c_excl - c_excl.gather(1, last_reset(grid_start | is_row))
    seg = (excl(is_sep) - excl(is_ex)).clamp(0, 1)
    ex = excl(is_ex).clamp(0, MAX_EX - 1)
    row = torch.where(is_color | is_row, row.clamp(0, NULL_POS - 1), NULL_POS)
    col = torch.where(is_color, col.clamp(0, NULL_POS - 1), NULL_POS)
    return row, col, seg, ex


class Block(nn.Module):
    def __init__(self, d_model: int, n_head: int):
        super().__init__()
        self.ln1 = nn.LayerNorm(d_model)
        self.attn = nn.MultiheadAttention(d_model, n_head, batch_first=True)
        self.ln2 = nn.LayerNorm(d_model)
        self.mlp = nn.Sequential(
            nn.Linear(d_model, 4 * d_model),
            nn.GELU(),
            nn.Linear(4 * d_model, d_model),
        )

    def forward(self, x: torch.Tensor, causal_mask: torch.Tensor) -> torch.Tensor:
        h = self.ln1(x)
        a, _ = self.attn(h, h, h, attn_mask=causal_mask, need_weights=False)
        x = x + a
        x = x + self.mlp(self.ln2(x))
        return x


class TinyTransformer(nn.Module):
    """Decoder-only LM. `residual_layer` marks the mid-stack residual
    stream."""

    def __init__(
        self,
        d_model: int = 128,
        n_head: int = 4,
        n_layer: int = 4,
        max_len: int = 256,
    ):
        super().__init__()
        self.d_model = d_model
        self.n_layer = n_layer
        self.residual_layer = n_layer // 2
        self.tok = nn.Embedding(VOCAB_SIZE, d_model)
        # 2D positions replace the 1D absolute embedding 
        # ([…]); max_len is kept in the signature so checkpoint
        # configs stay loadable, but no longer bounds anything.
        self.row_pos = nn.Embedding(NULL_POS + 1, d_model)
        self.col_pos = nn.Embedding(NULL_POS + 1, d_model)
        self.seg_pos = nn.Embedding(2, d_model)
        self.ex_pos = nn.Embedding(MAX_EX, d_model)
        self.blocks = nn.ModuleList(Block(d_model, n_head) for _ in range(n_layer))
        self.ln_f = nn.LayerNorm(d_model)
        self.head = nn.Linear(d_model, VOCAB_SIZE, bias=False)

    def forward(
        self, tokens: torch.Tensor, return_residual: bool = False
    ) -> torch.Tensor | tuple[torch.Tensor, torch.Tensor]:
        b, l = tokens.shape
        row, col, seg, ex = grid_positions(tokens)
        x = (
            self.tok(tokens)
            + self.row_pos(row)
            + self.col_pos(col)
            + self.seg_pos(seg)
            + self.ex_pos(ex)
        )
        mask = torch.triu(
            torch.full((l, l), float("-inf"), device=tokens.device), diagonal=1
        )
        residual = None
        for i, block in enumerate(self.blocks):
            x = block(x, mask)
            if i == self.residual_layer:
                residual = x
        logits = self.head(self.ln_f(x))
        if return_residual:
            return logits, residual
        return logits

    def lm_loss(self, tokens: torch.Tensor) -> torch.Tensor:
        logits = self(tokens)
        targets = tokens[:, 1:]
        logits = logits[:, :-1]
        return nn.functional.cross_entropy(
            logits.reshape(-1, VOCAB_SIZE), targets.reshape(-1), ignore_index=PAD
        )


def count_params(model: nn.Module) -> int:
    return sum(p.numel() for p in model.parameters())


def set_determinism(seed: int) -> None:
    torch.manual_seed(seed)
    torch.use_deterministic_algorithms(True)
