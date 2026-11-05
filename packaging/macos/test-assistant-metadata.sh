#!/usr/bin/env bash
# Synthetic plist regression; never builds, signs, installs, or changes settings.
set -euo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/gridcraft-metadata.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for variant in standard preview; do
  cp "$ROOT/packaging/macos/Info.plist.in" "$fixture/$variant.plist"
  if [[ "$variant" = preview ]]; then
    plutil -replace CFBundleIdentifier -string ai.storyteller.gridcraft.assistant-preview "$fixture/$variant.plist"
  fi
  CRAFT_UPDATE_REPOSITORY=example/gridcraft CRAFT_ASSISTANT_BRIDGE_PROTOCOL=1 write_assistant_update_metadata "$fixture/$variant.plist"
  [[ "$(plutil -extract CraftUpdateRepository raw -o - "$fixture/$variant.plist")" = example/gridcraft ]]
  [[ "$(plutil -extract CraftAssistantBridgeProtocol raw -o - "$fixture/$variant.plist")" = 1 ]]
  plutil -lint "$fixture/$variant.plist"
done
cp "$ROOT/packaging/macos/Info.plist.in" "$fixture/ordinary.plist"
CRAFT_UPDATE_REPOSITORY= CRAFT_ASSISTANT_BRIDGE_PROTOCOL= write_assistant_update_metadata "$fixture/ordinary.plist"
if plutil -extract CraftUpdateRepository raw -o - "$fixture/ordinary.plist" >/dev/null 2>&1; then exit 1; fi
for pair in 'example/gridcraft|' '|1' 'https://github.com/example/gridcraft|1' '../gridcraft|1' 'example/gridcraft|0' 'example/gridcraft|4294967296' 'example/gridcraft|0001'; do
  before="$(sha256 "$fixture/ordinary.plist")"
  if CRAFT_UPDATE_REPOSITORY="${pair%|*}" CRAFT_ASSISTANT_BRIDGE_PROTOCOL="${pair#*|}" write_assistant_update_metadata "$fixture/ordinary.plist"; then
    echo "error: invalid metadata pair accepted" >&2; exit 1
  fi
  [[ "$(sha256 "$fixture/ordinary.plist")" = "$before" ]]
done
python3 - "$fixture" <<'PYTEST'
import pathlib, plistlib, sys
root = pathlib.Path(sys.argv[1])
for name, identity in [('standard', 'ai.storyteller.gridcraft'), ('preview', 'ai.storyteller.gridcraft.assistant-preview')]:
    value = plistlib.loads((root / (name + '.plist')).read_bytes())
    assert value['CFBundleIdentifier'] == identity
    assert type(value['CraftAssistantBridgeProtocol']) is int
PYTEST
echo 'assistant metadata: both bundle variants, omitted metadata, and seven invalid pairs passed'
