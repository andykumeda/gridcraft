# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                    workspace root
#   VERSION                 [workspace.package] version from Cargo.toml (override: GRIDCRAFT_VERSION)
#   DIST                    output directory for release artifacts (default: $ROOT/dist/release)
#   GRIDCRAFT_BUILD_SHA    git commit baked into the binaries (see crates/engine/src/build_info.rs)
#   GRIDCRAFT_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR        cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
  return
}

VERSION="${GRIDCRAFT_VERSION:-$(workspace_version)}"
if [[ -z "$VERSION" ]]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [[ -z "${GRIDCRAFT_BUILD_SHA:-}" ]]; then
  GRIDCRAFT_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export GRIDCRAFT_BUILD_SHA
export GRIDCRAFT_BUILD_DATE="${GRIDCRAFT_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
  return
}

# Copy licence and readme files that exist into a package directory.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE LICENSE-MIT LICENSE-APACHE COPYRIGHT NOTICE ATTRIBUTION.md; do
    if [[ -f "$ROOT/$f" ]]; then cp "$ROOT/$f" "$dest/"; fi
  done
  copy_font_licences "$dest"
  return
}

# Builds made with craft-fonts (CRAFT_FONTS_DIR, set by the release workflow) embed its fonts, so
# the package carries their licences as OFL-<family-dir>.txt. Nothing without CRAFT_FONTS_DIR.
copy_font_licences() {
  local dest="$1" f
  [[ -n "${CRAFT_FONTS_DIR:-}" ]] || return 0
  for f in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
    if [[ -f "$f" ]]; then cp "$f" "$dest/OFL-$(basename "$(dirname "$f")").txt"; fi
  done
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null; then sha256sum "$file" | cut -d' ' -f1; else shasum -a 256 "$file" | cut -d' ' -f1; fi
  return
}

# Optional signed assistant compatibility metadata. Both values are supplied by the
# packager's explicit release configuration; ordinary packages have neither field.
write_assistant_update_metadata() {
  local app_info="$1" app_repo="${CRAFT_UPDATE_REPOSITORY:-}" bridge="${CRAFT_ASSISTANT_BRIDGE_PROTOCOL:-}"
  if [[ -z "$app_repo" && -z "$bridge" ]]; then return 0; fi
  if [[ -z "$app_repo" || ! "$app_repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ||
        "${app_repo%/*}" = . || "${app_repo%/*}" = .. || "${app_repo#*/}" = . || "${app_repo#*/}" = .. ]]; then
    echo "error: configure CRAFT_UPDATE_REPOSITORY as owner/repository together with CRAFT_ASSISTANT_BRIDGE_PROTOCOL" >&2
    return 2
  fi
  if [[ ! "$bridge" =~ ^[1-9][0-9]*$ || ${#bridge} -gt 10 ]]; then
    echo "error: CRAFT_ASSISTANT_BRIDGE_PROTOCOL must be a nonzero u32" >&2
    return 2
  fi
  if (( 10#$bridge > 4294967295 )); then
    echo "error: CRAFT_ASSISTANT_BRIDGE_PROTOCOL exceeds u32" >&2
    return 2
  fi
  plutil -replace CraftUpdateRepository -string "$app_repo" "$app_info"
  plutil -replace CraftAssistantBridgeProtocol -integer "$bridge" "$app_info"
}
