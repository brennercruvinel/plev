---
id: task-75
title: "task-75: accessibility de plataforma production-ready"
project: plev
type: tarefa
status: em-andamento
date: 2026-09-25
date_basis: cabecalho
topics: [acessibilidade, plataforma, input, semantica]
relations: [adr-0068, task-18, task-19, task-57, task-58, task-59, task-60, task-70, task-74]
evidence: [d811876f, f3e9f914, 3badee06, 917cdd2a, 2a19ad66]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-75: accessibility de plataforma production-ready

Registra a task que faz a accessibility projetada pela task-60 funcionar com os leitores de tela de cada plataforma e com teclado completo nas páginas de componente. A fase 1 mostrou que só o macOS roda com VoiceOver hoje, e o registro de decisão desta task é o [ADR 0068](../decisoes/interface/0068-accessibility-is-production-ready-per-platform.md). As fases 2a a 2d corrigiram a árvore das páginas de app e de componente, projetaram as 30 páginas de componente com widgets projetáveis e levaram o teclado a 29 das 30 páginas, com o gate verde em cada fase segundo os handoffs. Continuam abertos o roteiro com o leitor real, que o Brenner executa, a decisão sobre os 102 pares de contraste abaixo do AA e as pendências de contrato do teclado, então a task segue em andamento.

## objetivo

a accessibility que a task-60 projeta do contrato passa a funcionar bem
com os leitores de tela reais de cada plataforma, com navegação de teclado
completa e com as semânticas de foco e ação que um produto grande exige
para poder adotar o plev.

## ligação com a tese

seção 21 (esse ponto sozinho mata frameworks alternativos; estar presente
e estar production-ready são coisas diferentes; VoiceOver, NVDA, TalkBack,
keyboard navigation, focus semantics e action semantics são cidadãos do
runtime).

## estado atual no código

- depois da 60: árvore AccessKit a partir do snapshot, `NodeId` estável,
  ações do leitor pelo dispatcher, ordem de foco semântica, ponte no
  showcase, verificado com VoiceOver no macOS.
- AccessKit 0.24 com `accesskit_winit` 0.32. alvos do plev: macOS e web
  shipando; android e ios em andamento (tasks 18 e 19); linux e windows
  pendentes.
- web: AccessKit não tem adapter para navegador.
- recipe de focus ring em `rust/engine/src/widgets/recipe.rs`; reduced motion e
  contraste wcag declarados como requisitos nativos no AGENTS.md.

## escopo

1. matriz por plataforma, com o suporte real do AccessKit verificado (não
   presumido): VoiceOver no macOS e no iOS, TalkBack no Android, NVDA e
   Narrator no Windows, Orca no Linux (AT-SPI). onde o adapter não existir
   ou não servir, o registro diz isso.
2. web: decisão por ADR entre um espelho DOM oculto com ARIA gerado a
   partir do snapshot e nenhuma accessibility no web por enquanto; se o
   espelho for o caminho, ele é outra projeção do mesmo snapshot, nunca uma
   segunda fonte.
3. teclado: Tab e Shift-Tab pela ordem semântica, setas em listas, menus,
   tabs e árvores (padrões WAI-ARIA), trap de foco em modal e retorno do
   foco ao fechar, atalhos anunciados.
4. live regions: toast e status anunciados sem roubar foco.
5. reduced motion (a preferência do sistema desliga transições e springs)
   e contraste (verificação dos pares de token de texto e fundo contra
   wcag AA nos temas).
6. roteiros manuais por plataforma, executados e com resultado registrado
   neste arquivo ou num anexo.

## fora de escopo

accessibility de texto rico (74, fase 9); bring-up de linux e windows
como alvo (arc: pendente; esta task só entra lá quando o alvo existir).

## dependências

60 integrada; por plataforma, o alvo precisa rodar.

## superfície sob responsabilidade

`rust/engine/src/control/accessibility/**`, os shells dos apps na parte da
ponte, `rust/engine/src/widgets` na parte de teclado e foco dos widgets, testes de
contraste em `rust/engine/src/styling/theme` (só testes), ADRs por decisão.

## interfaces que consome

a ponte e o construtor de árvore (60), `Snapshot` (57), dispatcher (58).

## interfaces que pode alterar

`rust/engine/src/control/accessibility` por inteiro; widgets de comps na parte de
teclado com aprovação.

## critérios de aceitação

- para cada plataforma com alvo rodando: roteiro executado com o leitor
  real sobre o showcase (todo, formulários, modal, tabs, lista) e o ide,
  resultado registrado.
- testes automáticos de árvore para toda página do showcase.
- teste de contraste dos tokens de texto nos temas.
- teclado completo nas páginas de componente, com teste.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

asserções de árvore por página, navegação de teclado simulada, contraste,
reduced motion desligando animação.

## riscos e perguntas abertas

- suporte do AccessKit por plataforma muda por versão; atualizar a versão é
  decisão com aprovação.
- teste manual não roda em CI; o registro é a evidência.

## commits esperados

um grupo por plataforma e por item (teclado, live regions, reduced motion,
contraste), com `task-75:` no início.

## pare e reporte se

- o web exigir o espelho DOM (decisão por ADR);
- um leitor real se comportar diferente do que a árvore promete.

## Fases

1. [fase 1: a sonda e os roteiros manuais](task-75-platform-accessibility/fase-1-sonda-e-roteiros.md): a matriz do AccessKit por plataforma pela fonte do lock, a árvore das páginas, live regions, contraste, reduced motion e o roteiro do VoiceOver.
2. [fase 2a: páginas de app que já projetam](task-75-platform-accessibility/fase-2a-paginas-de-app.md): ações sem alcance viradas em controles dentro do item, foco projetado, status com mensagem, toast, modal e menu na projeção da view.
3. [fase 2b: páginas de componente](task-75-platform-accessibility/fase-2b-paginas-de-componente.md): o trait `Component`, a projeção das 30 páginas com widgets projetáveis, as 46 páginas sem projeção com o motivo de cada uma, e as 60 baselines.
4. [fase 2c: teclado nas páginas de componente](task-75-platform-accessibility/fase-2c-teclado.md): Tab na ordem da projeção, setas por widget, Enter e Espaço como pedido da pessoa, modal com trap e retorno de foco.
5. [fase 2d: listas e tabelas com teclado](task-75-platform-accessibility/fase-2d-listas-e-tabelas.md): listas e tabelas com uma linha focável por vez, o teclado em 29 das 30 páginas, e a correção que devolveu abas e árvore à fase 2c.
