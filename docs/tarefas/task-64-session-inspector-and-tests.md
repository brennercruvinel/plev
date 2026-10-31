---
id: task-64
title: "task-64: inspector de sessão, bug report e sessão para teste"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [gravacao, semantica, agentes]
relations: [adr-0050, estado-codigo-2026-09, task-63, task-65, task-66, task-83]
evidence: [e6560a2, d611fa4]
source: docs/log/steps/task-64-session-inspector-and-tests.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-64: inspector de sessão, bug report e sessão para teste

Registra a task que transformou uma sessão gravada em ferramenta de leitura e de teste: um inspector que responde às perguntas da tese sobre um controle ou um request (`plev-inspect` na fase a, a janela `plev-inspector` na fase b), um bug report exportado de um trecho (`--export`) e um teste de regressão gerado da sessão, rodado no gate com harness isolado (fase c). Foi fechada em 2026-09-25, com os critérios de aceite conferidos item por item. Importa porque o inspector só lê a gravação e nunca a executa; a reexecução controlada fica restrita ao teste, em ambiente isolado.

## objetivo

abrir uma sessão gravada, navegar no tempo com scrub, escolher um controle
num instante e ver o estado semântico, a ação, o ator, a precondição, o
delta e o resultado; exportar um trecho como bug report; e transformar um
trecho escolhido em teste de regressão.

## ligação com a tese

seções 11 (open session, scrub timeline, select control, inspect semantic
state, action, actor, result), 12 (bug report deixa de ser mídia; sessão
vira teste), 13 (no replay, selecionar um controle leva ao código daquela
versão), 28 (a pergunta do núcleo: o que alguém precisa descobrir no
primeiro `.monster` para confiar na delegação).

## estado atual no código

- `monster_player` (`demo/player/src`) toca
  animação com seek e scrub pelo player determinístico de
  `rust/motion/src/play/mod.rs`.
- depois de 63: `Session::open`, estado semântico em t, receipts por
  `SemId`, trilha visual.
- depois de 66: render headless de frame.

## escopo

### inspector

- um app ou uma página do showcase (decisão no briefing; recomendação:
  crate de app próprio, porque abre arquivo e vai crescer) que abre uma
  sessão.
- timeline com os eventos marcados por tipo e por ator; scrub com seek
  O(1) no keyframe mais próximo.
- o frame do instante, com as ligações desenhadas por cima (contorno do
  controle, `SemId` no hover).
- clique num controle no instante t: painel com o nó (role, label, state),
  os receipts que tocam esse `SemId` até t, e para cada receipt a cadeia
  causal: observação (quem, qual revisão), request (ator, `observed`),
  veredito (com o delta quando stale), execução, resultado.
- as perguntas da tese respondidas na tela: qual controle era esse, qual
  era o estado, qual foi a sequência anterior, qual ação ocorreu, quem
  iniciou, qual era a precondição, o que mudou, qual foi o resultado.
- link para o código quando a task-65 estiver integrada: o `source` do nó
  mais a identidade do build no cabeçalho da sessão; se a árvore de
  trabalho diverge daquele sha, mostrar o arquivo da versão gravada
  (`git show <sha>:<arquivo>`), não o atual.

### bug report

exportar `[t0, t1]` a partir do inspector (usa o export da task-63).

### sessão para teste

- o desenvolvedor escolhe, no inspector, os estados e resultados que
  viram asserções.
- a saída é um arquivo de dados (o trecho com as asserções marcadas) e um
  harness genérico que roda o trecho contra uma fixture do app: aplica as
  requests na ordem, confere os vereditos e os valores marcados.
- isso é reexecução controlada, e o documento diz isso: o harness roda num
  ambiente isolado (tempdir, relógio injetado, sem rede), a fixture é
  fornecida pelo app, e nada toca estado externo real. replay no
  inspector continua sem executar nada.

## fora de escopo

reexecução de efeitos externos reais; edição de sessão; compartilhamento
remoto.

