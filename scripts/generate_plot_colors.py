#!/usr/bin/env python3
"""
Regenerate PLOT_COLORS with the same hues as the current palette,
but with S=100, L=50 for each color.
"""

import colorsys

# Paste your current PLOT_COLORS here:
PLOT_COLORS = [
    "#d32f2f",  # Red
    "#1976d2",  # Blue
    "#388e3c",  # Green
    "#f57c00",  # Orange
    "#7b1fa2",  # Purple
    "#c2185b",  # Pink
    "#0097a7",  # Cyan
    "#ff8f00",  # Amber
    "#6d4c41",  # Brown
    "#5d4037",  # Dark Brown
]


def hex_to_hue(hex_color):
    """Extract the hue (0-1) from a hex color."""
    hex_color = hex_color.lstrip("#")
    r, g, b = [int(hex_color[i : i + 2], 16) / 255.0 for i in (0, 2, 4)]
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    return h  # colorsys uses HLS, so h is in [0,1]


def hsl_to_hex(h, s, l):
    """Convert HSL (all in 0-1) to hex."""
    r, g, b = colorsys.hls_to_rgb(h, l, s)
    return f"#{int(r*255):02x}{int(g*255):02x}{int(b*255):02x}"


if __name__ == "__main__":
    new_colors = []
    for color in PLOT_COLORS:
        hue = hex_to_hue(color)
        # S=1.0, L=0.5
        new_hex = hsl_to_hex(hue, 1.0, 0.5)
        new_colors.append(new_hex)

    print("// PLOT_COLORS with original hues, S=100, L=50")
    print("export const PLOT_COLORS = [")
    for i, color in enumerate(new_colors):
        print(f"    '{color}', // Color {i+1}")
    print("];")
