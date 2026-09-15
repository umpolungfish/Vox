# Membrane speeds and complete IMASM modules

Latest run: [87-test regression suite and five-run execution tables](membrane_test_tables.md).

The linked `.imasm` files contain the lifted executable operations, symbols and
baked data. Each listed run executes the saved module through `vox run`, exits
with status zero inside the VM, and matches its native executable's stdout and
stderr exactly. `membrane_one.sh` performs and checks the whole pipeline.

| Membrane | Baked input | Complete executable IMASM | Vox elapsed | VM steps |
|---|---|---|---:|---:|
| Factor | 8051 | [Module](membranes/factor_one/8051/payload.elf.imasm) | 7.34 s | 7,113,242 |
| ABC | epsilon 1/10; cutoffs 9, 32, 1000 | [Module](membranes/abc_one/1_10_9_32_1000/payload.elf.imasm) | 58.57 s | 56,272,600 |
| Divisor ring | 2^50, 360, 97 | [Module](membranes/divisor_one/1125899906842624_360_97/payload.elf.imasm) | 0.29 s | 238,939 |
| Shor | a=2, N=21, 12 qubits | [Module](membranes/shor_one/2_21_12/payload.elf.imasm) | 1.90 s | 1,958,437 |
| Schütte | 23 vertices; k=2,3 | [Module](membranes/schutte_one/23_2_3/payload.elf.imasm) | 0.17 s | 148,959 |
| Landau | 10, 15, 45 | [Module](membranes/landau_one/10_15_45/payload.elf.imasm) | 0.11 s | 63,650 |
| Distinct triple sums | 24 | [Module](membranes/tripsum_one/24/payload.elf.imasm) | 11.50 s | 11,934,728 |
| Binomial-row GCD | 0, 1, 2, 8, 9, 12, 25, 27, 30, 81, 120, 81 | [Module](membranes/binomial_one/0_1_2_8_9_12_25_27_30_81_120_81/payload.elf.imasm) | 0.14 s | 99,220 |
| Cumulative LCM | 0, 1, 10, 20, 40, 80, 100, 10 | [Module](membranes/lcm_one/0_1_10_20_40_80_100_10/payload.elf.imasm) | 0.12 s | 70,751 |
| Rep-tiling | 1,2,3,4,5,6,7,8,9,10,18,20,25,26,100,1000,6 | [Module](membranes/reptiling_one/1_2_3_4_5_6_7_8_9_10_18_20_25_26_100_1000_6/payload.elf.imasm) | 1.80 s | 2,717,665 |

Each module's directory contains `native.stdout`, `vox.stdout`, both stderr
streams, `output.diff` (empty on agreement), and `vox.time`. Vox elapsed time is
a single whole-process wall-clock reading, including module loading and
interpretation. Compilation and lifting are excluded. These are different
measurements from the native kernel timings below; they do not establish a Vox
speedup. The historical 160-bit factor payload has not been verified in Vox.

The prime-power membranes share preparation across each payload's readouts.
LCM uses checked u128 and reports overflow at n=100. Its n=80 result is
32433859254793982911622772305630400, verified with standard decimal formatting.
See [source, controls and instruction diagnosis](membrane_prime_power_notes.md).

The first native timing sample in `membrane_wave4_checks.log` records 1,000
sweeps of rows 2..30 and LCM queries 0..40. Shared preparation took 14.400 µs;
row readouts took 25.100 µs versus 37.462488 ms for repeated original row
construction (948.42× including preparation). LCM readouts took 60.800 µs
versus 21.994393 ms for repeated checked-u128 LCM scans (292.48× including
preparation). These are amortized native controls, not Vox speedup measurements.

## Resident circuit controls

`vox circuit` retains one prepared VM across gate changes. These measurements
execute the circuit's IMASM prepare and activation functions. Every readout is
compared with a direct transform, including feedback and closed-gate controls.
Topology and buffers remain resident; the gate patterns are supplied at runtime.
Depth is a build parameter.

