#!/usr/bin/env python3
"""Check the reviewed public file allowlist, working tree, index and reachable history.

Print finding categories and paths only, never matched credential values.
This is a publication guard, not a substitute for human ownership/security review.
"""
import argparse
import json
import re
import subprocess
from pathlib import Path, PurePosixPath

SECRET_PATTERNS = {
    'private key material': rb'-----BEGIN (?:[A-Z]+ )*PRIVATE KEY-----',
    'provider credential': rb'\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{24,}',
    'Google API credential': rb'\bAIza[A-Za-z0-9_-]{30,}',
    'AWS access credential': rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b',
    'GitHub credential': rb'\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})',
    'credential-bearing database URL': rb'postgres(?:ql)?://[^\s:/]+:[^\s@]{8,}@',
    'personal source path': rb'/(?:Users|home)/[A-Za-z0-9_.-]+/',
}
SENSITIVE_SUFFIXES = ('.key', '.p12', '.p8', '.pem', '.sqlite', '.sqlite3', '.db', '.dmg', '.zip', '.tar.gz')
PUBLIC_BINARY_PATHS = {
    'apps/desktop/src-tauri/icons/icon.png',
    'apps/desktop/public/art/command-surface.png',
    'apps/desktop/public/art/selection-brush.png',
    'apps/desktop/public/art/settings-landscape-v2.png',
    'apps/desktop/public/art/voice-ink.png',
    'apps/desktop/public/fonts/cormorant-medium.ttf',
    'apps/desktop/public/fonts/cormorant-regular.ttf',
    # Owner-approved 2026-09-25; see docs/PUBLICATION_REVIEW.md "Owner decisions".
    'apps/desktop/public/art/daily-ink-approved-source.png',
    'apps/desktop/public/art/daily-ink-landscape.png',
    'apps/desktop/public/art/daily-ink-landscape-v2.png',
    'apps/desktop/public/provider-logos/giphy.png',
    'apps/desktop/public/provider-logos/google-drive.png',
    'apps/desktop/public/app-icons/0.png',
    'apps/desktop/public/app-icons/1.png',
    'apps/desktop/public/app-icons/2.png',
    'apps/desktop/public/app-icons/3.png',
    'apps/desktop/public/app-icons/4.png',
    'apps/desktop/public/app-icons/5.png',
    'apps/desktop/public/app-icons/6.png',
    'apps/desktop/public/app-icons/7.png',
    'apps/desktop/public/app-icons/8.png',
    'apps/desktop/public/app-icons/9.png',
    'apps/desktop/public/app-icons/10.png',
    'apps/desktop/public/app-icons/11.png',
    'apps/desktop/public/app-icons/12.png',
    'apps/desktop/public/app-icons/13.png',
    'apps/desktop/public/app-icons/14.png',
    'apps/desktop/public/app-icons/15.png',
    'apps/desktop/public/app-icons/16.png',
    'apps/desktop/public/app-icons/17.png',
    'apps/desktop/public/app-icons/18.png',
    'apps/desktop/public/app-icons/19.png',
}

def git(root, *args, check=True):
    return subprocess.run(['git', '-C', str(root), *args], stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=check).stdout

def allowed_path(name, allowed):
    path = PurePosixPath(name)
    if path.is_absolute() or '..' in path.parts:
        return False
    if path.name.startswith('.env') and name != 'services/backend/.env.example':
        return False
    if name.lower().endswith(SENSITIVE_SUFFIXES):
        return False
    if name in allowed:
        return True
    # Upstream text notices only; no arbitrary source or binaries in this directory.
    return (name.startswith('third_party/licenses/') and len(path.parts) == 4
            and path.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE', 'COPYRIGHT')))

def inspect(name, data, mode, allowed):
    issues = []
    if mode not in ('100644', '100755'):
        issues.append('symlink, submodule or unsupported file mode')
    if not allowed_path(name, allowed):
        issues.append('outside reviewed publication scope')
    for label, pattern in SECRET_PATTERNS.items():
        if re.search(pattern, data):
            issues.append(label)
    if b'\0' in data and name not in PUBLIC_BINARY_PATHS:
        issues.append('unexpected binary file')
    return issues

def audit(root):
    allowed = set(json.loads((root / 'scripts/public-files.json').read_text())['files'])
    if git(root, 'rev-parse', '--is-shallow-repository').strip() == b'true':
        raise ValueError('Full history is required; fetch with --unshallow before auditing.')
    issues, checked, seen = [], 0, set()

    def blob(name, oid, mode, location):
        nonlocal checked
        identity = (name, oid, mode)
        if identity in seen:
            return
        seen.add(identity)
        data = git(root, 'cat-file', 'blob', oid) if mode != '160000' else b''
        checked += 1
        issues.extend((location, name, label) for label in inspect(name, data, mode, allowed))

    # The index matters even when a developer has subsequently cleaned the worktree.
    indexed = git(root, 'ls-files', '--stage', '-z').split(b'\0')
    names = set()
    for row in indexed:
        if not row:
            continue
        meta, raw_name = row.split(b'\t', 1)
        mode, oid, stage = meta.decode().split()
        name = raw_name.decode()
        names.add(name)
        if stage != '0':
            issues.append(('index', name, 'unresolved merge'))
        blob(name, oid, mode, 'index')

    for raw in git(root, 'ls-files', '--others', '--exclude-standard', '-z').split(b'\0'):
        if raw:
            names.add(raw.decode())
    for name in sorted(names):
        path = root / name
        if not path.exists() and not path.is_symlink():
            continue
        mode = '120000' if path.is_symlink() else '100644'
        data = str(path.readlink()).encode() if path.is_symlink() else path.read_bytes()
        issues.extend(('worktree', name, label) for label in inspect(name, data, mode, allowed))

    commits = git(root, 'rev-list', '--all').decode().splitlines()
    for commit in commits:
        # Commit messages are public as well as file contents. Personal Git author
        # email policy is a separate owner choice, not a credential detector.
        message = git(root, 'show', '-s', '--format=%B', commit)
        for label, pattern in SECRET_PATTERNS.items():
            if re.search(pattern, message):
                issues.append(('history', commit[:12], label + ' in commit message'))
        for row in git(root, 'ls-tree', '-r', '-z', commit).split(b'\0'):
            if not row:
                continue
            meta, raw_name = row.split(b'\t', 1)
            mode, _, oid = meta.decode().split()
            blob(raw_name.decode(), oid, mode, 'history ' + commit[:12])
    return sorted(set(issues)), checked, len(commits)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    try:
        issues, checked, commits = audit(args.root.resolve())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print('Publication audit could not complete:', type(error).__name__)
        return 2
    for location, name, label in issues:
        print(f'{location}: {name}: {label}')
    if issues:
        print(f'FAILED: {len(issues)} publication finding(s). No credential values printed.')
        return 1
    print(f'PASS: {checked} distinct source blobs; {commits} reachable commits; working tree and index checked.')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
