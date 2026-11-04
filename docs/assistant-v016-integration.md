# GridCraft v0.16 assistant integration

Updated 2026-11-03. Status: GridCraft 0.16.0 production source `e8f93495` and helper 0.1.2 are locally deployed; final native gear/settings and read-only MCP smoke passed on both Macs.
This isolated task branch reconciles the assistant-enabled source with upstream `85e51e00`
(GridCraft 0.16.0). Installation is managed separately; current installed verification is in
the candidate ledger.

| Workstream | State | Acceptance and verification |
|---|---|---|
| Upstream merge | Verified | Preserve v0.16 engine, GUI, file-format and control-channel changes alongside the shared assistant. |
| Shared update contract | Verified | Use one assistant-enabled shared revision with v0.16 update branding; retain the configured fork source, opt-out and unsaved-work guards. |
| MCP parity | Verified | Run complete catalog equality, formerly blocked tools, asynchronous screenshot, cancellation and reentrant assistant regressions against the merged source. |
| Delivery | Verified | Commit verified source integration. Public releases and installed-app replacement remain separate steps. |

The updater merge retains v0.16's branded `UPDATE_APP` for update offers. The existing custom
result modal retains the effective configured release source instead of linking to upstream
releases. Assistant-enabled builds continue to construct their updater from the explicit shared
source configuration. Neither startup nor manual checks can override `--disable-updates`.

The modified fork omits restricted upstream ArtCraft graphics and README references, retains
their legal terms and original open asset attribution, and identifies the fork plainly in About.
The original GridCraft ledger icon is retained.

The assistant's full MCP catalog and raw result envelopes continue through `gridcraft_mcp::Server`
and the same live GUI dispatcher as external MCP. Upstream's control-channel changes are retained;
no assistant-only document, range, command or filesystem filter is introduced.

Original reconciliation verification on shared revision `3ff2754e`:

- `cargo test -p gridcraft-ui-egui --lib`: 323 passed, including all assistant, control and updater regressions.
- `cargo test -p gridcraft-mcp`: 25 passed, using isolated temporary token files and ephemeral loopback sockets.
- `cargo clippy -p gridcraft-ui-egui -p gridcraft -p gridcraft-cli --all-targets -- -D warnings`: passed.
- `cargo xtask assets`: all 26 asset files attributed; `cargo xtask layers`: 17 crates, no violations.
- Native-menu regressions: 4 passed.
- `cargo check -p gridcraft-web --target wasm32-unknown-unknown`: passed; existing upstream `backstage::Modified` dead-code warning remains.

Full workspace CI is not claimed; verification is scoped to this integration. The merge fixes preserve assistant focus
ownership without exceeding the upstream keyboard handler complexity limit and qualify the
native-menu regression's `super::tree()` after upstream removed its import.

Workspace formatting and diff checks passed after final changes. All shared dependencies and the actual Git
lock now use `andykumeda/craft-libs@986cb16234cfdac1ee1337fb4cdf00945cad4e5a`; no local
path override is used. Native composition/clipboard/VoiceOver, real provider inference, approved
in-app update activation and the three-app unchanged-binary demonstration remain unverified.

Signed local standard/Preview bundles and a matching release CLI are prepared; artifact receipts
record source commits, identities and checksums. The deployment owner handles installation;
this verification workstream does not modify installed apps, wrappers or personal settings.

## Installed follow-up and restart safety

The initial signed candidate and independent installed smoke on both Macs are recorded in
[the candidate ledger](assistant-candidates.md). Actual helper rendering and read-only MCP passed.
A subsequent review found that app updates needed an additional active-assistant restart guard.
The verified correction uses shared revision
`986cb16234cfdac1ee1337fb4cdf00945cad4e5a`; the initial candidate was superseded by source `40305767`.
Its full UI suite passed 326 tests; scoped all-target Clippy and WASM checks passed.
Corrected signed candidates and repeated installed smoke passed on both Macs; see the candidate
ledger for exact hashes, process/socket cleanup and verification limits. No deployment changes
are made by this correction/verification workstream.

