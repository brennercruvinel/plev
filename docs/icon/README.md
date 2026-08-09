# App icon

`book-artwork.png` is the source illustration (transparent background). `dock-icon.svg`
composes it over a squircle in the app's own palette — `--bg-card` (#201f3c) fading to `--bg`
(#121124), with a violet glow and a contact shadow so the book sits on the surface rather than
floating over it.

Two things are easy to get wrong here, and both were:

**Centre by the opaque bounding box, not the canvas.** The PNG carries 70px of empty space
above the book and 28px below, 1px left and 24px right. Centring the file leaves the book
visibly high and off to one side.

**Restore the alpha after rasterising.** `qlmanage` composites SVG onto *opaque white*, so the
transparent margin around the squircle comes out as solid white — which macOS renders as a
white frame around the icon in the Dock. `squircle_alpha.py` masks it back to the rounded-rect
body, feathering one pixel *inside* the shape so the white already blended into the antialiased
edge is cut away instead of left as a fringe.

## Regenerating

```
qlmanage -t -s 1024 -o . dock-icon.svg          # -> dock-icon.svg.png, with white corners
python3 squircle_alpha.py dock-icon.svg.png icon-final.png
# then sips -Z <size> icon-final.png into Spellbook.iconset/icon_<n>x<n>[@2x].png for
# 16, 32, 32, 64, 128, 256, 256, 512, 512, and a straight copy at 1024
iconutil -c icns Spellbook.iconset -o icon.icns
```

All ten representations matter: an `.icns` holding only 512px makes macOS fall back to the
generic "prohibited" badge at Dock sizes.

The Dock also caches the icon in its own preferences, so a rebuilt bundle alone won't show a new
icon. Clearing `com.apple.iconservices*` under `/private/var/folders/.../C` and re-adding the
`persistent-apps` entry is what actually refreshes it.

This icon is the Dock/bundle icon only. The sidebar mark and favicon stay on the flat SVG in
`index.html` — this illustration is unreadable at 18px.
