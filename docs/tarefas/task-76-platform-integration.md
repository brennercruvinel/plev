---
id: task-76
title: "task-76: integração de plataforma"
project: plev
type: tarefa
status: concluido
date: 2026-09-24
date_basis: cabecalho
topics: [plataforma, renderizacao, agentes, input]
relations: [adr-0054, adr-0055, adr-0056, adr-0057, adr-0058, adr-0059, adr-0060, adr-0061, adr-0062, adr-0063, task-58, task-60, task-66, tese-runtime]
evidence: [7b9f860f, 3173d2de]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-76: integração de plataforma

Registra a task-76, que faz o plev responder às bordas entre renderer e application runtime: perda de device de GPU, múltiplas janelas, diálogos e drop de arquivo, menus, tray, notificações, sleep e wake, alertas nativos, impressão, câmera, microfone e deep links. Cada integração que é uma ação vira capacidade do contrato, com uma representação na tela, no menu ou na porta do agente, e o mesmo dispatcher decide quem pode pedi-la. O resultado são os dez itens na `main`, cada um com o seu ADR (0054 a 0063); o último a entrar, o dos deep links, foi integrado em `7b9f860f` e fechou a task em `3173d2de`. As provas na tela ficaram para os roteiros do brenner.

## objetivo

o plev responde de forma consistente às bordas que separam renderer de
application runtime: menus, tray, notificações, diálogos de arquivo, drag
and drop, clipboard, IME, múltiplas janelas, diálogos nativos, impressão,
câmera, microfone, deep links, sleep e wake, perda de device de GPU e
accessibility. e cada integração que é uma ação vira capacidade no
contrato: o item de menu nativo e o botão da toolbar são duas
representações da mesma capacidade.

## ligação com a tese

seção 22 (integração de plataforma continua existindo; é parte da
diferença entre renderer e application runtime), seção 6 (uma capacidade,
várias representações).

## estado atual no código

- clipboard: `rust/engine/src/runtime/platform/clipboard.rs` (arboard no desktop,
  `LocalClipboard` em teste, wasm e mobile).
- diálogo de arquivo nativo (rfd) e drop de arquivo: só no urnaui
  (`apps/urnadbapp/src/app.rs`, `apps/urnadbapp/src/view/explorer/mod.rs`).
- IME básico: `rust/engine/src/runtime/platform/ime.rs`.
- lifecycle (estados de app, aviso de memória):
  `rust/engine/src/runtime/platform/lifecycle.rs`; safe areas em
  `rust/engine/src/runtime/platform/mod.rs`.
- sem menus nativos, tray, notificações do sistema, múltiplas janelas,
  diálogos nativos no engine, impressão, câmera, microfone, deep links,
  sleep e wake, ou recuperação de device de GPU perdido.

## escopo

ordem recomendada (robustez primeiro, depois o que os apps atuais pedem):

1. perda de device de GPU: callback de device lost do wgpu, recriação de
   surface, pipelines, atlas de glifo, texturas; teste forçando a perda.
2. múltiplas janelas no engine (hoje cada app tem uma).
3. diálogos de arquivo e drag and drop: mover do urnaui para
   `engine::platform`, sem segunda implementação; o urnaui passa a usar o do
   engine.
4. menus nativos: cada item é um `SemId` com ação no contrato; o menu é
   outra representação da capacidade, e o clique no menu vai pelo
   dispatcher como `Actor::Human`.
5. notificações do sistema e tray.
6. deep links (esquema de URL do app) que viram `Request` pelo dispatcher,
   com policy.
7. sleep e wake: pausar animação e trabalho de fundo, retomar sem frame
   velho.
8. diálogos nativos (alerta, confirmação), incluindo a confirmação humana
   da task-58 como opção.
9. impressão (render headless da task-66 para o sistema de impressão).
10. câmera e microfone (captura para textura e buffer de áudio), com
    permissão do sistema.

cada item: API no engine, a capacidade no contrato quando é ação, teste, e
dependência nova justificada (por exemplo menus e tray em crate
especializado) e aprovada antes.

## fora de escopo

bring-up de linux e windows como alvo; cada item entra nessas plataformas
quando o alvo existir.

## dependências

independente; os itens que viram capacidade usam 58.

## superfície sob responsabilidade

`rust/engine/src/runtime/platform/**`, `rust/engine/src/runtime/window` na parte de
múltiplas janelas e device lost (coordenado com a 60 se ela estiver
rodando), o urnaui na troca para o diálogo do engine, ADR por item.

## interfaces que consome

`wgpu`, `winit`, `sem` (58) para capacidades.

## interfaces que pode alterar

`engine::platform` por adição; `engine::window` com aprovação.

## critérios de aceitação

por item: funcionando no macOS (alvo principal), variante mobile ou web
registrada (feita ou declarada fora), teste automático onde é possível,
roteiro manual onde não é. device lost sobrevive a uma perda forçada sem
reiniciar o app.

## testes esperados

device lost forçado; múltiplas janelas com estado separado; menu como
capacidade (request pelo dispatcher); deep link com policy.

## riscos e perguntas abertas

- cada item pode puxar dependência grande; o custo de binário é medido.
- permissões de câmera e microfone mudam por sistema.

## commits esperados

um grupo por item, `task-76:` no início.

## pare e reporte se

- um item exigir mudar o shell de todos os apps;
- dependência nova (sempre aprovação).

## Fases

1. [item 1: perda de device de GPU](task-76-platform-integration/item-1-perda-de-device.md): o callback de device lost, a recriação no lugar e a prova headless com `Device::destroy`.
2. [item 3: diálogos de arquivo e drag and drop](task-76-platform-integration/item-3-arquivos-e-drop.md): o trait de diálogo do engine, o evento `FileDrop` e a página Files com `open_file` fora do alcance do agente.
3. [item 2: múltiplas janelas](task-76-platform-integration/item-2-multiplas-janelas.md): `WindowHost` e `Windows`, e a página do showcase numa segunda janela sobre o mesmo estado.
4. [item 4: menus nativos](task-76-platform-integration/item-4-menus-nativos.md): o `MenuModel` da projeção, o `muda` no desktop e o clique como o request do botão.
5. [item 8: diálogos nativos de alerta e confirmação](task-76-platform-integration/item-8-alertas-nativos.md): o trait `Alerts`, a confirmação humana do ide e o aviso da Files.
6. [item 5: notificações do sistema e tray](task-76-platform-integration/item-5-tray-e-notificacoes.md): o ícone da barra, a notificação com ação e o bundle que o macOS exige.
7. [item 7: sleep e wake](task-76-platform-integration/item-7-sleep-e-wake.md): coberta e dormindo sem frame nem deadline, e um frame novo ao mostrar ou acordar.
8. [item 9: impressão](task-76-platform-integration/item-9-impressao.md): o frame a 300 dpi pelo render headless e o CUPS como filho, sem destino nesta máquina.
9. [item 10: câmera e microfone](task-76-platform-integration/item-10-camera-e-microfone.md): o ffmpeg por dispositivo, a imagem de fluxo e os interruptores só de pessoa, soltos quando a página não está à vista.
10. [item 6: deep links](task-76-platform-integration/item-6-deep-links.md): a gramática do link, o ator `link` e a lista de capacidades abertas, na branch à espera da integração.
11. [fecho da task-76](task-76-platform-integration/fecho.md): nove itens na main, o que cada um deixou e o que fica para o bring-up.
