#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$REPO_ROOT/web/src-tauri"
PROFILE="release"

if [[ "${1:-}" == "--debug" ]]; then
  PROFILE="debug"
elif [[ "${1:-}" == "--release" || -z "${1:-}" ]]; then
  PROFILE="release"
else
  echo "usage: $0 [--debug|--release]" >&2
  exit 2
fi

TARGET="$(rustc --print host-tuple)"
cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" --package gloamy --bin gloamy --target "$TARGET" --profile "$PROFILE"

SOURCE="$REPO_ROOT/target/$TARGET/$PROFILE/gloamy"
if [[ ! -x "$SOURCE" ]]; then
  echo "built daemon executable missing at $SOURCE" >&2
  exit 1
fi

mkdir -p "$TAURI_DIR/resources"
install -m 0755 "$SOURCE" "$TAURI_DIR/resources/gloamy"
