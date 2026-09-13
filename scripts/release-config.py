#!/usr/bin/env python3
"""Write release-only signing/updater configuration; never emit secret values."""
import json
import os
import sys
from pathlib import Path
from urllib.parse import urlsplit

required = ['APPLE_SIGNING_IDENTITY', 'DESTROY_UPDATER_PUBLIC_KEY', 'DESTROY_UPDATE_URL', 'TAURI_SIGNING_PRIVATE_KEY']
for key in required:
    if not os.environ.get(key):
        raise SystemExit(f'Missing release configuration: {key}')
if not os.environ['APPLE_SIGNING_IDENTITY'].startswith('Developer ID Application:'):
    raise SystemExit('A Developer ID Application identity is required')
try:
    feed = urlsplit(os.environ['DESTROY_UPDATE_URL'])
    if feed.scheme != 'https' or not feed.hostname or feed.username or feed.password or feed.fragment:
        raise ValueError('Invalid update feed')
    _ = feed.port
except ValueError:
    raise SystemExit('Update feed must be a valid HTTPS URL without embedded credentials or a fragment')
config = {
    'bundle': {'createUpdaterArtifacts': True, 'macOS': {'signingIdentity': os.environ['APPLE_SIGNING_IDENTITY'], 'hardenedRuntime': True}},
    'plugins': {'updater': {'pubkey': os.environ['DESTROY_UPDATER_PUBLIC_KEY'], 'endpoints': [os.environ['DESTROY_UPDATE_URL']]}}
}
Path(sys.argv[1]).write_text(json.dumps(config, indent=2) + '\n')
