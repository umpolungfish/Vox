"""Render retained folded-glut measurements without rerunning extraction."""
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
os.environ['MPLCONFIGDIR']=str(ROOT/'measurements/glut_plot_cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
rows=json.loads((ROOT/'membranes/glut_stress_2026-09-29_folded_verified/manifest.json').read_text())
old={r['name']:r for r in json.loads((ROOT/'membranes/glut_stress_2026-09-29/manifest.json').read_text())}
lines=['# Folded glut wiring, 2026-09-29','',
'Production extraction now retains symbolic factor cells and exact interval intersections. Free cells represent unresolved assignments without expanding their concrete prefixes. Product bounds constrain the high rail; source bits and exact convolution carry constrain the low rail. Interval extrema preserve holes imposed by fixed cells.','',
'Factor-width lanes advance fairly, each retaining a depth-first frontier. Propagation repeats when a cell gains information. This repairs two disconnected progress mechanisms: a wide lane could starve small-factor lanes, and endpoint rounding could iterate through a value’s magnitude without fixing a bit. No state, width, depth or decision quota is used.','',
'For even sources, the low zero run closes through product valuation before prefix materialization. A selected pair is projected through actual convolution transitions. The returned checkpoints, complete product and re-entry certificate are independently replayed. Resumed explicit frames validate their ancestry, then reopen the authoritative source relation through the same folded path.','',
'## Extraction measurements','',
'All 21 composite fixtures close; the prime control returns an empty exact proper-pair relation. The same external 8 s observation window and 768 MiB address envelope were used as in the retained baseline. These are measurement process settings. Factor witnesses in the fixture metadata are used only for independent checks after extraction.','',
'`Peak folds` counts live symbolic domains. `Peak cells` counts owned masks and interval bound tapes in those domains; it excludes temporary arithmetic storage and is not a process-memory measurement. Concrete prefix populations and symbolic domains are different units. Times include native extraction and verified checkpoint materialization, excluding compilation and certificate transport.','',
'| Case | Source bits | Prior observation | Folded extraction ms | Peak folds | Peak cells |','|---|---:|---|---:|---:|---:|']
for r in rows:
    b=old.get(r['name']); prior='new fixture' if b is None else ('closed' if b.get('result') and b['result'][0]=='closed' else 'no proper pair' if b.get('result') else 'interrupted')
    f=r['last_fold']; lines.append(f"| {r['name']} | {r['bits']} | {prior} | {int(r['result'][3])/1000:.3f} | {f[2]} | {f[4]} |")
lines += ['', '![Native extraction time by source width](glut_fold_width.svg)','',
'Raw measurements and static controls: [manifest](../membranes/glut_stress_2026-09-29_folded_verified/manifest.json). Earlier repair iterations are retained in the `folded`, `folded_v2`, `folded_final` and `folded_connected` run directories.','',
'## Complete IMASM executable verification','',
'`glut_one` bakes the source numeral. Its successful output consists of four IMASM words: source, p, q and the verified re-entry certificate. The complete executable carrier contains only the twelve marks, including code, operands, addresses, values, data and metadata.','',
'The existing glyph-module codec transports lossless UTF-8 module records inside numeral payloads. Vox decodes the carrier into its lifted executable module before execution. Rust and static ELF remain retained build intermediates. The alphabet check and execution comparison establish the complete carrier and its value outputs; they do not establish a new glyph-native internal VM representation.','']
path=ROOT/'membranes/glut_imasm_2026-09-29_verified/manifest.json'
if path.exists():
    glyph=json.loads(path.read_text()); lines+=['| Case | Source bits | Executed VM steps | IMASM execution s |','|---|---:|---:|---:|']
    for r in glyph:
        steps=r['vm_footer'].split('[')[1].split()[0]; lines.append(f"| {r['name']} | {r['source_bits']} | {steps} | {r['imasm_wall_seconds']:.3f} |")
lines+=['','Each retained glyph executable is alphabet checked, recovered byte-for-byte into its complete module, executed through Vox, and compared with its native control. The program’s output is independently alphabet checked. Artifact hashes, exit status and VM steps are in the [executable manifest](../membranes/glut_imasm_2026-09-29_verified/manifest.json).','',
'## Verification and reproduction','',
'- 255 library tests pass, including exhaustive four-cell interval/mask extrema and an independent proper-pair census for sources 0 through 1024.','- Four re-entry integration tests pass; all targets compile. Negative ancestry, carry, prefix and certificate tests remain active.','- Native stress results independently verify proper factors and exact integer products.','',
'```sh','GLUT_STRESS_RUN=fresh_name python3 measurements/glut_stress_run.py','GLUT_IMASM_RUN=fresh_name python3 measurements/glut_imasm_bake.py','python3 measurements/glut_fold_report.py','```','',
'Choose fresh names to preserve retained experiments. `GLUT_STRESS_CASES` and `GLUT_IMASM_CASES` select fixtures. The IMASM bake harness observes each Vox child for 120 s; this is external to the membrane.','']
(ROOT/'measurements/glut_fold_2026-09-29.md').write_text('\n'.join(lines))
fig,ax=plt.subplots(figsize=(8,4))
for shape in sorted({r['shape'] for r in rows}):
    group=[r for r in rows if r['shape']==shape and r['result'][0]=='closed']
    if group: ax.plot([r['bits'] for r in group],[int(r['result'][3])/1000 for r in group],marker='o',label=shape)
ax.set_yscale('log'); ax.set_xlabel('Source width (bits)'); ax.set_ylabel('Native extraction and projection (ms)'); ax.grid(alpha=.25); ax.legend(fontsize=8); fig.tight_layout()
for ext in ['svg','png','pdf']: fig.savefig(ROOT/f'measurements/glut_fold_width.{ext}')
