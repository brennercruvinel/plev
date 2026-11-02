---
id: task-71
title: "task-71: plev as compiler target (importers to IR)"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [importing, ir, formats]
relations: [adr-0011, adr-0012, adr-0064, adr-0069, task-66, task-68, task-69, generative-runtime-thesis]
evidence: [0c08e77a, 9f7e9cfd, 14cd3dfd, 7a240577, 9707a2a6, e3201af7, 6e99cec8, 7ca7d41e, 0cb09a90, 4e16c349]
source: docs/log/steps/task-71-compiler-target.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-71: plev as compiler target (importers to IR)

Records the task that turned the parser into a front end for a compiler target: UI sources converge on plev IR, with a report that says what was supported, transformed, approximated or impossible. Closed on 2026-09-25, it delivered the second emitter (`ParserNode` to plev IR) with the report in four categories and `binding required`, the pixel comparison against the builder code, and two new sources, HTML/CSS (subset) and Figma, each with a fixture, goldens and a report. It matters because the runtime does not carry the source's execution model, and the conversion marks what does not pass instead of faking a silent equivalence.

## objetivo

o parser deixa de ser só um conversor de React para código builder e vira
front-end de um compiler target: fontes de UI (React, gpui hoje; HTML/CSS,
Figma, SwiftUI e Flutter depois) convergem para plev IR, com um relatório
honesto do que foi suportado, transformado, aproximado ou é impossível. o
runtime nunca carrega o modelo de execução da origem.

## ligação com a tese

seção 3 (os front-ends convergindo para plev IR), seção 14 (plev como
compiler target; suportado, transformado, aproximado, impossível; nunca
fingir equivalência silenciosa), e a nota das anotações: "Figma -> plev IR
<- React importer <- LLM -> live application". goal 9 do programa.

## estado atual no código

- `rust/import`: parse (tree-sitter: `tsx.rs`, `sass.rs`, `gpui.rs`),
  resolve (`resolve_react.rs` via `css_map.rs`, `resolve_gpui.rs`) para o
  `ParserNode` IR, emit (`emit.rs`) de código Rust contra
  `engine::builder`. o droplist reporta com arquivo:linha tudo que não
  passa (`rust/import/src/lib.rs`, ADR
  `transpiler-reports-every-unmapped-construct`). escopo brutalmente
  pequeno: um componente react (o card) e um widget gpui (o separador).
  marcado experimental.
- ADR `import-foreign-formats-by-conversion-not-embedding`: conversão, nunca
  runtime estrangeiro embutido.
- deps já presentes: `tree-sitter`, `tree-sitter-typescript`,
  `tree-sitter-rust`.

## escopo

1. segundo emissor: `ParserNode -> plev IR` (68), ao lado do emissor de
   código builder existente. o emissor de código continua até uma decisão
   explícita de removê-lo.
2. relatório em quatro categorias substituindo ou estendendo o droplist:
   `supported`, `transformed` (com a regra aplicada),
   `approximated` (com a diferença estimada, por exemplo a cor fora do
   token mais próximo), `impossible` (com o motivo). arquivo:linha em
   todas.
3. inferência de semântica honesta: tag e handler que indicam controle
   (button, `onClick`, input) viram componente com role e um
   `binding required` no relatório; o importer nunca inventa capacidade, só
   marca onde o app precisa ligar uma.
4. as duas fixtures atuais produzem IR que verifica contra um registro de
   capacidades de teste e materializa (69) com pixels próximos do código
   builder emitido hoje (comparação por headless, 66).
5. próximas fontes, cada uma com o mesmo formato de parse, resolve, emit:
   - HTML/CSS (subset; dependência nova de grammar justificada);
   - Figma (export JSON da API REST: frames e auto layout para
     `Stack`/`Grid`, estilos para tokens pelo token medido mais próximo, com
     `approximated` quando fica fora da tolerância);
   - SwiftUI e Flutter registrados como próximas, sem implementação nesta
     task.

## plano por fase

- fase 1 (aberta em 2026-09-25): itens 1 a 4 do escopo. o segundo
  emissor, o relatório em quatro categorias, o `binding required` e as
  fixtures em IR com a comparação de pixels. dependências autorizadas pelo
  orquestrador: `uir` no parser e, nos testes, `surface`, `sem` e `comps`,
  todos do workspace; nenhum pacote novo no lock.
