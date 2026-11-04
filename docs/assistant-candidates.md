# Local assistant-enabled candidates and installed smoke

Updated 2026-11-03. Locally signed GridCraft 0.16.0 (`e8f93495`) and shared helper 0.1.2 are deployed on both Macs; no notarization or public release.

## Initial v0.16 candidate

Source `442be9e1ec48ea26b8e188df38d27eb160f7bbc6` (task branch
`codex/independent-assistant-v016`) merged upstream GridCraft 0.16.0. Its optimized desktop and
CLI build passed with `CRAFT_FONTS_REQUIRED=1` and release-workflow font input
`b076417b5880bd4154bd22f00695b1d28c6bea83`. Both are arm64; the app's Mach-O minimum is macOS 11.0.

Candidate root: `/private/tmp/gridcraft-local-candidates-442be9e1`. `receipt.json` includes
signed executable and archive SHA-256 values, full bundle/signature metadata, font/source pins
and the reproducible local packaging procedure. Strict signatures and ZIP-extracted app/CLI
copies passed verification. The CLI reported `gridcraft-cli 0.16.0`; its formula smoke returned 6.
Required project/dependency attribution, MIT/Apache and pinned-font OFL notices were copied
before signing. Restricted trademark graphics are absent.

| Candidate | Signed executable SHA-256 |
|---|---|
| Standard `GridCraft.app` | `97237b002f8de6d2ed664c71bf34f6ddc8f3a6d0e0b31643597872fe826ae787` |
| `GridCraft Assistant Preview.app`, native executable | `0a76747f8ed721c9ee97c2426c141a3de77c6b5472d8ab753ed2d49d19f31cdc` |
| Standalone `gridcraft-cli` | `58c4cd2d282f2079fc1a1d8ae49be805019245c4968aca7804187c9730893ead` |

The standard bundle ID is `ai.storyteller.gridcraft`; the Preview is
`ai.storyteller.gridcraft.assistant-preview`. Signed metadata declares bridge protocol 1 and
repository `andykumeda/gridcraft`. Preview retains its separate document profile and unchanged
`--disable-updates` launcher. These are explicitly locally signed with certificate SHA-1
`D7B1B5524FFEB37AB01D3960017BDF6743E3EDFD`; ordinary Apple-team verification is unchanged.

**This initial candidate needs replacement for the app-update restart guard fix below.**
The deployment owner installed it on both Macs; this workstream made no installation or
MCP-wrapper/configuration changes.

## Independent installed smoke

| Host | Evidence root | Result |
|---|---|---|
| Mini | `/private/tmp/gridcraft-installed-smoke-t9dei68g` | Passed; owned app PID 46805, ephemeral loopback 64014 |
| MacBook | `/private/tmp/gridcraft-installed-smoke-etb2_6fb` | Passed; owned app PID 42368, ephemeral loopback 52245 |

Each run used a synthetic XLSX and temporary document/AI profiles. Only public component/source
configuration was copied, pointing to the actual installed shared assistant. No personal chats,
model settings or credentials were copied. Both installed helper 0.1.0 processes connected and
rendered inside the sidebar; native screenshots were captured and inspected. No provider request
was sent, so the isolated panel correctly showed that no model was selected.

External MCP listed all 22 tools and read `A1:B2` as `[[2,5],[3,null]]`; a separate native control
read confirmed unchanged values. MacBook's existing wrapper `--diagnose` also passed and resolved
0.16.0. Both new app instances quit normally, both helper children exited, and both listeners were
verified released. Existing processes were preserved. Token-free service inventory notes were
updated on each host; those inventories are outside Git and cannot be committed here.

These checks establish installed embedding and read-only MCP. They do not claim live inference,
native composition/clipboard/VoiceOver, active update replacement or the three-app assistant-only
update demonstration.

## App-update restart guard correction

Status: source and corrected candidates verified; corrected installed smoke passed on both Macs. The initial app updater checked only dirty workbooks;
a clean workbook could permit restart while assistant work was active. The corrected guard also
checks native job/turn/dispatch state, remote turn/dispatch state and shared
`Controller::restart_safe()` from revision `986cb16234cfdac1ee1337fb4cdf00945cad4e5a`.
That accessor covers helper readiness, unacknowledged input, queued events, restart barriers,
update work and helper checkpoint safety. Missing/failed transport does not retain stale UI busy
flags forever. There is no protocol change; the installed helper remains compatible.

Finish chats/settings edits, then close the Assistant panel before app replacement if it is
deferred: pending render/input frames conservatively block restart. The existing unsaved-workbook
and Preview update opt-out safeguards remain in force. Focused regressions cover clean workbooks
with active chat/settings UI, native turns/dispatch/jobs, remote dispatch and stale widget flags.
The full UI library suite passed 326 tests; all-target Clippy for UI/desktop/CLI and the WASM
check passed (only the existing upstream dead-code warning). Formatting/diff checks passed.
Corrected candidates and installed follow-up are recorded below.

