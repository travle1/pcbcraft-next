# Original PCBcraft icons

These EDA glyphs and the `pcbcraft.svg` application mark are original project
artwork, licensed MIT OR Apache-2.0 under the repository licenses. They do not
incorporate SolidCraft's logo, the preserved GPL fork or KiCad artwork.

The app mark draws a routed P on a layered board. `pcbcraft.svg` is the vector
source used by the palette-aware title-bar icon. `pcbcraft.png` is its transparent
256×256 native window icon, generated with:

```sh
rsvg-convert -w 256 -h 256 assets/original/pcbcraft.svg -o assets/original/pcbcraft.png
```

The native icon keeps its brand colors. The in-app icon follows semantic theme
roles. Browser symbol categories/items use `place-symbol.svg`, the same glyph as
the Capture ribbon; the app mark does not stand in for an electrical symbol.
