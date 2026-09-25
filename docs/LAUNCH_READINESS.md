# Open-source launch readiness

Status: source-preview candidate only. This checklist does not authorize publication, signing, notarization, upload, or a change to repository visibility. A passing test suite is not evidence of real microphone, provider, editor, installer, or update acceptance.

Last checked: 2026-09-19, on the public checkout, with the existing dirty worktree preserved. Native checks ran on Apple Silicon with Rust 1.95.0, Xcode 27.0, and the macOS 27.0 SDK. The known Node 22.20.0 runtime was available for offline script checks; the default Homebrew Node launcher is currently broken against a missing ICU 73 dylib. See [release-test results](RELEASE_TEST_RESULTS.md) for command-level evidence.

## Verified in this pass

| Area | Evidence | Status |
|---|---|---|
| Locked frontend prerequisites | Current handoff evidence: `npm ci` completed from `apps/desktop/package-lock.json`; npm reported 0 vulnerabilities for the installed 80-package tree | Verified this turn by frontend handoff |
| Frontend static check | Current handoff evidence: `npm run check` — 0 errors, 0 warnings | Verified this turn by frontend handoff |
| Frontend tests | Current handoff evidence: `npm test` — 131 tests passed across 20 files, including the Home SSR regression | Verified this turn by frontend handoff |
| Frontend production build | Current handoff evidence: `npm run build` — 4,061 modules transformed successfully | Verified this turn by frontend handoff |
| Native formatting | `cargo fmt --all -- --check` | Verified |
| Native verification | Exclusive `cargo test --locked --workspace`: 80 desktop tests passed, 15 backend tests passed, 2 desktop model-dependent tests ignored, 0 failed. The run compiled the desktop, backend, and protocol crates. | Verified for source; no hardware/provider claim |
| Apple toolchain | `xcodebuild -version` reported Xcode 27.0; `xcrun --sdk macosx --show-sdk-path` resolved the macOS 27.0 SDK. The literal `xcrun sdk query` subcommand is unavailable on this host. | Verified with command distinction |
| Backend build | The workspace test run compiled the backend crate; standalone `cargo build --locked -p dictation-backend` was not rerun in this pass | Covered by workspace compile |
| Python setup/release/publication tests | `python3 -m unittest discover -s scripts -p 'test_*.py'` — 15 tests passed | Verified |
| Source boundary and secret scan | `python3 scripts/audit-public.py` — 4 findings in reachable history: `savedLink.ts` and `savedLink.test.ts` across two reachable commits. No credential values were printed. | Blocked pending candidate-scope review |
| Factual license inventory | `docs/MODEL_LICENSES.md` covers the seven model catalog entries; `THIRD_PARTY_NOTICES.md` now reconciles the resolved Whisper/Symphonia dependency notices | Verified inventory, owner clearance pending |
| Isolated backend HTTP smoke | Prior handoff evidence: existing built backend passed startup, auth/isolation, provider gates, encrypted data, export and corrupt-record deletion using temporary data | Inherited, test-only |
| Allowlist review | Current audit reports 4 historical outside-scope findings for `savedLink.ts` and `savedLink.test.ts` across two reachable commits; no allowlist changes were made in this pass | Blocked pending candidate-scope review |

## Blocking gaps

### Source-publication gates

#### P0 — owner candidate approval and provenance

**Update 2026-09-25:** The owner (waseemkhalo) states they own the extracted ("Sky-derived") source and approve publishing it under MIT. They also state they generated the Daily Ink artwork. Third-party app and provider logos are kept as marks used only to show compatibility. The owner must still approve the exact commit before making the repository public. The Cormorant font license text is still not in `third_party/licenses`. The paragraph below is the 2026-09-19 assessment.

Blocked. The public history records extraction commits for the standalone desktop/backend slice, but it does not provide a per-file origin and permission ledger for Sky-derived code. The factual model/license inventory and dependency notices are now documented, but they are not owner provenance or redistribution clearance. Creator/source and redistribution rights for the remaining icon and artwork are still unverified. Cormorant font claims and generated-art notes need retained source/license evidence, not just file presence. The newly approved `settings-landscape-v2.png` is recorded as built-in ImageGen output from an owner-supplied approved mock reference; this is not a third-party borrowed image, but the owner should retain that approval record outside the public repository.

Before publishing a source candidate, the owner must approve the concrete candidate and an exact-commit ledger covering every extracted or contributed file, including source/permission records, upstream license, required attribution, and files excluded from the candidate. This pass does not infer permissions or license clearance from file presence or this repository's `LICENSE`.

