#!/usr/bin/env python3
"""Regenerate all app icons from the source logo.png.

The original logo is 1373x784 (wide banner), which was stretched into
square icons resulting in the default Windows blue box. This script:
1. Creates a proper square version (centered crop + smart padding)
2. Generates all required icon sizes
3. Creates a proper Windows .ico with multiple sizes
"""

import struct
from PIL import Image
import os, io

BASE = os.path.dirname(os.path.abspath(__file__))
ICONS_DIR = os.path.join(BASE, '..', 'src-tauri', 'icons')
LOGO_PATH = os.path.join(ICONS_DIR, 'logo.png')

def create_square_source(logo_path, target_size=2048):
    """Convert the wide banner logo to a proper square icon source."""
    img = Image.open(logo_path).convert('RGBA')
    w, h = img.size

    # Step 1: Smart crop to square — take center, but prefer keeping
    # the full height since banners are typically horizontal
    # Strategy: place on a square canvas with the logo centered
    square_size = max(w, h)
    canvas = Image.new('RGBA', (square_size, square_size), (0, 0, 0, 0))

    # Center the logo on the canvas
    x_offset = (square_size - w) // 2
    y_offset = (square_size - h) // 2
    canvas.paste(img, (x_offset, y_offset), img)

    # Resize to target
    canvas = canvas.resize((target_size, target_size), Image.LANCZOS)
    return canvas

def generate_png_icons(square_src, output_dir):
    """Generate PNG icons at the sizes Tauri expects."""
    # Tauri v2 expects specific filenames:
    #   32x32.png  (32px)
    #   128x128.png  (128px)
    #   128x128@2x.png  (256px, retina variant of 128px)
    icon_map = [
        (32, '32x32.png'),
        (128, '128x128.png'),
        (256, '128x128@2x.png'),
    ]
    for size, filename in icon_map:
        resized = square_src.resize((size, size), Image.LANCZOS)
        path = os.path.join(output_dir, filename)
        resized.save(path)
        print(f'  ✓ {filename}  ({size}x{size})')

def generate_ico(square_src, output_dir):
    """Generate a proper Windows .ico file with multiple sizes.
    
    PIL's append_images doesn't work reliably for ICO, so we manually
    construct the ICO container with PNG-compressed image entries.
    """
    import struct
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_path = os.path.join(output_dir, 'icon.ico')

    # Generate PNG data for each size
    png_data = []
    for size in ico_sizes:
        img = square_src.resize((size, size), Image.LANCZOS)
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        png_data.append(buf.getvalue())

    # Build ICO directory entries + image data
    num_images = len(ico_sizes)
    header_size = 6 + num_images * 16  # ICO header + directory entries
    
    with open(ico_path, 'wb') as f:
        # ICO header
        f.write(struct.pack('<HHH', 0, 1, num_images))  # reserved=0, type=1(ICO), count
        
        # Directory entries (will update offsets later)
        entries = []
        data_offset = header_size
        for i, size in enumerate(ico_sizes):
            data_size = len(png_data[i])
            # w/h: 0 means 256 for >= 256
            w = 0 if size >= 256 else size
            h = 0 if size >= 256 else size
            entry = struct.pack(
                '<BBBBHHII',
                w, h,            # width, height
                0,               # colors (0 = no palette)
                0,               # reserved
                1,               # color planes
                32,              # bits per pixel
                data_size,       # size of image data
                data_offset      # offset in file
            )
            entries.append(entry)
            data_offset += data_size
        
        # Write directory entries
        for entry in entries:
            f.write(entry)
        
        # Write image data (PNG data = modern ICO format)
        for png in png_data:
            f.write(png)
    
    print(f'  ✓ icon.ico ({len(ico_sizes)} sizes: {ico_sizes[0]}x{ico_sizes[0]}..{ico_sizes[-1]}x{ico_sizes[-1]})')

def generate_icns(square_src, output_dir):
    """Generate macOS .icns file."""
    icns_path = os.path.join(output_dir, 'icon.icns')
    # macOS icns typically uses 1024x1024 source
    icns_img = square_src.resize((1024, 1024), Image.LANCZOS)
    icns_img.save(icns_path, format='ICNS')
    print(f'  ✓ icon.icns (1024x1024)')

def verify_ico(ico_path):
    """Verify the ICO file is well-formed."""
    with open(ico_path, 'rb') as f:
        header = f.read(6)
    assert header[:4] == b'\x00\x00\x01\x00', 'Invalid ICO header'
    count = struct.unpack('<H', header[4:6])[0]
    print(f'  ✓ ICO verified: {count} embedded images')
    return count

def main():
    print('=== Regenerating App Icons ===')
    print(f'Source: {os.path.basename(LOGO_PATH)} ({Image.open(LOGO_PATH).size})')

    # Create square source at 2048x2048 for maximum quality
    square = create_square_source(LOGO_PATH, 2048)

    # Save as high-res master
    master_path = os.path.join(ICONS_DIR, 'icon-square.png')
    square.save(master_path)
    print(f'  ✓ Square master: icon-square.png (2048x2048)')

    # Generate PNGs
    print('\nPNG icons:')
    generate_png_icons(square, ICONS_DIR)

    # Generate .ico
    print('\nWindows icon:')
    generate_ico(square, ICONS_DIR)
    verify_ico(os.path.join(ICONS_DIR, 'icon.ico'))

    # Generate .icns
    print('\nmacOS icon:')
    generate_icns(square, ICONS_DIR)

    print('\n=== Done! Icons regenerated successfully ===')

if __name__ == '__main__':
    main()
