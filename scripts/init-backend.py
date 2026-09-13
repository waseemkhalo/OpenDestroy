#!/usr/bin/env python3
"""Create local backend secrets. Existing configuration is never overwritten."""
import argparse
import base64
import hashlib
import json
import os
import secrets
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def initialize(directory):
    directory = Path(directory).resolve()
    configuration = directory / '.env'
    data = directory / 'data'
    token_path = data / 'desktop-access-token.txt'
    if any(path.exists() or path.is_symlink() for path in (configuration, token_path, data / 'dictation.sqlite')):
        raise SystemExit('Configuration, desktop token or database already exists; nothing was changed.')
    directory.mkdir(parents=True, exist_ok=True)
    data.mkdir(mode=0o700, exist_ok=True)
    token = secrets.token_urlsafe(48)
    values = {}
    for line in (ROOT / 'services/backend/.env.example').read_text().splitlines():
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        key, value = line.split('=', 1)
        values[key] = json.loads(value)
    values.update(
        DESTROY_DATA_KEY=base64.b64encode(secrets.token_bytes(32)).decode(),
        DESTROY_USERS=json.dumps({hashlib.sha256(token.encode()).hexdigest(): 'local-user'}),
    )
    created = []
    try:
        for path, content in (
            (token_path, token + '\n'),
            (configuration, ''.join(key + '=' + json.dumps(value) + '\n' for key, value in values.items())),
        ):
            descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            created.append(path)
            with os.fdopen(descriptor, 'w') as output:
                output.write(content)
    except BaseException:
        for path in created:
            path.unlink()
        raise
    print('Created private .env and data/desktop-access-token.txt. Add speech provider keys to .env; keep them off the desktop app.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=ROOT, help='Configuration directory (default: repository root)')
    initialize(parser.parse_args().directory)
