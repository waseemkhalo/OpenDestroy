# Contributor contract

This repository contains only the standalone dictation app, its local service, and the shared dictation protocol. Keep additions within that scope. Review changes to scripts/public-files.json explicitly; never regenerate the publication allowlist blindly.

Keep provider secrets in the backend. Fence asynchronous work to its captured connection and recording session. Selected text and raw recordings are transient. Insertion, correction and undo must target the exact original editable field. Preserve clipboard changes made by the user. Never send messages or change external sharing permissions from speech.

Expose unavailable capabilities honestly. Do not replace provider responses with fixture output in the app. Bound recording time, payloads, queues and provider requests. Keep hardware/provider verification separate from automated tests and preserve docs/PARITY.md.

No public release or deployment without owner approval of the concrete candidate. Prepare a draft first. Do not describe the app as shipped until its signed, notarized installer is downloadable and verified for real dictation.
