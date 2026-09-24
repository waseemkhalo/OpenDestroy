# Architecture

```text
Shortcut hold in another Mac app
  → native exact-field/session capture
  → native AudioQueue microphone capture + recording HUD
  → one selected speech route:
      native local model, native BYOK provider, or self-hosted service/provider
  → dictation cleanup or explicit requested transform
  → native insertion into the captured field, or explained clipboard fallback
```

The desktop is a Tauri application. Its web renderer supplies onboarding, settings and capture UI; native Rust supplies global shortcuts, local model/provider speech, permissions, Keychain, focus checks, clipboard handoff and the independent recording HUD. A browser preview does not supply those native capabilities. These are source-level paths; real model, provider, microphone and editor acceptance remain separate release gates.

- `apps/desktop/src/lib/Onboarding.svelte`: first-run speech, permission, shortcut and optional-integration setup, coordinated by `apps/desktop/src/App.svelte`.
- `apps/desktop/src/lib/dictation`: capture orchestration helpers, intent grammar, preferences, snippets and pickers.
- `apps/desktop/src-tauri/src/public_setup.rs`: native speech selection, pinned model download/checking, native provider calls and local data routes.
- `apps/desktop/src-tauri/src/native_audio.rs`: session-owned, memory-only macOS PCM capture for global dictation. Uses System default input, a 120-second native deadline, bounded buffers, and batch speech transcription. It does not depend on a background WebKit capture request. Explicit WebKit-only device IDs are rejected rather than silently replaced.
- `apps/desktop/src-tauri/src/integrations.rs`: opt-in Keychain-backed Google Drive/Composio and GIPHY requests.
- `apps/desktop/src-tauri`: macOS integration and authenticated backend bridge.
- `crates/dictation-protocol`: shared bounded request/response types.
- `services/backend`: speech relay, optional writing transforms, media adapter and encrypted personal settings in SQLite.

The local service accepts a bearer access token whose SHA-256 hash maps to a stable user ID. The authenticated identity determines the data scope. The frontend never chooses another account by supplying its ID. Backend provider keys stay in the service environment; native provider keys stay in the macOS Keychain. The local model is downloaded into app support after checksum validation and is not included in this source tree.

Recorded audio and selections are transient. A voice note is an explicit file handoff with timed cleanup. Saved preferences, vocabulary, snippets, links, media favorites and usage aggregates are persistent personal data. See PRIVACY.md for exceptions and external copies.

The recording HUD is separate from the settings window. Starting or finishing speech must not resize or reposition the user's settings window. Its destination display uses the app icon. This community app has separate production/development bundle and credential namespaces.
