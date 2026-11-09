# GridCraft 0.16.1 integrated assistant rollout

Status: **Deployed for GridCraft on both Macs**, 2026-11-05 (America/Los_Angeles).
The active app is `/Applications/GridCraft.app` with the integrated Assistant; its separate
Assistant Preview app and old MCP shortcut have been retired. This closes this app's workstream only. The
[suite consolidation ledger](https://github.com/andykumeda/pdfcraft/blob/codex/independent-assistant/docs/single-app-assistant-rollout.md) owns overall progress; this per-app deployment does not claim all four apps are complete.

## Official 0.18.0 reconciliation — 2026-11-09

| Workstream | State | Acceptance and evidence |
| --- | --- | --- |
| Assistant source on official 0.18.0 | In progress | Reapply the 0.16.1-to-assistant delta over official `17cb3c83`; retain assistant panel, managed updater protection, and release ancestry. Formatting and shell syntax checks pass. Shared commit b347, resolved from a local Git checkout, passed a UI check, strict locked all-target UI Clippy, and 37 locked assistant unit tests. Earlier local-path checks passed three updater tests and the title-toolbar integration test. The full ten-gate CI passed with b347: 3,655 tests passed and four were ignored, including doctests. The prior macOS WASM dead-code warning remains. Source integration and delivery qualification remain pending. |
| Protected installation | Not started | Run the managed launcher route only after source, build, package and assistant verification pass. Verify the installed app and underlying release on each Mac. |

The current deployed 0.16.1 installation below remains the last verified release. The shared
revision is published on the existing fork; no release build or installation is claimed.

### Full CI attempt and upstream test race

`cargo --offline --locked xtask ci` reached its workspace test gate after translation report,
formatting and workspace Clippy passed. The desktop control-server test
`a_silent_or_slow_peer_is_closed_at_the_auth_deadline` failed with macOS `EINVAL` when setting
a read timeout on the slow socket; later gates were skipped. The unchanged official test file
sets that timeout after waiting for another peer, by which time its peer can already have closed.
A standalone synthetic Rust loopback probe reproduced `EINVAL` on 30/30 closed peers, and the
original candidate test passed once and failed once in isolated retries. The test-only candidate
fix sets both client timeouts immediately after connecting; production server behavior is unchanged.
The corrected test passed in the focused workspace invocation and ten consecutive direct test-binary runs. A later user-requested pause stopped the first rerun during UI tests. After resumption, the full ten-gate CI passed at low priority with one build job and one test thread: 3,655 tests passed and four were ignored, including doctests. The WASM build retains its existing dead-code warning. Log: `/Volumes/X10 Pro/codex-craft-build-20261109/grid-b347-full-ci-low-priority.log`. No release package or installation is implied.

### GridCraft 0.18.0 integrated delivery — 2026-11-09

| Workstream | State | Evidence and remaining limitations |
| --- | --- | --- |
| Integrated application | Deployed | Signed standard app built from merge `b28dd31f3b703572a7815f93c0207a935761e6e8`, including official v0.18.0 `17cb3c83d9e73df6a88ed174cec612f1de2e8244` and shared assistant revision `b3474c019d28054396931e8e295cec1b44e60009`. Installed signature and version verified on Mini and MacBook. The release build used one low-priority job and the pinned release fonts and translations. The configured launcher adapter was not exercised; it currently hardcodes two build jobs. |
| Matching CLI and MCP | Deployed | Signed 0.18.0 CLI SHA-256 `6d122a5556060c1eb2fd52fbecceb4b6e700a4374d2e52eb9dd034a26b0592cd` installed with a digest record on each Mac. Mini's existing MCP command now points to that CLI with its existing `mcp --connect 7979` arguments. MacBook's integrity wrapper and `mcp --connect 127.0.0.1:7979` arguments remain unchanged. Both configured routes initialized, listed 22 tools, and edited and read back a disposable workbook against a task-owned app. The previously missing MacBook 0.16.1 digest remains historical; its 0.18.0 wrapper check passes. |
| Assistant panel | Verified | Both installed apps passed actual toolbar open/close, helper 0.1.7 handshake and configured-model display in an isolated profile, with protected profiles unchanged. Receipts: Mini `/private/tmp/gridcraft-toolbar-native-t4be7hsc/report.json`; MacBook `/private/tmp/gridcraft-toolbar-native-mieljkcb/report.json`. No live model inference was requested, so this does not establish a new inference or assistant edit test. |
| Rollback | Verified | Each host retains an independently extracted and signature-verified 0.16.1 rollback ZIP: Mini `~/Library/Caches/Craft Integrated Builds/gridcraft/codex-b28-stage/rollback-0.16.1.zip`; MacBook `~/Library/Caches/codex-grid-b28-stage/rollback-0.16.1.zip`. Their extracted native binary hashes match the old installations. Automatic approval review rejected recursive removal of the redundant `old-GridCraft.app` and `rollback-check` directories under those same staging directories; those copies remain in the caches and were not removed. The active app on each host is `/Applications/GridCraft.app`. |

The integrated standard app package contains the GUI executable; the matching CLI is distributed
separately. The configured MCP routes attach to an explicitly started local app and retain their
existing connection arguments. Existing user documents were not used for smoke testing.

## Top-toolbar Assistant placement — 2026-11-05

| Workstream | Owner | State | Acceptance / evidence | Delivery |
| --- | --- | --- | --- | --- |
| Title toolbar | Grid toolbar agent | Deployed | One sparkle Assistant control immediately beside Dark Mode; unique icon/accessibility name, pointer and keyboard toggles; 640/1200-point light/dark regression and reviewed renders | Normal app installed and independently checked on both Macs |

The current request supersedes the old far-right View-ribbon placement. Use a contributor-original
code-drawn sparkle (distinct from the adjacent Comments icon) and the existing `ui.assistant` command; remove the duplicate ribbon control.
Scope is title/ribbon layout, existing appearance regression and user docs. Workbook commands,
provider configuration, external assets, shared bridge, packaging contracts and stored data do not change.
Local `plan/STATUS.md`, execution plan and architecture files are absent; committed architecture
and assistant delivery docs supply the current context.

The replacement placement regression failed on the old layout (two Assistant controls) and passed
on the change. The existing native-panel test now expects the requested single toolbar control;
Chats and Close behavior still pass. Final `cargo xtask ci` passed all ten gates, 3,437 tests
passed / one ignored, including doctests, strict workspace Clippy, assets/layers and WASM.
An additional final scoped UI `--all-targets -D warnings` check passed after adding the distinct
sparkle. Existing WASM dead-code warnings remain. Logs: `/private/tmp/grid-toolbar-ci-final.log`,
`/private/tmp/grid-toolbar-final-lint.log`, `/private/tmp/grid-toolbar-{red,green}.log`.
Four final offscreen 640/1200-point light/dark renders were inspected under
`/private/tmp/grid-toolbar-final-*.png`; the icon remains adjacent to theme and visible without
ribbon scrolling. No new external asset/dependency, provider request, native installation,
physical keyboard, VoiceOver or private-workbook check was claimed at that source checkpoint. The later delivery below supersedes installation status.

### Top-toolbar package and two-Mac acceptance

The optimized source `f43086bc7fce24d273549653956382106b7a3922` built successfully with
required pinned fonts/i18n inputs. The existing standard packager produced signed ZIP/DMG;
strict ZIP/extracted/mounted signatures, exact local certificate and standard metadata checks
passed. `test-standard-package.sh` passed, including negative underlying-tag/commit checks.
The first local packaging invocation omitted the optional-font licence input; that package was
rejected before admission and its newly generated outputs were removed. The corrected package
contains all 13 font licences byte-for-byte; prior installed resources match except the authorized
README update. No first-package output is eligible for installation.

Qualified archive: `dist/integrated/f43086bc-licensed/gridcraft-0.16.1-macos-aarch64.zip`,
SHA-256 `647c7a107fa0fe5b1ea2dbf3875224d83ce6fa80a3e517e246a56de216386049`.
The package verifier report is adjacent `verification.json`; build/package logs are
`/private/tmp/grid-toolbar-{release,package-final}.log`.

The managed launcher's protected route installed this exact signed native on both Macs.
Old bundles are retained as verified ZIPs under each host's
`~/Library/Application Support/Craft/retired-apps/2026-11-05-top-toolbar.noindex/`.
The extracted qualification candidate was unregistered after its owned process exited;
no second active GridCraft or Preview installation was introduced.

| Device | State | Installed source / native | Independent acceptance evidence |
| --- | --- | --- | --- |
| AKsMini | Deployed | `f43086bc` / `fe4f8391` | `/private/tmp/gridcraft-toolbar-native-87tseav5/report.json` |
| AKs-MBPro.local | Deployed | `f43086bc` / `fe4f8391` | `/private/tmp/gridcraft-toolbar-native-a_5n75yz/report.json` |

Both installed-path trials used a disposable synthetic workbook and isolated document/AI profiles.
An actual top-toolbar pointer click opened the panel and another closed it; helper0.1.7 and the
configured model became ready. Light/dark screenshots were reviewed on each host, including the
selected sparkle beside theme. The workbook values/formula, baseline file bytes, normal/retired
Preview profiles and shared settings/chats stayed unchanged. Managed update guidance passed.
No inference was requested. All owned processes exited normally, their OS-assigned loopback
listeners were released, and private control keys were removed. Candidate and installed smoke
scripts reused the existing opt-in control protocol; no permanent service or provider change.
Physical keyboard/IME, VoiceOver, clipboard and private-workbook checks were not repeated.

## Current single-app deployment — 2026-11-05

| Component | Installed / verified state |
| --- | --- |
| Normal app | GridCraft 0.16.1, native source `f43086bc7fce24d273549653956382106b7a3922` |
| Underlying official release | `v0.16.1`, commit `e581bc95b2a17d2a744a115478a20a1c2ad1ef72` |
| Signed native SHA-256 | `fe4f83914ed46259d99cd242381d8dc94487c19fa251d8036f378c724909ec48` |
| Assistant helper | 0.1.7 on Mini and MacBook; bridge protocol1 unchanged |
| Update owner | Managed ArtCraft Launcher 0.14.1/source19170137; actual installed-launcher replacements passed on both Macs |
| Local signing | Exact `stt-dev` certificate `D7B1B5524FFEB37AB01D3960017BDF6743E3EDFD`; not notarized or a public release |

Use the **managed ArtCraft Launcher** to check official release freshness and update this app.
Its trusted source builder retains the Assistant, reconciles the requested official tag,
runs the required checks and produces a signed normal bundle. The launcher verifies the
standard identity, exact certificate, assistant protocol/repository and signed
`CraftUnderlyingTag`/`CraftUnderlyingCommit` before replacement. Failed builds, unavailable
build hosts or source conflicts leave the installed app intact; there is no stock-package
fallback. The MacBook uses signed artifacts from the Mini, which must be reachable for a new
build. The app's own updater is disabled by its standard wrapper. Preserve the managed launcher,
including its self-update protection; ordinary upstream app/launcher packages are not this route.

The previous native `f5785668` passed configured Codex/Terra/helper 0.1.6 candidate acceptance
independently on both Macs (28.00s Mini / 14.38s MacBook): requested synthetic edit, no early save,
Undo/Redo, explicit save/reopen and rendered output. After those protected installations,
no-model checks on the normal paths passed with helper 0.1.7 on both hosts: panel/status,
control responses, light/dark composer outlines and update guidance. Screenshots were
inspected; the bottom composer border is complete without clipping. The helper update was
not qualified by repeating a paid model turn. Full source CI passed all ten gates;
this documentation-only closeout does not rerun or imply new code checks.

Evidence: `/private/tmp/gridcraft-standard-ci.log`,
`/private/tmp/craft-single-app-evidence/grid-{mini,book}/` (actual-model candidates),
and `/private/tmp/craft-single-app-evidence/grid-{mini,book}-installed/` (installed reports/rendered panels).
Owned processes exited normally; listeners and task-owned control keys were cleaned up.
Standard and old Preview profiles, recovery data, shared AI settings and conversations were
preserved, without profile merging or credential copying. Active Preview bundles and obsolete
MCP shortcuts were retired, and Preview installers archived under the per-host
`~/Library/Application Support/Craft/retired-apps/2026-11-05-single-app.noindex/` record.
No owner process was force-quit. Physical IME, clipboard and VoiceOver checks were not repeated.

## Historical integration and Preview delivery (superseded)

Everything below records earlier checkpoints, not the active installation policy.

Historical checkpoint: Assistant Preview was deployed on both Macs on2026-11-04. Official source target:
[`v0.16.1`](https://github.com/storytold/gridcraft/releases/tag/v0.16.1), commit
`e581bc95b2a17d2a744a115478a20a1c2ad1ef72`. Preview binary source:
`2b995d6d16303a2c49a2094c12d39cddc1b52265` on `codex/independent-assistant`.

| Workstream / device | State | Acceptance and evidence | Delivery |
|---|---|---|---|
| Source integration | Verified | Official 0.16.1 merged; Assistant settings, far-right icon control, full app dispatcher and updater opt-out retained. | Source committed and pushed to the existing fork branch. |
| Exact checks and package | Verified | All ten CI gates, 3,437 ordinary tests passed / one ignored, doctests, optimized locked release build, signed ZIP/DMG and extraction/mount/integrity checks passed. | Local artifacts; no public release. |
| Mac mini / AKsMini | Deployed | Source/version/signature verified. Configured Terra edit/read-back and Undo/Redo passed in 16.29s; separate fresh processes reopened and rendered edited/restored files. | Existing Preview replaced with rollback retained. |
| MacBook / AKs-MBPro.local | Deployed | Same artifact independently verified. Configured Terra edit/read-back and Undo/Redo passed in 14.31s; separate fresh processes reopened and rendered edited/restored files. | Preview newly installed; standard app is separate. |

Both installations are `/Applications/GridCraft Assistant Preview.app`, identity
`ai.storyteller.gridcraft.assistant-preview`, version **0.16.1 (2b995d6d)**. Their signed native
executable SHA-256 is `c319cb1db89ebd37a5617284c8af4b25c898675ea8183e54a3eb4a5bed05fe7f`.
The launcher retains `--disable-updates` and its separate profile. The Mini's prior signed
0.16.0 Preview is retained under `~/Library/Application Support/GridCraft Preview Rollbacks`.
The MacBook had no prior Preview to replace. No user process was force-quit.

The compatible host bridge remains pinned to shared revision `83e61084`; independently
installed helper **0.1.6** owns native Codex/profile behavior. Both live tests used the existing
Codex connection and `gpt-6.1-terra`, with helper 0.1.6 observed as a child of the installed app.
Only public component/connection/runtime metadata was copied into disposable AI stores.
Normal Codex account/profile discovery remained intact; no credentials or old isolated Codex
homes were copied. Workspaces and document profiles were disposable. Existing user settings
and conversations were not replaced; profile hash checks in the final read-only qualification
passed on both hosts, and the MacBook's full native trial hash check also passed.

## Verification

Integration resolutions retain both Assistant settings and upstream workbook display options.
The tooltip fixture adopts upstream's explicit macOS test platform. The official release
workflow delta is inherited from the tag; no custom workflow changes were introduced.

`cargo xtask ci` passed all ten gates: i18n report, formatting, full-workspace clippy with warnings
denied, tests, doctests, assets, layers, WASM, parity and keytips. The official font input
`b076417b5880bd4154bd22f00695b1d28c6bea83` was clean; all 13 selected font hashes and licence
paths were verified. The locked optimized release build passed in 7m19s. ZIP extraction,
mounted/copied DMG app signatures, checksums and independent DMG integrity checks passed on
both hosts. Packages are ad-hoc signed, not notarized or Developer ID signed.

Each native model turn changed B1:B3 from 1/2/3 to 10/20/30, preserving `=SUM(B1:B2)` and both
sentinels. Independent application reads verified the result; Undo and Redo restored the exact
expected values. The model did not save the baseline. Edited and restored copies were saved,
then reopened in separate fresh single-document processes and checked again, including PNG
rendering. Installed View-ribbon, edited-result and reopened-result images were inspected.
All task-owned GUI processes quit normally and their opt-in loopback listeners were released.

Initial QA attempted a capture through a control window after opening another workbook. The
document-ID guard refused that capture rather than returning stale pixels; bounded read-only
retries did not change the result. Final qualification used a fresh process for each saved file.
The original receipts are retained, no application code or assertions were weakened, and the
paid model edits were not repeated.

## Artifacts

Files under `dist/assistant-preview/2b995d6d/`:

| Artifact | SHA-256 |
|---|---|
| `gridcraft-assistant-preview-0.16.1-2b995d6d-macos-aarch64.dmg` | `8b2f378906214359834873a11b90c284de089ba9d3cf774bf72053f242ebc1ca` |
| `gridcraft-assistant-preview-0.16.1-2b995d6d-macos-aarch64.zip` | `0d1a5e0ed0c4468231821ea2ddfc01dcec4216e84a57700a1ab31857602e2d96` |

Adjacent checksums and installation instructions accompany the packages. Local evidence
includes per-host installation/native/read-only receipts and qualification summaries under
`evidence/`; private operational details remain in the ignored handoff.

Physical keyboard/IME, VoiceOver and private-workbook fidelity checks were not performed.
These synthetic checks do not qualify every upstream feature or a public notarized release.

Final documentation closeout reran the exact `cargo xtask ci` pipeline successfully: all ten
gates passed again with the deployed source unchanged.
