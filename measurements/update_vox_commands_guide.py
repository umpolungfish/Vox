from pathlib import Path
import sys
root = Path(__file__).resolve().parents[1]
path = Path('/home/mrnob0dy666/imsgct/ig-docs/vox_commands_guide.md')
s = path.read_text()
old = """### `factor_one`
**Path:** `src/bin/factor_one.rs`
**Purpose:** Single factorization operation

```bash
./target/release/factor_one <N>
```
"""
new = """### `factor_one`
**Path:** `src/bin/factor_one.rs`
**Purpose:** Executes a native IMASM numeral baked at compilation.

```bash
cd /home/mrnob0dy666/imsgct/Vox
./bake_factor.sh "$N" membranes/my_case
membranes/my_case/factor_one > membranes/my_case/run.stdout 2> membranes/my_case/run.stderr
```

The executable receives no numeric argument. N is supplied to the builder.
"""
if old not in s:
    sys.exit('Expected factor_one section missing; inspect the current commands guide.')
s = s.replace(old, new, 1)
section = (root / 'measurements/vox_dynamic_baking_commands.md').read_text()
anchor = '## Factorization Commands\n'
s = s.replace(anchor, section + anchor, 1)
s = s.replace('10. [Utility Commands](#utility-commands)', '10. [Utility Commands](#utility-commands)\n11. [Dynamic baking and Vox diagnosis](#dynamic-baking-and-vox-diagnosis)', 1)
path.write_text(s)
(path.parent / 'commit.txt').write_text('I document dynamic Vox baking and native diagnosis\n\nI replace the stale factor_one runtime argument example with the prepared binary builder. I document independent semiprime generation, baking without execution, captured execution, Gödel closure commands, native register profiling and hardware counters. I retain the current measured limit while the under-one-minute objective remains active.\n')
