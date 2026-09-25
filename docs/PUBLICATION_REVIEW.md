# Publication scope evidence — 2026-09-24

Status: binary-scope findings resolved by owner decision on 2026-09-25 (see "Owner decisions"); the text below is the 2026-09-24 snapshot. This review covers the dirty working tree based on `72c33f377e07709a52776c5ecc3307577fcb37b0`, not an approved release candidate. Existing changes were preserved. No ownership or redistribution rights are inferred from file presence, UI approval, hashes, or the project license.

## Narrow scope corrections

The initial `python3 scripts/audit-public.py` run reported 30 findings: four historical outside-scope findings, one personal source path in `design-qa.md`, and 25 unexpected binaries. The exact historical files reviewed are:

| Path | Commit | Git blob |
|---|---|---|
| `apps/desktop/src/lib/dictation/savedLink.ts` | `399e94b5c01d446c71590715f073994e94b5c396` | `07b880898513dd84abf98ae3bab65ec32ef05524` |
| `apps/desktop/src/lib/dictation/savedLink.test.ts` | `399e94b5c01d446c71590715f073994e94b5c396` | `a9b4e144c953cea95de8576dca5727f01cd8af70` |
| `apps/desktop/src/lib/dictation/savedLink.ts` | `96ad75ea64b5a58de4026f6bbfea0628499102cc` | `d2b4beb224ea93f8f00db04a6b33945331fd82a5` |
| `apps/desktop/src/lib/dictation/savedLink.test.ts` | `96ad75ea64b5a58de4026f6bbfea0628499102cc` | `fe1361b385469f9f3552301b207ce47f73648d47` |

The parser derives saved-link drafts from pasted HTTPS URLs, refuses URL credentials, and bounds names/keywords for the dictation backend's `/v1/links` API. Tests use example/provider-shaped URLs and exercise recognition, rejection and UTF-8 limits. The versions differ in spelling and the corresponding internal function name. Their behavior matches the standalone dictation scope; the current backend still implements `/v1/links` and `/v1/links/search`. Both exact paths are now allowlisted for reachable-history inspection, without restoring the deleted modules. This is a functional scope decision, not evidence of code ownership.

The personal source path in `design-qa.md` was replaced with the reference filename and repository asset path, preserving the recorded hash. This ledger is separately allowlisted. No audit implementation change was justified or made in this pass; all existing content, binary, index and history checks remain active.

A concurrent release agent added `scripts/test_package_macos.py` during review. Its contents were inspected: offline packaging regressions use temporary fixtures and mocked build/sign/upload commands with test-only values. This exact standalone release-test path was also allowlisted; its source was not edited in this pass.

The same concurrent work added `docs/APPLE_RELEASE_PREFLIGHT.md`. Its release prerequisites, mocked-test results and remaining signing/notarization gates were inspected for standalone publication scope and the exact path was allowlisted. Its factual claims remain that agent's evidence, not independently repeated checks by this reviewer.

## Asset evidence and current usage

This ledger records repository evidence only; upstream downloads, terms and provenance records were not independently authenticated in this pass. Existing notices are claims to substantiate, not new clearance.

| Exact asset or set | Source/usage evidence | Unresolved evidence |
|---|---|---|
| `apps/desktop/public/art/daily-ink-approved-source.png` | `design-qa.md` records a user-selected generated-image reference. Local SHA256 matches its recorded `342215fc70528fd6d755735ff05b4e39f939346ff2c8468e77f3694d2668b7b1`. `HomeScreen.svelte` uses it for the panorama/sidebar; `Settings.svelte` uses it for drawer art. | Retain the original generation/reference record and owner confirmation of source and redistribution rights. A matching hash and visual approval do not establish them. |
| `apps/desktop/public/art/daily-ink-landscape.png`, `apps/desktop/public/art/daily-ink-landscape-v2.png` | Present in public assets and the existing allowlist. No references found in current desktop source. | Source/generation and rights records; owner decision about their inclusion. Files were preserved. |
| `apps/desktop/public/art/command-surface.png` | Existing notices mention UI usage, but no reference was found in current desktop source. | Notices explicitly leave remaining-art ownership/permission unverified. |
| `apps/desktop/public/art/settings-landscape-v2.png` | `docs/LAUNCH_READINESS.md` describes ImageGen output based on an owner-supplied reference. Current Settings uses the approved Daily Ink source instead. | Retained generation/reference approval and redistribution evidence. |
| `apps/desktop/public/art/selection-brush.png`, `apps/desktop/public/art/voice-ink.png` | Existing notices report ImageGen origin. Current references are in `InkBrush.svelte`, `HomeScreen.svelte` and `style.css`. | Retain generation/reference records supporting the notices. |
| `apps/desktop/public/provider-logos/google-drive.png`, `apps/desktop/public/provider-logos/giphy.png` | `THIRD_PARTY_NOTICES.md` gives provider source links and branding qualifications. `SettingsConnections.svelte` references both locally. | Exact downloaded-byte provenance and applicable provider-use approval/attribution review remain unverified here. |
| `apps/desktop/public/app-icons/0.png` through `apps/desktop/public/app-icons/19.png` (20 individually allowlisted files) | Notices give the ordered product mapping and report retrieval via Google's favicon service, except the direct Google Docs favicon. `HomeScreen.svelte` has the same ordered 20-product list and constructs these paths. | Per-file source URL, source-byte/conversion evidence and permission/attribution review. Retrieval and attribution claims alone do not establish redistribution permission. |
| `apps/desktop/public/fonts/cormorant-regular.ttf`, `apps/desktop/public/fonts/cormorant-medium.ttf` | `style.css` loads both. Notices identify Cormorant/OFL. No Cormorant/OFL-named notice file was found under `third_party/licenses`. | Exact font source/version and retained applicable font license text. |
| `apps/desktop/src-tauri/icons/icon.png` | Existing audit exception and app-icon notice. | Notices explicitly leave creator/source and redistribution rights unverified. |

