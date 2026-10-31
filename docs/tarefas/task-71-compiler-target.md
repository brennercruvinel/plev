---
id: task-71
title: "task-71: plev como compiler target (importers para IR)"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [importacao, ir, formatos]
relations: [adr-0011, adr-0012, adr-0064, adr-0069, task-66, task-68, task-69, tese-runtime]
evidence: [0c08e77a, 9f7e9cfd, 14cd3dfd, 7a240577, 9707a2a6, e3201af7, 6e99cec8, 7ca7d41e, 0cb09a90, 4e16c349]
source: docs/log/steps/task-71-compiler-target.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-71: plev como compiler target (importers para IR)

Registra a task que transformou o parser num front-end de compiler target: fontes de UI convergem para plev IR, com um relatório que diz o que foi suportado, transformado, aproximado ou é impossível. Fechada em 2026-09-25, entregou o segundo emissor (`ParserNode` para plev IR) com o relatório em quatro categorias e `binding required`, a comparação de pixels com o código builder e duas fontes novas, HTML/CSS (subset) e Figma, cada uma com fixture, goldens e relatório. Importa porque o runtime não carrega o modelo de execução da origem, e a conversão marca o que não passa em vez de fingir equivalência silenciosa.

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

`rust/import/**`, proposta de ADR [`0069-importers-target-plev-ir.md`](../decisoes/importacao/0069-importers-target-plev-ir.md).
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

## Fases

1. [Fase 1: sonda e handoff do emissor de plev IR](task-71-compiler-target/fase-1-emissor-ir.md): a sonda contra o `ParserNode` e o `uir`, o segundo emissor, o relatório em quatro categorias, `binding required`, as fixtures e a comparação de pixels.
2. [Fase 2: sonda e handoff das fontes HTML/CSS e Figma](task-71-compiler-target/fase-2-html-e-figma.md): a sonda das duas fontes, os campos aditivos do `ParserNode`, o script do export do Figma e o relatório de cada fonte.
3. [Fechamento do orquestrador](task-71-compiler-target/fase-fechamento.md): o fechamento item por item do critério de aceite e a pendência do texto quebrado no IR do Figma.
