#!/usr/bin/env bash
# Automated tmux-based visual test for slash commands.
# Usage: ./run_tests.sh [--update-golden]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCREENSHOTS="$SCRIPT_DIR/screenshots"
GOLDEN="$SCRIPT_DIR/golden"
SESSION="meow-test"
BINARY="$SCRIPT_DIR/../../target/release/meow"

rm -rf "$SCREENSHOTS"
mkdir -p "$SCREENSHOTS"

echo "Building meow in release mode..."
(cd "$SCRIPT_DIR/../.." && ~/.cargo/bin/cargo build --release 2>&1)

if [ ! -f "$BINARY" ]; then
  echo "ERROR: Binary not found at $BINARY"
  exit 1
fi

echo "Starting meow in tmux session..."
tmux new-session -d -s "$SESSION" -x 120 -y 30 \
  "$BINARY -S localhost,1433 -U sa -P TestPass123! --trust-cert"
sleep 3

run_command() {
  local name="$1"
  local keys="$2"
  echo "  Testing: $name"
  tmux send-keys -t "$SESSION" "$keys" ""
  # Execute with Ctrl+Enter
  tmux send-keys -t "$SESSION" C-m ""
  sleep 1
  # Use F5 instead for reliability
  tmux send-keys -t "$SESSION" F5 ""
  sleep 2
  tmux capture-pane -t "$SESSION" -p > "$SCREENSHOTS/${name}.txt"
  # Clear editor with Ctrl+L
  tmux send-keys -t "$SESSION" C-l ""
  sleep 0.5
}

# Type command then press F5
send_cmd() {
  local name="$1"
  local cmd="$2"
  echo "  Testing: $name"
  tmux send-keys -t "$SESSION" -l "$cmd"
  sleep 0.5
  tmux send-keys -t "$SESSION" F5
  sleep 2
  tmux capture-pane -t "$SESSION" -p > "$SCREENSHOTS/${name}.txt"
  tmux send-keys -t "$SESSION" C-l
  sleep 0.5
}

send_cmd "slash_d" '\d'
send_cmd "slash_dt" '\dt'
send_cmd "slash_dv" '\dv'
send_cmd "slash_di" '\di'
send_cmd "slash_df" '\df'
send_cmd "slash_ds" '\ds'
send_cmd "slash_dn" '\dn'
send_cmd "slash_conninfo" '\conninfo'
send_cmd "slash_x" '\x'
send_cmd "slash_timing" '\timing'
send_cmd "slash_help" '\?'

echo "Killing tmux session..."
tmux kill-session -t "$SESSION" 2>/dev/null || true

echo ""
echo "Screenshots saved to: $SCREENSHOTS/"

# Compare with golden if exists
if [ -d "$GOLDEN" ] && [ "$(ls -A "$GOLDEN" 2>/dev/null)" ]; then
  echo "Comparing with golden screenshots..."
  FAIL=0
  for f in "$GOLDEN"/*.txt; do
    base="$(basename "$f")"
    if [ -f "$SCREENSHOTS/$base" ]; then
      if ! diff -q "$GOLDEN/$base" "$SCREENSHOTS/$base" > /dev/null 2>&1; then
        echo "  MISMATCH: $base"
        diff "$GOLDEN/$base" "$SCREENSHOTS/$base" || true
        FAIL=1
      fi
    else
      echo "  MISSING: $base"
      FAIL=1
    fi
  done
  if [ "$FAIL" -eq 0 ]; then
    echo "  All golden comparisons PASSED!"
  else
    echo "  Some golden comparisons FAILED."
    exit 1
  fi
else
  echo "No golden/ directory with files. Run update_golden.sh to create baselines."
fi
