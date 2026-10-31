---
id: task-60
title: "task-60: accessibility as a projection of the contract"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [accessibility, semantics]
relations: [task-57, task-58, task-59, task-61, task-62, task-66, task-73, task-74, task-75, generative-runtime-thesis, adr-0026, adr-0038]
evidence: [3ba5b04]
source: docs/log/steps/task-60-accessibility-projection.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-60: accessibility as a projection of the contract

Takes the accessibility tree out of the hit-region geometry and derives it from the same `Snapshot` that the agent and the tests use, with the bounds of task-59's bindings. The screen reader's action enters the `Dispatcher` as `Actor::Assistive`, focus follows the semantic order, and the old path, derived from hit regions, is removed entirely. Merged in `3ba5b04` (eight commits), with a green gate in an isolated directory (1834 tests) and the showcase's whole as the first page with a tree; the VoiceOver test remains pending with Brenner, because the machine was on the login screen. It matters because the structure that explains the interface to an agent now also explains it to assistive technology, and ide and urnaui do not yet expose a tree.

## objetivo

a árvore de accessibility deixa de ser derivada de geometria e passa a ser
derivada do mesmo `Snapshot` que agente e teste usam, com os bounds da
ligação da task-59. a ação que vem do leitor de tela entra no mesmo
`Dispatcher`, como `Actor::Assistive`. e a accessibility chega nos apps
que shipam.

## ligação com a tese

seção 21 (accessibility é parte do contrato; uma estrutura semântica boa
para agentes explica a interface para tecnologia assistiva, e isso é teste
de qualidade da própria semântica), seção 6 (um controle, cinco
consumidores), e as ideias de implementação do arquivo de concepção
("completar o caminho semântico do AccessKit": ação vinda da
accessibility, do agente ou do humano chega no mesmo caminho, respeitando
disabled, foco, modais e regras da interface).

## estado atual no código

- `rust/engine/src/runtime/window/render.rs:99-125`: com a feature
  `accessibility` ligada e o leitor de tela ativo, cada hit region vira um
  nó AccessKit; a role é `Button` se `focusable`, senão
  `GenericContainer`; label sempre `None`.
- `rust/engine/src/runtime/window/mod.rs:221`: `PlevActionHandler::do_action`
  só faz log. a árvore inicial tem label fixa "plev Showcase".
- `rust/engine/src/control/accessibility/` (`state.rs`, `id_map.rs`,
  `focus.rs`): `AccessibilityState`, `NodeId` derivado do `ViewId` posicional
  (`view_id_to_node_id`), `FocusGraph` construído de hit regions.
- tudo isso vive em `engine::window::App`. showcase, ide e urnaui têm o
  próprio `App` e usam o `present_frame` compartilhado
  (`apps/uigallery/src/shell/mod.rs:367`, `apps/idestudio/src/main.rs:466`,
  `apps/urnadbapp/src/app.rs:432`). resultado: nenhum app que shipa expõe
  árvore.
- feature `accessibility` ligada por padrão
  (`rust/engine/Cargo.toml:21-22`), deps `accesskit` 0.24 e
  `accesskit_winit` 0.32.

## fechamento (2026-09-23)

integrada na `main` em `3ba5b04` (oito commits), rebaseada pelo
orquestrador sobre a 62, gate verde num diretório de build isolado (1834
testes). a árvore AccessKit sai do mesmo `Snapshot` e das mesmas ligações
que agente e teste usam (`engine::accessibility::tree::build`); NodeId
estável derivado do `SemId` (FNV-1a, colisão resolvida de forma
determinística); ação do leitor de tela vira `Request` com
`Actor::Assistive` (Click para a ação primária, SetValue, Increment,
Decrement); ordem de foco semântica; `Bridge` cria o adapter antes da
janela ficar visível. o caminho antigo, derivado de hit region, saiu por
inteiro (o grep confirma). o showcase liga a ponte e o todo é a primeira
página com árvore. ADR: [`0038-accessibility-projects-the-semantic-snapshot.md`](../../adr/interface/0038-accessibility-projects-the-semantic-snapshot.md).

