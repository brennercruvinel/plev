---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-25
domain: generative-runtime
task: task-97
wave: 8
depends: [task-68, task-69, task-96]
branch: task/97-code-in-ir
---

# task-97: código e diff no vocabulário do IR

## objetivo

a demo 5 da task-80 (a revisão de três commits no ide, task-96) ficou
pior que o painel fixo no diff: a superfície gerada mostra o diff numa
tabela, cinco linhas por vez e sem cor, contra o diff inteiro do painel.
o motivo não é do ide: o vocabulário fechado do IR (`uir::Component`,
`crates/uir/src/vocab.rs`) não tem um componente para código nem para
diff. esta task põe um, e a demo 5 é medida de novo pelo mesmo critério
escrito antes.

## ligação com a tese

seção 28 (a direção é reavaliada com os dados, não defendida: o dado
disse onde a superfície perde); a demo 5 da task-80.

## estado atual no código

- `comps::content::CodeBlock` (`crates/comps/src/content/code_block.rs`)
  desenha texto no passo mono da rampa, com legenda; não sabe o que é
  uma linha adicionada ou removida.
- o painel de diff do ide desenha o próprio diff
  (`crates/ide/src/views/diff_view.rs`).
- a revisão das regras (`crates/propose/src/review.rs`) liga o diff a
  uma `table`.

## escopo

1. um componente `code` no vocabulário do IR, ligado a um estado de
   texto (ou lista de linhas) do app, com a legenda e, quando o app diz,
   o tipo de cada linha (contexto, adicionada, removida) desenhado pelos
   tokens do tema; o verificador e o schema do prompt o conhecem.
2. o materializador desenha pelo `CodeBlock` do `comps`, estendido de
   forma aditiva para o tipo de linha, com rolagem quando o diff é maior
   que a área.
3. as regras da revisão ligam o diff ao `code` em vez da `table`.
4. a demo 5 medida de novo pelo laboratório, sem janela, pelo mesmo
   critério da task-80; o resultado escrito lá, ao lado do primeiro.

## fora de escopo

realce de sintaxe por linguagem; edição; o urnaui.

## dependências

68, 69 e 96 integradas.

## superfície sob responsabilidade

`crates/uir/src/**` (vocabulário, verificador, schema), `crates/surface`
(materializar e desenhar), `crates/comps/src/content/code_block.rs`
(aditivo), `crates/propose/src/review.rs`, o laboratório do ide para a
demo, testes.

## critérios de aceitação

- o `code` verifica, materializa e desenha um diff com o tipo de cada
  linha pelos tokens, sem cor fora do tema.
- a demo 5 medida de novo, e o resultado escrito ao lado do primeiro,
  qualquer que seja.
- os baselines existentes iguais; `./script/gate` verde.

## pare e reporte se

- o componente pedir mudar tipo do `sem` ou o protocolo v1;
- desenhar o tipo de linha pedir cor que o tema não tem (é decisão de
  design do brenner).
