#!/usr/bin/env python3
"""Gödel encode, shift evaluation frames, separate, and return the source."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import time
import uuid


def rotat(bits, cut):
    """Cyclically shift an LSB-first evaluation frame by its cut position."""
    if not bits:
        return bits
    cut %= len(bits)
    return bits[cut:] + bits[:cut]


def numeral_word(bits):
    bits = bits.rstrip("0")
    if not bits:
        return "⊢⊙⊡⊣"
    body = "".join("≻⋈∈" + ("⊥" if bit == "1" else "⊤") + "∋" for bit in bits)
    return "⊢" + body + "⊙⊡⊣"


def decode_bits(bits):
    return int(bits[::-1], 2) if "1" in bits else 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("value", type=int)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    artifact_dir = root / "measurements" / ("godel_frame_" + uuid.uuid4().hex)
    artifact_dir.mkdir()
    started = time.monotonic()

    encoded = subprocess.run(
        [str(root / "target/release/godel"), "analyze", str(args.value)],
        capture_output=True, text=True, timeout=60, check=True)
    if encoded.stderr:
        raise RuntimeError(encoded.stderr)
    bits = re.search(r"^bits-le\s+([01]+)$", encoded.stdout, re.M).group(1)
    if int(bits[::-1], 2) != args.value:
        raise RuntimeError("Gödel bit frame does not decode to the source value")
    (artifact_dir / "godel_analysis.txt").write_text(encoded.stdout)

    cuts = range(len(bits))
    shifted = [rotat(bits, cut) for cut in cuts]
    commands = ["factor_membrane separate " + numeral_word(frame) for frame in shifted]
    remaining = 60 - (time.monotonic() - started)
    if remaining <= 0:
        raise TimeoutError("Gödel encoding used the 60 second execution window")
    separated = subprocess.run(
        ["/home/mrnob0dy666/imsgct/G-mOMonadOS/target/release/g-momonados"],
        input="\n".join(commands + ["quit"]) + "\n",
        capture_output=True, text=True, timeout=remaining, check=True)
    if separated.stderr:
        raise RuntimeError(separated.stderr)
    lanes = list(zip(re.findall(r"^Λ\(W\)\.p=(⊢.*?⊣)$", separated.stdout, re.M),
                     re.findall(r"^Λ\(W\)\.q=(⊢.*?⊣)$", separated.stdout, re.M)))
    if len(lanes) != len(shifted):
        raise RuntimeError(f"factor_membrane returned {len(lanes)} of {len(shifted)} frame separations")

    def lane_bits(word):
        return "".join("1" if cell[3] == "⊥" else "0"
                       for cell in re.findall(r"≻⋈∈[⊥⊤]∋", word))

    # Repeat the same native operation on each daughter word. The resulting
    # four streams are a second separation dimension for every parent ROTAT cut.
    nested_inputs = []
    for cut, (left_word, right_word) in enumerate(lanes):
        nested_inputs.extend(((cut, 0, left_word), (cut, 1, right_word)))
    nested_commands = ["factor_membrane separate " + word
                       for _, _, word in nested_inputs]
    remaining = 60 - (time.monotonic() - started)
    if remaining <= 0:
        raise TimeoutError("first separation dimension used the 60 second execution window")
    nested = subprocess.run(
        ["/home/mrnob0dy666/imsgct/G-mOMonadOS/target/release/g-momonados"],
        input="\n".join(nested_commands + ["quit"]) + "\n",
        capture_output=True, text=True, timeout=remaining, check=True)
    if nested.stderr:
        raise RuntimeError(nested.stderr)
    nested_pairs = list(zip(re.findall(r"^Λ\(W\)\.p=(⊢.*?⊣)$", nested.stdout, re.M),
                            re.findall(r"^Λ\(W\)\.q=(⊢.*?⊣)$", nested.stdout, re.M)))
    if len(nested_pairs) != len(nested_inputs):
        raise RuntimeError(f"nested separation returned {len(nested_pairs)} of {len(nested_inputs)} pairs")

    records = []
    for cut, (frame, (left_word, right_word)) in enumerate(zip(shifted, lanes)):
        left, right = lane_bits(left_word), lane_bits(right_word)
        recomposed = "".join((left[i] if i < len(left) else "0")
                             + (right[i] if i < len(right) else "0")
                             for i in range(max(len(left), len(right))))
        recomposed = recomposed[:len(frame)].ljust(len(frame), "0")
        if recomposed.rstrip("0") != frame.rstrip("0"):
            raise RuntimeError(f"factor_membrane separation failed its Γ return at cut {cut}")
        returned = rotat(recomposed, len(recomposed) - cut)
        if returned != bits or int(returned[::-1], 2) != args.value:
            raise RuntimeError(f"inverse evaluation-frame shift failed at cut {cut}")
        # Preserve the lane's full frame width before inverse rotation: separator
        # words are canonical and omit high zero cells.
        lane_widths = ((len(frame) + 1) // 2, len(frame) // 2)
        left = left.ljust(lane_widths[0], "0")
        right = right.ljust(lane_widths[1], "0")
        left_unrotated = rotat(left, len(left) - (cut % len(left))) if left else left
        right_unrotated = rotat(right, len(right) - (cut % len(right))) if right else right
        left_value, right_value = decode_bits(left), decode_bits(right)
        left_unrotated_value = decode_bits(left_unrotated)
        right_unrotated_value = decode_bits(right_unrotated)
        nested_values = []
        for child_index in (2 * cut, 2 * cut + 1):
            child_frame_width = lane_widths[child_index % 2]
            child_words = nested_pairs[child_index]
            child_widths = ((child_frame_width + 1) // 2, child_frame_width // 2)
            child_bits = [lane_bits(word).ljust(width, "0")
                          for word, width in zip(child_words, child_widths)]
            nested_values.extend(decode_bits(child) for child in child_bits)
        nested_product = 1
        for child_value in nested_values:
            nested_product *= child_value
        nested_inverse_values = [decode_bits(rotat(
            format(child_value, "b")[::-1].rjust(width, "0"),
            width - (cut % width))) if width else 0
            for child_value, width in zip(nested_values,
                ((lane_widths[0] + 1) // 2, lane_widths[0] // 2,
                 (lane_widths[1] + 1) // 2, lane_widths[1] // 2))]
        nested_inverse_product = 1
        for child_value in nested_inverse_values:
            nested_inverse_product *= child_value
        records.append({"cut": cut, "left_bits": len(left), "right_bits": len(right),
                        "left_lane_value": str(left_value),
                        "right_lane_value": str(right_value),
                        "lane_product_closes": left_value * right_value == args.value,
                        "left_inverse_rotat_value": str(left_unrotated_value),
                        "right_inverse_rotat_value": str(right_unrotated_value),
                        "inverse_rotat_product_closes": left_unrotated_value * right_unrotated_value == args.value,
                        "nested_lane_values": [str(value) for value in nested_values],
                        "nested_lane_product": str(nested_product),
                        "nested_lane_product_closes": nested_product == args.value,
                        "nested_inverse_rotat_values": [str(value) for value in nested_inverse_values],
                        "nested_inverse_rotat_product_closes": nested_inverse_product == args.value,
                        "left_lane_word": numeral_word(left),
                        "right_lane_word": numeral_word(right),
                        "separation_recomposes_frame": True,
                        "inverse_frame_returns_source": True})

    (artifact_dir / "frames.json").write_text(json.dumps(records, indent=2) + "\n")
    report = {"value": str(args.value), "godel_bits": len(bits),
              "closed_frames": len(records), "all_frames_closed": len(records) == len(bits),
              "tested_cuts": len(records),
              "lane_product_closure_cuts": [r["cut"] for r in records if r["lane_product_closes"]],
              "inverse_rotat_product_closure_cuts": [r["cut"] for r in records if r["inverse_rotat_product_closes"]],
              "nested_lane_product_closure_cuts": [r["cut"] for r in records if r["nested_lane_product_closes"]],
              "nested_inverse_rotat_product_closure_cuts": [r["cut"] for r in records if r["nested_inverse_rotat_product_closes"]],
              "first_separated_values": {key: records[0][key] for key in
                                   ("left_lane_value", "right_lane_value")},
              "elapsed_seconds": time.monotonic() - started,
              "artifacts": str(artifact_dir)}
    (artifact_dir / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
