---
id: task-79
title: "task-79: cena por socket"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [agentes, gravacao, desempenho]
relations: [adr-0065, task-59, task-61, task-63, task-66, task-77]
source: docs/log/steps/task-79-scene-over-socket.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-79: cena por socket

A task 79 leva a cena de uma janela por socket: o app desenha, e um visualizador no desktop espelha o frame com as ligações por cima e devolve os cliques como `act` pelo contrato semântico. A fase 1 (task-79a) entregou o canal de cena na versão 1 do protocolo, o codec por layer suja no `rec`, o espelho em processo e entre dois processos com hash de frame igual ao do app nas páginas medidas, e a banda por página. A fase 2 (task-79b) trouxe o clique de volta como ato de agente e a máscara de segredo, sem mudar o protocolo; a imagem de fluxo da câmera nunca cruza o canal, só tamanho e lugar. A cena continua restrita ao localhost: sair dele, com pareamento e transporte cifrado, segue como pendência.

## objetivo

a UI roda num lugar e desenha noutro. o app no android manda a cena para o
desktop, que renderiza com o inspector por cima; ou o contrário, lógica
num servidor e cliente fino renderizando (server-driven UI no nível de
cena, não de JSON de widget). o urnaui explorando um corpus remoto sem
baixar nada é o caso óbvio.

## ligação com a tese

seção 27 (cena por socket), seção 2 (a cena é um vetor de nós puros,
hasheado por layer, gravável com delta).

## estado atual no código

- `SceneNode` é dado puro, `Clone + Debug + PartialEq`, sem serde
  (`rust/engine/src/drawing/compositor/scene.rs:6-7`).
- layers com hash: só as sujas precisariam viajar.
- o monster já sabe codificar cena com delta descoberto (modo B).
- depois da 61: transporte local e protocolo versionado para o agente.
  depois da 66: render sem janela.

## escopo

1. um transporte só: cena e semântica viajam pelo mesmo transporte da
   task-61, em canais separados, com o protocolo versionado. nenhum segundo
   protocolo paralelo.
2. codificação por frame: só as layers sujas, pelo hash; texto como
   `TextStyle` mais string (o receptor rasteriza com as mesmas fontes
   embutidas, determinismo da 77); imagem enviada uma vez por hash de
   conteúdo. avaliar reusar o codec de cena do monster em vez de serde.
3. visualizador no desktop que recebe e renderiza, com as ligações da
   task-59 por cima (inspector).
4. fase 2: input de volta (o visualizador manda eventos ou, melhor,
   requests semânticas pelo canal do agente).
5. rede: fora do local só com pareamento explícito e nunca execução
   remota; o ADR decide se esta task sai do localhost.

## fora de escopo

streaming de vídeo de pixels; multiusuário.

## dependências

61 e 66 integradas; 63 se o codec de cena do monster for o escolhido.

## superfície sob responsabilidade

o canal de cena no crate de protocolo (com aprovação, porque o protocolo é
da 61), o visualizador (exemplo ou app), serde ou codec para `SceneNode`
(mudança no engine, com aprovação), ADR
[`0065-scene-travels-as-data.md`](../decisoes/gravacao/0065-scene-travels-as-data.md).

## interfaces que consome

protocolo e transporte (61), headless e renderer (66), ligações (59),
codec do monster (63).

## interfaces que pode alterar

`SceneNode` por derive de serialização, com aprovação.

## critérios de aceitação

- o showcase rodando num processo (ou no simulador iOS) espelhado num
  visualizador desktop com pixels iguais (hash de frame renderizado nos
  dois lados).
- banda medida por página, com e sem mudança.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

roundtrip da codificação de cena, delta só com layers sujas, espelho em
processo com dois compositores.

## riscos e perguntas abertas

- segurança: cena pode conter dado do usuário; nada sai da máquina sem
  pareamento explícito.

## commits esperados

1. `task-79: serialização de cena por layer suja`
2. `task-79: canal de cena no transporte da 61`
3. `task-79: visualizador com ligações por cima`
4. `task-79: input de volta como request semântica`

## pare e reporte se

- o protocolo da 61 não comportar um segundo canal sem versão nova;
- `SceneNode` precisar mudar além de derive.

## Fases

1. [fase 1 (task-79a): a sonda](task-79-scene-over-socket/fase-79a-sonda.md): a medição do protocolo, do codec, das imagens e das ligações; a parada sobre os pixels; o canal, o espelho e a revisão de privacidade.
2. [fase 2 (task-79b): a sonda](task-79-scene-over-socket/fase-79b-sonda.md): a medição do clique e da redação; o clique de volta como `act`; a máscara de segredo; os testes e o handoff.
3. [fechamento (orquestrador, 2026-09-25)](task-79-scene-over-socket/fase-fechamento.md): o fechamento pelo critério de aceitação e as pendências.
