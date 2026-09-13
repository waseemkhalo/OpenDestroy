import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('public_audit', ROOT / 'scripts/audit-public.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)

class PublicationAuditTests(unittest.TestCase):
    def git(self, root, *args):
        return subprocess.run(['git', '-C', str(root), *args], check=True, capture_output=True)

    def repository(self, root):
        self.git(root, 'init', '-b', 'main')
        self.git(root, 'config', 'user.name', 'Test')
        self.git(root, 'config', 'user.email', 'test@example.invalid')
        (root / 'scripts').mkdir()
        (root / 'scripts/public-files.json').write_text(json.dumps({'files': ['README.md', 'scripts/public-files.json']}))
        (root / 'README.md').write_text('Safe public source.\n')
        self.git(root, 'add', '.')
        self.git(root, 'commit', '-m', 'Initial safe source')

    def test_removed_secret_in_reachable_history_still_fails(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.repository(root)
            # Construct dummy material so the regression fixture itself is safe.
            (root / 'README.md').write_text('sk-' + 'x' * 35)
            self.git(root, 'commit', '-am', 'Unsafe content')
            (root / 'README.md').write_text('Clean again.\n')
            self.git(root, 'commit', '-am', 'Remove content')
            issues, _, _ = audit.audit(root)
            self.assertTrue(any(location.startswith('history') and label == 'provider credential'
                                for location, _, label in issues))

    def test_staged_secret_cannot_be_hidden_by_clean_worktree(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.repository(root)
            (root / 'README.md').write_text('sk-' + 'y' * 35)
            self.git(root, 'add', 'README.md')
            (root / 'README.md').write_text('Safe unstaged edit.\n')
            issues, _, _ = audit.audit(root)
            self.assertIn(('index', 'README.md', 'provider credential'), issues)

    def test_scope_rejects_secrets_and_private_modules(self):
        allowed = {'services/backend/.env.example', '.env', 'services/auth/key.pem'}
        self.assertTrue(audit.allowed_path('services/backend/.env.example', allowed))
        for name in ['.env', 'services/auth/key.pem', 'website/index.html', 'data/state.sqlite', '../README.md']:
            self.assertFalse(audit.allowed_path(name, allowed))
        self.assertTrue(audit.allowed_path('third_party/licenses/example/LICENSE-MIT', set()))
        self.assertFalse(audit.allowed_path('third_party/licenses/example/private.json', set()))

    def test_untracked_out_of_scope_file_and_symlink_fail(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.repository(root)
            (root / 'unreviewed.txt').write_text('Not approved for publication.')
            (root / 'README.md').unlink()
            (root / 'README.md').symlink_to(root / 'unreviewed.txt')
            issues, _, _ = audit.audit(root)
            self.assertTrue(any(name == 'unreviewed.txt' and label == 'outside reviewed publication scope'
                                for _, name, label in issues))
            self.assertTrue(any(name == 'README.md' and 'symlink' in label for _, name, label in issues))

if __name__ == '__main__':
    unittest.main()
