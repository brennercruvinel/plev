---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: comps
task: task-81
depends: [task-59]
branch: task/81-select-scroll
---

# task-81: painel do `Select` com rolagem

## objetivo

um `Select` com mais opções do que cabem em `size.dropdown_max_h` hoje só
desenha as que cabem; as outras não são alcançáveis pelo ponteiro (a
task-59 fase b corrigiu o hit test que pegava uma opção invisível, e
deixou a rolagem para esta task). o painel ganha rolagem, pelo
`ScrollState` que o resto do comps já usa, e a opção selecionada fica
visível ao abrir.

## escopo

`crates/comps/src/form/select.rs` e os testes dele. o desenho, o hit test
e a ligação leem a mesma janela de linhas (`drawn_rows` passa a considerar
o offset); roda do mouse e teclado (setas) rolam; as opções fora da janela
continuam projetadas como filhos enquanto o painel está aberto, mas só as
desenhadas são ligadas.

## aceite

teste com 20 opções: rolar até a última, clicar, ela é selecionada;
`hit` e o clique concordam em toda linha desenhada depois da rolagem;
matriz do comps verde; gate verde.
