---
id: task-70
title: "task-70: adaptive transformations"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [generated-surfaces, ir, semantics, accessibility, agents]
relations: [task-63, task-68, task-69, task-58, task-75, generative-runtime-thesis, adr-0067]
evidence: [ec2ab5db, 5d1ca12f, f74ce129, 8124e8a7]
source: docs/log/steps/task-70-adaptive-transformations.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-70: adaptive transformations

Task-70 makes the interface change without being rebuilt from scratch: a safe and verified transformation that preserves the identity of the elements and, from the history of receipts, composed capabilities when a sequence repeats. The work was merged in four phases (operations on the IR, transformation proposer, composed capabilities and accessibility as a transformation), with the green gate in each one. The proof with the real Claude from phase 2 remains with the orchestrator, waiting for spending to be released.

## objetivo

a interface muda sem ser reconstruída do zero: a partir da interface
atual, do estado, do objetivo e do contexto, o sistema aplica uma
transformação segura e verificada que preserva identidade. e, a partir do
histórico de receipts, o sistema oferece capacidades compostas quando
percebe uma sequência que o usuário repete.

## ligação com a tese

seção 16 inteira (interface adaptativa, não apenas generativa: "tem
informação demais aqui", "não consigo comparar esses números", "tenho
dificuldade para enxergar esses números", a sequência review, approve,
publish virando "[aprovar e publicar]"; adaptação contextual real, não
responsive design). seção 4 (o espectro: parametrizada, adaptativa,
recomposta).

## estado atual no código

- depois de 68 e 69: `VerifiedIr`, host de superfícies com reconciliação
  por `SemId`.
- depois de 63: histórico de receipts gravado.
- telas fixas (showcase, ide) não são IR; para serem adaptadas, precisam
  ser expressas em IR (a fixture do formulário de commit da task-68 prova
  que dá) ou receber transformações no nível de projeção. a escolha é
  parte desta task.

## escopo

### operações de transformação

sobre um `VerifiedIr`: `Hide(SemId)`, `Show(SemId)`, `Group(ids, into)`,
`Reorder(parent, ids)`, `Emphasize(SemId)`, `Density(level)` dentro dos
tokens, `Restyle(SemId, token role)`, `SwapView(SemId, component)` quando
os dados ligados são compatíveis (tabela para gráfico de barras),
`Promote(capability)`.

toda transformação:

- produz IR que passa no verifier de novo;
- preserva identidade: nenhum `SemId` some sem um `Hide` explícito, e o
  estado dos que continuam é mantido pela reconciliação da task-69;
- é reversível (a transformação inversa é gerada junto) e registrada.

### proponentes de transformação

- pedidos do usuário ("tem informação demais", "não consigo comparar",
  "números maiores") viram transformações pelo mesmo `Proposer` da task-69
  (saída em transformações em vez de IR inteiro).
- preferências de accessibility (escala de fonte, contraste, densidade,
  reduced motion) como transformações dentro dos tokens.

### capacidades compostas

- detector sobre o histórico de receipts (63): a mesma sequência de ações,
  do mesmo ator, na mesma ordem, repetida N vezes.
- proposta de composta (`review -> approve -> publish` vira "aprovar e
  publicar"); instalar a composta exige aceite humano.
- executar a composta é uma sequência de requests encadeados: cada passo
  confere as próprias precondições contra o estado depois do passo
  anterior; o primeiro rejeitado para a sequência e o receipt da composta
  diz em qual passo e por quê. não há promessa de rollback, a menos que as
  capacidades ofereçam desfazer (declarado no ADR).

## fora de escopo

aprendizado de preferência entre sessões; transformação de tela fixa que
não passa por IR, se a análise da task concluir que não cabe.

## dependências

68, 69 e 63 integradas.

## superfície sob responsabilidade

módulo de transformação junto do IR (68) ou do host (69), conforme o ADR;
o detector de sequências junto do leitor de sessão (63); proposta de ADR
[`0067-adaptive-transforms-preserve-identity.md`](../../adr/generated/0067-adaptive-transforms-preserve-identity.md).

## interfaces que consome

`VerifiedIr`, `verify` (68), host e reconciliação (69), sessão e receipts
(63), `Dispatcher` (58).

## interfaces que pode alterar

as próprias. mudança no IR ou no host passa pelo orquestrador.

## critérios de aceitação

- testes: cada operação preserva identidade e passa no verifier; a inversa
  volta ao IR original.
- demo: na revisão de silêncios, "tem informação demais" reduz densidade e
  agrupa; "não consigo comparar" troca a lista por gráfico; o estado do que
  continua fica.
- composta: sequência repetida detectada, proposta, aceita, executada; um
  passo que envelhece no meio para a sequência com o motivo.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

unitários por operação, propriedade de preservação de identidade
(proptest), detector de sequência com histórico sintético, execução
encadeada com rejeição no meio.

## riscos e perguntas abertas

- interface que muda sozinha atrapalha memória muscular. transformação só
  por pedido ou com aceite; nunca automática sem o usuário ver.

## commits esperados

1. `task-70: operações de transformação com verificação e inversa`
2. `task-70: proponente de transformação`
3. `task-70: preferências de accessibility como transformação`
4. `task-70: detector de sequência sobre receipts`
5. `task-70: capacidade composta encadeada`

## pare e reporte se

- telas fixas não couberem sem virar IR;
- a composta exigir transação que as capacidades não oferecem.

## Phases

1. [phase 1: the operations, the inverse, the verification and the host](phase-1-operations.md): the nine operations on the IR, each re-verified and with an inverse, and the host with history and undo.
2. [phase 2: the transformation proposer](phase-2-proposer.md): an intent becomes operations through the rules or through Claude, with per-surface policy and emphasis on stat and badge.
3. [phase 3: the composed capabilities](phase-3-composite-capabilities.md): sequence detector over the receipts, acceptance by the person, and a chain that stops at the first refused step.
4. [phase 4: accessibility as a transformation](phase-4-accessibility.md): the person's preferences as token operations, which can be switched off, and the `promote` of a template into the `uir`.
5. [wrap-up](closing.md): the four phases integrated, the green gate and the open items that carry on in the program.
