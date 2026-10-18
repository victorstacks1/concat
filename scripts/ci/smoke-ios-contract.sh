#!/usr/bin/env bash

# A clean iOS simulator smoke needs a rendered frame and a successful exit.
# Depending on the simulator lifecycle, quit_event_loop can end the app
# process before its final log marker reaches simctl's console stream.
smoke_result_ok() {
  local status=$1 output=$2
  [ "$status" -eq 0 ] || return 1
  grep -q 'smoke: ok' "$output" || grep -q 'smoke: first frame rendered' "$output"
}