| Levels | Ports | Preparation | Resident activation | Cold parse + prepare + activation | Cold / resident |
|---:|---:|---:|---:|---:|---:|
| 3 | 8 | 112.336 µs | 545.599 µs | 37.305347 ms | 68.38× |
| 6 | 64 | 755.225 µs | 8.004777 ms | 45.928677 ms | 5.74× |
| 8 | 256 | 23.401976 ms | 39.995148 ms | 103.725391 ms | 2.59× |

The activation and cold columns are medians of five executions with the same
gate pattern. Cold timings start with the module text already read and include
parsing, preparation and activation. Resident timings cover activation alone;
output extraction and the direct-transform check follow the timer. Preparation
is the initial single reading, separate from module loading. These ratios measure
the cost saved by retaining the prepared circuit.

Complete modules and measurements:
[3 levels](membranes/qft_circuit/8/payload.elf.imasm),
[3-level log](membranes/qft_circuit/8/resident.log);
[6 levels](membranes/qft_circuit/64/payload.elf.imasm),
[6-level log](membranes/qft_circuit/64/resident.log);
[8 levels](membranes/qft_circuit/256/payload.elf.imasm),
[8-level log](membranes/qft_circuit/256/resident.log).
The [interactive log](membranes/qft_circuit/64/interactive.log) records gate
changes supplied after preparation, with one preparation and five activations.

## Recorded native kernel controls

Recorded controls from this conversation. Setup is included in the six port controls; compilation is excluded. The factor comparison is historical, reported earlier in the conversation; its earlier local log is no longer at the recorded path. Ratios are calculated from unrounded recorded times.

The numeral appendix below records input payloads emitted by `vox numeral`.
The complete executable modules are linked above. Numeral strings alone do not
contain their membrane operations.

| Membrane | Original | Nested | Ratio | Measurement |
|---|---:|---:|---:|---|
| [Factor, 160-bit](#factor-payload) | 50.04 s | 20.86 s | 2.40× | Earlier recorded MPQS run versus baked full factor run |
| [ABC windows](#abc-payload) | 138.317 ms | 13.048 ms | 10.60× | Cutoffs 200, 400, 600, 800, 1000; epsilon 1/10 |
| [Divisor ring](#divisor-payload) | 210.598 ms | 21.174 µs | 9,946.08× | 2^50; original divisor scan versus nested full ring analysis |
| [QFT / Shor](#shor-payload) | 25.347 ms | 38.768 µs | 653.80× | Dense 1,024-amplitude transform; payload below is the Shor example a=2, N=21, qubits=12 |
| [Schütte](#schutte-payload) | 11.236 ms | 4.375 µs | 2,568.33× | 23 vertices, all 253 pairs |
| [Landau](#landau-payload) | 4.247 ms | 5.851 µs | 725.81× | n=45; partition descent versus table for all capacities through 45 |
| [Distinct triple sums](#tripsum-payload) | 8.510 ms | 1.041 ms | 8.17× | Limit 24; identical seven-element witness |

Sources: `membrane_checks.log`, `membrane_wave2_checks.log`, `membrane_wave3_checks.log`, and the earlier factor timing in this conversation. The dense QFT control constructs a deterministic complex state in its test; the Shor payload below is the separately verified example, whose full runtime is not the QFT timing above.

## factor payload

Input values in order: 846698979836362929782388334525136360538050361829.

846698979836362929782388334525136360538050361829:

```text
⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```


## abc payload

Input values in order: 1, 10, 200, 400, 600, 800, 1000.

1:

```text
⊢≻⋈∈⊥∋⊙⊡⊣
```

10:

```text
⊢≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

200:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```

400:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```

600:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

800:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```

1000:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```


## divisor payload

Input values in order: 1125899906842624.

1125899906842624:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```


## shor payload

Input values in order: 2, 21, 12.

2:

```text
⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

21:

```text
⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

12:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```


## schutte payload

Input values in order: 23, 2.

23:

```text
⊢≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

2:

```text
⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```


## landau payload

Input values in order: 45.

45:

```text
⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```


## tripsum payload

Input values in order: 24.

24:

```text
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣
```
