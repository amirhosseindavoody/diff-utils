#!/usr/bin/env bash
# Record the diff-tool demo with VHS and burn key-hint overlays onto the video.
set -euo pipefail

cd "$(dirname "$0")"

vhs diff-demo.tape

echo "Adding key-hint overlays…"
ffmpeg -y -loglevel error -i diff-tool-python-demo.raw.mp4 \
  -vf "ass=key-hints.ass" \
  -c:v libx264 -preset fast -crf 23 -pix_fmt yuv420p \
  -c:a copy \
  diff-tool-python-demo.mp4

rm -f diff-tool-python-demo.raw.mp4
echo "Wrote demo/diff-tool-python-demo.mp4"
