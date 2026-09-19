#!/usr/bin/env python3
"""
Mark the ten known VOX grammar/pseudocode rustdoc blocks as `text`.

Idempotent: already fenced blocks are left unchanged.
No real Rust doctest examples are globally disabled.
"""
from pathlib import Path
import re
import sys

TARGETS = {
    "src/router_object.rs": [
        r"\(object,\s*router\)\s*->",
    ],
    "src/frame_work.rs": [
        r"tau.*->",
    ],
    "src/tape_delete.rs": [
        r"EDIT_WORD\s*=",
    ],
    "src/reducer_store.rs": [
        r"REDUCE_WORD\s*=",
    ],
    "src/router_store.rs": [
        r"IStore\s*\{",
        r"VINIT\s+cursor\s*=",
    ],
    "src/trace_algebra.rs": [
        r"candidate is admissible",
        r"judge\(a\)\s*=",
    ],
    "src/trace_word.rs": [
        r"<repr>.*<judgment>.*<recognised>.*<next>",
    ],
    "src/router_marks.rs": [
        r"RouteClauseG\s*\{",
    ],
}

DOC_RE = re.compile(r"^(\s*//[!/])(.*)$")

def doc_parts(line):
    m = DOC_RE.match(line.rstrip("\n"))
    if not m:
        return None
    return m.group(1), m.group(2)

def is_indented_code(line):
    p = doc_parts(line)
    if not p:
        return False
    rest = p[1]
    # Rustdoc/CommonMark indented code. The conventional one space after
    # /// or //! plus four code spaces commonly yields five here.
    return bool(re.match(r"^\s{4,}\S", rest))

def fence_prefix(line):
    p = doc_parts(line)
    if not p:
        raise RuntimeError("not a doc line")
    return p[0]

def already_fenced(lines, start):
    j = start - 1
    while j >= 0:
        p = doc_parts(lines[j])
        if not p:
            return False
        rest = p[1].strip()
        if not rest:
            j -= 1
            continue
        return rest.startswith("```")
    return False

def wrap_trigger_block(lines, regex, path):
    rx = re.compile(regex)
    hits = []
    for i, line in enumerate(lines):
        p = doc_parts(line)
        if p and rx.search(p[1]):
            hits.append(i)

    if not hits:
        raise RuntimeError(f"{path}: trigger /{regex}/ not found")
    if len(hits) > 1:
        # Prefer the indented-code occurrence if there are explanatory mentions.
        ind = [i for i in hits if is_indented_code(lines[i])]
        if len(ind) == 1:
            hits = ind
        else:
            raise RuntimeError(
                f"{path}: trigger /{regex}/ matched {len(hits)} locations; refusing ambiguous edit"
            )

    i = hits[0]
    if already_fenced(lines, i):
        return lines, False

    if not is_indented_code(lines[i]):
        raise RuntimeError(
            f"{path}:{i+1}: trigger found but is not an indented rustdoc code block"
        )

    # Expand over the contiguous indented doc-code lines. Blank doc lines are
    # delimiters, not part of the code block.
    start = i
    while start > 0 and is_indented_code(lines[start - 1]):
        start -= 1

    end = i
    while end + 1 < len(lines) and is_indented_code(lines[end + 1]):
        end += 1

    prefix = fence_prefix(lines[start])
    indent = re.match(r"^\s*", lines[start]).group(0)
    open_fence = f"{indent}{prefix.strip()} ```text\n"
    close_fence = f"{indent}{prefix.strip()} ```\n"

    new_lines = lines[:start] + [open_fence] + lines[start:end+1] + [close_fence] + lines[end+1:]
    return new_lines, True

def main():
    total_changed = 0
    missing = []

    for name, triggers in TARGETS.items():
        path = Path(name)
        if not path.exists():
            missing.append(name)
            continue

        lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
        changed_here = 0

        for trigger in triggers:
            try:
                lines, changed = wrap_trigger_block(lines, trigger, path)
            except RuntimeError as e:
                print(f"ERROR: {e}", file=sys.stderr)
                return 2
            changed_here += int(changed)

        if changed_here:
            path.write_text("".join(lines), encoding="utf-8")
            print(f"{name}: wrapped {changed_here} pseudocode block(s) as text")
            total_changed += changed_here
        else:
            print(f"{name}: already fixed")

    if missing:
        print("ERROR: missing expected source files:", file=sys.stderr)
        for name in missing:
            print(f"  {name}", file=sys.stderr)
        return 2

    print(f"\nchanged {total_changed} block(s)")
    print("running doctests...")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
