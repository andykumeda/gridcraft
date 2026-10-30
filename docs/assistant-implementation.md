# Assistant preview implementation ledger

Updated 2026-10-29 (America/Los_Angeles). **Implemented, verified, packaged and locally deployed: 0.13.0 (`00cb4830`).**

Private local preview. No push or public release is authorized.

| Workstream | Owner | State | Acceptance / dependencies |
| --- | --- | --- | --- |
| Shared stack and native panel | Primary | Verified | One model, visible toolbar/View toggles, Settings, app-local history; depends on shared bf932b9 |
| Workbook adapter | workbook_host | Verified | Bounded scoped commands, validation/protection, exact read-back, Undo and synthetic tests |
| Preview packaging | preview_package | Verified | Separate identity/profile, updater opt-out, ARM64 DMG/checksum and extracted-app checks |
| Integrated verification | Primary | Verified | cargo xtask ci, synthetic workbook acceptance, panel/Settings visual inspection |
| Local installation | Primary | Deployed | Install verified bundle only; preserve original app and running unsaved work |
| Live Codex acceptance | Primary | Verified | Installed Terra edit, actual read-back, one Undo, edited/restored XLSX fresh-process reopen |
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

## Delivered preview — final checkpoint

Source: **`00cb483042bed283c8b35d0e8513ea0f025495fb`**, branch `codex/assistant-preview`.
Shared libraries: **`bf932b91643c287502d539b88f08abd5796c9447`**.
The implementation commit passed all ten CI stages; 44 assistant regressions passed among
2,772 ordinary tests (one unrelated test ignored). No push or public release was performed.
The delivery documentation commit is separate from the binary source; find it with
`git log -1 --format=%H -- docs/assistant-implementation.md`.

| State | Evidence |
| --- | --- |
| Implemented | Shared native single-model pane/settings/providers/history; guarded workbook tools and automatic read-back; separate profile and updater opt-out |
| Tested | Full CI, candidate and exact installed-app synthetic tests, native panel/Settings/View screenshots, narrow layout/long history titles, safe connection/context failures, fresh-process XLSX reopen |
| Packaged | ARM64 release, version/source checked; assembled app, ZIP extraction, mounted DMG and DMG-extracted copy passed strict deep signature verification |
| Deployed | `/Applications/GridCraft Assistant Preview.app`, version `GridCraft 0.13.0 (00cb4830)`; no prior preview existed to replace; main app and main preferences hashes unchanged |
| Other Mac | Not started: independent launch/provider/credential/tool/Undo/save-reopen acceptance required |

Release build: `cargo build --release --locked -p gridcraft` with `GRIDCRAFT_BUILD_SHA` set
to the source commit and the required pinned font input. Existing dependency cache:
`/private/tmp/wordcraft-preview-delivery-build`; no sibling source changed. Build passed in
3m17s. Log: `/private/tmp/gridcraft-assistant-release-build.log`. Package log:
`/private/tmp/gridcraft-assistant-package.log`; install receipt:
`/private/tmp/gridcraft-assistant-install.json`.

Artifacts under `/Users/andy/Dev/gridcraft/dist/assistant-preview/00cb4830/`:

- `gridcraft-assistant-preview-0.13.0-00cb4830-macos-aarch64.dmg` and `.dmg.sha256`
- `gridcraft-assistant-preview-0.13.0-00cb4830-macos-aarch64.zip` and `.zip.sha256`
- `gridcraft-assistant-preview-0.13.0-00cb4830-macos-aarch64-INSTALL.md`

DMG SHA-256: `4a8519282f1fe6be4dd4587658778cd3d90f5ca4aeba509543f816da8c348214`.
ZIP SHA-256: `53d28840b87302fcda7f5edd8cfed86f283402e6785d34c0f342094183a303a4`.
Installed signed native executable SHA-256:
`6e24d4411d3b701e175bfc4e21ff65292a3f9b8282c7a1e59987b4920438f28a`.
Main GridCraft executable remains:
`8d24fd49968536842afb640185251a54fd7432fb2609ffb9ecc3f82199c0d33a`.

Signing is **local ad-hoc, not Developer ID signed and not notarized**. Minimum macOS from the
binary/bundle is 11.0; only Apple silicon is packaged. Font licences accompany the app. The
portable launcher derives its profile from the receiving user's HOME and ships no credentials,
settings, conversations, Codex installation or Ollama weights.

### Installed native acceptance and honest limits

