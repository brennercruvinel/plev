---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-02-16
domain: task-tracking
---

# task-51: reestruturacao do workspace e rebrand de naming

## objetivo
levar o repo de crates soltos com naming inconsistente para uma estrutura de tres tiers profissional, e fechar o rebrand: phi (φ) e boitata viram plev, anm vira monster, prs vira parser.

## dependencias
- nenhuma; afeta o workspace inteiro
- precede task-52 (modularizacao SRP)

## contexto
o naming inicial do projeto era phi (φ) e boitata; o codec era anm (.anm, magic ANM0); o transpiler era prs. nomes inconsistentes e crates sem hierarquia clara. esta foi a passada de organizacao antes da modularizacao por SRP.

## o que foi entregue
- tres tiers: engine na raiz (`crates/engine`), libs e apps irmas em `crates/`, demos em `examples/`.
- renames de crate: editor_core -> rope, git_backend -> git, basic-ide -> ide, prs -> parser, anm -> monster, narrate_macro -> narrate-macro.
- rebrand de produto: boitata / phi -> plev; .anm -> .monster, magic ANM0 -> MON0.
- shaders movidos para `src/gpu/shaders` (junto de quem os carrega).
- cargo a nivel profissional: `workspace.package` (version, edition 2024, authors), `workspace.dependencies` (fonte unica de versao por dep), `workspace.lints` (clippy unificado), profiles tunados; cada crate `publish = false` com `description`.
- demos scene-3d e snake viraram examples; exemplos padronizados (sem sufixo _demo nem prefixo makepad_).

## numeros honestos
- 12 crates como members, ~17 examples.
- nenhuma mudanca de API publica, so organizacao e naming.

## referencias
- adr [workspace-engine-at-root-libs-in-crates-demos-in-examples](../../../adr/workspace-engine-at-root-libs-in-crates-demos-in-examples.md)
- commits a4ad0c0 (rebrand anm/boitata), b08bce8 (phi -> plev, kdb), 99929b9 (crates com nomes curtos), c2a90f1 (cargo workspace), 8aa4a8f (cargo profissional), e6f7091 (demos viram examples), 92c3aaa (shaders para src/gpu/shaders)

## fora de escopo
- mudanca de comportamento ou de API publica
