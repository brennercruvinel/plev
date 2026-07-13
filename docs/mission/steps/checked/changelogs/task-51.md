---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-02-11
domain: changelog
---

# task-51 changelog: reestruturacao do workspace e rebrand

## tiers
- [x] engine na raiz (crates/engine)
- [x] libs e apps em crates/
- [x] demos em examples/ (scene-3d e snake deixam de ser crate)

## renames de crate
- [x] editor_core -> rope
- [x] git_backend -> git
- [x] basic-ide -> ide
- [x] prs -> parser
- [x] anm -> monster
- [x] narrate_macro -> narrate-macro

## rebrand de produto
- [x] boitata / phi (φ) -> plev
- [x] .anm -> .monster, magic ANM0 -> MON0

## cargo profissional
- [x] workspace.package (version, edition 2024, authors)
- [x] workspace.dependencies (fonte unica de versao)
- [x] workspace.lints (clippy unificado)
- [x] profiles tunados, cada crate publish=false com description
- [x] shaders -> src/gpu/shaders

## validacao
- [x] 12 crates members, ~17 examples
- [x] API publica inalterada (so organizacao e naming)
