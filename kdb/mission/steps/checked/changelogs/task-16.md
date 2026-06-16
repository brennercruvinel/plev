---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-16: effects <-> layer integration

## sessão 1 (2026-03-08)

### análise
- effectprocessor é serviço standalone: apply_blur, apply_shadow, composite_pass
- compositor só faz cpu-side prep (resolve), effects precisam de GPU passes
- texturepool grow-only, keyed por (width, height, format)
- layer tem offscreen texture, effects aplicados entre layer render e composite pass
- integration: add layereffect enum ao layer, aplicar no render loop
