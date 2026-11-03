# Assistant and external MCP parity

Status: source implementation verified with synthetic tests, 2026-11-02. Not released or deployed.

The assistant advertises GridCraft's complete external MCP catalog, using the server's original
names, descriptions, argument schemas and descriptor metadata. There is no assistant-only tool allowlist, command
allowlist, rewritten schema, explicit-range rule, read-before-edit requirement, document/selection
lock, filesystem ban, or special filtering of tool results.

Calls go through `gridcraft_mcp::Server` (`tools/list` and `tools/call`). Its backend queues the
same `ControlRequest` handled by the live application's external control channel. Both transports
use one control dispatcher, including asynchronous screenshot admission, capture budgets, PNG
writing, command validation, document/window targeting, file handling and normal Undo. No listener
is started for the assistant. Cancellation and expiry abandon requests that have not started;
uncertain operations are never retried.

This includes `execute_command`, `list_windows`, `new_workbook`, `open_workbook`, `save_workbook`
and `screenshot`, alongside the existing spreadsheet tools. The actual MCP `tools/call` result
is returned without extracting only its first text block, adding a read-back wrapper, dropping
errors, or truncating its content. Existing external-server and engine validation remain the
authority for each operation.

Selected request files provide model context; they are not an exclusive filesystem allowlist.
The app's ordinary MCP file operations validate and handle their own paths.

Verification against shared libraries `c28f430ee4e932e7cbfa9e149b92f24e0f0bc202`: all 34 assistant regressions (including seven MCP-parity cases), all 33 control
regressions and all 24 external MCP tests passed. The MCP socket/token tests required an
unsandboxed run with an isolated temporary runtime directory after sandbox permission failures.
Workspace formatting and scoped all-target Clippy with `-D warnings` also passed.
The screenshot regression uses the real GUI
and capture callback pipeline with synthetic pixels; it is not evidence of a live GPU capture
or model inference. No personal workbook or proprietary asset is used.

The v0.16 reconciliation at shared revision `3ff2754eeb1c079612247f4d288a22a20c169ac9`
passed the full 323-test UI library suite (including the 34 assistant regressions) and all
25 external MCP tests. See [the integration ledger](assistant-v016-integration.md) for the
other checks and remaining installed/native acceptance limits.
