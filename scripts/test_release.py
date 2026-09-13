import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REQUIRED = ['APPLE_SIGNING_IDENTITY', 'DESTROY_UPDATER_PUBLIC_KEY', 'DESTROY_UPDATE_URL', 'TAURI_SIGNING_PRIVATE_KEY']

class ReleaseTests(unittest.TestCase):
    def metadata(self, directory):
        return subprocess.run([sys.executable, str(ROOT/'scripts/release-metadata.py'), '--repo', 'example/dictation', '--version', '0.1.0', '--artifacts', directory], capture_output=True)

    def test_metadata_requires_complete_architectures_and_signature(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.assertNotEqual(self.metadata(folder).returncode, 0)
            self.assertFalse((root/'latest.json').exists())
            for target in ['aarch64-apple-darwin', 'x86_64-apple-darwin']:
                name = 'DestroyDictation_0.1.0_' + target
                for suffix in ['.dmg', '.app.tar.gz', '.app.tar.gz.sig']:
                    (root/(name+suffix)).write_text('test artifact')
            sig = root/'DestroyDictation_0.1.0_x86_64-apple-darwin.app.tar.gz.sig'
            sig.write_text('  ')
            self.assertNotEqual(self.metadata(folder).returncode, 0)
            self.assertFalse((root/'latest.json').exists())
            sig.write_text('test signature')
            self.assertEqual(self.metadata(folder).returncode, 0)
            metadata = json.loads((root/'latest.json').read_text())
            self.assertEqual(set(metadata['platforms']), {'darwin-aarch64', 'darwin-x86_64'})
            downloads = json.loads((root/'release.json').read_text())['downloads']
            self.assertEqual(len(downloads), 2)
            self.assertTrue(all('/releases/download/v0.1.0/DestroyDictation_' in item['url'] for item in downloads))

    def test_release_configuration_fails_without_secrets(self):
        env = {k:v for k,v in os.environ.items() if k not in REQUIRED}
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)/'config.json'
            result = subprocess.run([sys.executable, str(ROOT/'scripts/release-config.py'), str(output)], env=env, capture_output=True)
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

if __name__ == '__main__':
    unittest.main()
