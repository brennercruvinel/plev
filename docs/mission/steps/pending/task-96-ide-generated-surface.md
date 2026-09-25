---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-25
domain: generative-runtime
task: task-96
wave: 8
depends: [task-68, task-69, task-70, task-63, task-64]
branch: task/96-ide-surface
---

# task-96: uma superfície gerada no ide

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

- `crates/surface` (`SurfaceHost`, `Surface::route`) e `crates/propose`
  (regras e `Llm`) servem a Silences e a página Surface do showcase.
- o `uir` já verifica IR no domínio do ide contra o registro que a
  projeção do ide dá: `crates/uir/fixtures/ide-review-commit.ir.json`,
  `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por
  `crates/ide/tests/uir_fixture.rs`.
- `crates/ide/Cargo.toml` não depende de `surface`, `propose` nem `uir`.

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

`crates/ide/src/**` (a janela e a projeção), `crates/ide/Cargo.toml`
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
- `./script/gate` verde.

## o que a derruba

a superfície pior que o painel fixo para revisar três commits; um ato
fora do dispatcher; a superfície deixando rastro.

## pare e reporte se

- o ide precisar de dependência externa nova;
- a projeção do ide não der o registro que a superfície precisa sem
  mudar tipo do `sem`.
