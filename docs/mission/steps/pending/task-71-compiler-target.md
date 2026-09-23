---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-71
wave: 7
depends: [task-68]
branch: task/71-compiler-target
---

# task-71: plev como compiler target (importers para IR)

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

- `crates/parser`: parse (tree-sitter: `tsx.rs`, `sass.rs`, `gpui.rs`),
  resolve (`resolve_react.rs` via `css_map.rs`, `resolve_gpui.rs`) para o
  `ParserNode` IR, emit (`emit.rs`) de código Rust contra
  `engine::builder`. o droplist reporta com arquivo:linha tudo que não
  passa (`crates/parser/src/lib.rs`, ADR
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

## fora de escopo

LLM como fonte (já é o proponente da 69); runtime de qualquer framework de
origem.

## dependências

68 integrada; 69 e 66 para a comparação de pixels.

## superfície sob responsabilidade

`crates/parser/**`, proposta de ADR `docs/ADR/importers-target-plev-ir.md`.
dependência nova (grammar HTML, cliente do export Figma) só com
justificativa aprovada.

## interfaces que consome

o IR e o verifier (68), host de superfícies (69), headless (66).

## interfaces que pode alterar

`crates/parser` inteiro; o formato do relatório é contrato com quem o lê
(documentado).

## critérios de aceitação

- as duas fixtures em IR verificado, com relatório em quatro categorias.
- comparação de pixels entre o IR materializado e o builder emitido, com a
  diferença explicada quando houver.
- uma fonte nova (HTML/CSS subset ou Figma) com fixture e relatório.
- `./script/gate` verde.

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
