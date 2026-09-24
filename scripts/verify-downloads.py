#!/usr/bin/env python3
"""Verify public versioned download bytes before advertising an installer."""
import hashlib
import json
import re
import sys
import urllib.request
from pathlib import Path
from urllib.parse import urlsplit


def invalid(message):
    raise SystemExit(f'Invalid release metadata: {message}')


path = Path(sys.argv[1] if len(sys.argv) > 1 else 'artifacts/release.json')
try:
    data = json.loads(path.read_text())
except (OSError, ValueError) as error:
    invalid(f'cannot read JSON ({type(error).__name__})')
if not isinstance(data, dict):
    invalid('top-level value must be an object')
version = data.get('version')
if not isinstance(version, str) or not re.fullmatch(r'\d+\.\d+\.\d+', version):
    invalid('version must be semantic version text')
if data.get('available') is False:
    if data.get('downloads'):
        invalid('unavailable metadata must have no downloads')
    print('Download state is honestly unavailable')
    raise SystemExit(0)
if data.get('available') is not True:
    invalid('available must be true or false')
downloads = data.get('downloads')
if not isinstance(downloads, list) or not downloads:
    invalid('available metadata must include downloads')
expected_path = f'/releases/download/v{version}/'
for item in downloads:
    if not isinstance(item, dict):
        invalid('each download must be an object')
    url = item.get('url')
    if not isinstance(url, str):
        invalid('download URL must be text')
    parts = urlsplit(url)
    if (parts.scheme != 'https' or parts.hostname != 'github.com'
            or parts.username or parts.password or parts.fragment
            or expected_path not in parts.path):
        invalid('download URL must be a credential-free GitHub HTTPS release URL')
    size = item.get('bytes')
    if isinstance(size, bool) or not isinstance(size, int) or size <= 0:
        invalid('download size must be a positive integer')
    digest = item.get('sha256')
    if not isinstance(digest, str) or not re.fullmatch(r'[0-9a-fA-F]{64}', digest):
        invalid('download sha256 must be 64 hexadecimal characters')
    h = hashlib.sha256()
    actual_size = 0
    with urllib.request.urlopen(url, timeout=60) as response:
        while block := response.read(1024 * 1024):
            h.update(block)
            actual_size += len(block)
    if actual_size != size or h.hexdigest() != digest.lower():
        raise SystemExit('Release download checksum mismatch')
print('All public downloads match their exact release checksums')
