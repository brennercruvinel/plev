---
id: task-80
title: "task-80: prova de software qualitativamente diferente"
project: plev
type: tarefa
status: em-andamento
date: 2026-09-25
date_basis: cabecalho
topics: [superficies-geradas, agentes, gravacao, pesquisa]
relations: [adr-0050, adr-0070, task-60, task-62, task-63, task-64, task-68, task-69, task-70, task-84, task-86, task-87, task-89, task-92, task-93, task-94, task-96, task-97, tese-runtime]
evidence: [e44e4234, c8a6c3a9, f008c897, 46f272d3, b26dc988, ae5be30e, 677b1ad9]
source: docs/log/steps/task-80-proof-qualitatively-different-software.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-80: prova de software qualitativamente diferente

A task-80 reúne cinco demonstrações de que o modelo produz software qualitativamente diferente, e registra onde elas estão em 2026-09-25. A fase 1 foi entregue ao orquestrador nessa data, com a sonda do código, o método das cinco demos escrito antes de rodar, a demo 3 (um bug real da task-84 gravado sem janela, lido no instante pelo plev-inspect e transformado em teste de regressão) e a demo 4 (a interface efêmera medida pela gravação); a demo 5 foi feita pela task-96. A demo 1 e a demo 2 têm provas parciais, e a pergunta do núcleo e a pessoa de fora seguem abertas, por isso a task continua em andamento. Importa porque o próprio programa define que, se as demos não sustentarem a direção, ela é reavaliada com os dados, não defendida.

## objetivo

provar que o modelo produz software qualitativamente diferente, não só
mais rápido: interfaces efêmeras, handoff humano-agente, stale-state
rejection, inspectability e UI generativa, em aplicações reais, com
método, com números onde números cabem e com a pergunta do núcleo
respondida por quem usa.

## ligação com a tese

goal 10 do programa. tese seções 28 (não afirmar primeiro; a hipótese é o
contrato unificado; a frase "interfaces que humanos e agentes operam, e que
qualquer pessoa pode inspecionar depois"; a pergunta do núcleo), 33 (o que
o plev pode se tornar), e o experimento do arquivo de concepção.

## estado atual no código

depende das entregas: 62 (ide com rejeição e handoff, experimento com
métricas), 63 e 64 (sessão gravada e inspecionada, bug report, sessão para
teste), 69 (revisão de silêncios e comparação geradas), 70 (transformação
adaptativa e capacidade composta).

## escopo

cinco demonstrações, cada uma com roteiro, gravação (`.monster`) e
resultado escrito:

1. stale-state rejection: o experimento do ide completo (task-62, fase c),
   com as métricas (sucesso, tempo, dados e tokens para o agente,
   recuperação de erro, custo de gravação) e as comparações contra
   screenshot e contra a alternativa estruturada existente.
2. handoff humano-agente: o agente prepara um commit, o brenner assume,
   muda parte, devolve, o agente termina; o receipt de cada passo diz quem
   fez.
3. inspectability: um bug real (ou provocado) enviado como trecho de
   sessão, aberto no inspector no instante exato, com as perguntas da tese
   respondidas na tela, e virando teste de regressão que pega a quebra.
4. interface efêmera: a revisão de silêncios nasce da intenção, vira
   comparação, some; nunca existiu como tela.
5. UI generativa em aplicação real: além da demo de vídeo, uma intenção num
   app que já existe (no urnaui, "compara esses dois resultados de busca";
   ou no ide, "revisa esses três commits") gera uma superfície verificada
   que o usuário opera.

método:

- cada demo tem uma alternativa de controle (tela fixa bem feita; agente
  por screenshot; agente por snapshot de accessibility de terceiros) e o
  critério de comparação escrito antes.
- a pergunta do núcleo, com o brenner: ao abrir o primeiro `.monster` de uma
  delegação real, o que ele consegue descobrir, e isso basta para confiar
  na delegação? a resposta é escrita com as palavras dele.

critérios de parada, herdados do programa: se a rejeição e os receipts não
mudarem como a delegação é feita, ou se as superfícies geradas forem piores
que as fixas nas mesmas tarefas, a direção é reavaliada com os dados, não
defendida.

## fora de escopo

marketing, texto de blog (o destino blog passa pela skill de escrita, em
outra sessão), benchmarks isolados sem demonstração.

## dependências

62, 64, 69 e 70 integradas.

## superfície sob responsabilidade

roteiros e resultados em `docs/log/`, gravações de sessão (fora
do git se forem grandes), páginas Apps Demo do showcase para as demos que
couberem lá, ADR de resultado [`0070-generative-runtime-results.md`](../decisoes/geracao/0070-generative-runtime-results.md).

## interfaces que consome

tudo do programa, sem alterar.

## interfaces que pode alterar

nenhuma. defeito encontrado vira task nova ou volta para a task dona.

## critérios de aceitação

- as cinco demonstrações executadas, gravadas, com resultado escrito.
- a comparação de cada uma contra a alternativa de controle, com o
  critério definido antes.
- a pergunta do núcleo respondida.
- a decisão do checkpoint c registrada no programa.

## testes esperados

as demos que viram teste (a 3 obrigatoriamente) entram no gate.

## riscos e perguntas abertas

- viés de quem constrói avaliando o que construiu: sempre que possível,
  uma pessoa de fora usa as demos e o relato dela entra no resultado.

## commits esperados

um por demonstração com roteiro e resultado, `task-80:` no início, e o ADR
de resultado.

## pare e reporte se

- uma demo exigir funcionalidade que nenhuma task entregou (vira task, não
  atalho).

## Fases

1. [fase 1: sonda, método das cinco demos, demos 3 e 4 e handoff](task-80-proof-qualitatively-different-software/fase-1-sonda-e-demos.md): a sonda do código, o método escrito antes de rodar, o que já estava provado, as demos 3 e 4 e o handoff de 2026-09-25.
2. [demo 5: UI generativa em aplicação real](task-80-proof-qualitatively-different-software/fase-demo-5-ui-generativa.md): a superfície gerada no ide, feita pela task-96 e medida com o `code` do IR pela task-97 e pela fase b da task-96.
