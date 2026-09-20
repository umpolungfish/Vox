# Semiprime execution campaign

Run `cargo build --release --bin semiprime_probe`, then
`python3 semiprime_campaign.py --output results.jsonl`.
The Python harness requires SymPy. Output files are created exclusively and
each completed case is flushed immediately. The default minimum width is 175
bits, with five samples per family. There is no default execution timeout.
`--seconds` explicitly opts into an external measurement timeout.

Each producer receives only decimal N. Expected factors remain in the parent
process. Success requires both expected factors in the producer readout.
SymPy primality checks above 64 bits are probable-prime checks, not attached
primality certificates. Seeds and exact inputs are retained for reproduction.

Families currently implemented are close pairs, q near 2p, independently
sampled balanced pairs, and unbalanced pairs with p near one third of the
total width. Product widths are checked exactly. Close pairs use small gaps
and therefore principally measure arithmetic width. The multiplier family
currently probes k near two, not a sweep of multiplier depth.

The probe measures producer execution. Certificate and passive-extraction
timings, peak memory, persistent timeout checkpoints, and controlled
p-minus-one smoothness families remain to be added. Timeout means unfinished
within the selected budget. It does not establish a factoring limit.

`semiprime-calibration-175.jsonl` is an exploratory calibration log. Its first
resident close-pair result is a harness false failure: the initial probe omitted
the resident remainder. That defect was corrected and the case rerun in
`semiprime-close-sweep.jsonl`. The calibration crossed a probe rebuild and
must not be treated as a uniform binary benchmark.

`semiprime-close-sweep.jsonl` contains the corrected five-sample sweep from
175 through 512 bits for both producers, with the executable SHA-256 recorded.
All cases passed. Results include process startup and independent oracle
comparison outside the producer. Concurrent exploratory probes can affect timings.

The extension log covers five close-pair inputs each at 768 and 1024 bits.
Both producers passed every input. The 175-bit balanced calibration timed out
on both producers at sixty seconds; the multiplier-aligned dialectic case
closed in four descents while the resident timed out.

The structural battery now also exercises repeated neutral descent after
serialization at every checkpoint of the twelve-descent control:
`cargo test --release --test dialectic_neutral_every_cut`.
`dialectic_certificate_cuts` rejects every internal checkpoint deletion and
every checkpoint duplication, while verifying legitimate suffix restarts.

Next batteries cover exact Fermat interval boundaries, checkpoint splicing
between valid certificates, and an independent resident/dialectic provenance
comparison. The existing support projection comparison alone does not establish
independent producer agreement.

Direct producer options now include `phase`, `braid`, and `symbolic`.
The braid probe acquires the order, transports it in a braid, reads winding,
and closes factors. The symbolic probe invokes register construction directly,
then peak extraction and factor closure. Its current constructor computes the
order using a baby-step/giant-step table before forming a peak; removing the
braid cutoff does not change that algorithm. The phase probe invokes the nested
factor tower. These are local CPU execution paths.

The three additional 175-bit logs preserve the earlier sixty-second experiment.
Both braid inputs exhausted the former internal orbit cap. Phase and symbolic
inputs timed out. These logs precede removal of the braid cap.

The braid orbit now terminates on modular closure or invalid input. The scanned
API takes only N and returns the selected base as a tape; its former max-base
and orbit-cap arguments have been removed. No repository callers used that API.
The residue domain determines scan exhaustion. Regression coverage explicitly
crosses the former cutoff and checks factor reconstruction.
