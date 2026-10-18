#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/smoke-ios-contract.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

check() {
  local expected=$1 status=$2 content=$3
  printf '%s\n' "$content" > "$tmp/output.txt"
  if smoke_result_ok "$status" "$tmp/output.txt"; then
    actual=pass
  else
    actual=fail
  fi
  [ "$actual" = "$expected" ] || {
    echo "expected $expected for exit=$status output='$content', got $actual" >&2
    exit 1
  }
}

check pass 0 'smoke: ok'
check pass 0 'smoke: first frame rendered'
check fail 1 'smoke: first frame rendered'
check fail 1 'smoke: ok'
check fail 0 'startup log without a frame marker'

echo 'iOS smoke contract: ok'
