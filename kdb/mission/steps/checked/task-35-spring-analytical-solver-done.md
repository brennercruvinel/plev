---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-35: fix spring<t>, analytical solver, P0, done

## objetivo
substituir o forward euler integration do spring<t> por solver analitico com coeficientes pre-computados. corrige bug de corretude: resultados diferentes a 30fps vs 60fps.

## justificativa
bug de corretude. o euler solver diverge com springs rigidos ou dt grande. o solver analitico (pattern c1 de natura) e incondicionalmente estavel para qualquer dt e produz o resultado matematico exato independente de frame-rate.

## dependencias
- task-27 (animation system), concluida

## referencia
- pattern c1 em `mission/knowledge/extracted-patterns.md`
- natura source: `bunker/repos/web-frontend/animation/natura/natura/src/spring.rs` l73-271

## estimativa
~100 LOC

## checklist
- [x] pre-computar 4 coeficientes (`pos_pos_coef`, `pos_vel_coef`, `vel_pos_coef`, `vel_vel_coef`) no construtor de spring<t>
- [x] tratar 3 regimes: sub-amortecido (exp*sin/cos), criticamente amortecido (exp*polinomio), super-amortecido (duas exponenciais)
- [x] substituir `tick()` por multiply-add com coeficientes
- [x] adicionar `damping_ratio()` convenience method
- [x] manter API existente (`stiffness/damping/mass`)
- [x] re-computar coeficientes quando `set_target()` e chamado (se parametros mudam)
- [x] testes: comparar resultado a 30fps vs 60fps (devem ser identicos com solver analitico)
- [x] testes: spring convergencia com stiffness alta (que causaria divergencia euler)
- [x] verificar que testes existentes passam

## criterios de aceite
1. spring produz resultado identico a 30fps e 60fps (dentro de epsilon f32)
2. spring com stiffness=10000 nao diverge
3. zero regressao nos 35 testes de animacao
4. API publica nao muda (backward compatible)
