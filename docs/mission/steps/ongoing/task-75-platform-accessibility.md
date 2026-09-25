---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: generative-runtime
task: task-75
wave: trilha
depends: [task-60]
branch: task/75-a11y-platforms
---

# task-75: accessibility de plataforma production-ready

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
- recipe de focus ring em `crates/comps/src/recipe.rs`; reduced motion e
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

`crates/engine/src/accessibility/**`, os shells dos apps na parte da
ponte, `crates/comps` na parte de teclado e foco dos widgets, testes de
contraste em `crates/engine/src/theme` (só testes), ADRs por decisão.

## interfaces que consome

a ponte e o construtor de árvore (60), `Snapshot` (57), dispatcher (58).

## interfaces que pode alterar

`crates/engine/src/accessibility` por inteiro; widgets de comps na parte de
teclado com aprovação.

## critérios de aceitação

- para cada plataforma com alvo rodando: roteiro executado com o leitor
  real sobre o showcase (todo, formulários, modal, tabs, lista) e o ide,
  resultado registrado.
- testes automáticos de árvore para toda página do showcase.
- teste de contraste dos tokens de texto nos temas.
- teclado completo nas páginas de componente, com teste.
- `./script/gate` verde.

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