decisões do orquestrador sobre as duas perguntas do agente:

- `Focus` do leitor de tela não é capacidade: vira `Assist::Focus`,
  tratado pelo mesmo gancho de foco que o Tab usa. foco é navegação, como
  hover e cursor, e não entra no contrato.
- `Role::Media` mapeia para `EmbeddedObject` até alguma app usar mídia;
  dividir em `Audio`/`Video` fica para quando houver uso.

aceitos: requests do leitor são observados sobre o snapshot que ele
mostrou, então estado que mudou volta `Stale`; nós sem ligação ficam na
árvore sem bounds (o AccessKit 0.24 não tem offscreen); nós inertes ficam
escondidos; `App::render` do engine continua fora do `draw_frame` porque
compõe efeitos por layer que o `draw_frame` não compõe (anotado para a
fase 2 da task-73). o `Theme::intent_role`, sem chamador depois desta
task, foi apagado pelo orquestrador no fechamento.

pendente com o brenner (teste manual que só ele pode fazer): o VoiceOver
não rodou, a máquina estava na tela de login. roteiro:

1. `cargo run -p uigallery -- Todo`, depois Cmd-F5.
2. VO-Direita pela janela. esperado: janela "plev, design system
   showcase"; todo; campo "new todo"; grupo de abas com "All"
   selecionada; três linhas com a terceira marcada; "2 active · 1 done".
3. com o cursor do VoiceOver no campo, digitar "buy milk" e VO-Espaço.
   esperado: linha nova e "3 active · 1 done".
4. VO-Espaço em "Absorb the plev demos…". esperado: a linha fica marcada
   e "2 active · 2 done".
5. o Accessibility Inspector do Xcode mostra a mesma árvore sem VoiceOver.

limitações que seguem: só o todo projeta; o chrome do showcase (sidebar,
header, overlays) não; o Tab do showcase não usa `focus::next` ainda; só a
ação primária é alcançável (sem custom actions para `remove`); no macOS o
adapter nunca informa desativação, então a árvore é montada a cada frame
depois da primeira consulta (22 a 26 µs com 103 itens). ide e urnaui
entram depois.

## notas do orquestrador (2026-09-23)

- a task-59 fase a está na `main`: `Compositor::bindings()` e `hit()`
  existem, e só `TextField`, `Checkbox` e `Button` (mais o todo do
  showcase) ligam ids; a fase b liga os outros widgets. a árvore desta
  task sai do snapshot mais as ligações, então páginas sem projeção
  continuam sem árvore até serem ligadas: isso é esperado, não um
  fallback a construir.
- `engine::window::App::render` (`window/render.rs`) ainda tem um fim de
  frame próprio, fora do `window::draw_frame` da task-66; é o mesmo
  arquivo onde mora o caminho de accessibility que esta task substitui.
  ao remover o caminho antigo, fazer esse `render` passar por
  `draw_frame` ou registrar por que não.

## escopo

1. construtor puro `Snapshot + bindings -> accesskit::TreeUpdate`:
   - `NodeId` derivado de hash estável do `SemId` (estável entre frames,
     para o leitor de tela não perder foco quando a cena se reorganiza);
     colisão detectada e tratada (nunca dois nós com o mesmo `NodeId`);
   - mapeamento `Role -> accesskit::Role` completo e testado;
   - label, description, value (texto, número com range e step), toggled
     (checkbox, switch), disabled, expanded, selected, `secret` como campo
     de senha sem valor;
   - bounds da ligação; nó do snapshot sem ligação neste frame (fora da
     tela, virtualizado) entra marcado como offscreen quando AccessKit
     suportar, ou fica de fora com o motivo no ADR;
   - filhos na ordem do snapshot; foco a partir de `Snapshot.focused`.
2. ações: `accesskit::ActionRequest` vira `Request` com
   `Actor::Assistive`: `Click` para a ação primária do nó (convenção
   documentada: `activate`, `toggle` ou `submit`), `Focus`, `SetValue`,
   `Increment`/`Decrement` para slider. o handler do AccessKit roda fora
   da thread principal: a request viaja por `EventLoopProxy` (o mesmo
   padrão do ADR worker-results-wake-the-event-loop) e é despachada na
   thread principal.