## Corrected candidates and repeated installed smoke

Source `40305767ad5fcff58c6e7c12c77d0f3a1fe1c646` is committed/pushed on the same task branch.
The required-font optimized desktop/CLI build passed (7m21s); candidate root is
`/private/tmp/gridcraft-local-candidates-40305767`. `receipt.json` records complete metadata,
checksums and packaging procedure. Both standard/Preview signatures and ZIP-extracted app/CLI
copies passed strict verification. Version, architecture, minimum OS, identifiers, signed source
repository/protocol, Preview profile and update opt-out are unchanged. The CLI code is unchanged,
so the exact previously verified signed 0.16.0 bytes were retained.

| Candidate | Signed executable SHA-256 |
|---|---|
| Standard | `75e2feff3c87f9fa1f35b599c1048507c804cab134ff2b1f988547b2581744b4` |
| Preview native | `03ef3a5cb060f737b6e3869dc530d8e41fd47a510ad2bc7e7ffc5a401387a703` |
| CLI | `58c4cd2d282f2079fc1a1d8ae49be805019245c4968aca7804187c9730893ead` |

| Archive | SHA-256 |
|---|---|
| `gridcraft-0.16.0-40305767-macos-aarch64-local.zip` | `3e1bc60aa349348ec3f3f9187dbad715ecc4b5cdcddf48ad381e441e1f9d5cc3` |
| `gridcraft-assistant-preview-0.16.0-40305767-macos-aarch64-local.zip` | `f54a58598873d809bc1080f4e9495bf58e423a9763db96290146859e62a80ddc` |
| `gridcraft-cli-0.16.0-40305767-macos-aarch64-local.zip` | `96dffd3fad0cd5b447660845d9bf80f4d89c85b6cbafaff55cdafaade8d25d9b` |

The deployment owner replaced the initial standard app on both Macs and Mini Preview, retaining
backups. Independent repeated smoke asserted the exact installed source stamp and standard
executable hash above before launch:

| Host | Evidence root | Owned app / ephemeral loopback |
|---|---|---|
| Mini | `/private/tmp/gridcraft-installed-smoke-trwtphl0` | PID 78273 / 50081 |
| MacBook | `/private/tmp/gridcraft-installed-smoke-jzjpdtnf` | PID 45349 / 52508 |

Both passed: actual installed helper 0.1.0 connected/rendered; screenshots captured and visually
inspected; external MCP listed 22 tools and read exactly `[[2,5],[3,null]]`; native control reads
confirmed no changes. Each app quit normally, its helper child exited, and its listener was
verified released. The same temporary-profile/public-config-only isolation applied; no inference,
personal settings/documents or deployment edits were performed by this verification workstream.
Mini Preview additionally passed installed signature/source/hash, bundle identity and unchanged
profile/update-disabled launcher checks; its GUI was not separately exercised. Signature trust
verification used normal keychain access; the sandbox cannot evaluate this local certificate.
Earlier live-update, input-composition/clipboard/VoiceOver and independent-upgrade demonstration
limits still apply.

## Compact-panel final installation

The signed standard and existing Mini Preview candidates from production source
`e8f934958832f9a6efe7d1795b7b2582c9c6d806` were installed atomically. Standard is installed
on both Macs; Preview is installed only on the Mini. The signed native hashes are:

| App | SHA-256 |
| --- | --- |
| Standard | `ffc3cd4111677cb89b26549bb26d775a0d386cf1384595e965bcfd696db1a6c4` |
| Preview native | `e8a2e142fba63e0436322e45ca9f776c06794d3a4d3035751488ffeec2b415f2` |

Backups: `~/Applications/Craft Rollout Backups/2026-11-03-compact/` on each Mac. The CLI
retains its verified prior bytes (`58c4cd2d282f2079fc1a1d8ae49be805019245c4968aca7804187c9730893ead`); model/provider settings and personal
workbooks/history were not migrated. The host Git pin stays 83e61084; independently installed
helper 0.1.2 (3dda16ac) supplies New chat, Chat history, gear settings, file `+` and model controls.
Update version/check/status are hidden from chat and shown in settings.

Final installed native gear/settings and synthetic read-only MCP checks passed for Mini standard,
Mini Preview and MacBook standard. Each listed 22 tools, read `[[2,5],[3,null]]` unchanged and
released its test-owned process/listener. Exact source CI passed all 10 steps (3280 tests passed,
one ignored timing benchmark). Source/validation separation, final per-device evidence roots
and the resolved MacBook OCR cold-start retry are recorded in
[the integration ledger](assistant-v016-integration.md#final-two-mac-compact-panel-deployment--2026-11-03).

No live provider inference, IME, clipboard, VoiceOver or published-feed update activation was
performed for this revision. No public release was created.
