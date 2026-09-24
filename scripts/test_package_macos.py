"""Offline packaging regressions: all build/sign/upload commands are mocked."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class PackageMacOSTests(unittest.TestCase):
    def run_package(self, target_dir, team='TESTTEAM', npm_status=77):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'scripts').mkdir()
            (root / 'apps/desktop').mkdir(parents=True)
            for name in ['package-macos.sh', 'release-config.py']:
                shutil.copyfile(ROOT / 'scripts' / name, root / 'scripts' / name)
            commands = root / 'bin'
            commands.mkdir()
            stubs = {
                'npm': 'printf "BUILD_ROOT=%s\\n" "$CARGO_TARGET_DIR"\nexit "$TEST_NPM_STATUS"',
                'codesign': 'printf "TeamIdentifier=%s\\nflags=0x10000(runtime)\\n" "$TEST_TEAM"',
                # A sentinel failure ends the successful verification path before
                # any artifact creation. Never call the real packaging tools.
                'ditto': 'echo REACHED_ARCHIVE\nexit 78',
                'xcrun': 'echo UNEXPECTED_NOTARIZATION\nexit 79',
                'hdiutil': 'exit 79',
                'spctl': 'exit 79',
            }
            for name, body in stubs.items():
                executable = commands / name
                executable.write_text('#!/bin/bash\n' + body + '\n')
                executable.chmod(0o755)
            env = {
                'PATH': str(commands) + ':/usr/bin:/bin',
                'APPLE_TEAM_ID': 'TESTTEAM',
                'APPLE_NOTARY_PROFILE': 'test-only',
                'APPLE_SIGNING_IDENTITY': 'Developer ID Application: Test',
                'DESTROY_UPDATER_PUBLIC_KEY': 'test-only',
                'DESTROY_UPDATE_URL': 'https://example.invalid/latest.json',
                'TAURI_SIGNING_PRIVATE_KEY': 'test-only',
                'TEST_TEAM': team,
                'TEST_NPM_STATUS': str(npm_status),
            }
            (root / 'apps/desktop/package.json').write_text('{"version":"0.1.0"}')
            if target_dir is not None:
                env['CARGO_TARGET_DIR'] = target_dir
            result = subprocess.run(
                ['/bin/bash', str(root / 'scripts/package-macos.sh'), 'aarch64-apple-darwin'],
                cwd=root, env=env, capture_output=True, text=True,
            )
            expected = target_dir if target_dir and os.path.isabs(target_dir) else str(root / (target_dir or 'target'))
            return result, expected

    def test_build_and_packaging_share_absolute_target_directory(self):
        for target_dir in [None, 'relative build', '/tmp/test-only-destroy-target']:
            with self.subTest(target_dir=target_dir):
                result, expected = self.run_package(target_dir)
                self.assertEqual(result.returncode, 77, result.stderr)
                self.assertIn('BUILD_ROOT=' + expected + '\n', result.stdout)

    def test_team_identifier_requires_exact_match(self):
        result, _ = self.run_package(None, team='TESTTEAMEXTRA', npm_status=0)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('REACHED_ARCHIVE', result.stdout)
        self.assertNotIn('UNEXPECTED_NOTARIZATION', result.stdout)

    def test_exact_team_reaches_archive_sentinel(self):
        result, _ = self.run_package(None, npm_status=0)
        self.assertEqual(result.returncode, 78, result.stderr)
        self.assertIn('REACHED_ARCHIVE', result.stdout)
        self.assertNotIn('UNEXPECTED_NOTARIZATION', result.stdout)


if __name__ == '__main__':
    unittest.main()
