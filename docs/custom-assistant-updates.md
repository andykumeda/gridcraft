# Customized assistant-enabled app updates

Status: implementation added 2026-11-02; no compatible release published or live update claimed.

The independently installed assistant and the GridCraft app have separate update lifecycles.
Attaching the shared assistant calls `configure_assistant_updates(bridge_protocol)` before update
checks begin. The app then uses `craft-update` with an explicit custom release policy. Standard
builds retain their existing updater. `--disable-updates` always blocks both startup and manual
checks; configuring a source never overrides that opt-out.

## Configure the app source

Add this app to `sources.json` in the shared assistant settings directory (macOS:
`~/Library/Application Support/Craft/ai`; `CRAFT_AI_CONFIG_DIR` overrides that directory):

```json
{
  "schema_version": 1,
  "apps": {
    "gridcraft": {
      "repo": "YOUR_GITHUB_ACCOUNT/gridcraft",
      "macos_team_id": "YOURTEAMID",
      "bridge_protocol": 1
    }
  }
}
```

Replace the repository and team with the actual compatible release repository and its
10-character Apple Developer team identifier. The bridge protocol must match this build.
For explicitly approved local candidates, set `macos_team_id` to the schema placeholder
`LOCALTEAM0` and add `local_certificate_sha1` with the exact 40-character signing-certificate
fingerprint. This opts into strict local certificate verification; it does not enable notarization
or accept arbitrary ad-hoc signatures. Leave that optional field absent for normal Apple-team
verification.

Configure the other apps as additional entries in the same file. Restart the app after editing
source configuration. Missing, malformed, unavailable or incompatible sources report an error;
they never select upstream or another publisher.

## Release contract

Customized macOS releases must preserve the installed app's `ai.storyteller.gridcraft` bundle
identifier and the standard package/asset names understood by `craft-update`. Separately named
older Assistant Preview bundles remain update-disabled until replaced by a compatible package.

Each stable GitHub release includes `craft-assistant-app.json`:

```json
{
  "schema_version": 1,
  "app_id": "gridcraft",
  "release_tag": "v1.2.3",
  "repository": "YOUR_GITHUB_ACCOUNT/gridcraft",
  "bridge_protocol": 1
}
```

The tag must equal the release tag. Before code signing, the app's `Contents/Info.plist` must
include `CraftAssistantBridgeProtocol` (integer) and `CraftUpdateRepository` (string) matching
that source policy. The package must publish a SHA-256 digest through GitHub's asset digest or
`SHA256SUMS.txt`. Installation verifies the checksum, configured Developer ID signature, bundle
identity, version and signed bridge/source metadata before replacing anything. A release-side
manifest alone cannot establish compatibility or publisher identity.

Updates retain the existing save-before-restart safeguard. No app update starts over unsaved
work. App updates are macOS-only for this first delivery; no unsigned/ad-hoc package or unconfigured
publisher is accepted. Explicit local certificate mode is limited to its pinned identity. The assistant's own update remains independent of this flow.

## Delivery state

No source file has been written into a user's settings, no repository has been created, and no
release or installed app was changed by this implementation. The WordCraft fork
`andykumeda/wordcraft` returned HTTP 404 during read-only verification on 2026-11-02; do not
configure that destination until it exists and publishes compatible signed releases.

## Embedded component controls

The desktop entry point calls `enable_shared_assistant()` before enabling startup checks.
Once enabled, panel drawing, chat/settings operations and inference use the shared child process;
missing or incompatible components report errors rather than invoking the older in-process
assistant. The older adapter is retained for library compatibility and regression tests.
Document tools still execute through this app's existing guarded tool host on its UI thread.

Existing `ui.assistant.send`, `newChat`, `stop` and `status` commands retain their names.
Additional native control commands take an empty parameter object:

- `ui.assistant.updateCheck`: check the configured shared component release source.
- `ui.assistant.updateStatus`: report installed/offered component and update state.
- `ui.assistant.updateInstall`: explicitly approve the currently offered component update.
- `ui.assistant.reloadComponent`: reload configuration when no app-authorized turn is active.

These commands return a clear error when shared mode is not enabled. Checking or installing a
component never changes the app's release source or bypasses `--disable-updates` for app updates.

## Verification and remaining checks

The following initial implementation evidence is historical. Current v0.16 integration checks
and delivery status are recorded in [the integration ledger](assistant-v016-integration.md).

The production UI library passed its scoped Cargo check against the local shared component
implementation. Shared updater regressions passed (175 tests, four environment/live tests
ignored); the additional incompatible-DMG preservation test passed separately. Shared proxy
regressions passed for wrong targets, repeated request IDs, cancellation, backpressure and
argument limits.

New app-level command/grant regression tests are included but have not been executed. Native
app linking, installed-app update replacement, real helper-to-document edits, VoiceOver and
cross-app binary-unchanged update demonstrations remain unverified. The development volume
ran below 1 GiB free, so further app builds/test linking were stopped to preserve existing
work. No published release or installed application was changed.

Shared implementation pin: `andykumeda/craft-libs@986cb16234cfdac1ee1337fb4cdf00945cad4e5a` (task branch `codex/assistant-restart-safety`). All shared dependencies use that revision together. Cargo locks must resolve the published Git source, never the temporary local verification patch.

The shared assistant/config/installer/updater final test run passed **228 tests, 4 ignored**. PDF production UI Clippy and Word/Grid production UI checks passed against byte-identical reviewed local source before publishing this pin. Native consumer test links were deferred for disk headroom; do not treat compilation as installed acceptance. No release or user configuration was changed.

App replacement also waits for assistant turns, settings jobs, queued input and safe helper
checkpoint state. Finish chats/settings edits and close the Assistant panel if pending rendering
defers restart. Missing or failed helpers do not block app updates through stale busy flags.
See [candidate and installed-smoke evidence](assistant-candidates.md) for the correction and
remaining live-update verification.

## Current local rollout — 2026-11-03

GridCraft production source e8f93495 is installed as standard on both Macs and as Preview on
the Mini. Its reviewed shared-library pin remains 83e61084. The separately installed assistant
was updated to 0.1.2 (source 3dda16ac) without another host rebuild: New chat and Chat history
are explicit controls, while helper version/update checks/update status are inside gear settings.
Final installed native placement and read-only MCP smoke passed on all three installed bundles.
Exact source CI passed all ten gates; the validation-only 2002be68 test/doc commit did not change
the signed production binaries. See [the integration ledger](assistant-v016-integration.md)
for signed hashes and backup locations. Published-feed update activation remains unverified.
