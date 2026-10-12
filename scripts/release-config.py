#!/usr/bin/env python3
"""Write release-only signing/updater configuration; never emit secret values."""
import base64
import binascii
import json
import os
import sys
from pathlib import Path
from urllib.parse import urlsplit

# Installed 0.1.0 copies embed this minisign public key and this feed. An update
# signed by another key, or published anywhere else, is invisible to them.
# GitHub omits drafts and pre-releases from releases/latest.
SHIPPED_UPDATER_PUBLIC_KEY = (
    'dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY5NTkxNjc0NjM4RTQyMzYK'
    'UldRMlFvNWpkQlpaYVY3NWR2dnpudk5uMVNqeXdsV1dzL0pwR3NMZnRmNnI0V01tSkRSUGx4Wk8K'
)
SHIPPED_UPDATE_URL = 'https://github.com/waseemkhalo/OpenDestroy/releases/latest/download/latest.json'

def updater_public_key(value):
    compact = ''.join(value.split())
    expected = ''.join(SHIPPED_UPDATER_PUBLIC_KEY.split())
    expected_text = b''.join(base64.b64decode(expected).split())
    if compact == expected:
        return SHIPPED_UPDATER_PUBLIC_KEY
    try:
        decoded = b''.join(base64.b64decode(compact, validate=True).split())
    except (ValueError, binascii.Error):
        decoded = b''
    if decoded == expected_text or compact.encode() == expected_text:
        return SHIPPED_UPDATER_PUBLIC_KEY
    raise SystemExit('Updater public key does not match the key installed copies already trust')

required = ['APPLE_SIGNING_IDENTITY', 'DESTROY_UPDATER_PUBLIC_KEY', 'DESTROY_UPDATE_URL', 'TAURI_SIGNING_PRIVATE_KEY']
for key in required:
    if not os.environ.get(key):
        raise SystemExit(f'Missing release configuration: {key}')
if not os.environ['APPLE_SIGNING_IDENTITY'].startswith('Developer ID Application:'):
    raise SystemExit('A Developer ID Application identity is required')
try:
    feed = urlsplit(os.environ['DESTROY_UPDATE_URL'])
    if feed.scheme != 'https' or not feed.hostname or feed.username or feed.password or feed.fragment or feed.query:
        raise ValueError('Invalid update feed')
    _ = feed.port
except ValueError:
    raise SystemExit('Update feed must be a valid HTTPS URL without embedded credentials or a fragment')
if os.environ['DESTROY_UPDATE_URL'] != SHIPPED_UPDATE_URL:
    raise SystemExit('Update feed must be the releases/latest address already built into installed copies')
public_key = updater_public_key(os.environ['DESTROY_UPDATER_PUBLIC_KEY'])
config = {
    'bundle': {'createUpdaterArtifacts': True, 'macOS': {'signingIdentity': os.environ['APPLE_SIGNING_IDENTITY'], 'hardenedRuntime': True}},
    'plugins': {'updater': {'pubkey': public_key, 'endpoints': [SHIPPED_UPDATE_URL]}}
}
Path(sys.argv[1]).write_text(json.dumps(config, indent=2) + '\n')
