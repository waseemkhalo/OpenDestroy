# Verification — September 13, 2026

This is an independently extracted source candidate, not a shipped installer.

## September 24: private candidate and release preparation

The owner reports that actual dictation has been tested and works. This is
accepted as live dictation evidence; no repeat dictation test is requested.
Signed-installer clean-install and update acceptance remain separate.

Fresh checks: 148 frontend tests passed across 20 files; Svelte check reported
zero errors and warnings; the production frontend build passed (4,065 modules).
The locked offline Rust workspace suite passed 83 desktop tests and 15 backend
tests, with two model-dependent desktop tests ignored. Eighteen Python
setup/audit/release tests passed. Existing Rust dead-code warnings remain.
Rust formatting was normalized and its check passed.

Two packaging bugs were repaired: relative Cargo target directories are now
resolved before building, and signing-team verification requires an exact match.
The publication audit was reduced from 30 to 25 working-tree findings, all
artwork/logo binaries whose public redistribution evidence still needs review.
See `PUBLICATION_REVIEW.md`; subsequent index/history scans can repeat those
same assets across snapshots. The audit was not disabled or bypassed.

GitHub authentication and the standalone repository's private visibility were
verified with network access. The owner authorizes committing and pushing this
private candidate; this does not authorize changing repository visibility or
publishing an installer. No valid local code-signing identity was found, so
Apple notarization remains blocked on secure signing/notary configuration.
See `APPLE_RELEASE_PREFLIGHT.md` for prerequisites.

## September 23: quiet dictation completion and recovery HUD

Recording and processing now use brand red (`#e25345`). Native inline delivery
immediately hides the HUD and clears the completion message. Confirmed clipboard
fallback shows a readable explanation and the platform paste shortcut; a rejected
delivery/copy does not claim clipboard success. Recovery copies remain scoped to
the recording generation and account. No-speech-only markers are rejected before
delivery or history recording. The native idle HUD has room for the recovery UI.

Checks: 148 frontend tests passed; Svelte check reported zero errors/warnings;
production build passed; three native geometry tests passed (existing dead-code
warnings remain). An isolated browser preview of the actual components confirmed
red computed colors, readable 440px-wide recovery panels, and no HUD remaining
after simulated successful delivery. This is UI/lifecycle evidence, not a fresh
live microphone/paste acceptance run. The ignored `artifacts/hud-preview.html`
is a sample-state preview only and never records or accesses the clipboard.

## September 21: global microphone startup repair

A live user retry produced “Hold the shortcut until the microphone is ready, then speak.” Lifecycle diagnostics showed capture followed by cleanup without entering native delivery. Replaced global dictation's background WebKit microphone request with native AudioQueue capture. PCM stays in memory, bounded to 120 seconds; stop/cancel/account boundaries release it. Default input is supported; pinned WebKit device IDs require selecting System default. Batch transcription remains fenced to the connection owner and insertion remains fenced to the original field. No permission or target checks were bypassed.

Checks: 140 frontend tests passed; Svelte check zero errors/warnings; production frontend and native development builds passed; 83 native tests passed, 2 ignored. Native tests cover WAV framing/limits and ownership predicates; frontend regressions cover startup/key-up, cancellation and account changes. The user retried the running native build and confirmed successful dictation/insertion (“perfect”); its lifecycle trace reached the delivery claim without cancellation. The specific receiving app was not identified, so this is one live acceptance check, not an editor matrix or launch-readiness certification. Development-only session breadcrumbs are bounded to 64 static event/line records in a private process-specific temporary file; no audio, transcript, clipboard, target metadata or credentials are logged.

## Original landing snapshot

The results below are this landing snapshot. Locked frontend dependencies were installed and the bundled Node 24.19.0 runtime was used. Native changes now declare/register `public_setup` and `integrations`, and `Cargo.lock` contains the resolved `whisper-rs 0.16.0`/`symphonia 0.5.5` graph. The single owned Apple Silicon check compiled the speech stack, Tauri, and desktop crate successfully; the debug/ad-hoc app packaging step remains pending. CMake 4.4.3 was supplied from a task-local installation; no system packages were changed.

| Check | Result |
|---|---|
| Frontend tests | 108 passed |
| Svelte check | 0 errors, 0 warnings (UI worker final check) |
| Frontend production build | Passed: 4,041 modules; JS 175.93 kB (57.37 kB gzip), CSS 29.49 kB (5.86 kB gzip) |
| npm audit | Not rerun for this landing snapshot |
| Integrated Rust workspace check | Passed for `aarch64-apple-darwin` with `--offline --locked`; 12 unrelated dead-code warnings remain |
| Native desktop tests | 61 passed, 0 failed, 1 ignored; ignored structural test requires an explicit verified model/fixture smoke |
| Native backend tests | 15 passed, 0 failed |
| Rust formatting | Owned native files passed `rustfmt --edition 2021 --check` |
| Python setup/publication/release tests | 13 passed |
| Local backend HTTP smoke | Inherited candidate evidence; not rerun for this landing |
| Missing-provider behavior | Inherited candidate evidence; native/provider paths not rerun |
| Export/deletion/isolation | Inherited candidate evidence; not rerun for this landing |
| Source boundary/history | Targeted allowlist coverage passed; full history audit was stopped after exceeding 90 seconds |
| Live speech provider calls | Not verified |
| Physical microphone, notch, editors and media receivers | Not verified |
| Signed/notarized installation and updates | Not prepared or verified |
| Hosted CI | Workflow prepared; local results do not rely on hosted runs |