#### P0 — reviewed publication scope

**Update 2026-09-25:** Resolved. `python3 scripts/audit-public.py` passes after the owner approved the 25 binary assets recorded in `docs/PUBLICATION_REVIEW.md`. The paragraph below is the earlier snapshot.

`python3 scripts/audit-public.py` currently fails on four history findings: `apps/desktop/src/lib/dictation/savedLink.ts` and `savedLink.test.ts`, each appearing across two reachable commits. Recommended resolution: owner-review these exact source files and, if approved for the public candidate, restore only these two exact paths to `scripts/public-files.json`. This changes neither history nor audit behavior; the audit will continue scanning file contents and reachable history. If they are not approved, keep source publication blocked rather than hiding the findings. Do not rewrite history or broaden the allowlist blindly.

### Binary-release gates

Notarization is not required merely to publish source code. It is required only for the signed binary/installer path, which still needs owner-controlled candidate approval and release credentials.

#### P0 — model resource terms

Blocked on owner signoff for a binary release or bundled model. `docs/MODEL_LICENSES.md` records the seven catalog entries, pinned URLs, hashes, upstream-declared metadata, and the user-download-only boundary. This factual inventory does not establish permission to redistribute or promote use through this app. The owner must confirm the exact upstream commit, license, attribution and terms for every downloaded component before enabling a binary release or bundling any model.

#### P0 — real acceptance

Blocked. No live provider calls, physical microphone, Accessibility-driven exact-field insertion, editor matrix, media receivers, clean install, update, Gatekeeper acceptance, signed artifact, or notarized artifact was verified here. These remain manual gates in [Release preparation](RELEASE.md), and must be recorded with OS, app, CPU, receiving-app and provider details.

## P1 — required before a release candidate

- Keep the passing repository-wide Rust format check and locked workspace test result attached to the exact candidate; current frontend evidence is 131 tests across 20 files, check clean with 0 errors/0 warnings, and a 4,061-module production build.
- Review Newton's completed `THIRD_PARTY_NOTICES.md` reconciliation against the actual shipped target graph; resolve any remaining unavailable notice only if it applies to the chosen release artifacts.
- Review the explicit source allowlist additions already present in `scripts/public-files.json`, especially native modules, bundled fonts and artwork. The publication audit proves scope/pattern conditions only; it does not prove ownership or license rights.
- Keep `.env`, local databases, access tokens, provider keys, model files, build output and installers outside the candidate. Run the audit against the exact staged candidate and all reachable history from a complete clone.
- The isolated backend HTTP smoke passed with temporary data and the existing built binary. This does not cover live providers, a deployed service, or user data; rerun it from the exact candidate after any backend change.
- Have the owner review the onboarding/permissions path on a clean Mac: first-run route selection, backend token entry, Keychain storage, Microphone and Accessibility denial/recovery, shortcut collision, local model download failure, provider failure, and reset/delete behavior.
- Confirm hosted CI on the exact candidate. Local checks do not prove the GitHub runner, both macOS architectures, or a clean checkout.

## Release-only gates (not run)

Release inputs must be supplied by the owner through protected CI/environment storage or an ephemeral local keychain. Never save real credentials in source or this workspace. After owner approval of the exact candidate, the documented sequence is:

```sh
# From the repository root, after owner-controlled signing configuration exists.
bash scripts/package-macos.sh aarch64-apple-darwin
bash scripts/package-macos.sh x86_64-apple-darwin
python3 scripts/release-metadata.py --repo OWNER/REPOSITORY --version 0.1.0
python3 scripts/verify-downloads.py artifacts/release.json
```

The packaging script requires Developer ID signing, notarization, updater signing and an HTTPS feed. It must not be run by this launch worker. The workflow is draft-oriented, but creating a draft release, uploading artifacts, signing, notarizing, publishing, or changing repository visibility are owner actions outside this pass.

## Reproducible check sequence

Use a clean checkout of the exact candidate and the documented prerequisites (macOS 14+, Xcode command-line tools, Rust stable, Node 22+, Python 3):

```sh
cd apps/desktop
npm ci
npm run check
npm test
npm run build
cd ../..
cargo test --locked --workspace
cargo build --locked -p dictation-backend
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/audit-public.py
cargo fmt --all -- --check
```

Then complete the manual matrix in [Release preparation](RELEASE.md). Report automated results and hardware/provider results separately; do not call the app release-ready based on unit tests or a source build alone.
