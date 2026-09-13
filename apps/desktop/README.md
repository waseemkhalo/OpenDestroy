# Destroy Dictation desktop

This is the native community Mac app, with dictation settings and a separate recording HUD.

After setting up the local backend as described in the repository README:

```sh
npm ci
npm run dev
```

`dev` launches Tauri. `npm run dev:web` serves only an interface preview on port 1422; the browser preview cannot capture native application context or insert dictation.

Connect the localhost backend and its generated access token in **Connection & device**. The access token goes in macOS Keychain; provider API keys stay with the backend. Grant microphone and Accessibility access when you choose to use dictation. Hold the configured shortcut while editing a field in another app, then release to finish.

The settings window retains your position and size. Recording uses a separate, non-focusable HUD, with the waveform on the left and destination icon on the right. Closing settings hides the window; reopening the application restores it. Use **Quit Destroy Dictation** to stop the app.

Release identity: `org.destroy.dictation.community`; development identity: `org.destroy.dictation.community.dev`. These identities, Keychain services and temporary recording directories are separate from other Destroy installations. If another app uses the default shortcut, choose a different one here.

```sh
npm test
npm run check
npm run build
```

Real microphone capture, exact-field insertion, media receiving, signing and notarized installation require separate macOS hardware verification. A passing frontend build does not establish those results. Updates remain disabled until a maintainer configures a signed release feed.
