# Release preparation

This is a source preview. Do not label it a public, ready-to-download Mac release until the applicable checks below pass. A public source release and a signed binary release are separate decisions.

## Source candidate

Before the first publication, review the exact commit and its archive, not a working folder containing local data:

- Confirm the owner holds redistribution rights for every contributed/extracted source file and confirms the proposed MIT license. The presence of LICENSE alone is not evidence of those rights.
- Review THIRD_PARTY_NOTICES.md and preserve applicable upstream notices. Resolve any license obligations for the actual shipped dependency set.
- Run the source allowlist/credential audit against staged files and all reachable history. Use a new repository with fresh history; never change another repository's visibility to publish this slice.
- Run the frontend, native, backend and Python checks in CONTRIBUTING.md. Record limitations without implying live providers or hardware passed.
- Review README claims and docs/PARITY.md. No signed download URL, badges or provider promises should be invented.
- Obtain owner approval of this concrete source candidate before creating/publishing a public repository. The source preview can be published before installers if its limitations are explicit.

## Independent application identity

Production uses `org.destroy.dictation.community`; development uses `org.destroy.dictation.community.dev`. Each has its own app-support and Keychain namespace. Confirm these identifiers before a first public install; later changes need a migration plan.

The production product name is `Destroy Dictation`. The release package names are `DestroyDictation_VERSION_TARGET`. Supported build targets are `aarch64-apple-darwin` and `x86_64-apple-darwin`, with macOS 14.0 minimum. Do not claim an untested architecture is verified.

## Signed installer candidate

Configure a dedicated updater key pair and a verified repository/feed before enabling updates. Signing/notarization inputs are secrets, never committed:

- `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID`, and a local `APPLE_NOTARY_PROFILE`.
- `TAURI_SIGNING_PRIVATE_KEY` and optional password for updater signatures.
- `DESTROY_UPDATER_PUBLIC_KEY` and an HTTPS `DESTROY_UPDATE_URL`.

The manual draft workflow describes the corresponding GitHub environment values and imports signing credentials into an ephemeral Keychain. It prepares a draft only. Protect the `release` environment with owner review before storing credentials.

Run `scripts/package-macos.sh` for each target. It verifies Developer ID/hardened runtime, notarizes and staples the app and DMG, verifies Gatekeeper, and signs the stapled updater payload. Never ask a user to disable Gatekeeper.

Generate versioned metadata with:

```sh
python3 scripts/release-metadata.py --repo OWNER/REPOSITORY --version 0.1.0
```

Both architecture artifacts must exist before metadata is generated. Metadata generation is not a substitute for notarization and installation evidence. Prepare a GitHub draft, verify clean install plus update behavior, then obtain publication approval. After publication, use `python3 scripts/verify-downloads.py artifacts/release.json` to check the exact public bytes.

## Hardware acceptance

Use disposable text and record OS/app/CPU versions. Verify permission denial, repeated fast holds, quiet speech, cancellation, a missing or changed microphone, the two-minute cap, lost network, live-to-batch fallback, account changes during work and clipboard changes during handoff.

Check direct insertion/selection/correction/undo in Notes, TextEdit, Mail, browser editors and other supported apps. Verify secure fields and self-targets are rejected. Check emoji/media/link delivery and the two-step voice note in actual receiving apps. Confirm the HUD does not steal focus or move the user's settings window.

Verify a clean install from a quarantined signed DMG and an update from a previous signed build. Reject tampered updater signatures and preserve settings. A compile or ad-hoc launch does not satisfy these gates.
