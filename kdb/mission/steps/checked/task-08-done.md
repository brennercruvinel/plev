---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-08: efeitos de compositing (blur, shadow, opacity)

## objetivo
implementar efeitos visuais que operam sobre camadas: gaussian blur, drop shadow, opacity groups. efeitos são aplicados na composição de camadas, não por primitiva.

## contexto
com o layer system (task-07), efeitos são post-processing sobre texturas de camada. blur e shadow são os mais pedidos para UI de produção (backgrounds frosted-glass, elevation via shadow).

**decisão: fragment shader only** (sem compute shaders, conforme rules.md).

## dependências
- task-07 (layer system, integração final, fase e)
- fases a-d são independentes de task-07

## checklist de conclusão
- [x] gaussian blur (separável, two-pass horizontal+vertical)
- [x] drop shadow (silhouette extraction + blur + color)
- [x] opacity group (composite shader com alpha uniform)
- [x] texturepool para texturas temporárias (grow-only, reuso)
- [x] shaders WGSL para blur (fragment shader, 13-tap separável)
- [x] shaders WGSL para shadow (silhouette extraction)
- [x] shaders WGSL para composite (premultiplied alpha blending)
- [x] effectprocessor com apply_blur, apply_shadow, composite_pass
- [x] exemplo funcional: effects_demo com shadow + blur + opacity
- [x] `cargo check --examples` passa sem warnings
- [x] `cargo test` passa (10 testes, 0 falhas)
- [ ] efeitos configuráveis por camada via API (fase e, depende task-07)
- [ ] performance: blur em resolução 1080p < 2ms por frame (fase f)

## arquivos criados
| arquivo | propósito |
|---------|-----------|
| `src/effects.rs` | layereffect enum, effectprocessor, bluruniforms, compositeuniforms, shadowuniforms, gaussian_weights() |
| `src/texture_pool.rs` | texturepool com acquire/release, keyed por (width, height, format) |
| `shaders/blur.wgsl` | gaussian blur separável 13-tap, full-screen triangle |
| `shaders/shadow.wgsl` | extração de silhueta (alpha × shadow color) |
| `shaders/composite.wgsl` | composição com opacity uniform, premultiplied alpha |
| `examples/effects_demo.rs` | demo: quads -> offscreen -> blur + shadow -> composite to surface |

## arquivos modificados
| arquivo | o que muda |
|---------|-----------|
| `src/lib.rs` | +pub mod effects; +pub mod texture_pool; |

## armadilhas
- compute shaders não usados, fragment shader 13-tap é suficiente e portátil para WASM
- blur separável (h+v) é o(n) vs. o(n²) para kernel direto
- shadow: render silhueta -> blur -> composite (não re-renderiza cena inteira)
- texturepool grow-only, texturas nunca destruídas em steady state
- texturehandle armazena clone do textureview para evitar borrow conflicts com pool

## workflow
- ao iniciar: mover este arquivo para `mission/steps/ongoing/`
- ao concluir: renomear para `TASK-08-DONE.md`, mover para `mission/steps/checked/`
