#!/usr/bin/env python3
"""Build updater/download metadata from complete, local signed release outputs."""
import argparse,datetime,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--repo',required=True);p.add_argument('--version',required=True);p.add_argument('--artifacts',default='artifacts');args=p.parse_args()
if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',args.repo):raise SystemExit('Invalid GitHub repository')
if not re.fullmatch(r'\d+\.\d+\.\d+',args.version):raise SystemExit('Expected a release version without v')
root=Path(args.artifacts);base=f'https://github.com/{args.repo}/releases/download/v{args.version}'
platforms={};downloads=[]
for target,arch in [('aarch64-apple-darwin','aarch64'),('x86_64-apple-darwin','x86_64')]:
 name=f'DestroyDictation_{args.version}_{target}';dmg=root/(name+'.dmg');bundle=root/(name+'.app.tar.gz');sig=root/(name+'.app.tar.gz.sig')
 for item in [dmg,bundle,sig]:
  if not item.is_file() or not item.stat().st_size:raise SystemExit(f'Missing release artifact: {item.name}')
 signature=sig.read_text().strip()
 if not signature:raise SystemExit(f'Empty update signature: {sig.name}')
 platforms['darwin-'+arch]={'signature':signature,'url':base+'/'+bundle.name}
 downloads.append({'arch':arch,'url':base+'/'+dmg.name,'sha256':hashlib.sha256(dmg.read_bytes()).hexdigest(),'bytes':dmg.stat().st_size})
(root/'latest.json').write_text(json.dumps({'version':args.version,'notes':f'Destroy Dictation {args.version}. See release notes before updating.','pub_date':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platforms':platforms},indent=2)+'\n')
(root/'release.json').write_text(json.dumps({'available':True,'version':args.version,'minimumMacOS':'14.0','source':f'https://github.com/{args.repo}','downloads':downloads},indent=2)+'\n')
