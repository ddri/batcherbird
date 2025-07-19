#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

def create_icon(size, filename):
    # Create a simple icon with a blue background and white "B" for Batcherbird
    img = Image.new('RGBA', (size, size), (70, 130, 180, 255))  # Steel blue
    draw = ImageDraw.Draw(img)
    
    # Try to use a system font, fallback to default
    try:
        font_size = size // 2
        font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", font_size)
    except:
        font = ImageFont.load_default()
    
    # Draw "B" in the center
    text = "B"
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    
    x = (size - text_width) // 2
    y = (size - text_height) // 2
    
    draw.text((x, y), text, fill='white', font=font)
    
    img.save(filename)
    print(f"Created {filename}")

# Create required icon sizes
icons = [
    (32, "32x32.png"),
    (128, "128x128.png"),
    (256, "128x128@2x.png"),  # 2x version
]

for size, filename in icons:
    create_icon(size, filename)

print("Icons created successfully!")