The first installed trial completed the read/write and exact formula/value verification but its
final provider summary timed out at the unchanged 180-second default. A control request also
timed out during that trial. The app recovered without termination; thread samples did not
establish an AccessKit deadlock. Independent reads confirmed 10, 20, 30 and unchanged sentinels;
one Undo restored the baseline, both states were saved and the clean app quit. That interrupted
operation was never replayed. Evidence: `/private/tmp/gridcraft-assistant-installed-acceptance/`
and its `timeout-verification.json`. Resource pressure was observed but its cause is unproven.

A **fresh** installed-app conversation, with an explicit 300-second timeout in its disposable
`runtime.json` and a concise completion request, passed the complete turn. The same existing
Ollama model made one `write_range`, returned actual formula/value read-back and replied Done;
one Undo restored the complete baseline. No app defaults or normal shared bindings changed.
Evidence: `/private/tmp/gridcraft-assistant-installed-final/verification.json`, `live-result.json`,
`trace.json`, saved XLSX fixtures and visually inspected screenshots. Model latency/reliability
remains workload dependent; this is not a performance benchmark.

A normal **LaunchServices bundle launch** then reopened the edited XLSX: values 10, 20, 30,
`=SUM(B1:B2)`, and the other-sheet sentinel survived. The preview created and persisted its own
normal profile at `~/Library/Application Support/GridCraft Assistant Preview`; the main
GridCraft preferences hash was unchanged. An isolated 2,048-byte context budget failed clearly
before workbook edits. The native View and toolbar open indicators, Settings, provider-failure
and context-limit messages were visually inspected. The clean app quit and temporary loopback
17979 was verified released. There is no assistant-owned permanent listener or autostart.

Remaining acceptance: the other Mac's Gatekeeper approval and independent Codex/Ollama setup,
platform credential access, synthetic editing/read-back/Undo/save-reopen. At the initial delivery, live Codex and hosted
provider inference had not been performed. The Codex follow-up below closes this Mac's live test;
hosted-provider and other-Mac checks remain open. No private workbook was used.

## Live Codex acceptance — 2026-10-29

**Verified on the installed preview; no application changes or new package required.**
Installed binary: `0.13.0 (00cb4830)`, native SHA-256
`6e24d4411d3b701e175bfc4e21ff65292a3f9b8282c7a1e59987b4920438f28a`.
Shared implementation: `bf932b91643c287502d539b88f08abd5796c9447`; Codex CLI `0.165.0`.

The existing Craft Codex connection and WordCraft's configured `gpt-6.1-terra` model were selected
explicitly in disposable GridCraft settings. Only the existing isolated Craft Codex home was
referenced by a temporary directory symlink; no authentication files were copied into the
test artifacts, and the filesystem reference was removed after both test processes quit. The normal shared
connections/settings file hash, main GridCraft executable hash and preview executable hash were
unchanged. No coding-assistant account, model fallback or private workbook was used.

The native assistant completed one uninterrupted live turn with actual `read_range` and
`write_range` callbacks, ending with Done. Its effective route was **Codex account /
`gpt-6.1-terra`**. It read B1:B3, wrote exactly `[[10],[20],["=SUM(B1:B2)"]]`, and the tool returned
authoritative formula/value read-back. Independent application reads confirmed 10, 20 and 30,
with B3 retaining `=SUM(B1:B2)`. A1/D1 sentinels and a second sheet's sentinel stayed unchanged.
One normal `edit.undo` restored 1, 2 and 3 and the original formula.

The edited and Undo-restored workbooks were saved separately. A **fresh installed-app process**
reopened both XLSX files and independently confirmed the edited 10/20/30 and restored 1/2/3
values, the formula text, and the untouched second sheet. The real panel/result and reopened
workbook screenshots were inspected. Both clean synthetic app instances quit normally; temporary
loopback 17979 was verified released. No existing user instance was terminated or user work discarded.

Synthetic evidence: `/var/folders/cg/796c5bys74db15ww0b_qvw8c0000gn/T/gridcraft-codex-acceptance-0jcmp2o_`.
Key artifacts: `verification.json`, `reopen-verification.json`, `preservation-verification.json`,
`trace.json`, `route-status.json`, the completed local chat journal, `Edited.xlsx`, `Restored.xlsx`,
`panel-complete.png` and `codex-reopened.png`. No credential file or filesystem link to the account remains in
this artifact directory. This was one representative synthetic acceptance case, not broad model
reliability, adversarial containment or other-Mac acceptance.

After this acceptance, `cargo xtask ci` passed all ten required stages again; log:
`/private/tmp/gridcraft-codex-acceptance-ci.log`. Only verification documentation changed.
