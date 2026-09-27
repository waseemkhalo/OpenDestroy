# OpenDestroy 0.1.1 — free Mac preview

A free, open-source voice dictation app for macOS. Hold a shortcut, speak, and your words land in the app you're using. You can also add GIFs, stickers and emoji, or attach a Drive file link, by voice. Speech can run entirely on your Mac, or through your own OpenAI or Gemini key. No account and no subscription.

## What's new in 0.1.1

- **Setup explains why Continue won't advance.** If a local model is still downloading, isn't installed, failed its integrity check, or the microphone isn't allowed yet, the reason now appears right above the button instead of nothing happening.
- **A branded installer.** The DMG opens to a drag-to-Applications window.

## Install

Download the DMG for your Mac, open it and drag OpenDestroy into Applications. Apple Silicon builds include NVIDIA Parakeet and Whisper; Intel builds include Whisper and cloud speech. Requires macOS 14 or newer.

Already on 0.1.0? Open **Settings** and check for updates.

Bring your own provider keys for cloud speech, GIPHY and Google Drive (via Composio). This is an early preview: see [PARITY.md](https://github.com/waseemkhalo/OpenDestroy/blob/main/docs/PARITY.md) for current limits.
