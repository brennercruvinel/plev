---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-24
domain: semantic-contract
task: task-91
depends: [task-89, task-90]
branch: task/91-the-look
origin: a sonda da task-90 (a seleção por clique muda a revisão fora do dispatcher)
---

# task-91: o dispatcher sabe quando o agente olhou

## por que

a recusa stale nomeia quem agiu (`acted`, task-83; ampliado na task-90
para a cadeia que começa numa ação validada na revisão que o autor viu).
as duas regras ancoram no dispatcher, e o dispatcher só vê o que passa
por ele. a sonda da task-90 mostrou a cena mais comum da janela sem
nenhuma âncora: a pessoa clica numa linha nova (a seleção muda
`changes:selected` direto na view, sem receipt, e a revisão muda), depois
o `show_diff` e o `unstage` são validados em revisões seguintes. o
`acted` sai vazio, e o toast não diz quem tirou o arquivo do index.

a seleção não é a única mudança projetada fora do dispatcher no ide: a
digitação na mensagem de commit, o foco, o toast saindo pelo timer e as
linhas do diff chegando do worker também mudam a revisão sem receipt.

## direção

o dispatcher passa a saber do olhar: quem serve uma observação avisa o
dispatcher (ator, revisão, o último id despachado naquele instante), de
forma limitada. o `acted` de uma recusa passa a ser as ações de outros
atores despachadas depois do olhar em que o ato recusado se apoiou, com
a regra da task-90 como reserva quando não há olhar registrado (quem
observa pelo `Request::observe` no mesmo processo). com a task-89, o
servidor já sabe, por ator, de qual observação veio o valor de cada
`read`; a âncora é a mais antiga delas.

superfícies: API nova no `sem` (aditiva), o `wire` (servidor) e os
`Host` (`crates/ide/src/agent.rs`, o do showcase, o do laboratório). o
formato das mensagens não muda.

## aceite

a cena da janela pelo clique real (clicar numa linha nova, depois tirar
do index) nomeia o unstage; os testes das tasks 83 e 90 verdes; o ADR
com a regra e os limites que sobram (mudança de fora do app); gate
verde.
