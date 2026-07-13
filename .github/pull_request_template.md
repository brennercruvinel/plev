<!--
keep it short. the body explains the why; the diff shows the what. no
conventional-commits prefix.
-->

## what and why

<!-- one or two sentences. what changed, and the reason it had to. -->

## the gate (all four must be green)

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `cargo check --target wasm32-unknown-unknown -p showcase`

## contracts

- [ ] new crate or example is named in `kdb/arc/arc.yaml`, `kdb/arc/arc.md` and `README.md` (the arc sync guard enforces this)
- [ ] structure changes are reflected in the arc trio (arc.yaml is canonical)
- [ ] tests are real (happy + error + one edge), no mocks; visual claims measured by pixel
- [ ] no new dependency without a one-line justification of what it buys
