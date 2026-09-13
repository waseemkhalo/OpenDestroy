# Security

Please do not put tokens, raw recordings, selected text, exported personal data or unredacted logs in public issues.

Before enabling public reports, the repository owner should enable GitHub private vulnerability reporting. Use that channel when available. If it is not configured, ask the maintainer for a private reporting channel without posting exploit details or sensitive data publicly.

Version 0.1.0 is a pre-release candidate. It has not completed a third-party security audit or the real-hardware release matrix. Run the community backend on loopback by default and set provider spending limits.

Microphone access is explicit. Accessibility is used for the original editable field; secure and read-only fields are excluded. Provider API keys belong only on the backend. Backend access tokens are stored in a separate macOS Keychain namespace. Do not expose the service publicly without TLS, ingress limits, token management and a reviewed operational policy.

See docs/PRIVACY.md for retention, encryption boundaries and the limitations of deletion.
