# Portable assistant preview packages

`assistant-preview.sh` packages an explicitly supplied release binary without building,
installing or publishing it. The default is Apple silicon; `--arch x86_64` and
`--arch universal` require matching Mach-O slices. The minimum macOS version is read from
the binary. Version and full source commit are mandatory; the script runs the binary's
`--version` and checks its version and eight-character commit before writing outputs.
The full commit remains caller-supplied provenance. Package on a Mac that can execute that
architecture; this check does not establish source equivalence for an arbitrary binary.

```sh
packaging/macos/assistant-preview.sh \
  --binary /path/to/release/gridcraft \
  --version 0.1.0 \
  --build-sha FULL_40_CHARACTER_SOURCE_COMMIT \
  --output-dir /path/to/output
```

The default output directory is `dist/assistant-preview/<short-commit>/`.
Outputs are version/commit/architecture-named ZIP and drag-to-Applications DMG files,
adjacent SHA-256 checksums and an installation README. The ZIP contains its own installation README and **GridCraft Assistant Preview.app**.
Existing output files are refused. Temporary staging is removed on exit. For binaries built
with optional fonts, supply the same `CRAFT_FONTS_DIR` so `copy_docs` includes their licences.

The bundle uses `ai.storyteller.gridcraft.assistant-preview`. Its shell launcher derives the
document profile from the destination user's `HOME`, respects `GRIDCRAFT_PROFILE_DIR`, and
adds standard Homebrew paths and `~/.local/bin` for Codex discovery while preserving `CRAFT_CODEX_BINARY`.
There is no machine-specific `LSEnvironment`. The launcher executes the separately signed
`GridCraft-native` binary with `--disable-updates`. The supplied binary must advertise that
flag in `--help`; the packager refuses older binaries. Only that binary, original project assets, licences and generated
metadata are packaged: no settings, chats, credentials, Codex installation or Ollama models.

`MACOS_SIGN_IDENTITY` and optional `MACOS_KEYCHAIN` follow the main macOS packaging script.
The default is ad-hoc signing. This script does **not** notarize; a non-default identity is not
automatically a Developer ID identity. The installation README reports that limitation.
For public distribution with normal Gatekeeper trust, a valid Developer ID signature and
successful notarization remain separate delivery work. Do not treat local signature validation
as acceptance on another Mac. The standard GridCraft updater is not a preview update channel and is disabled by the
launcher. The main GridCraft app remains separate with its normal updater. New preview builds
are installed at `/Applications/GridCraft Assistant Preview.app`; existing profiles are retained.

The script signs inside-out and verifies both the assembled bundle and a fresh extraction of
the final archive. For a packaging smoke, use a temporary output directory and record the
actual binary source; an old installed binary is not evidence for a later repaired build.
Before delivery, independently check `--version` through the extracted launcher, architecture,
checksum, effective destination profile, and native acceptance using the intended release binary.

The packager verifies the mounted DMG app and a copy from the DMG, in addition to ZIP extraction,
before publishing outputs. It never installs the app itself. The owner's delivery workflow
installs the verified preview and retains a rollback copy. Upstream changes must be brought into the preview source deliberately, verified and rebuilt; no separate assistant service is included.

## Installation and rollback

Save work and quit an existing preview normally before replacing it. Never terminate an
unknown GridCraft process or replace a running preview; the main `GridCraft.app` is a separate
application and must remain unchanged. Inspect the bundle's display name, identity and native
executable before deciding which instance is the preview.

Copy the verified app from the mounted DMG to a staging directory, verify its signature and
run its launcher with `--version`, then install only `GridCraft Assistant Preview.app` under
`/Applications`. If a preview already exists, retain a timestamp/version-named copy in the
preview artifact directory with `ditto`, verify that copy's signature and executable checksum,
and keep it until the replacement passes native smoke. Preserve the profile at
`~/Library/Application Support/GridCraft Assistant Preview`; bundles do not contain that data.
To roll back, quit the preview normally and restore the retained bundle to that exact preview
path. Do not restore or replace the shared Craft credential/configuration store.

For every update, review upstream source changes, integrate them deliberately, run the required
repository checks and synthetic workbook acceptance, commit, rebuild and package a new preview.
The standard GridCraft updater is disabled for the preview and is not its delivery channel.
