# GridCraft Assistant Preview 0.16.1 rollout

Updated 2026-11-04. Status: **In progress**. Official source target:
[`v0.16.1`](https://github.com/storytold/gridcraft/releases/tag/v0.16.1), commit
`e581bc95b2a17d2a744a115478a20a1c2ad1ef72`.

| Workstream / device | State | Acceptance and evidence | Delivery |
|---|---|---|---|
| Source integration | Verified | Merge official 0.16.1 while retaining Assistant settings, far-right View control, complete app dispatcher and updater opt-out. | Existing custom branch. |
| Exact checks and package | In progress | Repository CI, signed native build, source-stamped ZIP/DMG, checksums and extracted-bundle verification. | No public release. |
| Mac mini / AKsMini | Not started | Install only Preview; verify version/signature and isolated configured Terra edit, read-back, Undo, save/reopen. | Preserve profile and rollback bundle. |
| MacBook / AKs-MBPro.local | Not started | Independently verify the same artifact and native workflow. | Preview initially absent; standard app is separate. |

The host bridge retains compatible shared revision `83e61084`; independently installed helper
0.1.6 owns the native Codex/profile behavior. No provider, credential, profile or conversation
configuration is changed. The preview launcher keeps `--disable-updates` and its separate bundle
identity/profile. Standard application updates are a separate rollout.

Integration resolutions retain both Assistant settings and upstream workbook display options.
The tooltip fixture adopts upstream's explicit macOS test platform; production platform behavior
is unchanged. The official release workflow change is inherited from the source tag, with no
custom workflow edits.

Verification and deployment remain pending until recorded below. Existing instances are never
force-quit; only task-owned synthetic sessions may be closed through their normal quit command.

Exact repository CI passed all ten gates with one Cargo job and the official release font input `b076417b5880bd4154bd22f00695b1d28c6bea83`. The native smoke uses disposable app/AI profiles, normal Codex account discovery and an explicit disposable workspace; it never copies credentials or old isolated Codex homes.

CI completed: 3437 ordinary tests passed, 1 ignored; doctests passed. Full workspace clippy with warnings denied, formatting, assets, layers, WASM and generated parity/keytip freshness all passed. The source changes preserve the existing toolbar test and all supported-locale rendering checks. Packaging and installation remain pending.