## Compact assistant panel follow-up — 2026-11-03

| Workstream | Owner | State | Acceptance and verification | Release |
| --- | --- | --- | --- | --- |
| Shared revision integration | Grid workstream | Verified | Locked shared revision `83e61084`: UI326, MCP25, strict all-target lint, assets26, layers17, WASM, format and diff checks passed | Production source `e8f93495`; installed records below |
| Signed desktop candidates | Grid workstream | Verified | Required pinned-font optimized desktop build passed; strict local-certificate signatures, source stamp and ZIP round-trip checks passed for standard and Preview | Installation owned by rollout coordinator |
| Exact repository CI | Grid workstream | Verified | All 10 `cargo xtask ci` steps passed with required pinned fonts, build/test jobs 1; 3280 tests passed, one timing benchmark ignored | Pre-install gate passed |

The helper owns chat presentation. Helper 0.1.2 provides visible New chat, a Chat history menu,
a settings gear, compact file/model composer controls and rounded message/composer styling. The app rebuild moves helper version and update
controls into settings, while retaining the update toolbar for an older helper that cannot
report its settings view. Provider selection, credentials, workbook state and chat storage
are not migrated by this source update. Existing Preview profile isolation and update opt-out
remain in place. Installed-app checks on each Mac are separate from candidate preparation.

Source validation used the locked Git dependency, not a path override. The original full UI
run passed325 tests and failed only the old assertion that New chat was always visible. The
updated test opens Chats, verifies New chat is reachable, closes the menu and closes the panel;
the full rerun passed326/326. MCP passed25/25. Scoped UI/desktop/CLI all-target Clippy passed
with warnings denied. WASM passed with the existing `backstage::Modified` dead-code warning.
The initial scoped checks did not constitute full-workspace CI; the exact-CI follow-up below
records that completed gate. Installed GUI, provider inference and signed feed-update activation
remain separate rollout checks. The previously verified signed CLI is retained; this
change does not change its engine/tool implementation.

### Signed compact-panel candidates

Source `e8f934958832f9a6efe7d1795b7b2582c9c6d806` is committed/pushed. The required-font
optimized desktop build passed in 10m 22s with the existing Noto CJK asset-policy exclusion.
Candidate root is `/private/tmp/gridcraft-compact-candidates-e8f93495`; `receipt.json` records
source/shared/font revisions, signed metadata, hashes and the adapted packaging script. Standard
and Preview archives were extracted and strictly verified. Both assembled apps additionally
satisfied an explicit identifier and configured-certificate requirement. No ad-hoc signing or
notarization claim is made. Existing Preview isolation/update opt-out remains unchanged.

| Candidate | Signed native executable SHA-256 |
| --- | --- |
| Standard | `ffc3cd4111677cb89b26549bb26d775a0d386cf1384595e965bcfd696db1a6c4` |
| Preview | `e8a2e142fba63e0436322e45ca9f776c06794d3a4d3035751488ffeec2b415f2` |
| Retained CLI (source 40305767) | `58c4cd2d282f2079fc1a1d8ae49be805019245c4968aca7804187c9730893ead` |

The rollout coordinator may stage these bundles while exact CI runs, but installation and
per-device native acceptance remain separate and are not claimed by candidate preparation.

### Exact-CI validation follow-up

The first exact `cargo xtask ci` run passed workspace strict lint and all but one UI integration
test. `ribbon_layout::big_split_button_separates_the_default_action_from_the_menu` expected the
compile-target macOS shortcut while egui_kittest now defaults its runtime OS to Nix. The fixture
now explicitly selects `OperatingSystem::from_target_os()` in that one harness. Production
widget behavior is unchanged. The original failing run was retained, and the focused corrected
case passed before the exact CI rerun.

The exact rerun passed all ten steps: i18n report, formatting, workspace/all-target strict Clippy,
workspace tests, doctests, assets, layers, WASM, parity and generated keytips. Tests totalled
3280 passed, zero failed and one ignored timing benchmark. UI integration passed 1058/1058.
The candidate production source remains `e8f934958832f9a6efe7d1795b7b2582c9c6d806`; the
validation-only follow-up changes one test fixture and this evidence document. A path audit
confirmed no production, asset or configuration changes, and both signed native hashes above
were recomputed unchanged. Candidate metadata is not relabelled with the validation commit.
The rollout coordinator records the separate validation commit and per-device installation/QA.

