#!/usr/bin/env python3
"""Test the μ∘δ=id loop closure for Vox's circular pipeline.

The loop: binary → imasm_module.emit() → decompile() → word
Compare: binary → vox.recompile_module() → word
Both words must be identical: μ∘δ=id.
"""
import sys
import os

# Add Vox directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import imasm_module
import vox
import vox_decompiler


def test_round_trip(binary_path):
    """Test μ∘δ=id: decompile(emit(binary)) should produce the same word
    as vox.recompile_module(binary).

    The μ (mu, fuse) and δ (delta, split) compose to identity:
    - lift (vox) splits bytecode into IMASM glyphs
    - decompile (vox_decompiler) reconstructs the word from IMASM module text
    - These must agree on the word structure
    """
    print(f"\n=== Testing μ∘δ=id on {binary_path} ===")

    # Step 1: Get the IMASM module text via emit()
    text = imasm_module.emit(binary_path)
    print(f"  emit() produced {len(text)} chars of IMASM module text")

    # Step 2: Decompile the module text back to a word
    decompiled_word = vox_decompiler.decompile(text, binary_path)
    print(f"  decompile() produced word: {''.join(decompiled_word)}")
    print(f"  word length: {len(decompiled_word)} glyphs")

    # Step 3: Get the reference word from vox.recompile_module()
    mod = vox.recompile_module(binary_path)
    reference_word = []
    for _, _, word in mod:
        reference_word.extend(word)
    print(f"  vox.recompile_module() produced word: {''.join(reference_word)}")
    print(f"  word length: {len(reference_word)} glyphs")

    # Step 4: μ∘δ=id verification — they must match
    if decompiled_word == reference_word:
        print(f"  ✓ μ∘δ=id CLOSED: words match exactly")
        return True
    else:
        print(f"  ✗ μ∘δ=id OPEN: words differ")
        print(f"    decompiled:  {''.join(decompiled_word)}")
        print(f"    reference:   {''.join(reference_word)}")
        # Show where they first diverge
        for i, (a, b) in enumerate(zip(decompiled_word, reference_word)):
            if a != b:
                print(f"    first divergence at position {i}: decompiled={a}, reference={b}")
                break
        return False


def test_imasm_text_round_trip():
    """Test that parse_module_text + reconstruct_word round-trips correctly."""
    print(f"\n=== Testing IMASM text round-trip ===")

    binary_path = os.path.join(os.path.dirname(__file__), "corpus_O2.so")
    text = imasm_module.emit(binary_path)

    # Parse and reconstruct
    parsed = vox_decompiler.parse_module_text(text)
    word = vox_decompiler.reconstruct_word(parsed)

    # Now convert word back to IMASM text and re-parse
    reconstituted_text = vox_decompiler._word_to_imasm_text(word)
    re_parsed = vox_decompiler.parse_module_text(reconstituted_text)
    re_word = vox_decompiler.reconstruct_word(re_parsed)

    if word == re_word:
        print(f"  ✓ Internal round-trip closed")
        return True
    else:
        print(f"  ✗ Internal round-trip failed")
        print(f"    word:      {''.join(word)}")
        print(f"    re_word:   {''.join(re_word)}")
        return False


def test_glyph_consistency():
    """Verify that the glyph mappings are consistent across all modules."""
    print(f"\n=== Testing glyph consistency ===")

    glyph_set = {"⊢", "⊣", "≻", "≺", "⋈", "⊤", "∈", "∋", "⊙", "⊥", "⊞", "⊡"}
    all_glyphs_used = set()

    for opt in ["O0", "O1", "O2", "O3", "Os"]:
        binary_path = os.path.join(os.path.dirname(__file__), f"corpus_{opt}.so")
        if not os.path.exists(binary_path):
            continue
        text = imasm_module.emit(binary_path)
        word = vox_decompiler.decompile(text)
        all_glyphs_used.update(word)
        print(f"  corpus_{opt}.so: {''.join(word)}")

    print(f"\n  All glyphs used: {sorted(all_glyphs_used)}")
    print(f"  Expected glyphs: {sorted(glyph_set)}")
    extra = all_glyphs_used - glyph_set
    missing = glyph_set - all_glyphs_used
    if extra:
        print(f"  Extra glyphs (unexpected): {extra}")
    if missing:
        print(f"  Missing glyphs: {missing}")
    if not extra and not missing:
        print(f"  ✓ All 12 glyphs accounted for")
        return True
    return False


if __name__ == "__main__":
    results = []

    # Test round-trip on each optimization level
    vox_dir = os.path.dirname(os.path.abspath(__file__))
    for opt in ["O0", "O1", "O2", "O3", "Os"]:
        binary_path = os.path.join(vox_dir, f"corpus_{opt}.so")
        if os.path.exists(binary_path):
            results.append(test_round_trip(binary_path))

    # Test internal consistency
    results.append(test_imasm_text_round_trip())

    # Test glyph consistency
    results.append(test_glyph_consistency())

    print(f"\n=== Summary ===")
    passed = sum(results)
    total = len(results)
    print(f"  {passed}/{total} tests passed")

    if all(results):
        print(f"\n  ✓ μ∘δ=id LOOP VERIFIED: decompiler closes the circular pipeline")
        sys.exit(0)
    else:
        print(f"\n  ✗ μ∘δ=id LOOP BROKEN")
        sys.exit(1)