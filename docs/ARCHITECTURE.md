# Architecture

```text
Shortcut hold in another Mac app
  → native exact-field/session capture
  → WebKit microphone capture + recording HUD
  → local service → configured speech provider
  → dictation cleanup or explicit requested transform
  → native insertion into the captured field, or explained clipboard fallback
```

The desktop is a Tauri application. Its web renderer supplies settings and capture UI; native Rust supplies global shortcuts, permissions, Keychain, focus checks, clipboard handoff and the independent recording HUD. A browser preview does not supply those native capabilities.

- `apps/desktop/src/lib/dictation`: capture orchestration helpers, intent grammar, preferences, snippets and pickers.
- `apps/desktop/src-tauri`: macOS integration and authenticated backend bridge.
- `crates/dictation-protocol`: shared bounded request/response types.
- `services/backend`: speech relay, optional writing transforms, media adapter and encrypted personal settings in SQLite.

The local service accepts a bearer access token whose SHA-256 hash maps to a stable user ID. The authenticated identity determines the data scope. The frontend never chooses another account by supplying its ID. Provider keys stay in the service environment.

Recorded audio and selections are transient. A voice note is an explicit file handoff with timed cleanup. Saved preferences, vocabulary, snippets, links, media favorites and usage aggregates are persistent personal data. See PRIVACY.md for exceptions and external copies.

The recording HUD is separate from the settings window. Starting or finishing speech must not resize or reposition the user's settings window. Its destination display uses the app icon. This community app has separate production/development bundle and credential namespaces.
