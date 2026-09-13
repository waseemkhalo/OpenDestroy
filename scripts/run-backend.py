#!/usr/bin/env python3
"""Load local configuration without shell expansion and launch the Rust service."""
import argparse
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def load_configuration(path):
    values = {}
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        try:
            key, value = line.split('=', 1)
            parsed = json.loads(value)
            if not key.startswith(('DESTROY_', 'OPENAI_', 'GEMINI_', 'GIPHY_')) or key in values or not isinstance(parsed, str):
                raise ValueError()
        except (ValueError, TypeError):
            raise SystemExit(f'Invalid configuration on line {number}; use JSON-quoted string values.') from None
        values[key] = parsed
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=ROOT, help='Configuration directory (default: repository root)')
    directory = parser.parse_args().directory.resolve()
    path = directory / '.env'
    if not path.is_file():
        raise SystemExit('No .env found. Run python3 scripts/init-backend.py first.')
    values = load_configuration(path)
    for key, value in values.items():
        os.environ.setdefault(key, value)
    db = Path(os.environ.get('DESTROY_DB', 'data/dictation.sqlite'))
    if not db.is_absolute():
        os.environ['DESTROY_DB'] = str(directory / db)
    os.chdir(ROOT)
    os.execvp('cargo', ['cargo', 'run', '--locked', '-p', 'dictation-backend'])


if __name__ == '__main__':
    main()
