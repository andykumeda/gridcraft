# Assistant preview implementation ledger

Updated 2026-10-29 (America/Los_Angeles).

Private local preview. No push or public release is authorized.

| Workstream | Owner | State | Acceptance / dependencies |
| --- | --- | --- | --- |
| Shared stack and native panel | Primary | Verified | One model, visible toolbar/View toggles, Settings, app-local history; depends on shared bf932b9 |
| Workbook adapter | workbook_host | Verified | Bounded scoped commands, validation/protection, exact read-back, Undo and synthetic tests |
| Preview packaging | preview_package | In progress | Separate identity/profile, updater opt-out, ARM64 DMG/checksum and extracted-app checks |
| Integrated verification | Primary | Verified | cargo xtask ci, synthetic workbook acceptance, panel/Settings visual inspection |
| Local installation | Primary | Not started | Install verified bundle only; preserve original app and running unsaved work |
| Other Mac | User acceptance | Not started | Launch, platform approval and independent provider setup/inference |

References inspected: GridCraft `39a10b1342c4896edb39d2020c65e296115e6271`, WordCraft
`4dd7cf7d3f1bd6bdb7f330e164da4dc0f1ad469e`, shared libraries
`bf932b91643c287502d539b88f08abd5796c9447`. All working trees were clean.
Local `plan/` and documented craftrules checkouts are absent; committed documentation is used.

Initial tool scope excludes filesystem access, arbitrary commands, scripts, protection changes,
sheet deletion and screenshots. Save/reopen uses ordinary application commands. The user chose
one model for conversation and workbook control. Shared legacy control bindings remain intact.

## Verification evidence

Pending: full gates, live provider acceptance, native UI, packaging and installation.
Cargo's first online dependency lookup failed DNS; the locked dependencies were then fetched with network permission. Compilation passed. Initial focused tests passed 37 cases and found an empty-tool-catalog fixture error; the corrected deterministic routing regression passed. Full CI is in progress.
The main app's executable SHA-256 before this work is
`8d24fd49968536842afb640185251a54fd7432fb2609ffb9ecc3f82199c0d33a`.

## Build input and gate follow-up

The first full CI run reached workspace tests: all assistant regressions passed, while two existing
interaction tests failed because the optional pinned font input was absent (Vietnamese and uppercase
German sharp-S glyphs). The exact CI font revision `b076417b5880bd4154bd22f00695b1d28c6bea83`
was fetched to `/private/tmp/gridcraft-assistant-fonts`; all 49 interaction tests then passed with
`CRAFT_FONTS_DIR` pointing there and `CRAFT_FONTS_REQUIRED=1`. Strict glyph/accessibility policies
remain enabled. GridCraft's existing asset policy excludes the barred CJK font; no fonts are added
to this repository. Release packaging must use the same input and include its licences.

The macOS no-harness menu test needed a scoped unused/dead-code allowance on its imported app
module so the required all-target Clippy gate could compile the existing installation-only probe.
The native View menu now also exposes Assistant. No unrelated production feature was changed.

## Verified candidate checkpoint

`cargo xtask ci` passed all ten stages with the pinned font input, offline locked dependencies,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0`.
Ordinary tests: **2,772 passed / 1 ignored / zero failed**; doctests and the 11-package WASM check
also passed. Log: `/private/tmp/gridcraft-assistant-ci6.log`. The one-way UI-to-MCP dependency's
old sideways-rejection test was updated to keep rejecting the reverse dependency.

A real native debug candidate completed the isolated Ollama acceptance using existing
`qwen3:4b-instruct-2507-q4_K_M` at `127.0.0.1:11434/v1`. The effective route, actual loaded model
and successful read/write results were captured. One `write_range` changed B1:B3 from 1, 2,
`=SUM(B1:B2)` to 10, 20, `=SUM(B1:B2)`; independent read-back confirmed 30. Sentinels on both
sheets remained unchanged and one normal Undo restored 1, 2, 3. Edited and restored XLSX files
were saved; fresh-process reopen remains a release/install checkpoint. No paid inference, model
download, private workbook or normal shared settings were used.

Evidence: `/private/tmp/gridcraft-assistant-acceptance/` contains synthetic configuration,
`verification.json`, route/model observations, action trace, XLSX fixtures and panel/Settings PNGs.
The panel and Settings screenshots were visually inspected. The native updater-disabled message
was verified. The clean test app quit and its temporary 17979 listener was released.
Release packaging/installation and other-Mac acceptance are **not yet performed** at this checkpoint.