## Final two-Mac compact-panel deployment — 2026-11-03

The rollout coordinator atomically installed the locally certificate-signed standard app on
both Macs and replaced the existing Mini Preview. Previous bundles are retained under
`~/Applications/Craft Rollout Backups/2026-11-03-compact/` on each device. No Preview was added
to the MacBook. The unchanged signed CLI remains installed.

| Device | Installed app | State | Final evidence root |
| --- | --- | --- | --- |
| AKsMini | `/Applications/GridCraft.app` | Deployed | `/private/tmp/gridcraft-installed-smoke-4vikfzy4` |
| AKsMini | `/Applications/GridCraft Assistant Preview.app` | Deployed | `/private/tmp/gridcraft-installed-smoke-4orcfvji` |
| AKs-MBPro.local | `/Applications/GridCraft.app` | Deployed | `/private/tmp/gridcraft-installed-smoke-p0ihfr_p` |

Each final receipt confirms production source `e8f934958832f9a6efe7d1795b7b2582c9c6d806`,
a valid app signature, connected installed helper 0.1.2, native gear/settings-only update
controls, all 22 MCP tools and an unchanged synthetic `A1:B2` read-back of `[[2,5],[3,null]]`.
Test-owned processes exited and ephemeral listeners were released. No provider inference was
sent, and the checks used isolated settings and synthetic workbooks. The initial MacBook OCR
helper cold-start failure was resolved by a retry without a product change; the final receipt
above passed all checks.

The host still pins reviewed shared revision `83e61084dee3117d34f95600d6ec68bad8b0b9b5`.
Independent helper 0.1.2, source `3dda16ac779f0ffbfd91a4eefe720346aca006d2`, adds visible
New chat and a Chat history dropdown. Helper version, update checks and update status remain
inside settings. That helper-only follow-up used the existing protocol/settings/history 1
contract and did not rebuild or relabel these host binaries. The validation-only commit
`2002be680fc7e99100dfa44a6b8b543a9375b5bf` changes only the tooltip fixture and evidence docs;
the signed candidate production-source stamp remains e8f93495. Exact CI passed all 10 steps
with 3280 passing tests and one ignored timing benchmark.

These builds are locally signed, not notarized or publicly released. Real provider inference,
IME, clipboard, VoiceOver and published-feed update activation remain separate checks.

## Assistant toolbar refinement — 2026-11-04

| Workstream | State | Acceptance and verification | Release state |
|---|---|---|---|
| View Assistant | Verified | Right-edge placement and pointer/keyboard toggle regression passed at 640 and 1200 points in light/dark themes; all four offscreen renders reviewed. Existing title-bar control retained. | Source only; no installation or release. |

The control reuses the existing contributor-original comment icon and large-ribbon button.
No new assets, dependencies, commands or provider settings are introduced.

Verification: `cargo fmt --all -- --check`, `cargo clippy -p gridcraft-ui-egui -j 2 --all-targets -- -D warnings`,
`xtask assets`, `xtask layers`, `xtask wasm`, `xtask keytips --check` and `xtask parity --check` passed.
The UI integration binary passed all six `system_theme` tests, six `ribbon_layout` tests, two
`widgets` tests and `renders_every_ribbon_tab_without_panicking`. The new regression checks
right-edge placement, an accessible Assistant button, pointer opening and keyboard closing.

A broader `ribbon` name-filter run passed 24 tests and failed the existing
`language_switch_renders_the_ribbon_and_persists` test: the optional craft-fonts input on this
machine lacks the current manifest, leaving no bundled glyph for Vietnamese U+1EBB.
That locale check remains blocked by its font input; no locale coverage is claimed.
The whole-workspace CI sweep, native VoiceOver and physical keyboard checks were not run.
No dependency, font, provider or installed app was changed.
