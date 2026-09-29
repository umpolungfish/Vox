"""Validate and summarize retained baked membrane measurements."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'membranes/glut_stress_2026-09-29'
rows=json.loads((BASE/'manifest.json').read_text())
lines=['# Glut stress measurements', '',
'The baked extraction ladder measures the production glut on increasing source',
'width, word length, factor balance and bit density. Each executable contains',
'its source as a Vox-generated IMASM numeral. No runtime numeral is supplied.',
'The observer records the parity seed and each completed production frame.',
'Peak state counts in this report refer to completed frames; intra-frame',
'population and ancestry allocation can be larger.', '',
'Each process receives an external eight-second observation window and a',
'768 MiB address-space envelope. The extended measurements use thirty seconds',
'with the same envelope. These settings belong to the measurement process.',
'The glut retains its dynamic tapes and complete live set.', '',
'## Extraction ladder', '',
'Closed rows pass exact product verification, execution replay, trace replay,',
'certificate transport and the glut-specific certificate verifier. Each also',
'rejects an altered terminal carry. The prime control completes the source',
'without selecting a proper pair.', '',
'| Case | Bits | Ones | Numeral characters | Reading | Search ms / last frame ms | Peak frame states | Last position |',
'|---|---:|---:|---:|---|---:|---:|---:|']
for r in rows:
    folder=BASE/r['name']
    source=next(line.split('\t') for line in (folder/'native.stdout').read_text().splitlines() if line.startswith('INPUT\t'))
    assert source[1]==r['n'] and int(source[2])==r['bits'] and int(source[4])==r['word_chars']
    result=r['result']
    if result:
        if result[0]=='closed':
            p,q=map(int,result[1:3]);assert p*q==int(r['n']) and p>1 and q>1
            reading=f'{p} × {q}'
        else: reading='complete, no proper pair'
        ms=int(result[3])/1000
    else:
        reading='external time stop' if r['interrupted'] else f'process exit {r["returncode"]}'
        ms=int(r['last_frame'][4])/1000
    lines.append(f'| {r["name"]} | {r["bits"]} | {r["popcount"]} | {r["word_chars"]} | {reading} | {ms:.3f} | {r["peak_completed_frame_states"]:,} | {r["last_frame"][0]} |')
lines += ['', 'The 40-bit balanced case reaches position 19 and 262,144 live states in',
'22.26 seconds during its extended thirty-second reading. The 33-bit sparse',
'case reports an allocation failure under the external envelope after 13.11',
'seconds. Its last completed frame is position 15 with 278,528 states; the',
'sampled resident-memory peak is 706,392 KiB.', '',
'## Folding and reverse-rail connections', '',
'The sparse census hands the production sieve a 257-bit power of two and',
'advances it to position 14. All 131,072 survivors pass the reverse rail and',
'share zero carry. They span 120 pairs of leading-zero valuations and contain',
'3,801,088 stored value cells, excluding path ancestry and deduplication keys.',
'Both orientations are materialized: 65,472 states have their mirrored ordering.',
'The two all-zero prefix arms each contain 16,384 states. This identifies the',
'live representation to fold: the leading-zero product relation is expanded',
'into concrete prefixes while its shared carry remains unchanged.', '',
'Execution replay reuses the verified frame boundary. Every stored frame still',
'passes its full carry and product check. Between boundaries, advance checks each',
'source bit and reconstructs the precise factor prefixes, and the returned state',
'must equal the next checkpoint. Each intermediate prefix product is bounded',
'by that checked checkpoint because extending nonnegative prefixes cannot',
'decrease the product.', '',
'## Supplied-witness transport ladder', '',
'These readings build a path from separately baked, supplied factors. They',
'measure reverse replay and folded transport independently of extraction.',
'Checkpoint positions double with the source extent, and the final checkpoint',
'is exact product closure. All rows verify through certificate transport.', '',
'| Shape | Product bits | Construct path ms | Full prefix replay ms | Reused boundary replay ms | Trace records | Trace characters | Certificate characters |',
'|---|---:|---:|---:|---:|---:|---:|---:|']
before=json.loads((ROOT/'membranes/glut_replay_stress_2026-09-29/manifest.json').read_text())
after=json.loads((ROOT/'membranes/glut_replay_stress_2026-09-29_connected/manifest.json').read_text())
for b,a in zip(before,after):
    assert (b['p'],b['q'],b['product_bits'])==(a['p'],a['q'],a['product_bits'])
    old=b['output'].splitlines()[1].split('\t')[1:]
    new=a['output'].splitlines()[1].split('\t')[1:]
    assert old[2:]==new[2:]
    lines.append(f'| {a["shape"]} | {a["product_bits"]} | {int(new[0])/1000:.3f} | {int(old[1])/1000:.3f} | {int(new[1])/1000:.3f} | {new[4]} | {new[3]} | {new[5]} |')
lines += ['', '## Lifted execution', '',
'The retained 35 failure control isolated a memory-push operand decoded at four',
'bytes in a 64-bit process. Its pushed trace pointer lost the high address bits.',
'The corrected decoder preserves the eight-byte operand for long-mode indirect',
'push, call and jump. A focused control pushes an address above 4 GiB and checks',
'the value on the guest stack.', '',
'The repaired lift of that same retained executable exits zero and agrees with',
'native execution on its input, frames, factor and certificate dimensions.',
'The separate mark-by-mark 35 trace control also agrees with native execution.',
'The retained 8051 stress executable exits zero after 20,440,457 VM steps.',
'The lifted 17-bit square reading reaches position 8 before its external',
'sixty-second observation window expires.', '',
'Current source checks pass all 251 library tests and all-target compilation.',
'The fresh connected 35 and 8051 membranes retain matching native and lifted',
'frame, factor, trace-length and certificate-length readings.', '',
'![Measured prefix growth and paired reverse replay](glut_stress_2026-09-29.svg)', '',
'## Reproduction', '', '```bash',
'python3 measurements/glut_stress_run.py',
'GLUT_STRESS_RUN=extended GLUT_STRESS_CASES=balanced_40,sparse_33 GLUT_STRESS_SECONDS=30 python3 measurements/glut_stress_run.py',
'GLUT_REPLAY_RUN=connected python3 measurements/glut_replay_stress_run.py',
'python3 measurements/glut_stress_plot.py',
'python3 measurements/glut_stress_report.py', '```', '',
'Each directory retains the generated input, static executable, build output',
'and measured stdout/stderr. JSON manifests preserve per-case process settings,',
'exit codes, observed memory and completed frame frontiers. The executable files',
'preserve the paired replay implementations; rerunning the driver bakes the',
'current source. Use a fresh run name when comparing a further change.', '']
(ROOT/'measurements/glut_stress_2026-09-29.md').write_text('\n'.join(lines))