Absence of a source reference does not establish exclusion from distribution: these assets remain in the desktop public directory, and the Vite configuration does not disable public-directory copying. Review actual packaged contents for the final candidate. No artwork was deleted or replaced.

Additional local artwork fingerprints (SHA256):

| File under `apps/desktop/public/art/` | SHA256 |
|---|---|
| `command-surface.png` | `64b074249c44fa2b6fcbc84d922c7cee9ea4eb9440ae46c6c47f10d04e0e370f` |
| `daily-ink-landscape.png` | `563d4c1d36b1eaa0b9f0ae3eaa5db452c622a4a5944cabc4b339517c6f94a4ed` |
| `daily-ink-landscape-v2.png` | `9d741f89ea2117eb64227551bc35d58620928767fd6f4813597bf5b0a235624d` |
| `selection-brush.png` | `785079e9d02006e74df47b3e163d46b2ff6976f1fa5186f64287243f7f8ca5ae` |
| `settings-landscape-v2.png` | `69440e56b14461b6ce92e8ca07dac031b07be5e20ba2755dfbcd1377439ac330` |
| `voice-ink.png` | `9640bbf6d89519fd780e7558caf5b7630097b59eaef86cfa5f3ca3a387cfd9df` |

## Owner decisions — 2026-09-25

The repository owner (waseemkhalo) made these decisions and asked for them to be recorded. Each file is now listed by exact path in `PUBLIC_BINARY_PATHS` in `scripts/audit-public.py`; no pattern or blanket exemption was added.

| Files | Decision |
|---|---|
| `art/daily-ink-approved-source.png`, `art/daily-ink-landscape.png`, `art/daily-ink-landscape-v2.png` | Owner states they generated these images and approves publishing them. The two landscape files remain unused by current source. |
| `provider-logos/giphy.png`, `provider-logos/google-drive.png` | Kept to identify the optional GIPHY and Google Drive integrations. They remain the providers' marks; follow each provider's brand and attribution guidelines. |
| `app-icons/0.png` … `app-icons/19.png` | Kept on the Home screen to show compatibility with the named products. They remain third-party trademarks, are not relicensed under this project's MIT license, and imply no endorsement. |

The owner also states they own the extracted ("Sky-derived") source and approve publishing it under MIT. See `docs/LAUNCH_READINESS.md`.

After these entries, `python3 scripts/audit-public.py` passes: 1,215 distinct source blobs, 13 reachable commits, working tree and index checked.

## Remaining publication gate (superseded 2026-09-25)

Verification: the final `python3 -B scripts/audit-public.py` run exited 1 with exactly 25 findings, all unexpected binaries listed below; no remaining outside-scope or personal-path findings were reported. `python3 -B -m unittest discover -s scripts -p 'test_public_audit.py'` passed all four existing guard regressions. Scoped Git whitespace checks passed. Audit counts are a working-tree snapshot and must be rerun against the final candidate after concurrent edits finish.

The 25 binary findings cover the 20 app icons, two provider logos and three Daily Ink images above. Their existing path entries do not satisfy the audit's separate binary gate. They remain unresolved; no blanket binary exemption or weakened pattern was introduced. Previously exempted assets still need provenance review even though they do not produce audit findings.

Owner action: provide or confirm source/permission records for the concrete code and asset candidate, resolve applicable notices, and approve that exact candidate before publication. The historical saved-link scope correction does not close the broader extracted-code provenance gap documented in `docs/LAUNCH_READINESS.md`. Signing, notarization and native acceptance are separate workstreams and are not certified by this review. Earlier four-finding snapshots in other release documents predate this working-tree audit.
