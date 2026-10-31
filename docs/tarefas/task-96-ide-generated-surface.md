---
id: task-96
title: "task-96: uma superfície gerada no ide"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [superficies-geradas, ir, agentes, llm]
relations: [task-63, task-64, task-68, task-69, task-70, task-80, task-85, task-93, task-97]
evidence: [73a318a7, 6bacaa3d, f53667c0, 73704549, f8f39298, e315e11e, 2e6c9e9a, 07650665, 9dadd352, 855b1a4d, 3df1297f, 205e90b3, 1283cc5b, e71222cf, 3959d958, 4be4bb2f, e42bd571, a427d2b8]
source: docs/log/steps/task-96-ide-generated-surface.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-96: uma superfície gerada no ide

A task põe uma superfície gerada (`SurfaceHost`) na janela do ide, ao lado das vistas fixas, para revisar três commits. A intenção "review these three commits" gera pelas regras uma superfície verificada, que a pessoa opera, e cada ato passa pelo dispatcher único do workbench. O resultado é a demo 5 da task-80 em aplicação real, medida contra o painel fixo: a superfície mostra melhor os arquivos e pior o diff. Por que importa: é o teste da hipótese do contrato unificado numa aplicação que já existe, não só na demo de vídeo. A fase b corrige o título do cartão do diff e a coluna da linha de intenção; a prova com o Claude de verdade e a demo 5 com janela ficam com o orquestrador.

## objetivo

a demo 5 da task-80 (UI generativa numa aplicação real) pede uma
intenção num app que já existe gerando uma superfície verificada que a
pessoa opera. hoje as superfícies geradas vivem só nas páginas Surface e
Silences do showcase: nem o ide nem o urnaui têm `SurfaceHost` (sonda da
task-80 fase 1, item (a)). esta task põe uma no ide.

## ligação com a tese

seção 28 (a hipótese é o contrato unificado, provada em aplicação real,
não só na demo de vídeo); goal 10 do programa; a demo 5 da task-80.

## estado atual no código

- `rust/screen` (`SurfaceHost`, `Surface::route`) e `rust/offers`
  (regras e `Llm`) servem a Silences e a página Surface do showcase.
- o `uir` já verifica IR no domínio do ide contra o registro que a
  projeção do ide dá: `rust/plevir/tests/fixtures/idestudio-review-commit.ir.json`,
  `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por
  `apps/idestudio/tests/plevir_fixture.rs`.
- `apps/idestudio/Cargo.toml` não depende de `surface`, `propose` nem `uir`.

## escopo

1. um `SurfaceHost` na janela do ide ao lado das vistas fixas; a linha de
   intenção (campo, proponente, propor, fechar) projetada como na
   Silences.
2. o registro de capacidades tirado do `Workbench::snapshot`; os atos da
   superfície roteados ao dispatcher único do workbench
   (`Surface::route`), com os receipts e a gravação do ide como estão.
3. as regras propondo "review these commits" sobre `stacks`; o Claude
   pelo `propose::Llm` fora da thread do app (a prova com o Claude de
   verdade é do orquestrador).
4. o laboratório grava a demo 5 sem janela, pelo roteiro e o critério
   escritos na task-80.

## fora de escopo

o urnaui; transformar as vistas fixas do ide (elas se exprimem em IR,
pela decisão da task-70 fase 1); qualquer chamada real ao Claude por um
agente.

## dependências

68, 69, 70, 63d e 64 integradas.

## superfície sob responsabilidade

`apps/idestudio/src/**` (a janela e a projeção), `apps/idestudio/Cargo.toml`
(três dependências de workspace: `surface`, `propose`, `uir`; nenhuma
externa), testes do ide, e a demo 5 no arquivo da task-80.

## critérios de aceitação

- "review these three commits" gera, pelas regras, uma superfície
  verificada que mostra os três commits com os arquivos e o diff de cada
  um, e que a pessoa opera.
- cada ato da superfície passa pelo dispatcher do workbench, com o
  receipt de quem pediu; nada da superfície fica depois de fechar.
- a demo 5 medida contra o painel de stacks fixo mais o diff, pelo
  critério escrito na task-80.
- `./tool/tasks/fullcheck.sh` verde.

## o que a derruba

a superfície pior que o painel fixo para revisar três commits; um ato
fora do dispatcher; a superfície deixando rastro.

## pare e reporte se

- o ide precisar de dependência externa nova;
- a projeção do ide não der o registro que a superfície precisa sem
  mudar tipo do `sem`.

## Fases

1. [fase a (2026-09-25): a superfície gerada no ide, com sonda, handoff e fechamento](task-96-ide-generated-surface/fase-a-superficie.md): a sonda de leitura antes do código, o handoff com os commits da branch `task/96a-ide-surface` e o fechamento que dá a fase como integrada.
2. [fase b (2026-09-25): o título do cartão do diff e a coluna só com superfície](task-96-ide-generated-surface/fase-b-titulo-coluna.md): a sonda da correção, o cartão do diff com o rótulo "diff", a linha numa faixa embaixo das vistas fixas, a demo 5 gravada de novo e o handoff da fase.
