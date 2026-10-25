"""Positional orders shared by Python operations and build wrappers.

All-order exploration is lazy. Limits are explicit and return a held cursor.
A family reruns the same command with each order; each run retains its own state.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def permutation(spec, width):
    if spec.startswith('file:'):
        spec = Path(spec[5:]).read_text().strip()
    if spec in ('canonical', 'forward'):
        return tuple(range(width))
    if spec in ('reverse', 'inverse'):
        return tuple(reversed(range(width)))
    if spec.startswith('swap:'):
        left, right = (int(value) for value in spec[5:].split(','))
        if left == right or min(left, right) < 0 or max(left, right) >= width:
            raise ValueError('swap order requires two distinct in-range positions')
        result = list(range(width))
        result[left], result[right] = result[right], result[left]
        return tuple(result)
    values = tuple(int(s) for s in spec.removeprefix('perm:').split(',') if s)
    if sorted(values) != list(range(width)):
        raise ValueError(f'order must contain each position 0..{width - 1} exactly once')
    return values


def inverse(order):
    result = [0] * len(order)
    for position, source in enumerate(order):
        result[source] = position
    return tuple(result)


def order_label(order):
    order = tuple(order)
    if order == tuple(range(len(order))):
        return 'canonical'
    if order == tuple(reversed(range(len(order)))):
        return 'reverse'
    moved = sum(position != source for position, source in enumerate(order))
    identity = 0xcbf29ce484222325
    for index in order:
        for byte in int(index).to_bytes(8, 'little'):
            identity = ((identity ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f'permutation(width={len(order)},moved={moved},id={identity:016x})'


def order_spec(order):
    order = tuple(order)
    if order == tuple(range(len(order))):
        return 'canonical'
    if order == tuple(reversed(range(len(order)))):
        return 'reverse'
    moved = [(position, source) for position, source in enumerate(order) if position != source]
    if len(moved) == 2 and moved[0][0] == moved[1][1] and moved[1][0] == moved[0][1]:
        return f'swap:{moved[0][0]},{moved[1][0]}'
    return ','.join(map(str, order))


def run_ordered(executable, forwarded, env, spec):
    env = dict(env)
    path = None
    if len(spec) > 4096:
        handle = tempfile.NamedTemporaryFile(mode='w', encoding='ascii', prefix='.imsgct-order-', suffix='.txt', dir=Path.cwd(), delete=False)
        path = Path(handle.name)
        with handle:
            handle.write(spec)
        env['IG_ORDER_SPEC'] = f'file:{path}'
    else:
        env['IG_ORDER_SPEC'] = spec
    try:
        return subprocess.call(executable + forwarded, env=env)
    finally:
        if path is not None:
            path.unlink(missing_ok=True)


def selected(site, width):
    if os.environ.get('IG_ORDER_SITE') != site:
        return tuple(range(width))
    if os.environ.get('IG_ORDER_DISCOVER'):
        print(f'ORDER_SHAPE={site} {width}', flush=True)
        raise SystemExit(0)
    return permutation(os.environ.get('IG_ORDER_SPEC', 'canonical'), width)


def apply(order, values):
    permutation(','.join(map(str, order)), len(values))
    return [values[i] for i in order]


def order_receipt():
    site = os.environ.get("IG_ORDER_SITE")
    raw = os.environ.get("IG_ORDER_SPEC", "canonical")
    orientation = "inverse" if raw == "inverse" else "forward"
    width = os.environ.get("IG_ORDER_WIDTH")
    if raw.startswith("file:"):
        raw = Path(raw[5:]).read_text().strip()
    if width is None:
        if raw.startswith("swap:"):
            positions = [int(value) for value in raw[5:].split(",")]
            width = str(max(positions) + 1)
        elif raw not in ("canonical", "forward", "reverse", "inverse"):
            width = str(len(raw.removeprefix("perm:").split(",")))
    inverse_label = raw
    if width is not None:
        order = permutation(raw, int(width))
        raw = order_label(order)
        inverse_label = order_label(inverse(order))
    elif raw == "inverse":
        raw = "reverse"
        inverse_label = "reverse"
    elif raw == "forward":
        raw = "canonical"
        inverse_label = "canonical"
    return {"site": site, "spec": raw, "inverse": inverse_label, "orientation": orientation}


def variant_path(path):
    path = Path(path)
    receipt = order_receipt()
    if receipt["site"] is None:
        return path
    key = hashlib.sha256(json.dumps(receipt, sort_keys=True).encode()).hexdigest()[:16]
    suffix = ".order-" + key
    return path if path.name.endswith(suffix) else path.with_name(path.name + suffix)


def intercept(command=None):
    """Consume ordering flags before an existing parser or shell wrapper runs."""
    args = list(sys.argv[1:] if command is None else command)
    if not any(a in ('--ordering', '--orderings') for a in args):
        return
    executable = [sys.executable, str(Path(sys.argv[0]).resolve())]
    if command is None and args and Path(args[0]).suffix == '.sh':
        wrapper = Path(args.pop(0)).resolve()
        if wrapper.is_file():
            executable = [str(wrapper)]
    flags = {}
    forwarded = []
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in ('--ordering', '--orderings', '--order-site', '--order-limit', '--order-start', '--order-width'):
            if i + 1 == len(args):
                raise SystemExit(f'{arg} requires a value')
            flags[arg] = args[i + 1]
            i += 2
        else:
            forwarded.append(arg)
            i += 1
    spec = flags.get('--ordering', flags.get('--orderings'))
    site = flags.get('--order-site', 'lanes')
    if command is not None:
        executable = []
    env = dict(os.environ, IG_ORDER_SITE=site)
    env.pop('IG_ORDER_DISCOVER', None)
    width = flags.get('--order-width')
    if width is None:
        probe = subprocess.run(executable + forwarded, env=dict(env, IG_ORDER_DISCOVER='1', IG_ORDER_SPEC='canonical'), capture_output=True, text=True)
        prefix = f'ORDER_SHAPE={site} '
        width = next((line[len(prefix):] for line in probe.stdout.splitlines() if line.startswith(prefix)), None)
        if width is None:
            sys.stderr.write(probe.stdout + probe.stderr)
            raise SystemExit(f'ordering site {site} was not reached')
    width = int(width)
    env['IG_ORDER_WIDTH'] = str(width)
    if spec != 'all':
        permutation(spec, width)
        raise SystemExit(run_ordered(executable, forwarded, env, spec))
    limit = int(flags['--order-limit']) if '--order-limit' in flags else None
    if limit is not None and limit < 1:
        raise SystemExit('order limit must be positive')
    start = permutation(flags.get('--order-start', 'canonical'), width)
    # itertools.permutations is lazy, but resumption must not walk a factorial prefix.
    order = list(start)
    count = 0
    failed = False
    while True:
        inverse_order = inverse(order)
        order_value = order_spec(order)
        print('ORDER_VARIANT ' + json.dumps(dict(site=site, selected=order_label(order), inverse=order_label(inverse_order))), flush=True)
        status = run_ordered(executable, forwarded, env, order_value)
        print(f"ORDER_RETURN selected={order_label(order)} exit={status}", flush=True)
        failed |= status != 0
        count += 1
        pivot = next((i for i in range(width - 2, -1, -1) if order[i] < order[i+1]), None)
        if pivot is None:
            print(f'ORDER_FAMILY status=complete completed={count}', flush=True)
            raise SystemExit(2 if failed else 0)
        last = next(i for i in range(width - 1, pivot, -1) if order[pivot] < order[i])
        order[pivot], order[last] = order[last], order[pivot]
        order[pivot+1:] = reversed(order[pivot+1:])
        if count == limit:
            print(f"ORDER_FAMILY status=held completed={count} next={order_label(order)}", flush=True)
            raise SystemExit(3)


if __name__ == '__main__':
    if len(sys.argv) == 3 and sys.argv[1] == '--variant-path':
        print(variant_path(sys.argv[2]))
    else:
        intercept()
