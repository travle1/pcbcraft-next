# Shared family visual resources

These are unmodified resource copies, not a dependency on the SolidCraft app.
The authoritative per-file source, revision, license, author and SHA-256 are in
[provenance.json](provenance.json). No reference project was modified.

- Sixteen flat SVG masters were obtained through the user's `solidcraft-icons`
  catalog, pinned to SolidCraft `60488a0837affcee79832bdb7bf6e860e84b10f2`, source
  `assets/icons/ui/design/svg/flat/`. SolidCraft contributors; MIT OR Apache-2.0.
  A subset appears in the app; retained unused masters remain in the register.
- Workspace drawing/render, move, mirror, list, configuration-table and select
  SVGs and the later delete/measure masters, plus `icon-palette.toml`, were subsequently
  imported from SolidCraft `c553365a2197a689e749389a6fbf707a38df9b61`. Exact source
  paths/hashes are in the register; MIT OR Apache-2.0. Masters remain unchanged.
- IBM Plex Sans variable font comes from SolidCraft
  `c553365a2197a689e749389a6fbf707a38df9b61`, source
  `assets/fonts/ibm-plex-sans/IBMPlexSans-Variable.ttf`; IBM Corp., SIL OFL 1.1.

Retain [MIT](SOLIDCRAFT-LICENSE-MIT), [Apache](SOLIDCRAFT-LICENSE-APACHE),
[upstream NOTICE](SOLIDCRAFT-NOTICE) and [font OFL](IBM-PLEX-OFL.txt) with distribution.
The upstream NOTICE is preserved verbatim and describes additional upstream
materials; it does not mean all those materials are included here.

`src/desktop/icons.rs` supplies a small egui SVG consumer adapter. It resolves
`currentColor` and qualified source paint roles through the current icon palette
in memory, without editing masters. Texture keys include that palette.
Replace this adapter/resource bundle with a qualified independent upstream icon
package when available, preserving identifiers and notices. No upstream request
has been sent. The narrow [workspace picker adaptation](../../docs/workspace-picker-provenance.md)
also retains SolidCraft's source attribution and notices. The subsequent bounded
[panel/control adaptation](../../docs/panel-controls-provenance.md) uses the same
pin and retained licenses in `src/desktop/controls.rs`. The remainder of the
app composition is original PCBcraft code following the user's references;
no full SolidCraft shell or UI crate was imported.

The EDA glyphs in `../original/` (including chip, symbol, wire, workspace layout,
power, port, bus, net label, rules, ERC, sync, Simulate and schematic sheet) were authored for PCBcraft
and use the root MIT OR Apache-2.0 license. They are not imported SolidCraft artwork.
[Theme mapping and future preset contract](../../docs/theme-foundation.md).
