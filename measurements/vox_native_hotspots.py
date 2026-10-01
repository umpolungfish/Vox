"""Join native RIP samples to Vox's decoded instructions and loop back edges."""
import bisect
from collections import Counter
from pathlib import Path
import sys

if len(sys.argv) != 3:
    raise SystemExit('usage: vox_native_hotspots.py SAMPLES.tsv VOX.disasm')
instructions = {}
for line in Path(sys.argv[2]).read_text().splitlines():
    fields = line.split('\t')
    if len(fields) >= 2:
        instructions[int(fields[0], 16)] = fields[1:]
addresses = sorted(instructions)
samples = Counter()
for line in Path(sys.argv[1]).read_text().splitlines()[1:]:
    address = int(line.split('\t')[1], 16)
    # Native RIP names the next instruction; addresses outside the image stay external.
    index = bisect.bisect_right(addresses, address) - 1
    if index >= 0 and address - addresses[index] < 16:
        samples[addresses[index]] += 1
    else:
        samples[-1] += 1
total = sum(samples.values())
print(f'{total} native samples')
print('Hot instructions: samples percent address decoded')
for address, count in samples.most_common(15):
    decoded = ' '.join(instructions.get(address, ['[external]']))
    print(f'{count}\t{100*count/max(total,1):.3f}\t{address:x}\t{decoded}')
loops = []
for address, fields in instructions.items():
    if not fields[0].startswith('j') or len(fields) < 2 or not fields[1].startswith('i:0x'):
        continue
    target = int(fields[1][2:], 16)
    if target < address:
        count = sum(n for a, n in samples.items() if target <= a <= address)
        loops.append((count, target, address))
print('Hot loop ranges: samples percent start end (nested ranges can overlap)')
for count, start, end in sorted(loops, reverse=True)[:15]:
    print(f'{count}\t{100*count/max(total,1):.3f}\t{start:x}\t{end:x}')
