#!/usr/bin/env python3
"""Verify public versioned download bytes before advertising an installer."""
import hashlib,json,sys,urllib.request
from pathlib import Path
p=Path(sys.argv[1] if len(sys.argv)>1 else 'artifacts/release.json');data=json.loads(p.read_text())
if not data['available']:
 assert not data.get('downloads'),'Unavailable metadata must have no downloads'
 print('Download state is honestly unavailable');raise SystemExit(0)
for item in data['downloads']:
 assert '/releases/download/v'+data['version']+'/' in item['url']
 h=hashlib.sha256();size=0
 with urllib.request.urlopen(item['url'],timeout=60) as response:
  while block:=response.read(1024*1024):h.update(block);size+=len(block)
 assert size==item['bytes'] and h.hexdigest()==item['sha256'],'Release download checksum mismatch'
print('All public downloads match their exact release checksums')
