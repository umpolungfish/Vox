"""Build-only inputs, canonical IMASM encoding, retained per-case executable."""
import os
import fcntl
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import time
from contextlib import contextmanager
from standalone import require_standalone, static_env

ROOT = Path(__file__).resolve().parent
DEFAULT_TARGET = 'x86_64-unknown-linux-musl'
MODES = {
    'divisor': 'One-divisor phase/Y producer; derive the quotient after return, then close the exact product (default).',
    'dialectic': 'N-only dialectic descent: short/extended square frontier, then Lehman multipliers.',
    'dialectic-lean': 'Resident dialectic descent with Lean-backed Y and clock operations.',
    'smart': 'General tape smart_factor dispatcher.',
    'resident': 'Unbounded factorization_31 resident membrane.',
    'synthesis': 'FactorSynthesis: retained dialectic descent and prepared GLUT ports.',
    'phase': 'Native numeral phase-factor route.',
    'braid': 'Braid order acquisition and winding factor closure, base 2.',
    'symbolic': 'Modular evolution and observed QFT order extraction, base 2.',
    'sparse': 'Sparse phase-register diagnostic; requires --width QUBITS; prints probabilities, not factors.',
    'observed': 'Observed phase-register order extraction; requires --width QUBITS.',
    'phaseB': 'Dedicated braid phase-factor executable, base 2; ignores --width.',
    'winding': 'Dedicated winding/phase-partner executable; --width sets qubits (default: 2*bit_length(N)+2).',
    'ring': 'Dedicated leaping-rotation order search, base 2.',
    'basemod': 'Dedicated modular-rotation order search, base 2.',
    'reduce': 'Dedicated reduce/delete coordinate order search, base 2.',
    'cylinder': 'Dedicated multiple-base cylinder search; --width sets exclusive base limit (default: 300).',
}


@contextmanager
def compilation_progress(label, enabled):
    """CLI-only stderr animation; stop before any executable runs."""
    if not enabled:
        yield
        return
    stopped = threading.Event()
    started = time.monotonic()
    animated = sys.stderr.isatty()

    def draw():
        index = 0
        while not stopped.is_set():
            sys.stderr.write(f'\r{"|/-\\"[index % 4]} {label} ({time.monotonic()-started:.0f}s)')
            sys.stderr.flush()
            index += 1
            stopped.wait(0.1)

    worker = threading.Thread(target=draw, daemon=True) if animated else None
    if worker:
        worker.start()
    else:
        print(f'{label}...', file=sys.stderr, flush=True)
    try:
        yield
    finally:
        stopped.set()
        if worker:
            worker.join()
            sys.stderr.write('\r\033[2K')
            sys.stderr.flush()


def compile_binary(command, env, progress, label):
    with compilation_progress(label, progress):
        return subprocess.run(command, cwd=ROOT, env=env, check=True, capture_output=True)


def build_case(n, mode, width=None, target=None, *, progress=False, radix=None):
    # Hold the lock through the copy: another case must not replace Cargo's
    # output between compilation and retention of this executable.
    (ROOT / 'target').mkdir(exist_ok=True)
    with (ROOT / 'target/baked-case.lock').open('a') as lock:
        with compilation_progress('Waiting for the build lock', progress):
            fcntl.flock(lock, fcntl.LOCK_EX)
        return _build_case(n, mode, width, target, progress, radix)


def _build_case(n, mode, width, target, progress=False, radix=None):
    target = target or DEFAULT_TARGET
    compile_binary(['cargo', 'build', '--release', '--target', DEFAULT_TARGET, '--bin', 'phase_case_encode', '--target-dir', 'target'],
                   static_env(), progress, 'Compiling the IMASM input encoder')
    parameter = (radix if radix is not None else 4294967296) if mode == 'divisor' else width
    values = [str(n), '2'] + ([str(parameter)] if parameter is not None else [])
    words = subprocess.check_output([str(ROOT / 'target/x86_64-unknown-linux-musl/release/phase_case_encode'), *values], text=True).splitlines()
    destination = ROOT / 'target/baked-semiprime' / str(time.time_ns())
    destination.mkdir(parents=True)
    inputs = destination/'inputs.imasm'
    inputs.write_text('\n'.join(words)+'\n')
    env = static_env()
    env['VOX_BAKED_INPUT_FILE'] = str(inputs)
    env.pop('VOX_PHASE_MODULUS_WORD', None)
    env.pop('VOX_PHASE_BASE_WORD', None)
    env['VOX_PROBE_MODE'] = mode
    env.pop('VOX_PHASE_WIDTH_WORD', None)
    target_bin = {'divisor': 'factor_2adic', 'phaseB': 'phaseB_fac', 'winding': 'winding_membrane', 'ring': 'ring_membrane', 'basemod': 'base_mod_membrane', 'reduce': 'reduce_delete_membrane', 'cylinder': 'cylinder_membrane'}.get(mode, 'semiprime_probe')
    build = ['cargo', 'build', '--release', '--bin', target_bin, '--target-dir', 'target']
    if target is not None:
        build += ['--target', target]
    compile_binary(build, env, progress, f'Compiling {mode} for {target}')
    binary = destination / mode
    output = ROOT / 'target'
    if target is not None:
        output /= target
    require_standalone(output / 'release' / target_bin)
    shutil.copy2(output / 'release' / target_bin, binary)
    return binary, words


