# Self-hosting

The default community setup runs a service on your own Mac at `127.0.0.1:8787`. The provider you configure performs speech recognition. Self-hosting the relay does not make provider processing offline or free.

## Configure

Run `python3 scripts/init-backend.py` from the repository root. It generates a private `.env` and `data/desktop-access-token.txt` without printing their values. It refuses to overwrite existing secrets. Preserve the encryption key securely; do not generate a replacement over an existing database.

Edit `.env` using the existing JSON-quoted value format:

- `OPENAI_API_KEY` enables batch transcription and optional writing transforms.
- `OPENAI_STT_MODEL` and `OPENAI_TEXT_MODEL` select compatible speech/writing models.
- `GEMINI_API_KEY` with `GEMINI_PAID_PROJECT="true"` enables the optional live adapter; the operator must verify its project eligibility and configured model.
- `GIPHY_API_KEY` and the three `GIPHY_*_APPROVED` flags gate search/favorites. Those flags describe the operator's reviewed integration permissions; setting them is not evidence of provider approval. Leave them false until that review is complete.

All four of those settings are required together, and the service reads them at startup: restart it after editing. The app's GIFs & stickers row in settings names whichever ones are still missing, and `GET /v1/dictation/media/status` returns the same list.

Provider models and terms can change. Verify the configured endpoints and current provider requirements before enabling them. No live provider request has been validated as part of preparing this repository.

Run `python3 scripts/run-backend.py`. The desktop connects to its origin using the generated access token, not a provider API key. Without a speech provider, setup and settings work but transcription is unavailable.

## Local secrets and accounts

`DESTROY_USERS` maps SHA-256 token hashes to stable user IDs. Use strong random tokens. To revoke a token, remove its hash and restart the service; a restart also ends existing live connections. A replacement token may map to the same user ID.

`DESTROY_DATA_KEY` is exactly 32 random bytes encoded as base64. It protects encrypted personal settings in SQLite. Store a recovery copy separately from database backups. Losing the key makes the encrypted data unrecoverable. Automated wrapping-key rotation is not included.

The `.env.example` is syntax documentation, not working credentials. Keep `.env` and `data/` private and outside Git. Provider keys must never go in desktop build variables, issue reports or downloadable artifacts.

## Remote operation

Loopback is the recommended first setup. For a remote service, use a reviewed HTTPS reverse proxy with WebSocket support, ingress request limits, rate limits, per-token quotas and provider spending limits. Keep the service's direct listener private. The native client rejects plaintext remote origins and credential-bearing URLs.

Do not log authorization headers, query text, audio or request bodies. This source preview does not supply a managed hosting SLA, automatic customer provisioning or a complete public-service abuse prevention system.

## Backup and deletion

Use SQLite's online backup API or stop the service before copying database/WAL files. Encrypt backups, document their retention, and test restoration with the original key. Data export/deletion is available through the app and authenticated account routes. A personal export is plaintext; protect or delete it separately. Provider records and copies inserted into other apps follow their own retention policies.
