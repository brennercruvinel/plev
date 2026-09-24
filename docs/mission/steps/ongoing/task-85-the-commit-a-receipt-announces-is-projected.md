---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: ide
task: task-85
depends: [task-84]
branch: task/85-commit-projected
origin: checkpoint a (agentes LLM) e validação da task-84
---

# task-85: o commit que o receipt anuncia já está na projeção, com os arquivos

## por que

duas coisas que agentes reais esbarraram, no mesmo nó:

- o nó `stacks.commit[sha]` não expõe os arquivos do commit como estado
  (apontado pelos agentes LLM do checkpoint a, que confirmaram o próprio
  commit por `show_diff` ou pelo receipt);
- o receipt do `submit` chega a `completed { commit, files }` antes de a
  projeção mostrar o commit: `changes:staged` ainda lista os arquivos
  commitados e `stacks` ainda não tem o commit até o refresh chegar
  (achado pelo agente da task-84: nas dez corridas do roteirizado, a
  observação logo depois do submit concluído viu só o commit inicial). um
  agente que confere o próprio commit na hora não o encontra; um que
  decide em cima dessa observação decide sobre um estado que contradiz o
  receipt que ele acabou de receber.

## invariante

depois que o receipt de um commit chega a `completed`, a próxima
observação não contradiz o receipt: os arquivos commitados não estão em
`changes:staged`, e `stacks.commit[<sha>]` existe com `files` igual ao
`files` do receipt. o refresh que vem depois reconcilia com o git sem
desfazer isso (regra da task-84).

## escopo

1. `files` como estado de todo `stacks.commit[sha]`: os caminhos que o
   commit muda contra o primeiro pai (commit raiz: contra a árvore vazia),
   lidos pelo worker junto com o log, com cache por sha (commit é imutável).
2. ao aplicar `Committed(Ok(NewCommit { sha, files }))`, a view mostra o
   commit na hora (palpite otimista, como as outras mutações), no mesmo
   passo em que o receipt conclui.

## aceite

teste de git (log com arquivos: commit normal, raiz, merge pelo primeiro
pai), teste do ide para a invariante sem nenhum refresh aplicado entre o
receipt e a observação, custo do log medido antes e depois, laboratório
roteirizado confirmando o commit pelo `files` do nó, gate verde.
