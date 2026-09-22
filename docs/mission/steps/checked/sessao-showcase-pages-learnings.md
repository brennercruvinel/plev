---
type: reference
tags: [learnings, showcase, comps, sidebar, refactor, session]
date: 2026-09-21
status: reference
---

# sessão showcase pages: o que funcionou e o que mordeu

registro da sessão que virou o showcase em uma página por componente
(adr showcase-one-page-per-component, task-55). escrito pra quem for
fazer a próxima reorganização grande, humano ou agente.

## o caminho que funcionou

1. **proposta antes de código, com a tabela item a item.** meia hora
   lendo `view/`, catalog.md e arc.toml deu uma tabela: cada item da
   lista nova classificado em "move" (o exemplo existe), "parcial" (o
   widget existe, o exemplo é o do vizinho) e "novo" (nada). a tabela
   virou o md temporário, o md virou o checklist, o checklist virou o
   PR. as quatro perguntas que restaram (placeholder sim ou não, os
   itens que faltavam na lista, grupos ou lista plana, tocar no comps)
   foram feitas de uma vez, antes de qualquer edição.
2. **página é dado, não enum.** `Section` era um enum posicional com
   `ALL`, `[ScrollState; N]` e um match em cada verbo. com 79 páginas
   isso vira `PAGES: &[Page]` (título, ícone, blurb, corpo) e um único
   arquivo que faz match em `Pane` (`dispatch.rs`). testes da tabela
   (ordem alfabética, título único, alias aponta pra página que existe)
   pegam erro de dado sem rodar o app.
3. **alias com dívida declarada em vez de página duplicada.** "Field"
   mostra o exemplo de "Input" com uma linha em cima dizendo o que falta.
   o estado é o mesmo struct, então digitar num aparece no outro: é o
   mesmo exemplo, não uma cópia. custa zero e mantém a ordem de migração
   visível na sidebar.
4. **split por verbo, não por widget, quando o arquivo é o roteador.**
   `view/mod.rs` crescia sem parar por causa dos 45 campos e do match
   de altura. `panes.rs` (o estado) e `dispatch.rs` (os quatro matches)
   resolveram; `input.rs` e `render.rs` ficaram com o que é deles. o
   mesmo molde serviu pro `Sidebar` do comps (mod / input / render).
5. **teste que conta nós dentro do clip, não no layer inteiro.** o
   `nodes > 10` antigo reprovava um placeholder legítimo (8 nós) e não
   distinguia página de chrome. contar o que está entre o `PushClip` do
   conteúdo e o `PopClip` dele responde a pergunta certa: a página
   desenhou alguma coisa?
6. **sidebar tall test com o grupo dobrado.** 79 links não cabem em
   1100 px. o teste "tall: nada rola" passou a dobrar Components antes,
   que é exatamente o que o usuário faz.
7. **`git mv` em lote com `git rm --cached` + `git add` quando o
   sistema de arquivos não distingue maiúscula.** `docs/adr` -> `docs/ADR`
   no macOS: `git mv` recusa ("bad source"), o disco já estava com o
   nome novo. `git rm -r --cached docs/adr && git add docs/ADR` registra
   os 59 renames sem tocar em nada.

## o que mordeu

- **"continua tudo igual".** o usuário rodou o app em outra branch e viu
  a versão antiga. antes de investigar código, perguntar o comando e a
  branch: `git branch --show-current`, `ls -la target/debug/<bin>`.
- **zsh e `for f in $files`.** a variável com nomes separados por linha
  não faz split no zsh: o sed recebeu um único "arquivo" gigante ("File
  name too long"). python ou `while read` pra lote de arquivos, sempre.
- **script de split que aborta no meio deixa os arquivos pela metade.**
  o python que extraía os matches escreveu mod/input/render antes de um
  `assert` falhar, e o `dispatch.rs` não existiu. em refactor de extrair
  e mover, escrever o arquivo novo primeiro e só depois apagar dos
  antigos; ou fazer o assert antes de qualquer `write`.
- **`Group::len` sem `is_empty`.** clippy `len_without_is_empty` pega
  qualquer método público chamado `len`. nome que diz o que conta
  (`pages()`) resolve melhor que um `is_empty` de fachada.
- **typos no root de docs/.** a exclusão cobria `docs/adr/`, `docs/how-to/`
  etc., mas `docs/catalog.md` e o changelog que subiu pra `docs/` são
  português e ficaram descobertos. a CI é advisory, mas o ruído esconde
  erro real.
- **`TaskCreate` não existe em toda sessão.** quando a tool não está
  carregada, um md com checkbox na raiz cumpre o papel e o usuário lê
  melhor. apagar antes do commit.

## fica pra próxima

- busca na sidebar (a árvore aberta passa da tela)
- as páginas parciais em ordem: Field (label + helper + erro), Button
  Group, Table estática, Toggle e Toggle Group, Dialog x Alert Dialog,
  Popover e Hover Card, Combobox, Item, Kbd, Label no comps
- `view/pages/table.rs` é dado (492 linhas): se crescer, um arquivo por
  grupo
