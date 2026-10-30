"""The output gate rejects dynamic and malformed images before publication."""
import importlib.util
from pathlib import Path
import struct
import os
import json
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('standalone', ROOT / 'standalone.py')
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class StaticMembranePolicy(unittest.TestCase):
    def image(self, kind=1, tag=0, entry=0x400000):
        raw = bytearray(144)
        raw[:7] = b'\x7fELF\x02\x01\x01'
        struct.pack_into('<H', raw, 16, 2)
        struct.pack_into('<Q', raw, 24, entry)
        struct.pack_into('<Q', raw, 32, 64)
        struct.pack_into('<HH', raw, 54, 56, 1)
        struct.pack_into('<I', raw, 64, kind)
        struct.pack_into('<Q', raw, 72, 128)
        struct.pack_into('<Q', raw, 96, 16)
        struct.pack_into('<Q', raw, 128, tag)
        return raw

    def check(self, raw):
        # Patch file reading rather than creating ephemeral executable fixtures.
        from unittest.mock import patch
        with patch.object(Path, 'read_bytes', return_value=bytes(raw)):
            return policy.require_standalone('membrane')

    def test_static_exec_and_static_pie(self):
        self.check(self.image())
        raw = self.image(kind=2)
        struct.pack_into('<H', raw, 16, 3)
        self.check(raw)

    def test_external_interpreter_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'interpreter'):
            self.check(self.image(kind=3))

    def test_needed_filter_and_auxiliary_libraries_are_rejected(self):
        for tag in (1, 0x7ffffffd, 0x7fffffff):
            with self.subTest(tag=tag), self.assertRaisesRegex(ValueError, 'shared library'):
                self.check(self.image(kind=2, tag=tag))

    def test_nonexecutables_and_truncated_metadata_are_rejected(self):
        for raw in (b'#!/bin/sh\n', self.image(entry=0), self.image()[:100],
                    self.image(kind=2)[:136]):
            with self.subTest(length=len(raw)), self.assertRaises(ValueError):
                self.check(raw)

    def test_flags_enforce_static_after_inherited_override(self):
        original = {'RUSTFLAGS': '-D warnings -C target-feature=-crt-static',
                    'CARGO_ENCODED_RUSTFLAGS': '-C\x1ftarget-feature=-crt-static'}
        env = policy.static_env(original)
        self.assertTrue(env['RUSTFLAGS'].endswith('-C target-feature=+crt-static'))
        self.assertTrue(env['CARGO_ENCODED_RUSTFLAGS'].endswith('-C\x1ftarget-feature=+crt-static'))
        self.assertIn('-crt-static', original['RUSTFLAGS'])
        self.assertEqual(policy.static_env({'CARGO_ENCODED_RUSTFLAGS': ''})['CARGO_ENCODED_RUSTFLAGS'],
                         '-C\x1ftarget-feature=+crt-static')

    def test_shell_and_python_enforce_the_same_encoded_flags(self):
        for original in ('', '-C\x1ftarget-feature=-crt-static'):
            env = dict(os.environ, CARGO_ENCODED_RUSTFLAGS=original)
            returned = subprocess.check_output(
                ['bash', '-c', 'source "$1"; python3 -c \'import os,json; print(json.dumps(os.environ["CARGO_ENCODED_RUSTFLAGS"]))\'',
                 'policy-test', str(ROOT / 'scripts/static_membrane.sh')], env=env, text=True)
            self.assertEqual(json.loads(returned), policy.static_env(env)['CARGO_ENCODED_RUSTFLAGS'])


if __name__ == '__main__':
    unittest.main()
