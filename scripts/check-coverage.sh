#!/usr/bin/env bash
# Coverage gate step: `make coverage` (the one home of the tarpaulin pin and
# invocation), with stdout HELD — tarpaulin's stdout carries the per-test roll
# and the report, which a passing run should not print, but a FAILING gate
# must name what failed; discarding stdout outright once reduced a named test
# failure to tarpaulin's opaque `Error: "Test failed during run"` (bl-0dff).
# Held, then replayed on failure: quiet when it passes, complete when it does
# not.
#
# Fingerprinted by bl-speculate as part of the gate identity (GATE_FILES,
# balls src/speculate.rs) — see scripts/pre-commit.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

out="$(mktemp)"
trap 'rm -f "$out"' EXIT
if ! make coverage >"$out"; then
  echo "error: the coverage gate failed. tarpaulin's held stdout follows." >&2
  cat "$out" >&2
  exit 1
fi
