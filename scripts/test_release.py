import base64
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REQUIRED = ['APPLE_SIGNING_IDENTITY', 'DESTROY_UPDATER_PUBLIC_KEY', 'DESTROY_UPDATE_URL', 'TAURI_SIGNING_PRIVATE_KEY']
SHIPPED_UPDATE_URL = 'https://github.com/waseemkhalo/OpenDestroy/releases/latest/download/latest.json'
SHIPPED_UPDATER_PUBLIC_KEY = (
    'dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY5NTkxNjc0NjM4RTQyMzYK'
    'UldRMlFvNWpkQlpaYVY3NWR2dnpudk5uMVNqeXdsV1dzL0pwR3NMZnRmNnI0V01tSkRSUGx4Wk8K'
)

class ReleaseTests(unittest.TestCase):
    def metadata(self, directory):
        return subprocess.run([sys.executable, str(ROOT/'scripts/release-metadata.py'), '--repo', 'example/dictation', '--version', '0.1.0', '--artifacts', directory], capture_output=True)

    def test_metadata_requires_complete_architectures_and_signature(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.assertNotEqual(self.metadata(folder).returncode, 0)
            self.assertFalse((root/'latest.json').exists())
            for target in ['aarch64-apple-darwin', 'x86_64-apple-darwin']:
                name = 'OpenDestroy_0.1.0_' + target
                for suffix in ['.dmg', '.app.tar.gz', '.app.tar.gz.sig']:
                    (root/(name+suffix)).write_text('test artifact')
            sig = root/'OpenDestroy_0.1.0_x86_64-apple-darwin.app.tar.gz.sig'
            sig.write_text('  ')
            self.assertNotEqual(self.metadata(folder).returncode, 0)
            self.assertFalse((root/'latest.json').exists())
            sig.write_text('test signature')
            self.assertEqual(self.metadata(folder).returncode, 0)
            metadata = json.loads((root/'latest.json').read_text())
            self.assertEqual(set(metadata['platforms']), {'darwin-aarch64', 'darwin-x86_64'})
            downloads = json.loads((root/'release.json').read_text())['downloads']
            self.assertEqual(len(downloads), 2)
            self.assertTrue(all('/releases/download/v0.1.0/OpenDestroy_' in item['url'] for item in downloads))

    def test_release_configuration_fails_without_secrets(self):
        env = {k:v for k,v in os.environ.items() if k not in REQUIRED}
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)/'config.json'
            result = subprocess.run([sys.executable, str(ROOT/'scripts/release-config.py'), str(output)], env=env, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(output.exists())

    def test_release_configuration_accepts_the_key_and_feed_installed_copies_trust(self):
        env = {k: v for k, v in os.environ.items() if k not in REQUIRED}
        env.update({
            'APPLE_SIGNING_IDENTITY': 'Developer ID Application: Test',
            'DESTROY_UPDATER_PUBLIC_KEY': SHIPPED_UPDATER_PUBLIC_KEY,
            'DESTROY_UPDATE_URL': SHIPPED_UPDATE_URL,
            'TAURI_SIGNING_PRIVATE_KEY': 'test-only',
        })
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / 'config.json'
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/release-config.py'), str(output)], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            config = json.loads(output.read_text())
            updater = config['plugins']['updater']
            self.assertEqual(updater['pubkey'], SHIPPED_UPDATER_PUBLIC_KEY)
            self.assertEqual(updater['endpoints'], [SHIPPED_UPDATE_URL])
            env['DESTROY_UPDATER_PUBLIC_KEY'] = base64.b64decode(SHIPPED_UPDATER_PUBLIC_KEY).decode()
            output.unlink()
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/release-config.py'), str(output)], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            config = json.loads(output.read_text())
            self.assertEqual(config['plugins']['updater']['pubkey'], SHIPPED_UPDATER_PUBLIC_KEY)

    def test_release_workflow_publishes_the_feed_installed_copies_poll(self):
        workflow = (ROOT / '.github/workflows/release.yml').read_text()
        self.assertIn(SHIPPED_UPDATER_PUBLIC_KEY, workflow)
        self.assertIn('/releases/latest/download/latest.json', workflow)
        self.assertIn('--latest', workflow)
        self.assertNotIn('--draft', workflow)
        self.assertNotIn('--prerelease', workflow)

    def test_release_configuration_rejects_a_different_updater_key_or_feed(self):
        env = {k: v for k, v in os.environ.items() if k not in REQUIRED}
        env.update({
            'APPLE_SIGNING_IDENTITY': 'Developer ID Application: Test',
            'DESTROY_UPDATER_PUBLIC_KEY': SHIPPED_UPDATER_PUBLIC_KEY,
            'DESTROY_UPDATE_URL': 'https://github.com/example/other/releases/latest/download/latest.json',
            'TAURI_SIGNING_PRIVATE_KEY': 'test-only',
        })
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / 'config.json'
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/release-config.py'), str(output)], env=env, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(output.exists())
            env['DESTROY_UPDATE_URL'] = SHIPPED_UPDATE_URL
            env['DESTROY_UPDATER_PUBLIC_KEY'] = 'dGhpcyBpcyBub3QgdGhlIHNoaXBwZWQga2V5'
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/release-config.py'), str(output)], env=env, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(output.exists())

    def test_update_feed_rejects_credentials_and_non_https(self):
        env = {k:v for k,v in os.environ.items() if k not in REQUIRED}
        env.update({'APPLE_SIGNING_IDENTITY':'Developer ID Application: Test', 'DESTROY_UPDATER_PUBLIC_KEY':'test public key', 'TAURI_SIGNING_PRIVATE_KEY':'test key'})
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)/'config.json'
            for url in ['http://example.invalid/feed.json', 'https://', 'https://user:password@example.invalid/feed.json', 'https://example.invalid/feed.json#secret']:
                env['DESTROY_UPDATE_URL'] = url
                result = subprocess.run([sys.executable, str(ROOT/'scripts/release-config.py'), str(output)], env=env, capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.exists())

    def test_download_verifier_rejects_untrusted_metadata_before_fetch(self):
        script = ROOT / 'scripts/verify-downloads.py'
        with tempfile.TemporaryDirectory() as folder:
            metadata = Path(folder) / 'release.json'
            metadata.write_text(json.dumps({
                'available': True,
                'version': '0.1.0',
                'downloads': [{
                    'url': 'http://example.invalid/releases/download/v0.1.0/app.dmg',
                    'bytes': 1,
                    'sha256': '0' * 64,
                }],
            }))
            result = subprocess.run([sys.executable, str(script), str(metadata)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'credential-free GitHub HTTPS release URL', result.stderr)

    def test_download_verifier_accepts_honestly_unavailable_metadata(self):
        script = ROOT / 'scripts/verify-downloads.py'
        with tempfile.TemporaryDirectory() as folder:
            metadata = Path(folder) / 'release.json'
            metadata.write_text(json.dumps({'available': False, 'version': '0.1.0', 'downloads': []}))
            result = subprocess.run([sys.executable, str(script), str(metadata)], capture_output=True)
            self.assertEqual(result.returncode, 0)

if __name__ == '__main__':
    unittest.main()
