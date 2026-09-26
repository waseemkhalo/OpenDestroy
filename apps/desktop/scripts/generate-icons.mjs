import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const icons = fileURLToPath(new URL('../src-tauri/icons/', import.meta.url));
const cli = fileURLToPath(new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url));
const output = mkdtempSync(join(tmpdir(), 'destroy-icons-'));

try {
  // Clip the original artwork with a vector mask so every size has smooth,
  // transparent corners. Keep the source image unchanged.
  const artwork = readFileSync(join(icons, 'source.png')).toString('base64');
  const roundedSource = join(output, 'rounded.svg');
  writeFileSync(roundedSource, `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs><clipPath id="rounded"><rect width="1024" height="1024" rx="224"/></clipPath></defs>
  <image width="1024" height="1024" xlink:href="data:image/png;base64,${artwork}" clip-path="url(#rounded)"/>
</svg>`);
  const result = spawnSync(process.execPath, [cli, 'icon', roundedSource, '--output', output], {
    stdio: 'inherit',
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Icon generation failed (${result.status ?? result.signal})`);

  // Keep desktop assets only; Tauri also generates mobile and Appx assets.
  for (const name of ['32x32.png', '64x64.png', '128x128.png', '128x128@2x.png', 'icon.png', 'icon.icns', 'icon.ico']) {
    copyFileSync(join(output, name), join(icons, name));
  }
} finally {
  rmSync(output, { recursive: true, force: true });
}
