---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-07: sistema de camadas independentes

## objetivo
implementar camadas (layers) independentes no compositor. cada camada renderiza para uma textura offscreen propria. camadas estaveis (toolbar, sidebar) nao re-renderizam quando conteudo dinamico muda.

## contexto
hoje o compositor tem uma lista flat de scenenodes. com camadas, grupos de nodes renderizam para texturas separadas que sao compostas no final. isso habilita: caching de camadas estaveis, efeitos por camada (blur, opacity), e z-ordering correto.

## dependencias
- task-01 (view trait, views declaram em qual camada pertencem)

## checklist de conclusao
- [x] `Layer` struct com textura offscreen propria (render target)
- [x] compositor gerencia multiplas camadas com z-order
- [x] dirty tracking por camada, camada limpa = zero re-render
- [x] API para criar/destruir camadas
- [x] composicao final: combinar texturas de todas as camadas no framebuffer
- [x] exemplo funcional: camada de background estatica + camada de conteudo dinamico
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa (89 testes, incluindo 7 novos de layers)
- [x] performance: camada estatica = zero draw calls apos primeiro frame

## armadilhas
- texturas offscreen consomem vram, reusar e pooling
- composicao de camadas com alpha precisa de blend mode correto (premultiplied alpha)
- nao criar camada por view, camadas sao agrupamentos logicos (poucas por cena)
- wgpu no WASM pode ter limites de texturas simultaneas, testar

## workflow
- ao iniciar: mover este arquivo para `mission/steps/ongoing/`
- ao concluir: renomear para `TASK-07-DONE.md`, mover para `mission/steps/checked/`
