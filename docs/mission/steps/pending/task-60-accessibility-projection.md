---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-60
wave: 4
depends: [task-57, task-58, task-59]
branch: task/60-a11y-projection
---

# task-60: accessibility como projeção do contrato

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

- `crates/engine/src/window/render.rs:99-125`: com a feature
  `accessibility` ligada e o leitor de tela ativo, cada hit region vira um
  nó AccessKit; a role é `Button` se `focusable`, senão
  `GenericContainer`; label sempre `None`.
- `crates/engine/src/window/mod.rs:221`: `PlevActionHandler::do_action`
  só faz log. a árvore inicial tem label fixa "plev Showcase".
- `crates/engine/src/accessibility/` (`state.rs`, `id_map.rs`,
  `focus.rs`): `AccessibilityState`, `NodeId` derivado do `ViewId` posicional
  (`view_id_to_node_id`), `FocusGraph` construído de hit regions.
- tudo isso vive em `engine::window::App`. showcase, ide e urnaui têm o
  próprio `App` e usam o `present_frame` compartilhado
  (`crates/showcase/src/app.rs:367`, `crates/ide/src/main.rs:466`,
  `crates/urnaui/src/app.rs:432`). resultado: nenhum app que shipa expõe
  árvore.
- feature `accessibility` ligada por padrão
  (`crates/engine/Cargo.toml:21-22`), deps `accesskit` 0.24 e
  `accesskit_winit` 0.32.

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

- `crates/engine/src/accessibility/**`
- `crates/engine/src/window/{mod.rs,render.rs,lifecycle.rs}` (só o que é
  accessibility)
- `crates/showcase/src/app.rs` (a ponte no shell do showcase)
- proposta de ADR `docs/ADR/accessibility-projects-the-semantic-snapshot.md`

proibido: `crates/ide` e `crates/git` (task-62 na mesma onda),
`crates/sem` (mudança só com aprovação), o socket (task-61).

## interfaces que consome

`Snapshot`, `SemId`, `Role`, `Request`, `Dispatcher` (57, 58); `Binding` e
`Compositor::bindings` (59).

## interfaces que pode alterar

`crates/engine/src/accessibility` inteiro (é dela). `engine::window::App`
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
- `./script/gate` verde.

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
