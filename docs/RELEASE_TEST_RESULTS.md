# Release-test results

Status: source-preview evidence only; this is not signed-installer or publication approval.

Checked: 2026-09-19. This pass preserved the dirty worktree, ran one exclusive Cargo job, and did not run signing, notarization, uploads, publication, permission acceptance, live providers or microphone access. No provider credentials or persistent test data were used.

Current handoff status: frontend verification this turn reports 131 tests across 20 files, including the Home SSR regression, check clean with 0 errors/0 warnings, and a production build with 4,061 modules transformed. The source candidate is not binary-release ready.

## Native verification — September 19, 2026

Commands:

```sh
cargo fmt --all -- --check
cargo test --locked --workspace
xcodebuild -version
xcrun --sdk macosx --show-sdk-path
```

Results: formatting passed. The locked workspace test run compiled the desktop, backend and protocol crates; 80 desktop tests and 15 backend tests passed, 2 desktop model-dependent tests were ignored, and 0 tests failed. Xcode reported 27.0 and the supported SDK-path query resolved `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk`. The literal `xcrun sdk query` invocation is not available as a developer tool on this host; no license acceptance was attempted.

The native run is source/build evidence only. It does not verify a physical microphone, Accessibility-driven exact-field insertion, provider calls, an installer, signing, notarization or Gatekeeper acceptance.

## Safe offline release checks

Command:

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
```

Result: **15 tests passed.** This includes release metadata completeness checks, missing-signing-configuration checks, HTTPS/update-feed validation, honest-unavailable download metadata, and rejection of untrusted download metadata before a network fetch.

The known Node 22.20.0 runtime was available through the configured Node version manager. The default `/opt/homebrew/bin/node` launcher was not used because it references a missing ICU 73 dylib.

## Public-boundary audit

Command:

```sh
python3 scripts/audit-public.py
```

Result: **failed with 4 findings**, without printing credential values. The findings are outside reviewed publication scope: `apps/desktop/src/lib/dictation/savedLink.ts` and its test, each present across two reachable commits. Recommended non-rewrite resolution: owner-review these exact source files and, if approved for the public candidate, restore only these two exact paths to `scripts/public-files.json`; audit content/history checks remain active. If they are not approved, keep source publication blocked. This is separate from binary signing/notarization.

## Isolated backend HTTP smoke

Command:

```sh
python3 scripts/smoke-backend.py
```

Result: **passed.** The harness used the existing `target/debug/dictation-backend`, one ephemeral loopback port, generated throwaway access tokens, a generated encryption key, and a temporary SQLite database. Provider environment variables were removed. It covered startup, authentication, user isolation, vocabulary, link resolution, provider-disabled responses, encrypted files, export, and deletion after a deliberately corrupted record. The temporary directory and backend process were cleaned up.

The first sandbox-only attempt was blocked before startup by loopback bind permission; the same isolated harness passed when run with the permission required to bind its ephemeral test port. This is an execution-environment note, not an application failure.

## Still blocked

- Source publication: owner approval of the concrete candidate, provenance/license records, and resolution or explicit review of the four current public-audit history findings.
- The download verifier has not fetched public release bytes because no release exists and no network release check is authorized here.
- No live provider, physical microphone, exact editor insertion, media receiver, clean install, update, Gatekeeper, signing or notarization check was performed.
- Binary release: owner-controlled signing/updater configuration, notarization and release verification remain required for an installer. These are not prerequisites to publish source code.
- Source ownership/provenance and owner signoff on model license/redistribution terms remain candidate-review gates. Existing inventories and notices are factual documentation, not legal clearance.

Keep these automated results separate from the hardware/provider/install matrix in [Release preparation](RELEASE.md). Do not describe the app as release-ready from these tests alone.