3. navegação de teclado a partir da ordem semântica: o `FocusGraph`
   derivado de hit region é substituído por ordem de foco derivada do
   snapshot (`focusable`, não inerte), com Tab e Shift-Tab.
4. substituir, não duplicar: o caminho derivado de hit region em
   `window/render.rs` e o `id_map` posicional saem. não pode existir dois
   caminhos de accessibility.
5. um helper que qualquer app shell usa (`engine::accessibility::Bridge`
   ou nome aprovado): cria o adapter `accesskit_winit` antes da janela ficar
   visível, recebe o snapshot e as ligações a cada frame em que o leitor
   está ativo, e entrega as requests do leitor para o app.
6. ligar no showcase (todo app da task-59 como referência). o ide entra
   depois da integração da task-62; o urnaui fica registrado como próximo
   passo.

## fora de escopo

qualidade production-ready por plataforma, testes manuais com NVDA,
TalkBack e Orca, live regions, web (sem adapter AccessKit): task-75.
texto rico acessível (runs, seleção): task-74.

## dependências

57, 58, 59 integradas.

## superfície sob responsabilidade

- `rust/engine/src/control/accessibility/**`
- `rust/engine/src/runtime/window/{mod.rs,render.rs,lifecycle.rs}` (só o que é
  accessibility)
- `apps/uigallery/src/shell/mod.rs` (a ponte no shell do showcase)
- proposta de ADR [`0038-accessibility-projects-the-semantic-snapshot.md`](../../adr/interface/0038-accessibility-projects-the-semantic-snapshot.md)

proibido: `apps/idestudio` e `rust/gitdrv` (task-62 na mesma onda),
`rust/accord` (mudança só com aprovação), o socket (task-61).

## interfaces que consome

`Snapshot`, `SemId`, `Role`, `Request`, `Dispatcher` (57, 58); `Binding` e
`Compositor::bindings` (59).

## interfaces que pode alterar

`rust/engine/src/control/accessibility` inteiro (é dela). `engine::window::App`
só no que toca accessibility.

## critérios de aceitação

- testes do construtor de árvore: roles, labels, estados, bounds, ordem,
  foco, `NodeId` estável entre dois frames com layout diferente.
- teste de ação: `ActionRequest` de click num nó do todo produz o mesmo
  estado que o clique humano, com receipt de `Actor::Assistive`.
- modal aberto: ação do leitor no fundo volta `Inert`.
- verificação manual no macOS com VoiceOver, roteiro escrito no handoff:
  navegar o todo app, ouvir role e label certos, ativar "adicionar" com
  VO+espaço, marcar item, ouvir o estado mudar. resultado registrado.
- nenhum resto do caminho antigo (grep por `view_id_to_node_id` e pelo
  bloco de `render.rs` vazio).
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

unitários do mapeamento e do construtor; teste de integração do todo app
com request `Assistive`; teste de ordem de foco; teste de colisão de
`NodeId`.

## riscos e perguntas abertas

- ciclo de vida do adapter por plataforma (macOS, iOS, Android) e em apps
  com o próprio `App`: se o helper não couber num dos shells, reportar.
- custo de construir a árvore a cada frame: só quando o leitor está
  ativo, e medido.

## commits esperados

1. `task-60: árvore AccessKit a partir do snapshot e das ligações`
2. `task-60: NodeId estável derivado do SemId`
3. `task-60: ações do leitor de tela entram no dispatcher`
4. `task-60: ordem de foco semântica no lugar do FocusGraph de hit region`
5. `task-60: remover o caminho derivado de hit region`
6. `task-60: ponte de accessibility no shell do showcase`

## pare e reporte se

- AccessKit não tiver equivalente para uma role ou um estado do contrato;
- o shell de algum app não permitir criar o adapter antes da janela;
- for preciso mudar `sem`.
