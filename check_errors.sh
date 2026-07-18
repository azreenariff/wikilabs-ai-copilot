#!/usr/bin/env bash
set -euo pipefail

cd ~/wikilabs-ai-copilot
source ~/.cargo/env

# Read the full cargo build output
ERRORS=$(cargo build --package wikilabs-knowledge 2>&1)
echo "$ERRORS" > /tmp/build_output.txt
ERROR_COUNT=$(echo "$ERRORS" | grep "^error" | wc -l)
echo "Current error count: $ERROR_COUNT"
echo "=== Top errors ==="
echo "$ERRORS" | grep "^error\[" | sed 's/.*error\[E[0-9]*\]: //' | sort | uniq -c | sort -rn

# Show first 30 unique error file locations
echo "=== Error locations ==="
echo "$ERRORS" | grep -oP 'src/knowledge/src/\S+' | sort -u | head -50