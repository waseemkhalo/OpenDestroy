# App icon

`source.png` is the owner's original 1254 × 1254 artwork, preserved byte-for-byte
from `exec-c907fd90-4d04-4818-90fa-d12873271fe2.png`.
SHA-256: `7f44181edaf251e6d6ec2e9cd56b23a34fb64e5cd723a465db7e044001197326`.

The generator clips the artwork to a rounded square, with a 224-pixel corner
radius on a 1024-pixel canvas. The corners outside the shape are transparent;
the composition and black background inside it are preserved. The vector mask
is applied before generating every size, and `source.png` remains unchanged:

- PNG: 32, 64, 128, 256 (`128x128@2x.png`), and 512 (`icon.png`) pixels square.
- macOS `icon.icns`: 16, 32, 128, 256, and 512 point representations at both 1×
  and 2×, up to 1024 pixels square.
- Windows `icon.ico`: 16, 24, 32, 48, 64, and 256 pixels square.

To regenerate with the project's installed Tauri CLI (2.11.4 when generated):

```sh
cd apps/desktop
npm run icons
```

The script retains desktop assets only. `tauri.conf.json` lists these assets in
`bundle.icon`; development builds inherit them from the base configuration.
The current bundle targets remain macOS `app` and `dmg`.
