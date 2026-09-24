# Daily Ink implementation QA — 2026-09-21

## Latest follow-up: normal-window layouts and provider icons

Verified the public app components in the local browser fixture at 1034×772 CSS pixels. Insights now retains three equal columns (each roughly 256px wide, aligned at the same vertical coordinate). Writing uses three short, wide tone choices (roughly 259×77px); its preview moves below the controls when the content area is narrower than 960px. Dictionary search retains one outer focus indicator; the focused input has no inner outline, border or shadow. Connections displays bundled official Google Drive and GIPHY icons, with provenance in THIRD_PARTY_NOTICES.md and explicit public asset allowlisting. Provider screenshot: `/private/tmp/destroy-home-review.E1uZOi/connections-layout-fix.png`.

Validation: Svelte check reported zero errors/warnings, all 136 tests across 20 files passed, production build and git whitespace checks passed. These are frontend layout checks, not native dictation or provider-authentication certification. Existing artwork and motion remain unchanged.

## Latest: approved secondary pages and motion

Final result: passed for the frontend design/interaction scope. This is not a native dictation or launch-readiness certification.

Implemented in the public `destroy-dictation` repository only. Dictionary, Writing, History, Connections and the Settings drawer follow the approved black/ivory/vermilion layouts. The existing approved Home painting remains the single persistent main artwork layer and scales down on secondary pages; returning Home scales it back up. Settings uses the same source artwork. Added eased page entrances, moving pill indicators, reversible drawer and disclosure transitions, and reduced-motion fallbacks. Dictionary/Writing drafts survive main-page switches; Data drafts survive settings-section switches.

Compared each supplied screen with the implementation in the same tool output at 1487×1058. Evidence in `/private/tmp/destroy-home-review.E1uZOi/`: `dictionary-final.png`, `writing-final.png`, `history-final.png`, `connections-final.png`, `settings-final.png`. Checked Writing and Settings at 540×700; DOM width matched the viewport with no body horizontal overflow. `writing-narrow.png` captures the compact view. The browser omits native window controls. Reference example values are fixture data only, not seeded in production. Library service icons remain instead of provider logos; extra capability/storage notes and native-only gates are intentional, not a pixel-identical claim.

Fixed review findings: duplicate headings and enclosing Dictionary card; micro-sized fields and labels; search icons stacked above inputs by legacy global CSS; crowded Advanced backend row; duplicate Connections content in Data; excessive settings row dividers; tiny shortcut caps; incorrect Settings artwork. Existing real callbacks, account scoping and destructive confirmations remain wired.

Verified by browser interaction: History filtering and Insights; Dictionary filtering and retained add-word draft; Writing tone selection and shortcut view; Settings open/close, Speech/System/Data changes, scoped search, microphone disclosure, and retained Data variable draft. Preview invokes a test-only local adapter, so no real credential, provider, microphone, or user data operations were performed. Production drawer focus-trap/inert code was inspected, not native-window tested. No current browser runtime errors were reported.

Validation: 136 tests passed across 20 files; final Svelte check 0 errors/0 warnings; final production Vite build passed. Native system-wide insertion, live recording, authenticated service connections and signed/notarized release remain outside this UI pass.

## Latest: full Home composition correction

Final result: passed (Home visual scope; supersedes earlier visual findings below).

Reference: original exec-84e5e408-8a01-4dd2-a27c-5a2a6f8ec6e5.png, 1487×1058.
Evidence: `/private/tmp/destroy-home-review.E1uZOi/full-layout-review.png`; browser viewport 1487×1058 at 1:1 density. Source and rendered image were emitted together for full-view comparison. At this resolution the hero, branch and typography are readable without another crop. Also inspected 540×700.

Corrected earlier P1/P2 mismatches: raised art by the native titlebar inset to restore reference moon/horizon position; increased headline and keycap sizes; matched keycap inset/gap; restored tracked sans-serif ritual copy without italic/red emphasis; matched sidebar label scale and navigation spacing; replaced oversized textured branch with the original reference branch; aligned body insets, weekly columns, lower headings and model status. Artwork uses the unchanged approved source; no new painting generated.

