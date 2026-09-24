# Capabilities and limitations

These statuses describe source implementation, not a shipped product. Automated logic tests and native/provider acceptance are separate. Record actual release evidence in VERIFICATION.md.

| Capability | Included behavior | Remaining limit |
|---|---|---|
| Hold/release shortcut | Configurable system-wide recording | Real keyboard/microphone acceptance pending |
| First-run setup | Select local speech, native BYOK speech, existing backend, permissions and optional integrations | Native onboarding/build acceptance pending |
| Recording HUD | Separate waveform, destination icon, cancellation state | Real notch/multiple-display checks pending |
| Input device | Native in-memory macOS capture for global dictation using System default | Pinned WebKit device IDs are rejected with a recovery instruction; native device selection and hardware acceptance pending |
| Streaming/fallback | Global dictation uses native PCM capture and batch transcription; WebKit streaming helper remains available separately | Native live streaming is not wired; real provider verification pending |
| Native speech | Local checksum-checked model or selected OpenAI/Gemini provider path | Model download/load, speech quality and provider checks pending |
| Direct insertion | Original editable field, secure/self/read-only exclusions | Editor acceptance matrix pending |
| Clipboard | Explained fallback; preserve later clipboard changes | Real editor/pasteboard matrix pending |
| Cleanup | Conservative cleanup and backtracking | Provider-backed transforms require configuration |
| Rewrite | Explicit requested rewrite with content guards | Live provider behavior pending |
| Selected text | Setting-controlled edit of an explicit selection | Original field must still be valid |
| Formatting | App-aware formatting with generic fallback | Unsupported editors use generic rules |
| Writing preferences | Language, vocabulary and style settings | Explicit style samples are transient |
| Voice snippets | Exact phrase/insert grammar and defined variables | Bounded snippet size/count |
| Correction/undo | Short-lived previous insertion in the same field | Native editor undo interaction pending |
| Emoji | Three choices, spoken/keyboard choice and cancel | Destination must accept text |
| GIF/sticker selection | Search, favorites and bounded rotation | Provider integration approvals/configuration required |
| Personal integrations | Opt-in native Google Drive/Composio lookup and direct GIPHY search with Keychain keys | Provider approval, OAuth scope, network and receiving-app checks pending |
| Voice note | Explicit ready step, then next hold creates a handoff file | Receiver may reject a file paste |
| Saved file links | Name/keyword lookup, up to three HTTPS links | No file-byte attachment; optional Drive lookup is a separate integration |
| Sharing | Existing link access remains unchanged | Manage access in the file provider |
| Imported media transcription | Not implemented | Do not advertise audio/video-file transcription |
| Usage | Daily word/speaking-time aggregates | No durable transcript archive |
| Personal data | Export, deletion and connection/session fencing | Backups/external copies require separate deletion |
| Signed installer/update | Packaging and draft preparation | Signing, notarization and install/update verification pending |

Do not substitute successful unit tests for real microphone, exact-field, receiving-app, provider or signed-installer evidence. Do not silently enable unavailable media providers or manufacture example provider responses in the running app.
