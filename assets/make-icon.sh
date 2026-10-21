#!/bin/bash
# Regenerate AppIcon.icns and icon.png from icon.svg (needs rsvg-convert: brew install librsvg).
set -euo pipefail
cd "$(dirname "$0")"
set_dir=$(mktemp -d)/AppIcon.iconset
mkdir -p "$set_dir"
for s in 16 32 128 256 512; do
    rsvg-convert -w $s -h $s icon.svg -o "$set_dir/icon_${s}x${s}.png"
    rsvg-convert -w $((s * 2)) -h $((s * 2)) icon.svg -o "$set_dir/icon_${s}x${s}@2x.png"
done
iconutil -c icns "$set_dir" -o AppIcon.icns
rsvg-convert -w 256 -h 256 icon.svg -o icon.png
echo "Wrote AppIcon.icns and icon.png"
