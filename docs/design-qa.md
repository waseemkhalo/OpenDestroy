# Design QA evidence

This is a source-level QA record, not release or installation evidence.

## Reference and harness

- Approved comparison board: two distinct Home and Settings screens.
- Source: the owner-supplied Home/Settings concept and subsequent screenshot corrections. The later request preserves watercolor artwork while replacing the uneven paper backdrop.
- Source crops use CSS clipping from the 1672×941 board; no raster crop or redraw was added.
- Isolated comparison harness: `/private/tmp/destroy-watercolor-qa`, served at `http://127.0.0.1:1424/`.
- Harness uses the real `HomeScreen.svelte`, `Settings.svelte`, production CSS, Cormorant fonts, and public artwork. Fixture callbacks remain harness-only.
- Harness stop note: the QA server on port 1424 was stopped after review.

## Current evidence

- `voice-ink.png`: present, opaque dark background, 1774×887.
- `selection-brush.png`: present, opaque charcoal background, 1536×1024; used as the CSS-selected-tab underline crop.
- Frontend tests: 122 passed across 19 files.
- Frontend production build: passed.
- Svelte check: passed with 0 errors and 0 warnings.
- Production diff review: passed.
- Final source review covered visible modal failure/reset shortcut behavior, failed-practice handling, and model controls.
- Publication audit passed: 1,076 distinct source blobs, 4 reachable commits, worktree and index checked. The exact allowlist includes the new brush and Settings components and form-validation helpers.
- Native desktop tests (Volta): 71 passed, 0 failed, 2 ignored; ignored coverage is opt-in real-model loading.
- Native synthetic-media regressions: AAC/MP4 decode, missing-metadata fallback, and input-rate duration tests passed.
- Native portability regression: 1 passed, 0 failed; the synthesized AAC fixture is now embedded and no runtime `ffmpeg` dependency remains.

## Refinement review

Visual result: passed for the reviewed browser component surfaces. Home and all four Settings tabs were inspected at the 680×700 target size. The moon, mountains, blossoms, and painted accents remain; the charcoal backdrop is uniform. Keys have an extruded lower base and a pressed state. The gesture label sits below the painted ribbon. Settings artwork occupies a separate area away from form labels.

Interaction evidence uses an isolated fixture, not a live provider account. A changed level deformed the painted segments; stopping recording reset their values to zero. Source wiring uses the live capture RMS for global dictation and practice. At a smaller 480×520 surface, Home scrolls to its lower controls. Connections expand into labeled credential panels. Structured variable inputs retain values; malformed JSON disables saving and structured edits until repaired, without losing existing values. No credentials were saved and no user data was reset during this review.

Runtime/release acceptance: blocked (not yet verified). Native microphone and practice transcription, global shortcut behavior, integrations, local reset semantics, receiving-app insertion, packaging, signing, notarization, and installation/update checks remain unverified. The public binary was not exposed to CUA, so this handoff does not add real microphone or native editor-insertion evidence.

## External-field and media refinement review

The later browser comparison caught and corrected a clipped/off-center Home moon, competing Settings artwork sizing rules, and a generic button rule overriding the black recording notch. The approved navigation remains unchanged. Settings uses a dedicated generated corner landscape rather than reusing the Home hero. Artwork is decorative and must not consume the settings scroll area's height or cover controls.

The current frontend suite passed 127 tests across 19 files; Svelte check reported zero errors and zero warnings. The recording fixture rendered a black 456-by-46 surface with a target-app label at the default camera geometry. This checks component rendering, not real display positioning, target-icon capture, audio capture, or paste delivery. An attempted browser viewport override was not reflected in the observed page dimensions, so this pass adds no minimum-window acceptance claim.

Source review also found personal-key GIPHY selections using text-link delivery. They now use the same bounded native media-delivery request as backend selections; ordinary receiver fallback remains native-owned. End-to-end GIPHY, Drive, voice-note, and external-field insertion still need real provider/receiver acceptance. A passing practice field does not satisfy that gate. See `LAUNCH_READINESS.md` for remaining release gates.
