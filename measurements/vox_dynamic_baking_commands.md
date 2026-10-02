## Dynamic baking and Vox diagnosis

Run these commands from `/home/mrnob0dy666/imsgct/Vox`. Inputs are generated
separately before baking. The factor binary receives only N's native IMASM
numeral at compilation; the generator's p and q remain checker data.

```bash
# Generate one random semiprime into a fresh input file.
python3 measurements/random_rsa_fixtures.py --bits 208 --count 1 --output measurements/random_inputs.jsonl

# Bake only. Prints the retained executable's path and does not execute it.
./bake_factor.sh --input measurements/random_inputs.jsonl membranes/my_case

# A supplied decimal can also be baked directly.
./bake_factor.sh "$N" membranes/my_other_case

# Execute the prepared binary without arguments. Capture terminal output.
membranes/my_case/factor_one > membranes/my_case/run.stdout 2> membranes/my_case/run.stderr

# Decode the actual binary with Vox.
./target/release/vox disasm membranes/my_case/factor_one > membranes/my_case/vox.disasm

# Sample its native execution through closure, including live registers.
./target/release/vox profile-native membranes/my_case/factor_one membranes/my_case/profile --registers

# Bake and verify every case in a separately generated input batch.
GODEL_BAKED_RUN=membranes/my_verified_batch python3 measurements/godel_baked_random_semiprime.py measurements/random_inputs.jsonl
```

`bake_factor.sh DECIMAL_N [OUTPUT_DIRECTORY]` and
`bake_factor.sh --input SINGLE_CASE.jsonl [OUTPUT_DIRECTORY]` retain
`factor_one`, `input.imasm`, `input.decimal`, its SHA256 and build diagnostics.
With no output directory, a fresh directory is created under
`membranes/baked_factor`. Explicit relative output directories are relative to
the Vox repository. Existing directories are refused. `--input` requires one
JSONL case. No p or q is baked. Cargo tracks the compile-time numeral, so changing
N rebuilds the executable. The emitted binary takes no runtime numeric input.

The generator requires `--bits` of at least 16, a positive `--count` and a fresh
`--output`. It checks both prime candidates with Vox and records the fixture's
seed. The batch runner checks both factor lanes, `godel check mul` and complete
ELF to IMASM to glyphs to IMASM recovery. Its results distinguish build time,
execution wall time and process CPU time.

`profile-native ELF PREFIX [--registers]` runs on Linux x86-64. It captures the
child's stdout and stderr into `PREFIX.stdout` and `PREFIX.stderr`; neither is
forwarded during execution. It writes native address samples, function shares,
exit status and hardware counters into `PREFIX.samples.tsv`, `.profile.tsv`,
`.status` and `.counters.tsv`. Hardware counter errors are recorded per event;
address sampling remains available. Counter counts are raw; `enabled_ns` and
`running_ns` expose multiplexing. `--registers` adds the live general registers
to each sample. Native tracing may require ptrace permission on the host.
When the ELF contains `VOX_SIEVE_COUNTERS`, Vox also reads its object storage
and records relation progress, surviving matrix dimensions and residual
cofactor counts in `PREFIX.sieve.tsv`. These observations come from program
memory and are written by the tracer.

After `vox imasm ELF` writes `ELF.imasm`, sampled addresses can be joined to
Vox's decoded instructions and symbol boundaries:

```bash
python3 measurements/vox_native_hotspots.py membranes/my_case/profile.samples.tsv membranes/my_case/vox.disasm membranes/my_case/factor_one.imasm
```

Gödel's commands read cell-binary words and check closure:

```bash
./target/release/godel encode "$N"
./target/release/godel analyze "$N" 8
./target/release/godel lte2 "$A" "$K"
./target/release/godel check mul "$WP" "$WQ" "$WN"
./target/release/godel unbraid "$N"
./target/release/godel braid "$GAMMA" "$LAMBDA"
```

`analyze` automatically factors after a bounded sieve miss, then verifies the
factor words' product and primality before releasing its report. `lte2` requires
odd A and even K. `braid` combines representation lanes; its value need not equal
the lanes' arithmetic product. The under-one-minute goal remains unverified:
the retained 208-bit example currently closes in about three minutes.

