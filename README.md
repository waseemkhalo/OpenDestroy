# Destroy Dictation

Speak into the Mac app you are already using. Hold a shortcut, speak, and release to insert your words into the original field.

**Community preview, version 0.1.0.** This is a native macOS app and a self-hosted dictation service. No public signed installer is available yet. Real microphone, editor, provider and installation checks remain release gates.

## What is included

- Hold-to-talk with a separate recording HUD, waveform and destination-app icon.
- Text cleanup, explicit rewriting, selected-text editing, correction and undo.
- Microphone selection, language, vocabulary, writing preferences and voice snippets.
- Emoji choices and spoken selection; configurable GIF/sticker integration and favorites.
- Two-stage voice-note recording and handoff to the current app.
- Spoken lookup of saved file links, with a three-choice picker.
- Personal settings export/deletion and a local encrypted SQLite service.

“File dictation” here inserts a saved HTTPS link. It does not upload file bytes, search a cloud drive, change sharing, or transcribe imported audio/video files. Media availability depends on configured providers and the receiving app. See [capabilities and limitations](docs/PARITY.md).

This repository provides dictation software, not trained model weights. Speech and optional writing transformations use your configured providers. It is not an offline speech engine. Provider charges may apply.

## Run from source

You need macOS 14 or newer, Xcode command-line tools, Rust stable, Node.js 22+, and Python 3. The macOS version and both CPU architectures are build targets; only tested combinations should be treated as verified.

From the repository root, initialize the local service:

```sh
python3 scripts/init-backend.py
```

Edit the generated private `.env` and set `OPENAI_API_KEY` for batch transcription. Optional live transcription and media settings are explained in [Self-hosting](docs/SELF_HOSTING.md). Keep provider keys on the service; do not enter them into the desktop app.

Start the service in one terminal:

```sh
python3 scripts/run-backend.py
```

Start the native app in another:

```sh
cd apps/desktop
npm ci
npm run dev
```

`npm run dev` launches the native Tauri application. `npm run dev:web` is only a browser preview and cannot perform system-wide dictation.

In Connection & device, use `http://127.0.0.1:8787` and the token from `data/desktop-access-token.txt`. Allow Microphone and, for direct insertion, Accessibility. Choose a free shortcut if another app already uses the default. Focus a disposable editable document, hold the shortcut, speak and release. Never disable Gatekeeper as an installation workaround.

The community app uses its own bundle identity, Keychain namespace and preferences. It can be installed alongside another app without sharing its credentials.

## Develop and contribute

Start with [Contributing](CONTRIBUTING.md), [Architecture](docs/ARCHITECTURE.md), [Privacy](docs/PRIVACY.md), and [Verification](docs/VERIFICATION.md). Run the frontend, Rust and Python checks before submitting changes. The publication audit checks the source allowlist, staged files and reachable Git history.

The source license is [MIT](LICENSE); dependency licenses remain their authors’ licenses. The first public release requires the ownership and redistribution review described in [Release preparation](docs/RELEASE.md). No provider credentials, accounts or service entitlements are included.
