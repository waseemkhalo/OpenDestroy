# Personal integrations

This build keeps Composio and GIPHY opt-in and local to the desktop app. It does not use a Destroy-hosted integration service, proxy provider media, provision provider accounts, or include provider credentials in the repository.

## Storage and account fencing

- Composio project keys, GIPHY keys, and the generated stable local integration user ID are stored only in the native macOS Keychain.
- JavaScript receives presence/status and provider results, never a key. Keys are not written to app storage, logs, exports, or URLs emitted by the app.
- Each async request captures an integration generation. Saving or removing either key invalidates older work and prevents its response from being applied.
- Google Drive connected accounts are filtered by the stable user ID and `googledrive` toolkit. The app accepts only `ACTIVE` accounts belonging to that identity.

## Composio project setup

Mandatory:

1. Create or use a dedicated personal Composio project and create a project API key for that project. A scoped project key should grant only the connected-account permissions needed for connection management and read-only tool execution; do not use an organization key.
2. Configure a Google Drive OAuth auth config in that project. The app uses Composio Connect Link (`POST /api/v3.1/connected_accounts/link`) and opens the returned `https://connect.composio.dev/link/...` URL in the native browser. No app localhost callback server is required; the UI calls `refresh_google_drive` after the browser flow.
3. The app executes only `GOOGLEDRIVE_FIND_FILE`, whose current documented search argument is `arguments.q`, alongside `user_id`, the selected connected account, and `version: "latest"`. Composio’s execution response is read from `data.files`. It does not call sharing, permission, write, upload, download, proxy, or arbitrary-tool endpoints.
4. The result is limited to three existing `https://drive.google.com`, `https://docs.google.com`, or `https://drive.usercontent.google.com` links. Provider ordering is retained.

If no suitable config exists, the app creates a project-local Composio-managed auth config automatically. It requests Google’s `https://www.googleapis.com/auth/drive.metadata.readonly` scope and restricts the config to `GOOGLEDRIVE_FIND_FILE`, the sole read-only tool used by this integration. Existing configs are used only when they are Google Drive, Composio-managed, enabled, restricted to that exact tool, and explicitly use a read-only Drive scope; the app never silently picks a wider or unrelated config.

Scope guidance:

- Required baseline: the automatically created config uses `drive.metadata.readonly`, which can view file metadata and links but cannot modify or download file content. `drive.file` is narrower but only covers files created by or explicitly granted to the app, so it is suitable if the workflow later adds a picker.
- Conditional: do not widen this integration to `drive.readonly` or `drive`; the current existing-link finder does not need content access. If a future product change needs a broader scope, verify it in Google Cloud and review the new action before changing this contract.
- Recommended: use a customer-owned verified Google OAuth app in the auth config when the project needs a custom scope, branding, or dedicated quota. Composio-managed OAuth is supported for the basic connection flow.

Official references: [Composio projects](https://docs.composio.dev/reference/api-reference/projects), [scoped project API keys](https://docs.composio.dev/reference/authenticating-to-composio/project-api-key-permissions), [auth config creation](https://docs.composio.dev/reference/api-reference/auth-configs/postAuthConfigs), [scope control](https://docs.composio.dev/docs/authentication/controlling-scopes), [Connect Link](https://docs.composio.dev/reference/v3/api-reference/connected-accounts/postConnectedAccountsLink), [connected accounts](https://docs.composio.dev/reference/api-reference/connected-accounts), [Google Drive toolkit](https://docs.composio.dev/kb/guide/toolkits-googledrive), and [Google Drive scopes](https://developers.google.com/workspace/drive/api/guides/api-specific-auth).

## GIPHY BYOK setup

Mandatory:

1. Create a GIPHY API key in the GIPHY Developer Dashboard and enter it in the app. The key is stored only in the native Keychain.
2. Use the default GIF search path unless sticker access has separately been reviewed for the intended use. The app sends the user query as entered, URL-encoded by the native HTTP client, with a maximum of 50 characters and three results.
3. Keep the UI attribution visible as `Powered by GIPHY`. Results retain GIPHY’s full HTTPS `preview_url`, `content_url`, and `source_url` values, including query parameters.
4. Load media directly from GIPHY URLs. This build does not reorder results, mix GIPHY results with another provider, or use a backend proxy for delivery. On supported macOS targets, both backend-routed and direct personal-key results use the native delivery path: the selected HTTPS asset is revalidated, downloaded into a bounded 8 MiB in-memory buffer, and pasted as GIF/WEBP/PNG with HTML and plain-text link fallbacks while preserving the prior clipboard. The app does not write a media file or maintain an app-managed media cache; the operating-system pasteboard may retain the pasted payload until replaced. Non-macOS builds use the plain link fallback.

This implementation does not establish permission to download, paste, or redistribute GIPHY media. Keep `Powered by GIPHY`, retain provider URLs, and complete the GIPHY terms/attribution review before release.

Recommended:

- Use separate GIPHY keys per platform and app section as GIPHY requests.
- Register view/click/send analytics with GIPHY when the product flow and review permit it; this build does not silently add analytics calls.
- Use smaller fixed renditions for previews and a larger rendition after selection. The implementation preserves provider URLs rather than rewriting them.
- Upgrade a beta key to production only after the app’s provider review and rate-limit needs justify it.

Official references: [GIPHY API requirements](https://developers.giphy.com/docs/api/), [search endpoint](https://developers.giphy.com/docs/api/endpoint/), and [GIPHY rendition guidance](https://developers.giphy.com/docs/).

## Honest limitations

The desktop cannot verify provider approval, Google OAuth verification, or the exact OAuth scopes configured in a personal project. Those remain owner/provider setup responsibilities. Network failures, expired/revoked connections, invalid keys, and malformed provider responses return bounded generic errors without exposing provider response bodies or query keys.
