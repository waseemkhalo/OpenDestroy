import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const args=process.argv.slice(2);
if(args[0]==='dev'||(args[0]==='build'&&args.includes('--debug')))args.push('--config','src-tauri/tauri.dev.conf.json');
const binary=fileURLToPath(new URL('../node_modules/.bin/tauri',import.meta.url));
const r=spawnSync(binary,args,{stdio:'inherit'});process.exit(r.status??1);
