# Apple release preflight

Checked 2026-09-24, updated with the main agent's fresh network-enabled GitHub
verification. The owner explicitly authorizes commit and push for a private
backup to the existing private repository. Public release still requires
exact-candidate approval and provenance review. Signing, notarization and
release-artifact uploads remain gated on approval of the exact candidate.
No release build or remote mutation was run by this preflight worker.

## Acceptance already established

The owner confirms actual dictation works. This preflight accepts that report
and does not request a repeat of that test. It does not establish clean install
or update acceptance for a newly signed installer, or both CPU architectures.
Older readiness documents that say no real dictation has been verified should
be read with this newer owner confirmation.

## Local prerequisites

| Check | Observation |
| --- | --- |
| Working tree | Many tracked modifications and untracked files; no fixed, reviewed release candidate established in this pass |
| Signing identity | `security find-identity -v -p codesigning` returned zero valid identities; identity details were filtered from output |
| Disk | Approximately 18 GiB available, volume reported 96% capacity; limited headroom for two architecture builds and packaging copies, not a measured build-space requirement |
| Apple tooling | `notarytool` is installed; `xcodebuild -checkFirstLaunchStatus` succeeded |
| Rust targets | Only `aarch64-apple-darwin` installed locally; Intel target is absent |
| Node | Default launcher fails because its required ICU library is missing; use a verified working Node 22+ runtime before packaging |
| GitHub CLI | Main agent reports fresh `gh auth status` with escalated network access passed: active login with `repo` and `workflow` scopes. The earlier invalid-login result was a sandbox artifact, not an authentication blocker |
| Git remote | Origin fetch/push use the intended standalone GitHub repository over HTTPS; main agent's `gh repo view` confirmed it is private with default branch `main` |
| Release environment | Secret/variable presence and owner approval protections have not yet been verified; successful authentication does not establish these |
| Existing artifacts | Source archive and its checksum only; no signed DMG or updater artifact in the artifacts directory |

The following variable names were checked for presence only. All were absent
from the current process environment; this does not prove absence from secure
storage or GitHub configuration:

- Local packaging: `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID`,
  `APPLE_NOTARY_PROFILE`, `TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `DESTROY_UPDATER_PUBLIC_KEY`,
  `DESTROY_UPDATE_URL`.
- CI credential inputs: `APPLE_CERTIFICATE_BASE64`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_API_KEY_BASE64`, `APPLE_API_KEY_ID`,
  `APPLE_API_ISSUER`, `DESTROY_UPDATER_PRIVATE_KEY`, `DESTROY_UPDATER_PASSWORD`.
- CLI/build overrides: `GH_TOKEN`, `GITHUB_TOKEN`, `CARGO_TARGET_DIR`.

The updater password is optional for an unencrypted key. The workflow sets its
own notary profile name and maps CI updater secrets to the Tauri variables.
No credential values or Keychain contents were retrieved. Stored notarization
credentials were not queried or exercised.

## Script and workflow inspection

- `scripts/package-macos.sh` verifies signing and hardened runtime, submits
  the app and DMG separately, staples both, assesses the app with Gatekeeper,
  and signs the final updater archive after stapling. It performs signing and
  uploads when executed; it is not a read-only preflight command.
- Fixed relative `CARGO_TARGET_DIR` handling: resolve and export the absolute
  repository-relative path before the build so packaging locates its output.
- Fixed signing-team verification: require a complete matching output line,
  preventing a longer team identifier from satisfying a prefix match.
- `.github/workflows/release.yml` is manually dispatched, uses the `release`
  environment for packaging and drafting, builds both architectures, imports
  temporary credentials, and creates a draft targeting the dispatched commit.
  Draft status does not prevent the earlier signing, notarization, or artifact
  uploads. Required owner review and bypass settings are external environment
  configuration, not guaranteed by the YAML; verify them before dispatch.
- `scripts/release-config.py` rejects missing signing/updater inputs and invalid
  HTTPS feeds. Presence and format checks do not establish key validity or
  matching public/private updater keys.
- `scripts/release-metadata.py` requires both architecture installers, updater
  archives, and nonempty signatures. It generates hashes and download metadata;
  it does not independently prove notarization or cryptographic signature
  validity. Generated `available: true` metadata must remain a draft artifact
  until the corresponding public bytes are released and verified.
- `scripts/verify-downloads.py` checks versioned GitHub URLs, sizes and hashes.
  Its live-download path was not run. It verifies installer bytes, not clean
  install behavior or updater signature acceptance.

## Verification in this pass

- Existing release tests: 5 passed.
- New packaging regressions: 3 passed, covering default/relative/absolute
  build directories, rejected team-prefix matches, and accepted exact matches.
  Build, signing, archive and notarization commands are mocked; successful
  verification ends at a sentinel before artifact creation.
- `bash -n scripts/package-macos.sh` and scoped `git diff --check` passed.
- Full application builds, downloads, notarization, signed install tests and
  hosted CI were intentionally not run.

## Remaining release gates

1. Before public release, establish and obtain owner approval of the exact
   candidate, including the final diff and intended release source, and
   reconcile the independent publication-audit and license/provenance review.
   Private backup commit/push is already authorized and is distinct from public
   release approval. This worker did not modify or rerun the other agent's
   publication audit.
2. Establish a working Node runtime and sufficient measured disk headroom.
   Install/verify the Intel toolchain only when the reviewed build is authorized.
3. Provision a valid Developer ID identity, notarization credentials, a dedicated
   matching updater key pair and approved HTTPS feed through secure storage.
   GitHub authentication is confirmed by the main agent's fresh check; verify
   required release secret/variable names and presence and owner approval
   protections without exposing values. Release environment verification is
   still outstanding.
4. After exact-candidate approval, build and verify each intended architecture,
   record notarization and stapling results, and test a quarantined signed DMG
   on a clean Mac. Verify onboarding and permissions for that installer, plus
   signed update/settings preservation and tampered-signature rejection using
   an appropriate signed baseline. Existing dictation confirmation remains valid.
5. Obtain publication approval and verify the exact downloadable installer
   bytes before advertising availability. No website or other repository was
   changed by this preflight.
