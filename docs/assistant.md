# Integrated assistant preview

Updated 2026-10-29 (America/Los_Angeles). Preview **0.13.0 (`00cb4830`)** is installed at
`/Applications/GridCraft Assistant Preview.app`. Full CI and installed Ollama edit/read-back/Undo
plus save/reopen checks passed. Implementation and delivery evidence:
[task ledger](assistant-implementation.md). This is a private preview, not an upstream release.

Open **Assistant** in the title toolbar or **View → Assistant**. Both controls indicate whether
the native side panel is open. Resize the panel or close it without losing its conversation.
**File → Options → Assistant**, or **Settings** inside the panel, configures one model for
questions and workbook tools. Advanced model options start collapsed. Desktop providers include
Codex, Ollama and the shared OpenAI-compatible API integrations. Browser inference is unavailable.

Connections, credentials and runtime configuration use the existing shared Craft assistant stack.
The effective provider, endpoint, model and context budget appear in Settings. The selected model
must support streaming and tools. Missing or unsupported connections fail visibly; no other model
or provider is substituted. Model discovery is a catalog request. The explicit tool-support check
uses synthetic inference and can incur provider charges.

Codex uses the separate Craft-managed account and compatibility checks. It never copies the coding
assistant's settings, skills, credentials, history or MCP registrations. The portable preview launcher
adds standard Homebrew and user binary paths; `CRAFT_CODEX_BINARY` remains an explicit override.
API keys use the platform credential store and stay out of JSON settings, workbook files and logs.

## Workbook work

The assistant can inspect workbook/sheet metadata, read bounded cell ranges (values, formulas and
styles), evaluate formulas, select cells, enter cells/ranges, format them, create tables/charts,
sort, filter and navigate Undo/Redo. Table creation uses the first row as existing headers; it never inserts a header row. Filtering ranges intersecting tables and sorting dynamic spill ranges are excluded in this preview. Reads require explicit finite ranges. Tools act in the
workbook, window and active sheet captured when the request begins. Switching targets or changing
the selection stops the old turn's tools. After an intervening workbook revision, a fresh read is
required before another edit. Relevant workbook text and prior chat history reach the selected
provider; unrelated questions do not automatically read workbook cells.

Writes must declare the exact rectangle matching their value matrix. Typed strings follow
GridCraft's normal parsing: `=SUM(A1:A2)` enters a formula and a leading apostrophe forces literal
text, such as `'=SUM(A1:A2)`. Formatting a cell as Text also affects parsing, as it does in the UI.
Formula parsing, calculation, locked-cell protection and validation use existing application paths.
Invalid validation results are rejected without a warning override. Calculated error/array values in validated cells are conservatively refused. Formulas that spill beyond the
write rectangle are refused. Workbooks remain subject to GridCraft's existing format limitations.

Each compound edit is atomic and uses normal **Undo**. Successful tools include actual read-back,
including formula text and calculated values where relevant. Inspect these results and the grid;
model narration is not proof. Selection changes are not undoable. Undo/Redo reverses/navigates
existing workbook history and is reported separately from a new edit. **Stop** cancels inference
and pending tools; completed edits remain undoable. Uncertain commands are never replayed.

Workbook text, comments and imported data are untrusted tool data, not user instructions.
Context/response limits apply before dispatch and result transmission. Large reads fail with
an actionable request for a smaller range; chat history is not silently discarded. Start a new chat
or change the documented runtime limits when a conversation exceeds its budget. Provider response
bodies are excluded from diagnostics because they can echo private content.

Arbitrary commands, scripts, protection changes, deleting sheets, screenshots and assistant file
access are excluded. Use normal **Open/Save** controls for workbook files. The reused shared panel's
Allowed files controls return an explicit unavailable error; they grant no file access in this
preview. No additional network listener or external MCP client is started by the assistant.

## Configuration and conversations

`CRAFT_AI_CONFIG_DIR` overrides the normal shared configuration directory, which on macOS is
`~/Library/Application Support/Craft/ai`.

| Location | Purpose |
| --- | --- |
| `connections.json` | Shared connections, capabilities, suite defaults and app overrides; credential references only |
| `runtime.json` | Shared validated request/context/response limits and tool-round budget |
| `chats/gridcraft/` | GridCraft-only saved conversations; separate from workbook files and WordCraft chats |
| Platform credential store | API keys |
| `codex/` | Isolated Craft Codex account/runtime state |

Use the canonical `assistant.chat` binding. A GridCraft override takes precedence over a suite
default; an invalid override fails. Legacy `assistant.control` bindings are preserved and never
used as fallback. Fresh GridCraft settings do not select a provider automatically. Explicitly
applying a suite-wide setting affects other Craft apps, as the settings button explains.

New chat, history selection and Delete chat manage local history. Interrupted chats reopen as
interrupted without restarting inference or edits. Chats can contain workbook data and tool
results; switching providers can send retained history to the new provider.

UI commands available through the existing control channel are `ui.assistant {value?}`,
`ui.assistant.send {text}`, `ui.assistant.stop {}`, `ui.assistant.newChat {}` and
`ui.assistant.status {}`. Sending and New chat start asynchronous work. Wait for `busy:false` and
`settingsBusy:false` before a subsequent request. Status excludes transcripts, workbook contents
and credentials. Legacy route fields alias the same single selected model.

## Preview maintenance

Install only **GridCraft Assistant Preview.app** at `/Applications/GridCraft Assistant Preview.app`.
Its distinct identity and profile preserve the main `/Applications/GridCraft.app`. The launcher
passes `--disable-updates`, preventing both automatic and manual standard updates from replacing
the preview. Upstream updates must be deliberately integrated into the preview source, checked,
rebuilt and repackaged. There is no separate assistant service.

See [packaging and rollback instructions](../packaging/macos/assistant-preview.md).
Other Macs require independent Codex sign-in or local provider setup. Ad-hoc signing is not
notarization; receiving-Mac acceptance is recorded separately from local delivery.

Local-model acceptance included a final-summary timeout after a correct, undoable edit. A fresh
conversation completed with an explicitly configured 300-second test timeout; the application
default remains 180 seconds. A timeout never authorizes replaying an uncertain operation. Inspect
the workbook and tool outcomes first. Other-Mac and live Codex acceptance remain separate checks.
