#!/usr/bin/env bash
# Record the diff-utils demo with VHS and burn key-hint overlays onto the video.
# Invoked by: pixi run demo-video
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/demo"

echo "Recording terminal session…"
vhs diff-demo.tape

echo "Adding key-hint overlays…"
ffmpeg -y -loglevel error -i diff-utils-python-demo.raw.mp4 \
  -vf "ass=key-hints.ass" \
  -c:v libx264 -crf 23 -preset medium \
  -c:a copy \
  diff-utils-python-demo.mp4

rm -f diff-utils-python-demo.raw.mp4
echo "Wrote demo/diff-utils-python-demo.mp4"
