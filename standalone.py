"""Build-time static membrane policy shared by shell and Python bakers."""
import os
from pathlib import Path
import struct
import sys


def static_env(env=None):
    result = dict(os.environ if env is None else env)
    result['RUSTFLAGS'] = result.get('RUSTFLAGS', '') + ' -C target-feature=+crt-static'
    if 'CARGO_ENCODED_RUSTFLAGS' in result:
        previous = result['CARGO_ENCODED_RUSTFLAGS']
        result['CARGO_ENCODED_RUSTFLAGS'] = (previous + '\x1f' if previous else '') + '-C\x1ftarget-feature=+crt-static'
    return result


def require_standalone(path):
    """Reject an ELF that needs a loader or shared library before retaining it."""
    path = Path(path)
    raw = path.read_bytes()
    if len(raw) < 16 or raw[:4] != b'\x7fELF' or raw[4] not in (1, 2) or raw[5] not in (1, 2):
        raise ValueError(f'{path}: expected a static ELF executable')
    endian = '<' if raw[5] == 1 else '>'
    wide = raw[4] == 2

    def number(offset, code):
        size = struct.calcsize(code)
        if offset < 0 or offset + size > len(raw):
            raise ValueError(f'{path}: truncated ELF')
        return struct.unpack_from(endian + code, raw, offset)[0]

    if number(16, 'H') not in (2, 3) or number(24, 'Q' if wide else 'I') == 0:
        raise ValueError(f'{path}: ELF is not an executable image')
    off = number(32 if wide else 28, 'Q' if wide else 'I')
    size = number(54 if wide else 42, 'H')
    count = number(56 if wide else 44, 'H')
    if not count or size < (56 if wide else 32) or off + size * count > len(raw):
        raise ValueError(f'{path}: invalid ELF program headers')
    for index in range(count):
        at = off + index * size
        kind = number(at, 'I')
        if kind == 3:
            raise ValueError(f'{path}: external ELF interpreter; rebuild statically')
        if kind == 2:
            start = number(at + (8 if wide else 4), 'Q' if wide else 'I')
            length = number(at + (32 if wide else 16), 'Q' if wide else 'I')
            stride = 16 if wide else 8
            if length % stride or start + length > len(raw):
                raise ValueError(f'{path}: invalid ELF dynamic table')
            for entry in range(start, start + length, stride):
                tag = number(entry, 'Q' if wide else 'I')
                if tag == 0:
                    break
                if tag in (1, 0x7ffffffd, 0x7fffffff):
                    raise ValueError(f'{path}: external shared library; rebuild statically')
    return path


if __name__ == '__main__':
    try:
        if len(sys.argv) < 2:
            raise ValueError('usage: standalone.py EXECUTABLE ...')
        for argument in sys.argv[1:]:
            require_standalone(argument)
    except (OSError, ValueError) as error:
        sys.exit(str(error))
