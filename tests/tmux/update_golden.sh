#!/usr/bin/env bash
# Copy current screenshots to golden/ for baseline comparisons.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCREENSHOTS="$SCRIPT_DIR/screenshots"
GOLDEN="$SCRIPT_DIR/golden"

if [ ! -d "$SCREENSHOTS" ] || [ -z "$(ls -A "$SCREENSHOTS" 2>/dev/null)" ]; then
  echo "No screenshots found. Run run_tests.sh first."
  exit 1
fi

mkdir -p "$GOLDEN"
cp "$SCREENSHOTS"/*.txt "$GOLDEN/"
echo "Golden screenshots updated from $SCREENSHOTS/ → $GOLDEN/"
