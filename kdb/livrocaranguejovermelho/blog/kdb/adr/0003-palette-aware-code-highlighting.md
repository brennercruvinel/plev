# 0003. palette-aware syntax highlighting over Zola's inline Solarized

- status: accepted
- date: 2026-06-23

## context

code blocks stayed green-on-navy in every palette, which looked wrong once the rest of the page followed the theme. the cause: this Zola fork bakes syntax colors inline on every token as `style="color:light-dark(#light,#dark)"` using a fixed Solarized palette. the class-based mode (`z-*` scope classes) is not emitted by this fork, i tried it and it broke the build, so ordinary CSS has nothing to hook onto. the inline color also wins the cascade against any normal rule.

## decision

an author `!important` layer in `sass/_code-theme.scss` matches each Solarized color by its `style` substring and remaps it to a theme variable. an author `!important` declaration outranks a normal inline style, which is the one lever that works against inline colors. keywords map to `--accent-color`, strings, functions, types, numbers, errors map to the theme's semantic vars (`--green-fg`, `--blue-fg`, and so on), comments and punctuation map to the muted scale, and the panel background becomes a tint of `--bg-color`. the exact Solarized hexes were extracted from the built HTML, not guessed.

## consequences

- one file re-themes code for every palette, light and dark, automatically.
- tokens stay distinct instead of collapsing into one accent color.
- it is coupled to Solarized's specific hexes. change the highlight theme in `config.toml` and this map has to change with it.
- `!important` is load-bearing here. normally a smell, but it is the only way to override an inline style without touching how Zola renders code.
