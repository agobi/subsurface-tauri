#!/bin/sh
# Regenerate visual regression baselines in the same Linux/amd64 environment as CI.
# Usage: npm run update-snapshots [-- <playwright-options>]
# Example: npm run update-snapshots -- --grep "prefs"
set -e

VITE_VISUAL_TEST=1 npx vite &
SERVER_PID=$!
trap "kill $SERVER_PID 2>/dev/null" EXIT INT TERM

until curl -sf http://localhost:1420 > /dev/null 2>&1; do
  sleep 0.3
done

docker run --rm --platform linux/amd64 \
  --add-host=host.docker.internal:host-gateway \
  -v "$(pwd):/work" -w /work \
  -e PLAYWRIGHT_BASE_URL=http://host.docker.internal:1420 \
  mcr.microsoft.com/playwright:v1.61.0-noble \
  node_modules/.bin/playwright test --update-snapshots "$@"
