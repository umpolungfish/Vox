"""Generate the period-finding ladder as closing IMASM words, two forms.

Form K (Krawtchouk): the weight-sector ladder the OPI word realizes, off-diagonal
sqrt((k+1)(m-k)); the period rides its diagonal deposits.

Form C (r-cycle): fork/fuse steps stepping around an r-cycle with the wrap, giving
the all-ones off-diagonal whose spectrum is 2 cos(2 pi k / r); the period is the
eigenphase gap.

vox verdict is the closure oracle: T closes, B holds a paradox, N never forked,
F is ill-typed. A word is accepted only when it reads T (or B if it carries the
hold mark) under the protocol graph.
"""
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
VOX = ROOT / "target/release/vox"

# marks
BEGIN, CLOSE = "⊢", "⊣"
FWD, REV, ID, LINK = "≻", "≺", "⊙", "⋈"
FORK, FUSE = "∈", "∋"          # S+, S-
DT, DF = "⊤", "⊥"              # deposit true / false  (D reads T)
HOLD, FIX = "⊞", "⊡"           # paradox hold, commit


def verdict(word: str) -> str:
    r = subprocess.run([str(VOX), "verdict", word], capture_output=True, text=True)
    for line in r.stdout.splitlines():
        if line.startswith("verdict"):
            return line.split()[1]
    return f"?({r.stdout.strip()}|{r.stderr.strip()})"


def form_K(rungs: int) -> str:
    """Weight-sector ladder: each rung banks a deposit in an outer frame opened
    before the computing frame, the banking that makes the OPI word close."""
    body = ""
    for _ in range(rungs):
        # outer holding frame opened, inner computing frame deposits, both fuse
        body += FORK + FORK + DT + DF + FUSE + FUSE
    return BEGIN + ID + body + HOLD + FIX + CLOSE + ID


def form_C(length: int) -> str:
    """r-cycle: fork/fuse steps stepping around the ring, deposit per step."""
    body = ""
    for _ in range(length):
        body += FORK + DT + FUSE
    return BEGIN + ID + body + HOLD + FIX + CLOSE + ID


def form_banked(rungs: int) -> str:
    """Banked rung: fork, deposit T, reverse (the clear), deposit F, fuse. The
    deposit is banked in the frame and must survive the ≺ reversal inside it,
    the OPI word's own banking site ∈⊤≺⊥∋. `banked` should read survival, not
    VACUOUS."""
    body = ""
    for _ in range(rungs):
        body += FORK + DT + REV + DF + FUSE
    return BEGIN + ID + body + HOLD + FIX + CLOSE + ID


if __name__ == "__main__":
    # form_banked is the twin: each rung banks a winding deposit across the ≺
    # reversal inside its frame, so the `banked` razor reads survival (not
    # VACUOUS). form_C and form_K carry phase but their banking is vacuous, no
    # reversal ever clears a live register. vox verdict alone cannot tell them
    # apart; the `banked` razor through the kernel is the instrument that does.
    print("form banked (winding survives the reversal), rungs 1..6:")
    for L in range(1, 7):
        w = form_banked(L)
        print(f"  rungs={L}  len={len(w)}  verdict={verdict(w)}  {w}")
    print("form C (r-cycle, phase-bearing, banking vacuous), length 1..4:")
    for L in range(1, 5):
        w = form_C(L)
        print(f"  len={L}  verdict={verdict(w)}  {w}")
    print("form K (weight-sector), rungs 1..4:")
    for L in range(1, 5):
        w = form_K(L)
        print(f"  rungs={L}  verdict={verdict(w)}  {w}")
    print("control (OPI word):", verdict("⊢⊙≻⊙∈⊤⊙≺⊥∋⋈⊙⊤⊥∈∋∈∈⊤⊥∋⋈≺⊤⊥∋∈⊤⊥∋⋈⊙⊤⊥⊞⊡⊣⊙"))
