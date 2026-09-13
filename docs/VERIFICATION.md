# Verification — September 13, 2026

This is an independently extracted source candidate, not a shipped installer.

| Check | Result |
|---|---|
| Frontend tests | 105 passed in 15 files |
| Svelte check | 0 errors, 0 warnings |
| Frontend production build | Passed; approximately 153 KB JavaScript before gzip |
| npm audit | 0 reported advisories at preparation time |
| Integrated Rust workspace tests | 58 passed: 43 desktop/native logic, 15 backend |
| Rust formatting | Passed |
| Python setup/publication/release tests | 13 passed |
| Local backend HTTP smoke | Passed using disposable credentials and local SQLite |
| Missing-provider behavior | Speech/media operations fail visibly when not configured |
| Export/deletion/isolation | Passed, including authenticated deletion of corrupt records |
| Source boundary/history | Checked by scripts/audit-public.py against explicit reviewed paths |
| Live speech provider calls | Not verified |
| Physical microphone, notch, editors and media receivers | Not verified |
| Signed/notarized installation and updates | Not prepared or verified |
| Hosted CI | Workflow prepared; local results do not rely on hosted runs |

The local HTTP smoke makes real loopback requests to SQLite-backed endpoints. Test fixtures are never provider fallbacks in the running app. No production database or cloud account is needed for these checks.

The candidate has fresh Git history and separate community app, Keychain and temporary-recording identities. Public source review must examine the exact commit/archive, including all history. No private infrastructure configuration or product planning documents are part of the export.

## Dependency review limits

The lockfile contains `glib 0.18.5` for the transitive Linux GTK graph, affected by [GHSA-wrw7-89jp-8q8g](https://github.com/advisories/GHSA-wrw7-89jp-8q8g). It is absent from the normal Apple Silicon macOS dependency graph checked with `cargo tree`. No Linux desktop release is offered. Do not force an incompatible GLib replacement into GTK 0.18; a Linux distribution needs a compatible upstream resolution first.

`cargo-audit` was unavailable on this machine, so a complete new Rust advisory scan was not run. npm's result does not establish that all dependencies are vulnerability-free. THIRD_PARTY_NOTICES.md lists exact-version upstream notice/cache gaps to review before distributing applicable binaries.

Before public source publication, confirm ownership/license rights and approve the exact source candidate. Before a binary release, complete the real-device/provider/installation matrix in RELEASE.md. Logic tests do not prove editor acceptance, provider accuracy or safe behavior in every app.
