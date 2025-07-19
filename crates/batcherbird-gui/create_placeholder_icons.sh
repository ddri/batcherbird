#!/bin/bash

# Create simple placeholder icons using ImageMagick or fallback methods
cd icons

# Check if ImageMagick is available
if command -v convert &> /dev/null; then
    echo "Creating icons with ImageMagick..."
    convert -size 32x32 xc:"#4682B4" -gravity center -pointsize 20 -fill white -annotate +0+0 "B" 32x32.png
    convert -size 128x128 xc:"#4682B4" -gravity center -pointsize 80 -fill white -annotate +0+0 "B" 128x128.png
    convert -size 256x256 xc:"#4682B4" -gravity center -pointsize 160 -fill white -annotate +0+0 "B" "128x128@2x.png"
    
    # Convert to ICO and ICNS (basic approach)
    cp 32x32.png icon.ico
    cp 128x128.png icon.icns
    
    echo "Icons created successfully!"
else
    echo "ImageMagick not found. Creating minimal placeholder files..."
    # Create minimal PNG files (just empty for now to satisfy build)
    touch 32x32.png
    touch 128x128.png
    touch "128x128@2x.png"
    touch icon.ico
    touch icon.icns
    
    echo "Placeholder files created. Install ImageMagick for proper icons: brew install imagemagick"
fi