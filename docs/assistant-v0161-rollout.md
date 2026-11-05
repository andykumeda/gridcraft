# GridCraft Assistant Preview 0.16.1 rollout

Updated 2026-11-04. Status: **Deployed on both Macs**. Official source target:
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
