<!--
keep it short. the body explains the why; the diff shows the what. no
conventional-commits prefix.
-->

## what and why

<!-- one or two sentences. what changed, and the reason it had to. -->

## the gate

- [ ] `./script/gate` green: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `cargo check --target wasm32-unknown-unknown -p showcase`, and the urnaui leg when `../urna` is checked out
- [ ] `cargo run -p showcase --example regress -- check` unchanged, or the change blessed in the same PR

## contracts

- [ ] new crate or example is named in `docs/arc/arc.toml` and `README.md` (the arc sync guard enforces this)
- [ ] structure changes are reflected in `docs/arc/arc.toml` (the tables are canonical)
- [ ] tests are real (happy + error + one edge), no mocks; visual claims measured by pixel
- [ ] no production file past 639 lines (the file hygiene guard enforces this)
- [ ] no new dependency without a one-line justification of what it buys

## design for ranges, not screens (when the change touches anything visible, on any platform)

- [ ] fluid scale: typography, spacing and dimensions scale continuously between limits, from design tokens; no loose number inside a component
- [ ] component-driven responsiveness: the component responds to the space, content and context it receives, not to the device or the global viewport; a breakpoint only where the composition has to change
- [ ] intrinsic layout: size, distribution, wrapping and proportion resolve by themselves; fixed sizing only where the dimension is a real part of the design
- [ ] relative and semantic values: relative measures, tokens and logical properties; no arbitrary pixel value, unnecessary physical coordinate or device-specific rule
- [ ] progressive adaptation: holds across input modalities, reduced motion, high contrast, font scaling and zoom; checked over continuous ranges of dimensions and states (longer content, translation, empty, overflow), not only device presets
