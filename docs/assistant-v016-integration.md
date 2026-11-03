# GridCraft v0.16 assistant integration

Status: Verified source integration; candidate packaging in progress. This isolated task branch reconciles the assistant-enabled source with
upstream `85e51e00` (GridCraft 0.16.0). No installed app, release or user configuration is changed.

| Workstream | State | Acceptance and verification |
|---|---|---|
| Upstream merge | Verified | Preserve v0.16 engine, GUI, file-format and control-channel changes alongside the shared assistant. |
| Shared update contract | Verified | Use one assistant-enabled shared revision with v0.16 update branding; retain the configured fork source, opt-out and unsaved-work guards. |
| MCP parity | Verified | Run complete catalog equality, formerly blocked tools, asynchronous screenshot, cancellation and reentrant assistant regressions against the merged source. |
| Delivery | In progress | Commit verified source integration. Public releases and installed-app replacement remain separate steps. |

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

Verified on the final shared pin:

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
lock now use `andykumeda/craft-libs@3ff2754eeb1c079612247f4d288a22a20c169ac9`; no local
path override is used. Native input/accessibility, real provider inference, installed updates and the three-app unchanged-binary demonstration remain unverified.

Signed local standard/Preview bundles and a matching release CLI are the next delivery step;
artifact receipts will record their source commit, identities and checksums. No installed app,
CLI wrapper or user settings have been changed.