## Fresh native evidence — September 14, 2026

Volta completed `cargo test -p destroy-desktop --lib`: 71 passed, 0 failed,
2 ignored. The ignored coverage remains opt-in real-model loading and is not
treated as hardware or release evidence. The native regression set also passed
synthetic AAC/MP4 decoding, missing-metadata fallback, and input-rate duration
tests. A small portability patch is pending to embed the synthesized media
fixture instead of depending on runtime `ffmpeg`.

These tests do not verify a physical microphone, live provider calls, exact
editor insertion, media receivers, packaging, signing, notarization, or clean
installation/update behavior.

## Final frontend integration handoff — September 14, 2026

The final reviewed source handoff reports 116 frontend tests passed across 18
files, Svelte check passed with 0 errors and 0 warnings, and the production
build plus diff review passed. Final source review covered the visible modal
failure state, local reset shortcut behavior, failed-practice handling, and
model controls. No further build rerun is required for this handoff.

The public binary is not exposed to CUA. Real microphone behavior and native
editor insertion therefore remain unverified.

The native portability follow-up is complete: the synthesized AAC fixture is
embedded, removing the runtime `ffmpeg` dependency. The focused portability
regression passed (1 passed, 0 failed).

## Current native verification addendum

The post-integration Apple Silicon native check passed with one Cargo job and
the existing target cache. The workspace test run passed with 61 desktop tests
and 15 backend tests, with one model-dependent structural test ignored. The
opt-in Parakeet fixture smoke then loaded the exact pinned 670,619,803-byte
bundle through the native `transcribe-rs` adapter and produced a non-empty JFK
transcript from the public `whisper.cpp` fixture. Free space was 2,954,288 KiB
before download, 2,285,924 KiB after download, and 2,201,944 KiB after the
targeted smoke build/run; all measurements stayed above the revised 1.5 GiB
floor. The four model digests matched the pinned values in `LOCAL_MODELS.md`.

The ignored test is opt-in only and requires
`DESTROY_PARAKEET_SMOKE_MODEL` and `DESTROY_PARAKEET_SMOKE_WAV`; ordinary
builds and automated tests do not download models. `cargo fmt --all --
--check` also passed. The targeted compiler fix made the two local-model
installation error conversions explicit as `String`; no release or commit was
created.

The temporary smoke directory was removed after verification. No model was
installed into the user model store; 670,971,784 bytes were reclaimed.

The local HTTP smoke makes real loopback requests to SQLite-backed endpoints. Test fixtures are never provider fallbacks in the running app. No production database or cloud account is needed for these checks.

The candidate has fresh Git history and separate community app, Keychain and temporary-recording identities. Public source review must examine the exact commit/archive, including all history. No private infrastructure configuration or product planning documents are part of the export.

## Remaining release blockers

The implementation audit is complete for this landing snapshot. The native
launcher now follows the invoking Node runtime, including the bundled Node 24
runtime used by the verification environment; `npm run dev` still requires a
working local dependency install and the native macOS toolchain.

- **Source ownership:** review creator/source, ownership and redistribution
  permission for the icon, command-surface art and bundled fonts. Preserve any
  required attribution in `THIRD_PARTY_NOTICES.md`.
- **Runtime model:** review the exact model URL, checksum, upstream project,
  license and redistribution/attribution terms in `public_setup.rs`. Runtime
  download is not evidence that a shipped model artifact is license-cleared.
- **Release credentials:** owner-controlled Developer ID identity, Apple team,
  notarization profile, updater key pair and HTTPS feed are still absent from
  this checkout by design. Do not put them in source control.
- **Disk budget:** the audit environment currently reports only 361 MiB free
  after the lightweight frontend build.
  Do not start the large native/package build or download a model until space
  is reclaimed; no package artifact was created by this audit.
- **Acceptance:** live provider calls, physical microphone, exact-field editor
  insertion, media receivers, clean install, update, signing, notarization and
  Gatekeeper acceptance remain unverified. Automated tests do not establish
  those gates.

## Dependency review limits

The lockfile contains `glib 0.18.5` for the transitive Linux GTK graph, affected by [GHSA-wrw7-89jp-8q8g](https://github.com/advisories/GHSA-wrw7-89jp-8q8g). It is absent from the normal Apple Silicon macOS dependency graph checked with `cargo tree`. No Linux desktop release is offered. Do not force an incompatible GLib replacement into GTK 0.18; a Linux distribution needs a compatible upstream resolution first.

`cargo-audit` was unavailable on this machine, so a complete new Rust advisory scan was not run. npm's result does not establish that all dependencies are vulnerability-free. THIRD_PARTY_NOTICES.md lists exact-version upstream notice/cache gaps to review before distributing applicable binaries.

Before public source publication, confirm ownership/license rights and approve the exact source candidate. Before a binary release, complete the real-device/provider/installation matrix in RELEASE.md. Logic tests do not prove editor acceptance, provider accuracy or safe behavior in every app.
