---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: semantic-contract
task: task-90
depends: [task-83, task-86]
origin: a prova da task-86 numa janela real (limitação lida no código e escrita no handoff)
---

# task-90: a recusa nomeia quem agiu desde que o agente olhou

## por que

a task-83 fez a recusa stale dizer quem agiu (`Rejection::Stale.acted`),
com uma regra exata e limitada: as ações de outros atores validadas
contra a mesma revisão que o autor viu e que chegaram a rodar
(`crates/sem/src/dispatch.rs`, `acted`). o ADR
a-delta-carries-the-node-a-refusal-names-who-acted escreve o limite: de
uma cadeia de ações de outra pessoa, só a primeira é nomeada.

a prova da task-86 na janela real mostrou que esse limite cai na
sequência mais comum de uma pessoa no ide. o agente olha (revisão R); a
pessoa clica na linha de um arquivo (seleção e `show_diff`, validados
contra R, a revisão muda para R'); a pessoa tira o arquivo do index
(validado contra R'). o `submit` do agente é recusado com o delta certo,
mas o `acted` nomeia a seleção e não o unstage, e o toast do ide repete
isso para a pessoa. a prova só nomeou o unstage porque a pessoa clicou
na linha antes de o agente decidir.

é a garantia da task-83 ("a recusa diz quem agiu") falhando no uso
normal: bug de validação real, visto numa janela, com a pessoa e o
agente ao mesmo tempo.

## direção (orquestrador; confirme no código)

o dispatcher numera os requests em ordem (`RequestId`, crescente). uma
ação de outro ator validada contra R rodou sobre o estado que o autor
viu, então rodou depois do olhar do autor; toda ação de outro ator que
rodou depois dela também. regra candidata: `acted` = as ações de outros
atores que rodaram, com id maior ou igual ao da primeira que rodou
validada contra a revisão do autor. sem nenhuma validada contra R (a
mudança veio de fora do app, um terminal), o `acted` fica como hoje. a
revisão continua não decidindo nada: `check` compara os `reads`, o
`acted` só nomeia.

o que ela não resolve, e continua escrito no ADR: mudança de fora do app
não tem receipt; um estado que volta exatamente a R pode nomear a mais;
nomear quem agiu não diz que a ação moveu o que o ato lê (o `changes`
diz).

## aceite

teste no `sem` e no ide (`crates/ide/tests/human_agent.rs`) da sequência
da janela (o agente olha, a pessoa seleciona, a pessoa tira do index, o
`submit` é recusado): o `acted` nomeia a seleção e o unstage, em ordem
de id; o texto do toast legível com mais de uma ação (diga o texto antes
e depois); os testes da task-83 verdes; o ADR com os limites
reescritos; gate verde.
