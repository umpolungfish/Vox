"""Exact native/hex numeral input shared by the Python command surfaces."""
from functools import lru_cache
from pathlib import Path
import re


@lru_cache(maxsize=1)
def hex_words():
    source = (Path(__file__).resolve().parent / "src/combined_factor.rs").read_text()
    table = source.split("pub const HEX_WORDS:", 1)[1].split("];", 1)[0]
    words = tuple(re.findall(r'"([^"\n]+)"', table))
    if len(words) != 16 or len(set(words)) != 16:
        raise ValueError("canonical hex numeral table must contain sixteen distinct motifs")
    return words


def decode_hex(word):
    word = word.strip()
    motifs = re.findall(r"[^⊣]*⊣", word)
    if not motifs or "".join(motifs) != word:
        raise ValueError("incomplete hex numeral")
    table = hex_words()
    digits = [table.index(motif) for motif in motifs]
    if len(digits) > 1 and digits[0] == 0:
        raise ValueError("hex numeral has a leading zero motif")
    value = 0
    for digit in digits:
        value = value * 16 + digit
    return value


def decode_word(word, canonical=True):
    word = word.strip()
    try:
        return decode_hex(word)
    except ValueError:
        pass
    if word == "⊢⊙⊡⊣":
        return 0
    if not word.startswith("⊢") or not word.endswith("⊙⊡⊣"):
        raise ValueError("expected a native or hex numeral word")
    body = word[1:-3]
    cells = re.findall(r"≻⋈∈([⊤⊥])∋", body)
    if not cells or "".join("≻⋈∈" + bit + "∋" for bit in cells) != body:
        raise ValueError("invalid native numeral cells")
    if canonical and cells[-1] != "⊥" and cells != ["⊤"]:
        raise ValueError("native numeral has redundant high zero cells")
    return sum((bit == "⊥") << index for index, bit in enumerate(cells))


def native_word(value):
    if value < 0:
        raise ValueError("numerals must be nonnegative")
    return "⊢" + "".join("≻⋈∈" + ("⊥" if value >> i & 1 else "⊤") + "∋"
                         for i in range(value.bit_length())) + "⊙⊡⊣"


def normalize_word(word):
    return native_word(decode_word(word))
