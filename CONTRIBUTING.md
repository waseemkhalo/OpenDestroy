# Contributing

Keep changes focused on dictation, media selection, saved file links, accessibility, reliability and the community setup. Describe the user-visible behavior, the exact checks you ran, and any hardware or provider checks still pending.

For the source-preview publication gates and the remaining open-source launch blockers, see [Launch readiness](docs/LAUNCH_READINESS.md). No source preview or installer is release-ready until the owner approves the exact candidate and the documented hardware, provider, ownership and installation gates are complete.

## Local checks

```sh
cd apps/desktop
npm ci
npm run check
npm test
npm run build
cd ../..
cargo test --locked --workspace
cargo build --locked -p dictation-backend
python3 scripts/smoke-backend.py
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/audit-public.py
```

Rust desktop checks require macOS. The service can be tested separately with `cargo test --locked -p dictation-backend` on a supported Linux toolchain.

Do not record customer speech or use personal documents in tests. Fixtures belong in tests, never in production provider fallbacks. Validate cancellation, exact-field insertion, clipboard preservation and connection/session boundaries when modifying capture or delivery.

Use meaningful commits. Do not commit `.env`, local databases, credentials, build output, generated installers, editor state or screenshots containing personal information. Intentional new source files require a reviewed update to `scripts/public-files.json`.

Use issue templates for reproducible bugs and focused requests. For vulnerabilities, follow SECURITY.md. Contributions must be yours to redistribute under the project license, with upstream notices preserved.

To refresh the dependency notice inventory, use Python 3.11+ and run `python3 scripts/generate-notices.py` after resolving Cargo dependencies and running `npm ci`. Review exact-version notice gaps before distributing the corresponding binaries.
