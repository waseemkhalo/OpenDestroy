<p align="center">
  <img src="apps/desktop/public/art/voice-ink.png" alt="A red sun over ink-wash mountains" width="720">
</p>

<h1 align="center">Destroy Dictation</h1>

<p align="center">
  <b>Your voice, in ink.</b><br>
  Hold a key, speak, and release. Your words land in the Mac app you’re already using.<br>
  Open source. No account. No subscription. Speech can stay entirely on your Mac.
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#what-you-can-say">What you can say</a> ·
  <a href="#privacy">Privacy</a> ·
  <a href="docs/PARITY.md">Status &amp; limitations</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

> **Community preview (0.1.0).** Build it from source today. A signed, notarized installer is not available yet, and real-hardware release checks are still in progress. See [status](#status).

<!-- Add a 30–60 second demo GIF or video here, recorded on a real Mac. -->

## Why Destroy

- **Works where you already type.** Slack, Gmail, Notion, Cursor, ChatGPT, Docs. If the field is editable, Destroy inserts into that exact field. Secure and read-only fields are never touched.
- **You choose where speech runs.** Run a local model such as NVIDIA Parakeet on your Mac, so audio never leaves the machine. Or use your own OpenAI or Gemini key, or your own self-hosted relay. Destroy never silently switches to a different provider.
- **More than dictation.** Pick an emoji, GIF or sticker by voice. Attach a Drive file by saying “attach my resume”. Hand off a voice note. Undo the last insertion by voice.
- **Nothing to sign up for.** Keys live in the macOS Keychain. Transcripts stay in memory, not on disk. There is no Destroy account and no hosted service.

## What you can say

| Say | What happens |
|---|---|
| *hold ⌘` and speak* | Clean text is inserted into the field you were typing in |
| “**undo that**” / “**scratch that**” | Removes your last insertion from the same field |
| “That demo was incredible. **Add a proud reaction GIF**” | Keeps your sentence, then offers three GIFs; say “**two**” or “**show me more**” |
| “**Insert a sticker of** thumbs up” | Sticker picker |
| “**Add a** celebration **emoji**” | Three emoji choices you can pick by voice or keyboard |
| “Hey Amy, **attach my pitch deck**” | Finds the file in Google Drive or your saved links and inserts its existing link; sharing is unchanged |
| “**Voice note**” | Your next hold records a voice note and hands it to the current app |
| *a saved snippet phrase* | Inserts the snippet, with your saved variables filled in |

Writing preferences, vocabulary and snippets are in **Dictionary** and **Writing** in the app.

**Needs the self-hosted service:** “…**rewrite**” at the end of an utterance, voice edits to selected text, and corrections like “**change Tuesday to Wednesday**”. In on-device and bring-your-own-key modes these are turned off rather than sent anywhere unexpected. See [Self-hosting](docs/SELF_HOSTING.md).

GIFs use your own [GIPHY](https://developers.giphy.com/) key. Drive search uses your own [Composio](https://composio.dev/) project with read-only metadata access. Both are optional and set up during onboarding or later in Settings. See [Integrations](docs/INTEGRATIONS.md).

## Quick start

**Requirements:** macOS 14+, Xcode Command Line Tools, CMake, Rust stable and Node.js 22+.

```sh
git clone https://github.com/waseemkhalo/Destroy-Dictation.git
cd Destroy-Dictation/apps/desktop
npm ci
npm run dev
```

The first launch walks you through three steps:

1. **Speech.** Choose **On this Mac** and download a local model. The pinned Parakeet model is about 640 MB and checksum-verified (see [local models](docs/LOCAL_MODELS.md)). Or paste an OpenAI or Gemini key.
2. **Your Mac.** Allow Microphone. Allow Accessibility so text goes straight into the field; without it, Destroy falls back to copy-and-paste. Pick a shortcut if ⌘` is taken.
3. **Connections.** Optionally add GIPHY or Google Drive.

Then click into any text field, hold your shortcut, speak and release.

> `npm run dev` builds and launches the native app. `npm run dev:web` only previews the interface in a browser; it can’t dictate. Never disable Gatekeeper to run a build.

<details>
<summary><b>Advanced: run the self-hosted service</b></summary>

The optional service adds provider-backed rewrite, selection editing, correction and live transcription. It runs on your own Mac or server, and provider keys stay on the service.

```sh
python3 scripts/init-backend.py   # creates a private .env; set OPENAI_API_KEY
python3 scripts/run-backend.py    # serves http://127.0.0.1:8787
```

In onboarding, choose **Advanced connection** and enter `http://127.0.0.1:8787` and the token from `data/desktop-access-token.txt`. Remote deployments need HTTPS and the controls described in [Self-hosting](docs/SELF_HOSTING.md).
</details>

## Privacy

- **On this Mac:** audio is transcribed in-process and never leaves your machine.
- **Your own key:** audio goes only to the provider you chose, billed to your account.
- **Transient data:** raw recordings and selected text are not kept. Recent transcripts live in memory for the session.
- **Local storage:** only settings, vocabulary, snippets, saved links, favorites and daily word counts are stored. You can export or delete them from Settings.
- **No actions on your behalf:** Destroy never sends messages or changes file sharing from speech.

Full details: [Privacy](docs/PRIVACY.md) · [Security](SECURITY.md)

## Status

Destroy is a **community preview**. The source builds and its automated suites pass. The following are still being verified on real hardware before a signed release:

- microphone and editor behavior across apps
- local model speed on Apple Silicon
- provider calls
- installing and updating from a notarized DMG

[PARITY.md](docs/PARITY.md) lists every capability with its remaining limits. [VERIFICATION.md](docs/VERIFICATION.md) records what has been tested and how.

**Not included:** transcribing imported audio or video files, Windows or Linux apps, and bundled model weights (models download on demand).

## Contributing

Bug reports, editor-compatibility reports and focused pull requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) and [Architecture](docs/ARCHITECTURE.md). Please don’t put recordings, tokens or personal text in public issues; report vulnerabilities as described in [SECURITY.md](SECURITY.md).

## License

Source code is [MIT](LICENSE). Dependencies keep their own licenses ([notices](THIRD_PARTY_NOTICES.md)). The optional Parakeet model is published by its authors under CC BY 4.0 and downloaded separately; see [model licenses](docs/MODEL_LICENSES.md). No provider credentials, accounts or service entitlements are included.
