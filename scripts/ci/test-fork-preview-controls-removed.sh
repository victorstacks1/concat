#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
files=(
  "$root/src/crates/concat/ui/workspace/preview-pane.slint"
  "$root/src/crates/concat/ui/workspace/seat.slint"
  "$root/src/crates/concat/ui/editor.slint"
)

if rg -n 'playback-rate|playback-rate-step|preview-rate|preview-fullscreen|fullscreen-toggle' "${files[@]}"; then
  echo "preview fullscreen or playback-rate controls are still wired" >&2
  exit 1
fi

echo "fork preview controls: removed"
