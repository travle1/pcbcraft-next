# README media

These are captures of the running **PCBcraft** desktop, using the original
fan-controller example. They are not FigCraft concept images, SolidCraft screenshots
or an AI-generated design. The example is an editing illustration, not an
electrically validated reference circuit.

Captured November 3, 2026 on Linux from desktop code at
`c4dd083baaf0b3505f80560c1f2249fc5115e73d` (egui/eframe 0.36.2).
The three PNGs are unmodified 1600 × 1000 app captures. The GIF retains that size
and uses a shared 256-color palette. No UI or circuit content was composited.

| File | Actual app state | Capture options |
| --- | --- | --- |
| [schematic.png](schematic.png) | Example schematic with U1 selected and Properties visible | `--demo` |
| [library.png](library.png) | Floating Parts library with the twelve built-in generic symbols | `--demo --capture-library` |
| [commands.png](commands.png) | Command-search palette, including sheet creation actions | `--demo --capture-search` |
| [desktop-tour.gif](desktop-tour.gif) | Loop through the three stills above, four seconds each | 12 seconds at 1 fps, palette-encoded with FFmpeg 8.1.3 |

The GIF is a **tour of captured states**, not a recording of pointer motion or
completed edits. The main README keeps the animation in an expandable section and
links all static alternatives. The frames were visually reviewed, and duration,
dimensions and decoding checked. Total image size is approximately 1.7 MB.

SolidCraft's [README at the reference pin](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/README.md)
informed the product introduction, real-app media, caption and build/status layout.
No upstream media was imported. Captures show existing licensed artwork/control
adaptations and IBM Plex Sans; their [provenance and notices](../../assets/family/README.md)
continue to apply. Original project contributions use the root MIT OR Apache-2.0
license. The original PCBcraft application mark is included; its [asset provenance](../../assets/original/README.md)
is separate from the capture process. No application dependency was added.

## Refresh the captures

Run from the repository root with a local graphical session. This starts three
brief, inactive app windows sequentially and closes each after capturing. It uses
isolated settings and only the built-in example. Review the new images before
replacing the committed presentation; do not reuse these captions for unsupported
features. The following is Bash, requiring Cargo and `timeout`:

```bash
cargo build --locked
mkdir -p work/readme-media/{config,data,cache} docs/screenshots

for state in schematic library commands; do
  extra=()
  case "$state" in
    library) extra=(--capture-library) ;;
    commands) extra=(--capture-search) ;;
  esac
  env XDG_CONFIG_HOME="$PWD/work/readme-media/config" \
      XDG_DATA_HOME="$PWD/work/readme-media/data" \
      XDG_CACHE_HOME="$PWD/work/readme-media/cache" \
    timeout 25s target/debug/pcbcraft --demo "${extra[@]}" \
      --capture "docs/screenshots/$state.png" --exit-after-capture
done
```

Then rebuild the animation with FFmpeg, leaving the PNGs unchanged:

```bash
cat > work/readme-media/tour.ffconcat <<'EOF'
ffconcat version 1.0
file '../../docs/screenshots/schematic.png'
duration 4
file '../../docs/screenshots/library.png'
duration 4
file '../../docs/screenshots/commands.png'
duration 4
file '../../docs/screenshots/schematic.png'
EOF

ffmpeg -hide_banner -loglevel error -y \
  -f concat -safe 0 -i work/readme-media/tour.ffconcat \
  -filter_complex 'fps=1,split[frames][colors];[colors]palettegen=stats_mode=full[palette];[frames][palette]paletteuse=dither=bayer:bayer_scale=5' \
  -t 12 -loop 0 docs/screenshots/desktop-tour.gif

ffprobe -v error -show_entries format=duration,size \
  -show_entries stream=width,height,nb_frames -of json \
  docs/screenshots/desktop-tour.gif
```

Update the source revision/date and verify all three states after a refresh.
Capture flags exercise presentation state only; app behavior is covered by the
existing core and interaction tests, not proven by these pictures.
