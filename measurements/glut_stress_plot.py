"""Plot retained completed-frame growth and paired reverse-rail measurements."""
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
os.environ['MPLCONFIGDIR']=str(ROOT/'measurements/glut_plot_cache')
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
fig, axes=plt.subplots(1,2,figsize=(11,4.5))
base=ROOT/'membranes/glut_stress_2026-09-29'
for case,label in [('control_8051','8051, 13 bits'),('balanced_27','Balanced, 27 bits'),('dense_32','Dense square, 32 bits'),('balanced_64','Balanced, 64 bits'),('sparse_33','Power of two, 33 bits')]:
    fields=[line.split('\t') for line in (base/case/'native.stdout').read_text().splitlines() if line.startswith('FRAME\t')]
    axes[0].plot([int(f[1]) for f in fields],[int(f[3]) for f in fields],marker='.',label=label)
axes[0].set_yscale('log',base=2)
axes[0].set_xlabel('Processed source bits')
axes[0].set_ylabel('Live states at completed frames')
axes[0].set_title('Concrete prefix growth')
axes[0].legend(fontsize=8)
for suffix,style,label in [('', '-', 'Full prefix replay at each bit'),('_connected','--','Reuse verified frame boundaries')]:
    rows=json.loads((ROOT/('membranes/glut_replay_stress_2026-09-29'+suffix)/'manifest.json').read_text())
    for shape,color in [('sparse','#1463a2'),('dense','#b13c30')]:
        points=[]
        for row in rows:
            if row['shape']==shape:
                parts=row['output'].splitlines()[1].split('\t')
                points.append((row['product_bits'],int(parts[2])/1000))
        axes[1].plot(*zip(*points),color=color,linestyle=style,marker='o',label=f'{shape}: {label}')
axes[1].set_xscale('log',base=2)
axes[1].set_yscale('log')
axes[1].set_xlabel('Product width in bits, supplied witness')
axes[1].set_ylabel('Verify and transport time (ms)')
axes[1].set_title('Reverse rail and folded transport')
axes[1].legend(fontsize=7)
for ax in axes: ax.grid(alpha=.25)
fig.tight_layout()
for ext in ['svg','pdf','png']: fig.savefig(ROOT/f'measurements/glut_stress_2026-09-29.{ext}')
