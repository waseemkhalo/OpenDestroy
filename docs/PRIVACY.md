# Privacy

Recording starts on an explicit shortcut hold or the explicitly armed voice-note flow. Release finishes recording; cancellation discards the active session. Capture is bounded to two minutes. Microphone permission is required. Accessibility allows safe insertion into the original editable field. Screen Recording is not required.

The active app icon is displayed locally. Accessibility reads field metadata and an explicit selected range when needed for the requested edit. The app does not capture arbitrary screens or continuously collect window contents as background context. Speech, explicitly selected text and relevant writing preferences may go to your configured backend and provider for the requested operation.

Ordinary recordings remain in memory and are released after completion/cancellation. The service does not maintain a raw audio or transcript history. The latest transcript may remain in the desktop dashboard's process memory. A failed composition/delivery may offer a manual copy for 30 seconds; it clears on a new hold, cancellation, connection change or expiry.

Preferences, vocabulary, snippets, user variables, saved links, media favorites and aggregate usage are persistent. The community service application-encrypts those personal records in SQLite using an operator-controlled wrapping key. This protects a database copy separated from the key; it does not prevent the running service or the speech provider from reading inputs needed for processing. It is not end-to-end encryption.

An explicit voice note creates a private temporary handoff file with timed cleanup. Receiving apps may keep their own copy. A successful injected paste shortcut does not prove that an editor accepted an attachment; check before sending anything. This app does not send messages automatically.

Clipboard fallback deliberately replaces the clipboard so you can paste manually. Native insertion attempts restore the prior bounded pasteboard snapshot unless the user or another app changed it during the operation. Clipboard managers and receiving applications may retain separate copies.

The desktop stores its backend access token in a community-specific macOS Keychain namespace. Provider secrets remain in the service environment. Connection/account changes invalidate asynchronous capture and delivery and clear transient personal state. The app includes no transcript telemetry or ambient screen recording.

Export creates a new private JSON file in Downloads, containing plaintext personal data. Delete removes the current service account's stored personal envelope and clears app caches; separately remove exports and backups. It cannot erase a recipient's copy, another app's draft, clipboard history or provider-retained records. Review your provider's current retention settings before handling sensitive speech.
