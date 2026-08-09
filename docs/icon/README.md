# App icon

`book-artwork.png` is the source illustration (transparent background). `dock-icon.svg`
composes it over a squircle in the app's own palette — `--bg-card` (#201f3c) fading to `--bg`
(#121124), with a violet glow and a contact shadow so the book sits on the surface rather than
floating over it.

The artwork is centred by its **opaque bounding box**, not by the PNG canvas: the file carries
70px of empty space above and 28px below, so centring the canvas leaves the book visibly high.

## Regenerating

`sips` cannot composite, so the SVG is rasterised through Quick Look and cut into an iconset:

```
qlmanage -t -s 1024 -o . dock-icon.svg
# then sips -Z <size> into Spellbook.iconset/icon_<n>x<n>[@2x].png for
# 16, 32, 32, 64, 128, 256, 256, 512, 512, 1024
iconutil -c icns Spellbook.iconset -o icon.icns
```

All ten representations matter. An `.icns` holding only 512px falls back to the generic
"prohibited" badge at Dock sizes.

This icon is the Dock/bundle icon only. The sidebar mark and favicon stay on the flat SVG in
`index.html` — this illustration is unreadable at 18px.
