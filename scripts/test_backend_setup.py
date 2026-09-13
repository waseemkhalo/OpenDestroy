"""Regression checks for community secret generation and .env parsing."""
import base64
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parent


def load_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


initializer = load_module('backend_initializer', 'init-backend.py')
runner = load_module('backend_runner', 'run-backend.py')


class BackendSetupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='destroy-setup-test-')
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)

    def initialize(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            initializer.initialize(self.directory)
        return output.getvalue()

    def test_generated_secrets_match_and_are_private(self):
        output = self.initialize()
        values = runner.load_configuration(self.directory / '.env')
        token = (self.directory / 'data/desktop-access-token.txt').read_text().strip()
        self.assertEqual(len(base64.b64decode(values['DESTROY_DATA_KEY'], validate=True)), 32)
        self.assertGreaterEqual(len(token), 48)
        self.assertEqual(json.loads(values['DESTROY_USERS']), {hashlib.sha256(token.encode()).hexdigest(): 'local-user'})
        self.assertNotIn(token, output)
        self.assertNotIn(values['DESTROY_DATA_KEY'], output)
        if os.name == 'posix':
            for relative in ('.env', 'data/desktop-access-token.txt'):
                self.assertEqual(stat.S_IMODE((self.directory / relative).stat().st_mode), 0o600)
            self.assertEqual(stat.S_IMODE((self.directory / 'data').stat().st_mode), 0o700)

    def test_existing_configuration_token_or_database_is_never_overwritten(self):
        for relative in ('.env', 'data/desktop-access-token.txt', 'data/dictation.sqlite'):
            with self.subTest(existing=relative), tempfile.TemporaryDirectory(dir=self.directory) as name:
                directory = Path(name)
                existing = directory / relative
                existing.parent.mkdir(parents=True, exist_ok=True)
                existing.write_bytes(b'preserve existing data')
                with self.assertRaises(SystemExit):
                    initializer.initialize(directory)
                self.assertEqual(existing.read_bytes(), b'preserve existing data')
                for candidate in (directory / '.env', directory / 'data/desktop-access-token.txt'):
                    if candidate != existing:
                        self.assertFalse(candidate.exists())

    @unittest.skipUnless(hasattr(os, 'symlink'), 'symlinks unsupported')
    def test_broken_secret_symlink_is_not_followed(self):
        destination = self.directory / 'absent-target'
        (self.directory / '.env').symlink_to(destination)
        with self.assertRaises(SystemExit):
            initializer.initialize(self.directory)
        self.assertFalse(destination.exists())
        self.assertTrue((self.directory / '.env').is_symlink())

    def test_partial_creation_failure_removes_only_new_secret(self):
        real_open = os.open
        count = 0

        def fail_second_open(path, flags, mode):
            nonlocal count
            count += 1
            if count == 2:
                Path(path).write_text('created by another process')
                raise FileExistsError('concurrent configuration creation')
            return real_open(path, flags, mode)

        with patch.object(initializer.os, 'open', side_effect=fail_second_open):
            with self.assertRaises(FileExistsError):
                self.initialize()
        self.assertFalse((self.directory / 'data/desktop-access-token.txt').exists())
        self.assertEqual((self.directory / '.env').read_text(), 'created by another process')

    def test_parser_rejects_foreign_duplicate_and_non_string_values(self):
        configuration = self.directory / '.env'
        cases = ('PATH="foreign"', 'OPENAI_API_KEY=unquoted', 'OPENAI_API_KEY=42',
                 'OPENAI_API_KEY=null', 'OPENAI_API_KEY=[]', 'OPENAI_API_KEY="one"\nOPENAI_API_KEY="two"')
        for content in cases:
            with self.subTest(configuration=content):
                configuration.write_text(content)
                with self.assertRaises(SystemExit):
                    runner.load_configuration(configuration)

    def test_parser_preserves_quoted_strings_without_shell_expansion(self):
        configuration = self.directory / '.env'
        literal = '$(touch should-not-exist) `echo literal` ${HOME}'
        configuration.write_text('# comment\n\nOPENAI_API_KEY=' + json.dumps(literal) + '\n')
        self.assertEqual(runner.load_configuration(configuration), {'OPENAI_API_KEY': literal})
        self.assertFalse((self.directory / 'should-not-exist').exists())


if __name__ == '__main__':
    unittest.main()
