import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const args=process.argv.slice(2);

const xcodeProbeTimeoutMs=5000;
const isDevOrBuild=args[0]==='dev'||args[0]==='build';

function checkMacOsXcodeLicense(){
  if(process.platform!=='darwin'||!isDevOrBuild)return;

  // This is intentionally read-only. A non-zero first-launch status alone is
  // not enough to identify a license refusal, so use xcrun for the signal.
  const firstLaunch=spawnSync('/usr/bin/xcodebuild',['-checkFirstLaunchStatus'],{
    encoding:'utf8',
    stdio:['ignore','pipe','pipe'],
    timeout:xcodeProbeTimeoutMs
  });
  if(firstLaunch.error?.code==='ENOENT'||firstLaunch.status===0)return;

  const sdk=spawnSync('/usr/bin/xcrun',['--sdk','macosx','--show-sdk-path'],{
    encoding:'utf8',
    stdio:['ignore','pipe','pipe'],
    timeout:xcodeProbeTimeoutMs
  });
  const output=`${sdk.stdout??''}\n${sdk.stderr??''}`;
  if(!/you have not agreed to the xcode license agreements?/i.test(output))return;

  console.error('Xcode license agreements have not been accepted. Review and accept them in Xcode, or run `sudo xcodebuild -license` in a terminal, then retry.');
  process.exit(1);
}

checkMacOsXcodeLicense();

if(args[0]==='dev'||(args[0]==='build'&&args.includes('--debug')))args.push('--config','src-tauri/tauri.dev.conf.json');
// Execute the CLI entrypoint with this process's Node. The package bin is a
// /usr/bin/env node shim, which can silently select a different (and broken)
// system Node when the bundled runtime is used for npm scripts.
const cli=fileURLToPath(new URL('../node_modules/@tauri-apps/cli/tauri.js',import.meta.url));
const r=spawnSync(process.execPath,[cli,...args],{stdio:'inherit'});
if(r.error)throw r.error;
process.exit(r.status??1);
