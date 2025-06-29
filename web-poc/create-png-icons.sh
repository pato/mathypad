#!/bin/bash
set -e

echo "🎨 Creating PNG icons from SVG..."

# Check if rsvg-convert is installed
if ! command -v rsvg-convert &> /dev/null; then
    echo "❌ rsvg-convert is required but not installed."
    echo "   On macOS: brew install librsvg"
    echo "   On Ubuntu/Debian: sudo apt-get install librsvg2-bin"
    exit 1
fi

# Check if ImageMagick is installed (needed for maskable icon padding)
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick is required for creating maskable icons."
    echo "   On macOS: brew install imagemagick"
    echo "   On Ubuntu/Debian: sudo apt-get install imagemagick"
    exit 1
fi

# Create PNG versions with rsvg-convert
echo "📱 Creating standard icons..."
rsvg-convert mathypad-icon.svg -w 512 -h 512 -f png -o mathypad-icon-512.png
rsvg-convert mathypad-icon.svg -w 192 -h 192 -f png -o mathypad-icon-192.png
rsvg-convert mathypad-icon.svg -w 180 -h 180 -f png -o mathypad-icon-180.png
rsvg-convert mathypad-icon.svg -w 152 -h 152 -f png -o mathypad-icon-152.png
rsvg-convert mathypad-icon.svg -w 144 -h 144 -f png -o mathypad-icon-144.png
rsvg-convert mathypad-icon.svg -w 128 -h 128 -f png -o mathypad-icon-128.png
rsvg-convert mathypad-icon.svg -w 96 -h 96 -f png -o mathypad-icon-96.png
rsvg-convert mathypad-icon.svg -w 72 -h 72 -f png -o mathypad-icon-72.png
rsvg-convert mathypad-icon.svg -w 48 -h 48 -f png -o mathypad-icon-48.png

# Create a maskable version with safe area padding
echo "🎭 Creating maskable icon with safe area..."
# First render at smaller size to leave room for padding
rsvg-convert mathypad-icon.svg -w 410 -h 410 -f png -o temp-maskable.png
# Then add padding with white background for contrast
convert temp-maskable.png -gravity center -background "white" -extent 512x512 mathypad-icon-maskable-512.png
rm temp-maskable.png

echo "📊 Generated icons:"
ls -lh mathypad-icon-*.png | awk '{print "   " $9 ": " $5}'

echo ""
echo "🔍 Checking icon rendering..."
# Quick check to see if the icons have proper colors
if command -v identify &> /dev/null && [ -f "mathypad-icon-512.png" ]; then
    COLORS=$(identify -format "%k" mathypad-icon-512.png 2>/dev/null || echo "0")
    if [ "$COLORS" -lt 10 ]; then
        echo "⚠️  Warning: Icons may not have rendered correctly (only $COLORS colors detected)"
    else
        echo "✅ Icons appear to have rendered correctly ($COLORS colors detected)"
    fi
fi

echo ""
echo "✅ PNG icons created successfully!"