def main():
    import argparse
    if hasattr(sys, 'set_int_max_str_digits'):
        sys.set_int_max_str_digits(0)
    parser = argparse.ArgumentParser(
        description='Compile decimal N into a retained standalone executable, then run it with an empty environment.',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='Modes:\n' + '\n'.join(f'  {mode:16} {description}' for mode, description in MODES.items()) + '''

Examples:
  python3 Vox/baked_case.py 143
  python3 Vox/baked_case.py 143 divisor --radix 4294967296 --build-only
  python3 Vox/baked_case.py N observed --width 16
  python3 Vox/baked_case.py N divisor --no-spinner
  (Replace N with a decimal integer.)

Build and execution:
  Requires Cargo/Rust, the musl target and its linker/toolchain for the default target.
  Builds run from this script's directory, regardless of your working directory.
  N and the fixed base 2 are imscribed by the Rust encoder and baked in at build time.
  Each build retains inputs.imasm and an executable under Vox/target/baked-semiprime/<id>/.
  Default divisor mode searches one divisor; its quotient is derived only afterward.
  Its --radix controls terminal prefix closure, not a candidate width or factor seed.
  Compilation time is separate from the executable's reported seconds, when present.
  The spinner uses stderr only during compilation or lock waiting; redirected stderr
  receives plain stage messages. Execution receives no spinner or added progress reports.
  Normal execution forwards the selected mode's output and returns its exit code.
  A zero exit code alone does not mean every diagnostic mode produced factors.
  There is no wrapper execution timeout. Ctrl-C interrupts the command.
  --build-only writes only the executable's absolute path to stdout and does not run it.
  Divisor mode expects composite N; a prime has no proper divisor to return.
  Its starting base is 2; a completed cycle without a divisor re-enters on the next
  internally generated base while retaining its evidence. Other modes ignore --width.
  This wrapper has no --base or supplied-divisor option.
''')
    parser.add_argument('n', type=int, metavar='N', help='decimal source integer, at least 2; no reference factors are accepted')
    parser.add_argument('mode', nargs='?', default='divisor', choices=MODES, metavar='MODE', help='one of the modes listed below (default: divisor)')
    parser.add_argument('--width', type=int, metavar='COUNT', help='positive qubit count for sparse/observed/winding, or exclusive base limit for cylinder')
    parser.add_argument('--radix', type=int, metavar='RADIX', help='post-production prefix-closure radix for divisor mode only (default: 4294967296); does not select candidate factors')
    parser.add_argument('--target', default=None, metavar='TRIPLE', help=f'Rust compilation target (default: {DEFAULT_TARGET}); custom output must be static and runnable here unless --build-only')
    parser.add_argument('--build-only', action='store_true', help='build and print the retained executable path; skip execution')
    parser.add_argument('--no-spinner', action='store_true', help='disable compilation animation and stage messages on stderr')
    args = parser.parse_args()
    if args.n < 2:
        parser.error('n must be at least 2')
    if args.width is not None and args.width < 1:
        parser.error('--width must be positive')
    if args.mode in ('sparse', 'observed') and args.width is None:
        parser.error(f'{args.mode} requires --width QUBITS')
    if args.mode == 'divisor' and args.width is not None:
        parser.error('divisor mode uses --radix, not --width')
    if args.radix is not None and (args.mode != 'divisor' or args.radix < 2):
        parser.error('--radix requires divisor mode and a value at least 2')
    try:
        binary, _ = build_case(args.n, args.mode, args.width, args.target, progress=not args.no_spinner, radix=args.radix)
    except subprocess.CalledProcessError as error:
        if error.stderr:
            sys.stderr.write(error.stderr.decode(errors='replace') if isinstance(error.stderr, bytes) else error.stderr)
        return error.returncode
    except (OSError, ValueError) as error:
        print(f'baked_case.py: {error}', file=sys.stderr)
        return 1
    if args.build_only:
        print(binary)
        return 0
    return subprocess.run([str(binary)], env={}).returncode


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