## dependências

63 e 66 integradas; 65 para o link de código.

## superfície sob responsabilidade

o crate ou página do inspector; o harness de teste de sessão (onde o ADR
de 63 colocou o leitor); proposta de ADR
[`0050-sessions-become-regression-tests.md`](../decisoes/gravacao/0050-sessions-become-regression-tests.md).

## interfaces que consome

`Session` (63), render headless (66), `Node.source` (57) e o resolvedor de
código (65), comps para a UI.

## interfaces que pode alterar

as próprias.

## critérios de aceitação

- a sessão do cenário do commit (gravada em 63) aberta no inspector: scrub
  até antes do commit, clique no botão, painel mostra o estado que o agente
  observou, o request, a rejeição com "removed: B", depois o aceite e o sha.
- export de trecho reabre com o mesmo conteúdo.
- um teste gerado a partir da sessão roda verde no gate e falha quando a
  regra de precondição é quebrada de propósito (prova que o teste pega
  regressão).
- inspector não cria request em nenhum caminho (teste).
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

testes do modelo do inspector sem UI (estado em t, cadeia causal por
`SemId`), testes de viewport estreito e largo da UI, teste do harness com
uma fixture de brinquedo.

## riscos e perguntas abertas

- asserção gerada acoplada demais (todo o estado) fica frágil: o default
  é só o que o desenvolvedor marcou.
- a fixture do app é trabalho do app; o harness só define a interface.

## commits esperados

1. `task-64: modelo do inspector (estado em t, cadeia causal)`
2. `task-64: UI com timeline, frame e ligações`
3. `task-64: painel do controle e perguntas da tese`
4. `task-64: export de trecho`
5. `task-64: sessão para teste com harness isolado`

## pare e reporte se

- a geração de teste exigir reexecução que toque estado externo;
- o formato de sessão não der acesso O(1) que o scrub precisa.

## fases (orquestrador, 2026-09-23)

a task é grande (modelo, UI, export, sessão para teste) e foi dividida:

- **fase a, agora:** o modelo do inspector sem UI e uma CLI
  (`plev-inspect`) que responde às perguntas da tese sobre uma sessão
  gravada: o que era o controle, qual era o estado, quem observou o quê e
  quando, qual request, qual veredito (com o delta e o `acted`), o que
  aconteceu entre a observação e o request (as mudanças no que a ação lê
  e os receipts de outros atores que rodaram nesse intervalo, pela
  gravação), e o resultado. responde por tempo a pergunta "quem mudou o
  que eu li", que o `acted` da task-83 responde só em parte. funciona já
  nas sessões dos dois checkpoints a (`target/agents/exp/`,
  `target/agents/exp2/`).
- **fase b:** a UI (timeline, frame com as ligações por cima, painel do
  controle). pede gravar as ligações por frame no `rec`, que hoje não
  grava ([`estado-do-codigo-2026-09.md`](../historia/estado-do-codigo-2026-09.md), 2.4).
- **fase c:** export de trecho como bug report e sessão para teste com o
  harness isolado.

## Fases

- [fase a: modelo do inspector e CLI `plev-inspect`](task-64-session-inspector-and-tests/fase-a-modelo.md): o modelo sem interface, a cadeia causal de cada request, o controle no instante e a CLI, com o handoff e a integração (2026-09-23).
- [fase b: UI do inspector](task-64-session-inspector-and-tests/fase-b-ui.md): a sonda sobre onde a UI mora e o que ela lê, o roteiro para abri-la, a janela `plev-inspector` e o handoff (2026-09-25).
- [fase c: sessão para teste e export](task-64-session-inspector-and-tests/fase-c-sessao-para-teste.md): a sonda que reaplica os requests gravados, o harness `rerun`, o teste gerado da a2ppa, o `--export` e o fechamento da fase (2026-09-24).
- [fechamento da task](task-64-session-inspector-and-tests/fechamento.md): a conferência final do critério de aceite, item por item (2026-09-25).
