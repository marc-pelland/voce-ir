#!/usr/bin/env bash
# Build the playground-wasm bundle and gate it against .size-baseline.
# Usage:
#   ./scripts/build-and-check.sh                    # build + check
#   ./scripts/build-and-check.sh --update-baseline  # build + write new baseline
#
# Exits non-zero if the bundle grew by more than 5% over the baseline,
# unless --update-baseline is set, in which case the new size is recorded.

set -euo pipefail

cd "$(dirname "$0")/.."

UPDATE=0
if [ "${1:-}" = "--update-baseline" ]; then
  UPDATE=1
fi

# Required tools.
for bin in rustc wasm-pack wasm-opt; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "error: '$bin' not on PATH" >&2
    case "$bin" in
      wasm-opt) echo "  install via: brew install binaryen   (or apt install binaryen)" >&2 ;;
      wasm-pack) echo "  install via: cargo install wasm-pack" >&2 ;;
    esac
    exit 1
  fi
done

# Build. wasm-pack's bundled wasm-opt is disabled in Cargo.toml; we run
# the system one explicitly below.
wasm-pack build --target web --release

# Run system wasm-opt. The feature flags match what rustc 1.94+ emits;
# without them the optimizer rejects valid input as "error validating".
wasm-opt -Oz --converge \
  --enable-bulk-memory \
  --enable-mutable-globals \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --enable-reference-types \
  --enable-multivalue \
  -o pkg/voce_playground_wasm_bg.wasm \
  pkg/voce_playground_wasm_bg.wasm

CURRENT=$(wc -c < pkg/voce_playground_wasm_bg.wasm | tr -d '[:space:]')
BASELINE_FILE=".size-baseline"
BASELINE=$(grep -E '^[0-9]+$' "$BASELINE_FILE" | head -1)

if [ -z "$BASELINE" ]; then
  echo "error: no numeric baseline in $BASELINE_FILE" >&2
  exit 1
fi

# 5% growth ceiling. Use awk for portable arithmetic on macOS (which has
# bash 3.x and no built-in floating-point math).
THRESHOLD=$(awk -v b="$BASELINE" 'BEGIN { printf "%d", b * 105 / 100 }')
DELTA=$(awk -v c="$CURRENT" -v b="$BASELINE" 'BEGIN { printf "%+d", c - b }')
PCT=$(awk -v c="$CURRENT" -v b="$BASELINE" 'BEGIN { printf "%+.1f", (c - b) * 100 / b }')

echo ""
echo "  bundle size: ${CURRENT} bytes"
echo "  baseline:    ${BASELINE} bytes"
echo "  delta:       ${DELTA} bytes (${PCT}%)"
echo "  threshold:   ${THRESHOLD} bytes (+5%)"
echo ""

if [ "$UPDATE" -eq 1 ]; then
  printf "# playground-wasm bundle size baseline (raw bytes, post wasm-opt).\n# CI fails if the built bundle exceeds this by more than 5%%.\n# Update via a \`chore(wasm): update size baseline\` commit so reviewers\n# can audit the delta. See docs/perf-investigation.md for context.\n%s\n" "$CURRENT" > "$BASELINE_FILE"
  echo "Baseline updated to ${CURRENT} bytes."
  exit 0
fi

if [ "$CURRENT" -gt "$THRESHOLD" ]; then
  echo "FAIL: bundle grew by more than 5% over baseline." >&2
  echo "       If this is intentional, run:" >&2
  echo "         ./scripts/build-and-check.sh --update-baseline" >&2
  echo "       and commit the change as 'chore(wasm): update size baseline'." >&2
  exit 1
fi

echo "OK: under 5% threshold."
