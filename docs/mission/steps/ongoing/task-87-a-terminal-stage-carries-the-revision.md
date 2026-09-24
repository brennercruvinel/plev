---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: agent-surface
task: task-87
depends: [task-61, task-83]
branch: task/87-stage-revision
origin: checkpoint a, segunda rodada (os três agentes pediram)
---

# task-87: um estágio terminal traz a revisão em que o app ficou

## por que

na segunda rodada do checkpoint a, os três agentes LLM disseram a mesma
coisa: o `completed` de um ato não traz a revisão nova, então para
encadear o próximo ato o agente precisa de uma observação só para saber
a revisão. é o maior custo de idas e voltas que sobrou, e idas e voltas
são onde o plev perde para o `git` direto (H4). um agente que age
`--from-revision` precisa de uma revisão servida; hoje a única fonte é o
`observe`.

## estado no código (main @ 5be83c5)

- `crates/wire/src/proto.rs:197-203`: `StageReply { request, stage, at,
  asked }`, sem revisão.
- `crates/wire/src/server.rs:225-261`: `Server::forward` manda os
  estágios que o sink ouviu, sem acesso ao `Host`; chamado em `:219` (fim
  do `handle`) e em `crates/wire/src/port.rs:296` (fim do `drain`).
- `crates/wire/src/ring.rs:52`: `Ring::serve(snapshot) -> Revision`
  guarda um snapshot servido; `since` e `from_revision` só aceitam
  revisão que esteja no anel.

## escopo

um estágio terminal (`completed`, `rejected`, `failed`) servido a um
agente leva `revision`: a revisão da projeção inteira do app no instante
em que o estágio sai, guardada no anel como toda revisão servida. aditivo
no protocolo v1. mais a frase do `--keys` no `plev --help` (ele corta só
mudanças de estado; outras mudanças passam), que o controle da segunda
rodada estranhou.

## fora do escopo

por que um controle está desabilitado; o que `constrained` restringe;
conexão persistente do `plev`.

## aceite

teste no wire: o `revision` de um `completed` é aceito por `observe
--since` e por `act --from-revision`; o laboratório roteirizado usa a
revisão do `completed` onde hoje observa só para saber a revisão, com as
mensagens antes e depois no arquivo da task; gate verde.

## medição (laboratório roteirizado)

`ide-lab --arm scripted`, binário de debug, cinco corridas por cenário
antes (base, `main @ f741246`) e depois (esta branch), todas com exit 0 e
verdict MATCH. mensagens e bytes são os do `Traffic` da porta (a linha
com o `\n`); as mensagens do cliente contam o `hello`.

| | none, antes | none, depois | unstage, antes | unstage, depois |
| --- | --- | --- | --- | --- |
| mensagens do cliente | 16 | 12 | 24 | 17 |
| mensagens do servidor | 34 | 30 | 46 | 39 |
| observações (snapshot / delta) | 9 / 0 | 5 / 0 | 14 / 1 | 7 / 1 |
| atos / estágios enviados | 6 / 24 | 6 / 24 | 8 / 30 | 8 / 30 |
| bytes do cliente | 1132 | 992 | 1601 | 1356 |
| bytes do servidor | 54453 a 54473 | 28782 a 28791 | 91502 a 91507 | 45874 a 45878 |
| recusa stale / medição válida / recuperou | 0 / não / não | 0 / não / não | 1 / sim / sim | 1 / sim / sim |

o cenário unstage é `unstage-after-observe:src/session.rs`. as cinco
corridas de cada linha deram os mesmos números, fora os bytes do
servidor, que variam na faixa escrita (os tempos dentro das mensagens).

o que saiu: a observação depois de cada ato de arquivo (três), a
observação depois do `open` e, no reparo, a observação depois de cada
arquivo (três). o que ficou, porque decide: a primeira observação (quais
arquivos entram e saem), a que lê se o formulário está aberto, a decisão
antes de cada submit (é nela que a pessoa do roteiro age), o delta depois
da recusa, a observação do índice que decide o reparo, a observação de
`stacks` que confirma o commit, e a do workspace inteiro depois dele (só
pelo peso, como antes). a linha de um estágio terminal ficou 30 bytes
maior (`,"revision":"<16 hex>"`); cada observação inteira que saiu pesava
uns 6,5 KB.

custo: a projeção do `forward` (projetar e servir no anel), medida com um
build instrumentado que não foi commitado, deu mediana de 158 us no
cenário none e 172 us no unstage, máximo de 306 us, em 70 chamadas (uma
por estágio terminal; nenhum `forward` mandou dois terminais juntos), em
debug. o limite da task era 2 ms. as chamadas a `project` do laboratório
foram de 15 para 17 (none) e de 31 para 28 (unstage), a uns 120 us cada.

a revisão engana? nas 25 vezes em que um ato foi seguido direto por uma
observação (o último ato de arquivo e a leitura do formulário, o
`set_value` e a decisão, o reparo e a nova decisão), a observação serviu
exatamente a revisão que o estágio terminal daquele ato trouxe: o ide
mostra o efeito de um ato quando ele termina (task-84), e a projeção do
`forward` vê isso. o que a revisão não traz, e uma observação no mesmo
instante também não traria, está no ADR
(`docs/ADR/a-terminal-stage-carries-the-revision.md`, consequências): o
`show_diff` termina antes das linhas chegarem, e um stage que falha no
git deixa o palpite da view até o refresh.

o relatório do laboratório rotula como `observed` o `form` dos atos que
se apoiam na revisão de um estágio (`FooFoF` no none, `FooFoFFF` no
unstage, onde F é `from_revision`): `story::acts` só conhece as
observações servidas. o teste do laboratório deixou de ler o `form` e
confere que cada ato se apoia numa revisão servida, por observação ou
por estágio (o script escreve "the app at <rev>" depois de cada ato).