Fidelity surfaces: editorial font retained and display sizing corrected; sidebar and body rhythm now follow source; charcoal/ivory/red palette retained; original artwork preserved with UI portions clipped out; text remains live and selectable. Content intentionally uses session-only history, real counters or empty states, and an estimate qualifier instead of the reference's invented values. Browser has no native traffic lights. Minor library-icon/font-rasterization differences remain P3, not a pixel-identical claim.

Interaction check: Dictionary navigation and return Home worked. Narrow layout keeps icon rail and live controls visible; secondary content scrolls. Narrow rail omits decorative branch to avoid source crop leaking mock controls. Native microphone/dictation behavior was not retested in this visual pass.

Validation: 135 frontend tests passed across 20 files; production Vite build passed; Svelte check zero errors/warnings before the final CSS-only refinement. No release/publish claim.

## Follow-up: exact approved artwork

Focused artwork correction final result: passed.

Source reference: `exec-84e5e408-8a01-4dd2-a27c-5a2a6f8ec6e5.png`; repository copy: `apps/desktop/public/art/daily-ink-approved-source.png` (SHA256 below). The reference filename and matching bytes establish identity, not ownership or redistribution permission; see `docs/PUBLICATION_REVIEW.md`.
Implementation screenshot: `/private/tmp/destroy-home-review.E1uZOi/approved-art-review.png`.
Compared both images in one tool output at 1487×1058 (CSS/pixels 1:1), Home idle. Source has illustrative statistics; implementation has unresolved-scope empty state. Browser omits native traffic lights and reserves a 48px top area, shifting the painting down by that amount. Also inspected 540×700.

Earlier P1: regenerated moon/landscape differed materially. Fixed by copying the exact approved source, verified identical SHA256 `342215fc70528fd6d755735ff05b4e39f939346ff2c8468e77f3694d2668b7b1`, then clipping only its artwork with CSS. No generated replacement, stretching, rasterized interactive UI, or image filtering. Original mock text, controls and sidebar are excluded. Live controls and content remain unchanged.
P2 during iteration: reflection overlapped live weekly heading/date. Fixed overview stacking; subsequent desktop screenshot shows both labels visible. Narrow viewport keeps controls visible and uses an intentional landscape crop.

Focused fidelity surfaces: original moon facets, peaks, reflection and palette retained; landscape aspect ratio preserved. Typography/copy unchanged. Painting composition reviewed at full screenshot resolution; no separate enlarged region needed for this focused artwork change. Existing whole-screen layout deviations and prior release/build verification gaps below are not certified by this scoped pass. Svelte check: zero errors/warnings. Git whitespace check passed.

Status: BLOCKED (final verification incomplete).

Reference: user-selected exec-84e5e408-8a01-4dd2-a27c-5a2a6f8ec6e5.png.
Production implementation: HomeScreen.svelte; App.svelte sidebar integration; Settings initial-tab routing; WritingSettings filtered modes; memory-only recent dictations and weekly usage aggregation.

Visually inspected Home at 1487×1058 in the local component preview. Sidebar, editorial heading, raised shortcut keys, vermilion moon and mountain panorama, weekly totals, recent activity, personalization links and bottom model status are present. Artwork is generated independently, not a screenshot embedded as UI. Moon scale and landscape details differ from the reference. Empty states intentionally replace illustrative transcripts and numbers. Native window controls are outside the browser preview.

Final visual fidelity gate is not passed: paired comparison and minimum-window verification remain outstanding. Preview Settings callbacks are not wired; production callbacks remain wired in App.svelte. Native dictation and drawer interactions were not reverified in this pass.

Agent checks passed: 21 dictation-state tests; Svelte reported zero errors and warnings. Final full run: 117 tests passed, four suites failed loading due to ENOSPC. Production build also stopped at ENOSPC. Need free disk space and rerun full checks/build before declaring this ready.

Privacy: recent transcript text is session memory only; unresolved scope returns no transcripts; clearing memory emits the update event. No sample transcript data was introduced.
