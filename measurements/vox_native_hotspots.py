"""Join native RIP samples to Vox's decoded instructions and loop back edges."""
import bisect
from collections import Counter
from pathlib import Path
import sys

if len(sys.argv) not in (3, 4):
    raise SystemExit('usage: vox_native_hotspots.py SAMPLES.tsv VOX.disasm [MODULE.imasm]')
symbols = []
if len(sys.argv) == 4:
    symbols = sorted({int(line.rsplit(' ', 1)[1], 16)
                      for line in Path(sys.argv[3]).read_text().splitlines()
                      if line.startswith('; sym ')})
instructions = {}
for line in Path(sys.argv[2]).read_text().splitlines():
    fields = line.split('\t')
    if len(fields) >= 2:
        instructions[int(fields[0], 16)] = fields[1:]
addresses = sorted(instructions)
samples = Counter()
register_values = {}
sample_lines = Path(sys.argv[1]).read_text().splitlines()
columns = {name: index for index, name in enumerate(sample_lines[0].split('\t'))} if sample_lines else {}
for line in sample_lines[1:]:
    fields = line.split('\t')
    address = int(fields[1], 16)
    # Native RIP names the next instruction; addresses outside the image stay external.
    index = bisect.bisect_right(addresses, address) - 1
    if index >= 0 and address - addresses[index] < 16:
        decoded_address = addresses[index]
        samples[decoded_address] += 1
        decoded = instructions[decoded_address]
        if decoded[0] == 'add' and len(decoded) > 1:
            operands = decoded[1].split()
            if len(operands) == 2 and operands[1].startswith('r:'):
                register = operands[1][2:]
                if register in columns:
                    if decoded_address not in register_values:
                        register_values[decoded_address] = (register, Counter())
                    register_values[decoded_address][1][int(fields[columns[register]], 16)] += 1
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
    if symbols and bisect.bisect_right(symbols, target) != bisect.bisect_right(symbols, address):
        continue
    if target < address:
        count = sum(n for a, n in samples.items() if target <= a <= address)
        loops.append((count, target, address))
print('Hot loop ranges: samples percent start end (nested ranges can overlap)')
for count, start, end in sorted(loops, reverse=True)[:15]:
    print(f'{count}\t{100*count/max(total,1):.3f}\t{start:x}\t{end:x}')
if register_values:
    print('Sampled source registers at hot additions: address register decimal-value samples')
    for address, _ in samples.most_common(15):
        if address not in register_values:
            continue
        register, values = register_values[address]
        for value, count in values.most_common(8):
            print(f'{address:x}\t{register}\t{value}\t{count}')
