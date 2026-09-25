#!/usr/bin/env python3
"""Regenerate notices from this repository's lockfiles and installed dependency caches.

Requires Python 3.11+. Run after npm ci and Cargo dependency resolution.
Never substitute a different version's notice for a missing dependency.
"""
import json
import shutil
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIXES = ('LICENSE', 'LICENCE', 'COPYING', 'NOTICE', 'COPYRIGHT')

def main():
    old = ROOT / 'third_party/licenses'
    rows, missing = [], []
    with tempfile.TemporaryDirectory(prefix='dictation-notices-') as temporary:
        output = Path(temporary) / 'licenses'
        output.mkdir()
        def preserve(folder, label):
            sources = []
            if folder and folder.exists():
                sources = [p for p in folder.iterdir() if p.is_file() and p.name.upper().startswith(PREFIXES) and p.stat().st_size < 1024*1024]
            if not sources and (old / label).exists():
                sources = [p for p in (old/label).iterdir() if p.is_file() and p.name.upper().startswith(PREFIXES)]
            for source in sources:
                target = output / label / source.name
                target.parent.mkdir(exist_ok=True)
                shutil.copyfile(source, target)
            return bool(sources)
        registry = Path.home() / '.cargo/registry/src'
        for package in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']:
            source = package.get('source', '')
            if not source:
                continue
            if not source.startswith('registry+'):
                raise SystemExit('Review non-registry dependency before generating notices.')
            name, version = package['name'], package['version']
            manifests = list(registry.glob('*/'+name+'-'+version+'/Cargo.toml'))
            folder = manifests[0].parent if manifests else None
            metadata = tomllib.loads(manifests[0].read_text()).get('package', {}) if manifests else {}
            license_name = metadata.get('license', 'Unresolved; review before distribution')
            copied = preserve(folder, 'rust-'+name+'-'+version)
            rows.append(('Rust', name, version, license_name))
            if not copied or not manifests:
                missing.append('Rust '+name+' '+version)
        for name_in_lock, package in json.loads((ROOT/'apps/desktop/package-lock.json').read_text())['packages'].items():
            if not name_in_lock:
                continue
            name = name_in_lock.split('node_modules/')[-1]
            version = package['version']
            folder = ROOT/'apps/desktop'/name_in_lock
            manifest = folder/'package.json'
            metadata = json.loads(manifest.read_text()) if manifest.exists() else package
            license_name = metadata.get('license', 'Unresolved; review before distribution')
            copied = preserve(folder, 'npm-'+name.replace('/','_').replace('@','')+'-'+version)
            rows.append(('npm', name, version, str(license_name)))
            if not copied:
                missing.append('npm '+name+' '+version)
        text = '''# Third-party notices

This inventory follows this repository's Cargo/npm lockfiles. Dependency licenses remain their authors' licenses; the project MIT license does not replace them. Original notice files available for the exact resolved versions are preserved under `third_party/licenses`.

The lockfiles include development, optional and cross-platform packages. Review the actual shipped dependency set and any reciprocal-license obligations before distributing binaries. An unavailable notice is a release-review item for artifacts that contain that dependency, not evidence that it is unlicensed.

| Ecosystem | Package | Version | Declared license |
|---|---|---|---|
'''
        for kind, name, version, license_name in sorted(rows):
            text += f'| {kind} | {name} | {version} | {license_name.replace("|", "or")} |\n'
        text += '\n## Notices requiring review\n\n'
        text += '\n'.join('- '+item for item in sorted(set(missing)))+'\n' if missing else 'None found missing in the resolved local dependency inventory.\n'
        (ROOT/'THIRD_PARTY_NOTICES.md').write_text(text)
        if old.exists():
            # Bundled assets (fonts, artwork) are not in any lockfile; keep their reviewed notices.
            for kept in old.glob('asset-*'):
                shutil.copytree(kept, output / kept.name)
            shutil.rmtree(old)
        old.parent.mkdir(exist_ok=True)
        shutil.copytree(output, old)
        print(f'Inventoried {len(rows)} dependencies; {len(set(missing))} exact-version notice/cache gaps require applicable distribution review.')

if __name__ == '__main__':
    main()
