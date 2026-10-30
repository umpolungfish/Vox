"""Exercise the public launchers without starting factor searches."""
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('programs', ROOT / 'tools/programs.py')
programs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(programs)


class ProgramOperations(unittest.TestCase):
    def command(self, repo, *args, code=0):
        result = subprocess.run([str(repo / 'programs'), *args], cwd=ROOT.parent,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, code, result.stderr)
        return result

    def test_public_help_and_catalogs(self):
        for repo in (ROOT, ROOT.parent / 'G-mOMonadOS'):
            with self.subTest(repo=repo.name):
                self.command(repo, '--help')
                self.command(repo, 'bake', '--help')
                rows = json.loads(self.command(repo, 'list', '--all', '--json').stdout)
                self.assertTrue(rows)
                self.assertTrue(all(row['available'] for row in rows))
                for row in rows:
                    # Every discoverable program has inspectable usage, including
                    # nested projects and measurement recipes in the baker index.
                    text = self.command(repo, 'help', row['name']).stdout
                    self.assertIn(f"./programs run {row['name']}", text)

    def test_nested_targets_and_short_name_resolution(self):
        repo = ROOT.parent / 'G-mOMonadOS'
        entries = programs.catalog(repo)
        for name in ('membranes/bin:membrane_ecm_extract',
                     'gpu_port/spine_probe/bin:spine_probe',
                     'gpu_port/fold_probe/bin:fold_probe'):
            self.assertIn(name, entries)
        self.assertEqual(programs.resolve(repo, 'membrane_ecm_extract'),
                         entries['membranes/bin:membrane_ecm_extract'])

    def test_invalid_inputs_fail_before_building(self):
        for arguments in (('bake', 'factor', '--number', '1'),
                          ('bake', 'factor', '--number', 'a'),
                          ('bake', 'word'), ('bake', 'cpu'),
                          ('run', 'no-such-program'),
                          ('bake', 'factor', '--base', '1')):
            with self.subTest(arguments=arguments):
                self.command(ROOT, *arguments, code=2)

    def test_dynamic_saved_executable_is_rejected(self):
        result = self.command(ROOT, 'execute', '/bin/true', code=2)
        self.assertIn('interpreter', result.stderr)

    def test_child_failure_and_argument_boundaries_survive(self):
        with patch.object(programs.subprocess, 'run') as run:
            run.return_value = subprocess.CompletedProcess([], 7)
            with self.assertRaises(programs.CommandFailure) as failure:
                programs.invoke(['tool', 'one argument', '$(literal)'], ROOT)
            self.assertEqual(failure.exception.code, 7)
            self.assertEqual(run.call_args.args[0], ['tool', 'one argument', '$(literal)'])


if __name__ == '__main__':
    unittest.main()
