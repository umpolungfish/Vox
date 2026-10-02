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
if old in s:
    s = s.replace(old, new, 1)
elif new not in s:
    sys.exit('Expected factor_one section missing; inspect the current commands guide.')
section = (root / 'measurements/vox_dynamic_baking_commands.md').read_text()
anchor = '## Factorization Commands\n'
heading = '## Dynamic baking and Vox diagnosis\n'
if heading in s:
    start = s.index(heading)
    end = s.index(anchor, start)
    s = s[:start] + section + s[end:]
else:
    s = s.replace(anchor, section + anchor, 1)
if '11. [Dynamic baking and Vox diagnosis]' not in s:
    s = s.replace('10. [Utility Commands](#utility-commands)', '10. [Utility Commands](#utility-commands)\n11. [Dynamic baking and Vox diagnosis](#dynamic-baking-and-vox-diagnosis)', 1)
path.write_text(s)
(path.parent / 'commit.txt').write_text('I document silent native program-state sampling\n\nI document the sieve progress, matrix dimensions and residual cofactor readings produced by Vox from prepared ELF object storage. I refresh the existing dynamic baking section without duplicating it. The under-one-minute objective remains active.\n')
