---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-01: view trait + viewcontext

## objetivo
criar a abstração base `View` que encapsula a produção de `SceneNode`s. o usuário do φ deve declarar views que produzem cena, sem tocar no compositor diretamente.

## contexto
hoje o `window.rs` empurra `SceneNode::Rect` e `SceneNode::Text` diretamente no compositor. a camada de view abstrai isso, cada view recebe um `ViewContext` e retorna scenenodes. isso é o passo fundamental para a API declarativa futura.

## dependências
- nenhuma (primeiro passo da nova fase)

## checklist de conclusão
- [x] trait `View` definido com método `render(&self, cx: &mut ViewContext) -> Vec<SceneNode>`
- [x] `ViewContext` struct com viewport info (width, height), sem ref ao compositor por testabilidade
- [x] duas views concretas: `RectView` e `TextView`
- [x] `window.rs` refatorado para usar views em vez de empurrar scenenodes diretamente
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo run --example text_demo` funciona identicamente ao antes
- [x] 4 testes unitários: rect_view, text_view, custom_view_composes, dyn_view_dispatch

## armadilhas
- não over-engineer: view é trait simples, não framework de widgets
- não introduzir alocações por frame, views devem poder reusar buffers
- manter compatibilidade com o dirty tracking existente do compositor
- o borrow checker pattern do textsystem::resolve() não pode ser afetado

## trabalho paralelo
este projeto tem múltiplos agentes e devs trabalhando simultaneamente em tasks diferentes. regras:
- criar branch `task/TASK-01-view-trait` a partir de `main` antes de começar
- verificar com `git branch -a` se a branch já existe (outro agente pode ter começado)
- nunca commitar na `main`, todo trabalho na branch da task
- não alterar arquivos que pertencem a outras tasks sem registrar no changelog
- ao concluir: pr para `main` ou avisar o usuário para merge

## workflow
- ao iniciar: criar branch, mover este arquivo para `mission/steps/ongoing/`
- ao concluir: renomear para `TASK-01-DONE.md`, mover para `mission/steps/checked/`