- fase 2: a fonte nova (item 5). HTML/CSS pede grammar nova e o Figma
  pede um export real; as duas passam pelo brenner.

## fora de escopo

LLM como fonte (já é o proponente da 69); runtime de qualquer framework de
origem.

## dependências

68 integrada; 69 e 66 para a comparação de pixels.

## superfície sob responsabilidade

`rust/import/**`, proposta de ADR [`0069-importers-target-plev-ir.md`](../../adr/Importing/0069-importers-target-plev-ir/0069-importers-target-plev-ir.md).
dependência nova (grammar HTML, cliente do export Figma) só com
justificativa aprovada.

## interfaces que consome

o IR e o verifier (68), host de superfícies (69), headless (66).

## interfaces que pode alterar

`rust/import` inteiro; o formato do relatório é contrato com quem o lê
(documentado).

## critérios de aceitação

- as duas fixtures em IR verificado, com relatório em quatro categorias.
- comparação de pixels entre o IR materializado e o builder emitido, com a
  diferença explicada quando houver.
- uma fonte nova (HTML/CSS subset ou Figma) com fixture e relatório.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

goldens de IR e de relatório por fixture; teste de que nenhuma construção
some sem aparecer numa categoria; teste de `binding required`.

## riscos e perguntas abertas

- Figma exige token de API e arquivo real; a fixture usa um export salvo,
  nunca rede no teste.
- a tolerância de "aproximado" para cor e espaço precisa de número; vem da
  medição dos tokens existentes, não chute.

## commits esperados

1. `task-71: emissor de plev IR a partir do ParserNode`
2. `task-71: relatório em quatro categorias`
3. `task-71: inferência de semântica com binding required`
4. `task-71: fixtures em IR e comparação de pixels`
5. `task-71: importer de <fonte nova>`

## pare e reporte se

- o `ParserNode` não carregar informação suficiente para o IR sem mudar o
  parse;
- uma fonte nova exigir dependência pesada.

## Phases

1. [Phase 1: probe and handoff of the plev IR emitter](phase-1-ir-emitter.md): the probe against `ParserNode` and `uir`, the second emitter, the report in four categories, `binding required`, the fixtures and the pixel comparison.
2. [Phase 2: probe and handoff of the HTML/CSS and Figma sources](phase-2-html-and-figma.md): the probe of the two sources, the additive fields of `ParserNode`, the Figma export script and the report of each source.
3. [Orchestrator's closing](closing.md): the item-by-item closing of the acceptance criterion and the open item for broken text in the Figma IR.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0011: import foreign formats by conversion, never by embedding a foreign runtime](../../adr/Importing/0011-import-foreign-formats-by-conversion-not-embedding/0011-import-foreign-formats-by-conversion-not-embedding.md)
- [ADR 0012: the transpiler reports every unmapped construct, never drops silently](../../adr/Importing/0012-transpiler-reports-every-unmapped-construct/0012-transpiler-reports-every-unmapped-construct.md)
- [ADR 0049: the plev IR is typed, references only declared capabilities, and runs only once verified](../../adr/Synthesis/0049-plev-ir-typed-and-verified/0049-plev-ir-typed-and-verified.md)
- [ADR 0064: what identical means](../../adr/Workflows/0064-what-identical-means/0064-what-identical-means.md)
- [ADR 0069: importers target plev IR, with a report in four categories](../../adr/Importing/0069-importers-target-plev-ir/0069-importers-target-plev-ir.md)
- [what the autonomous cycle taught: importers target plev IR, with a report in four categories](../../adr/Importing/0069-importers-target-plev-ir/learned-importers-target-plev-ir.md)
- [task-66: headless as a first-class citizen](../task-66-headless-first-class/task-66-headless-first-class.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-69: ephemeral and generated surfaces](../task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-69, checkpoint b: second round](../task-69-ephemeral-generative-surfaces/checkpoint-b-round-2.md)
- [task-71 closure by the orchestrator](closing.md)
- [task-71 phase 1: probe and handoff of the plev IR emitter](phase-1-ir-emitter.md)
- [task-71 phase 2: probe and handoff of the HTML/CSS and Figma sources](phase-2-html-and-figma.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
