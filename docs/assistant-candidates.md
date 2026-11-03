# Local assistant-enabled candidates and installed smoke

Updated 2026-11-02. Local signed builds only; not notarized or publicly released.

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

Status: source verified; corrected release candidate build pending. The initial app updater checked only dirty workbooks;
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
Corrected signed candidates will be recorded after the verified source commit/build.
