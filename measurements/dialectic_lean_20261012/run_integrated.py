"""Run hard N-only inputs through the resident source-carrier producer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
WORK = Path(__file__).resolve().parent
PROJECT = ROOT.parent / 'G-mOMonadOS'
CAMPAIGN = PROJECT / 'measurements/factor_source_carrier_20261011/run_campaign.py'
CAMPAIGN_ROOT = CAMPAIGN.parent


def execute(case, bound, repetitions, tag):
    source_path = WORK / 'inputs' / f'{case}.txt'
    campaign_source = CAMPAIGN_ROOT / 'inputs' / source_path.name
    if not source_path.exists():
        if not campaign_source.exists():
            raise FileNotFoundError(f'N-only input not found: {source_path}')
        source_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(campaign_source, source_path)
    source = source_path.read_bytes()
    if not source.strip().isdigit() or int(source) < 2:
        raise ValueError(f'{case} is not an N-only source numeral greater than one')
    if campaign_source.exists() and campaign_source.read_bytes() != source:
        raise ValueError(f'{case} conflicts with its canonical source input')
    if not campaign_source.exists():
        campaign_source.write_bytes(source)

    campaign_runs = []
    for repeat in range(repetitions):
        campaign_tag = f'{tag}_r{repeat + 1}'
        log_path = WORK / f'{campaign_tag}_{case}.campaign.log'
        command = ['python3', str(CAMPAIGN), '--tag', campaign_tag,
                   '--source-carrier', '--bound', str(bound), '--cases', case]
        started = time.perf_counter()
        process = subprocess.run(command, cwd=PROJECT, capture_output=True, text=True)
        elapsed = time.perf_counter() - started
        log_path.write_text(process.stdout + process.stderr)
        receipt_path = CAMPAIGN_ROOT / f'{campaign_tag}_{case}.receipt.json'
        if process.returncode or not receipt_path.exists():
            raise RuntimeError(f'source-carrier campaign failed; inspect {log_path}')
        receipt = json.loads(receipt_path.read_text())
        campaign_runs.append({
            'campaign_receipt': str(receipt_path.relative_to(ROOT.parent)),
            'campaign_log': str(log_path.relative_to(WORK)),
            'wrapper_seconds': elapsed,
            'status': receipt['status'],
            'product_verified': receipt['product_verified'],
            'producer': receipt['producer'],
            'preparation': receipt['preparation'],
            'production_clocks': receipt['production_clocks'],
            'binary_sha256': receipt['binary_sha256'],
            'source_sha256': receipt['source_sha256'],
            'vox_binding': receipt.get('vox_binding'),
        })

    report = {
        'source_n': str(int(source)),
        'source_bits': int(source).bit_length(),
        'source_only_file': str(source_path.relative_to(WORK)),
        'source_sha256': hashlib.sha256(source).hexdigest(),
        'runtime_arguments': [],
        'dispatch': 'resident source-carrier; N-only MPQS relation source',
        'bound_seconds': bound,
        'wrapper_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'runs': campaign_runs,
    }
    receipt_path = WORK / f'{tag}_{case}.receipt.json'
    receipt_path.write_text(json.dumps(report, indent=2) + '\n')
    print(case, json.dumps(campaign_runs), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--cases', nargs='+', required=True)
    parser.add_argument('--bound', type=float, default=90)
    parser.add_argument('--repetitions', type=int, default=1)
    parser.add_argument('--tag', default='source_carrier')
    args = parser.parse_args()
    for case in args.cases:
        execute(case, args.bound, args.repetitions, args.tag)
