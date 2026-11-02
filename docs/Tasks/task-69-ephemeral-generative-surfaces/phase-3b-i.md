---
id: task-69/phase-3b-i
title: "task-69, phase 3b-i: Claude as the proposer's model"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [llm, agents, process]
relations: [adr-0052]
evidence: [b771ead, 31ff396, 524332f]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 3b-i: Claude as the proposer's model

Records phase 3b-i: Claude as `propose::Model`, called by `claude -p` as a process, in the `claude-model` crate, with no tools, no saved session and no user settings. The result is the real calls, measured in time and cost: the four runs of the example passed the verifier on the first try, and the proposer stayed out of the IR and the engine.


branch `task/69d-claude-model`, worktree próprio, sobre a `main` @ ce9acee.
a decisão é a 13 do programa: o modelo é o Claude, pelo `claude -p` do
Claude Code desta máquina (`/Users/nn/.local/bin/claude`, 2.1.281, logado
na conta do brenner), atrás de `propose::Model`, num crate próprio.

## sonda da fase 3b-i (antes do primeiro tipo, 2026-09-24)

um exemplo descartável (`demo/intent/src/probe.rs`, rodado e
jogado fora, nunca commitado) monta o `propose::prompt` para a intenção
"only what is active and the add field" sobre o golden do todo
(`rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`), com a superfície
compacta da página como a atual (`showcase-todo.ir.json`, id
`todo_panel`), e escreve o prompt num arquivo: 17.704 bytes. o mesmo
exemplo, com `check <arquivo>`, passa uma resposta por
`propose_verified(Llm::with_attempts(<a resposta>, 1), ask)`.

três chamadas reais, todas de um diretório vazio (`probe-cwd`, no
scratchpad), sem as variáveis que a sessão do Claude Code que rodava a
sonda põe no ambiente dos processos dela (`CLAUDECODE`,
`CLAUDE_CODE_ENTRYPOINT`, `CLAUDE_CODE_SESSION_ID`,
`CLAUDE_CODE_CHILD_SESSION`, `CLAUDE_CODE_SESSION_ATTENDED`,
`CLAUDE_CODE_MESSAGING_SOCKET`, `CLAUDE_CODE_MESSAGING_TOKEN`,
`CLAUDE_CODE_EXECPATH`, `CLAUDE_PID`, `CLAUDE_EFFORT`, `AI_AGENT`). elas
são da sessão, não do app, e uma delas, `CLAUDE_EFFORT=xhigh`, mudaria o
esforço do filho. o `env -u ...` abaixo é abreviado; tira as onze.

a primeira, com o system prompt padrão do Claude Code e a saída em
stream-json para ver o `init` (ferramentas, MCP, skills) e cada evento:

```sh
env -u CLAUDECODE -u ... -u AI_AGENT \
  claude -p --model sonnet --output-format stream-json --verbose \
    --tools "" --no-session-persistence --setting-sources "" \
    --strict-mcp-config --disable-slash-commands --permission-prompts none \
    --max-budget-usd 0.50 --debug-file probe-1.debug \
  < probe-prompt.txt > probe-1.out
```

a segunda, com as flags que o crate usa (a saída em json, um objeto só, e
um system prompt curto no lugar do do Claude Code):

```sh
env -u CLAUDECODE -u ... -u AI_AGENT \
  claude -p --model sonnet --output-format json \
    --tools "" --no-session-persistence --setting-sources "" \
    --strict-mcp-config --disable-slash-commands --permission-prompts none \
    --max-budget-usd 0.50 \
    --system-prompt "you are called by a program, not by a person. its message is the whole task: answer exactly what it asks for, in the form it asks for, with nothing before or after it." \
    --debug-file probe-2.debug \
  < probe-prompt.txt > probe-2.out
```

| | 1 (system prompt do Claude Code) | 2 (as flags do crate) |
| --- | --- | --- |
| tempo de parede | 40,1 s (API 39,3 s, primeiro byte em 2,0 s) | 24,5 s (API 23,9 s) |
| saída do processo | 25.470 bytes de eventos | 3.145 bytes, um objeto json |
| a resposta (`result`) | 1.324 caracteres | 1.322 caracteres |
| tokens de entrada | 11.150 | 6.983 |
| tokens de saída (de pensamento) | 3.667 (3.063) | 2.628 (2.026) |
| custo que o `claude` mostra | 0,069 USD | 0,054 USD |
| exit code, stderr | 0, vazio | 0, vazio |
| `propose_verified` de primeira | passa | passa |

o custo é o de lista que o `claude` calcula (`costBasis: list`); a conta
é de assinatura (`apiKeySource: none`, login OAuth do keychain), então é
uma medida de tamanho, não uma cobrança. a diferença de tempo entre as
duas é mais pensamento do que prompt (mil tokens de pensamento a menos na
segunda); com uma chamada de cada não dá para separar. o system prompt
próprio tira uns 4.200 tokens de entrada (o prompt do Claude Code, que
fala de ferramentas que o filho não tem, e do diretório e do git dele).

nenhuma pediu interação, login ou permissão. o que o `claude` usou:

- `init` da primeira: `tools: []`, `mcp_servers: []`, `skills: []`,
  `slash_commands: []`, `permissionMode: default`, modelo
  `claude-sonnet-5`. as duas: `num_turns: 1`, `permission_denials: []`,
  `stop_reason: end_turn`. nenhuma ferramenta, nenhuma tentativa;
- o log de debug: "project memory is off" (nenhum CLAUDE.md), 0 hooks, 0
  skills, só o plugin embutido `agents-md` carregado (sem hooks
  registrados). o `claude` escreveu o próprio `~/.claude.json` (três
  escritas atômicas, o estado dele, como em qualquer uso) e buscou a
  lista de conectores do claude.ai (1 servidor), que não conectou
  (`mcp_servers: []`). nenhum arquivo do repositório: o diretório era
  vazio e não há ferramenta de leitura.

a resposta da segunda, um objeto json puro, sem prosa e sem cerca (a
primeira é igual, fora `gap: sm` e os ids `item_1`, `item_2`):

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo")
  todo_panel.entry (container)
    todo_panel.input (text_field, "new todo"): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.active_list (list, "active")
    todo_panel.active_list.item1 (checkbox, "Absorb the plev demos into the showcase"): checked <- todo.item[1]:completed, toggle -> todo.item[1] toggle
    todo_panel.active_list.item2 (checkbox, "Measure text, never estimate it"): checked <- todo.item[2]:completed, toggle -> todo.item[2] toggle
```

o Claude leu "o que está ativo" como os itens não concluídos agora
(`todo.item[1]` e `[2]` têm `completed: false`, o `[3]` tem `true`), e
não como o chip "Active" que as regras escolhem. os ids da entrada
(`todo_panel.input`, `todo_panel.add`) são os da superfície atual; os
chips e os números saíram. é uma leitura dos valores de agora, não um
filtro: um item concluído depois continua na lista, marcado, e um item
novo não entra. a lista que se repete sobre os itens é a task-95.

a terceira mediu o que acontece quando o orçamento estoura: o mesmo
comando da segunda com `--max-budget-usd 0.0001` e um prompt de uma linha
(`answer with the JSON object {"ok": true} and nothing else.`). 2,8 s,
exit 1, stderr vazio, e na stdout o objeto com `is_error: true`,
`subtype: error_max_budget_usd`, `errors: ["Reached maximum budget
($0.0001)"]`, sem `result`, custo 0,0013 USD. o limite é conferido
depois do turno: ele não corta uma chamada no meio. com um turno só e
sem ferramentas, quem limita uma chamada é o tempo máximo do crate e o
tamanho do prompt; o `--max-budget-usd` fica como teto.

o que a sonda decide:

- o `propose::prompt` não muda: as duas respostas vieram só com o json,
  a instrução do fim do primeiro parágrafo ("answer with the IR only: one
  JSON object, no prose") bastou, e o `read` do `Llm` já tira cerca e
  prosa se vierem;
- as flags do crate são as da segunda chamada, com o motivo de cada uma
  no handoff; a saída em json porque ela separa resposta de erro
  (`is_error`, `subtype`, `errors`) e diz o custo e os turnos; um erro
  vem com exit 1 e a explicação na stdout, não no stderr;
- o crate tira do ambiente do filho as variáveis da sessão do Claude
  Code acima e roda o filho num diretório dele;
- a condição de parada não disparou: sem interação, sem login, sem
  permissão, as duas chamadas longe dos 5 minutos, custo pequeno.

## corridas reais do exemplo (2026-09-24)

`cargo run -p intent -- --app todo|ide "<intenção>"`,
com o `claude` de verdade (sonnet, as flags do crate), da branch @ 31ff396
mais o exemplo; o todo com a superfície compacta mostrada, o ide sem
superfície. as quatro primeiras rodaram com o tempo máximo em 120 s; a
do ide levou 89 s, e o padrão subiu para 180 s antes do commit. a
quinta repete a terceira com o exemplo como foi commitado (ele passou a
imprimir o tamanho do prompt e o layout de cada nó).

| # | app | intenção | as regras | tentativas | tempo | custo | resposta |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | todo | the add field and the counts in two columns | verificam, 7 nós | 1, verificada | 4,5 s | 0,033 USD | 1.051 bytes |
| 2 | todo | a compact card with the input on top and the counts below | `"card", "top", "below" are not a word of the app` | 1, verificada | 5,6 s | 0,034 USD | 1.049 bytes |
| 3 | todo | show the filter as chips and hide the counts | `"chips", "hide" are not a word of the app` | 1, verificada | 6,3 s | 0,035 USD | 1.308 bytes |
| 4 | ide | review what goes into the commit: each changed file with stage and unstage, then the message and the commit button | `"review", "goes", "each", "changed", "file", "stage", "unstage", "then" are not a word of the app` | 1, verificada | 88,9 s | 0,111 USD | 5.653 bytes |
| 5 | todo | a 3 de novo | igual | 1, verificada | 8,7 s | 0,011 USD (prompt em cache) | 1.308 bytes, o mesmo IR |

os prompts: 17.710, 17.724 e 17.711 bytes no todo, 17.075 no ide (sem
superfície atual, mas com mais nós). nenhuma resposta foi recusada, então
o laço de conserto não rodou de verdade; ele está provado com o `claude`
falso (teste `a_refused_answer_is_repaired_by_the_second_call`).

os IRs, resumidos (id, componente, label, layout, o que liga):

1, "the add field and the counts in two columns":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.counts (container, grid min_col card_min_w, gap sm)
    todo_panel.active (stat, "active"): value <- todo.counter:active
    todo_panel.done (stat, "done"): value <- todo.counter:done
```

o Claude leu "two columns" como os dois números lado a lado num grid, e
não a superfície inteira em duas colunas, como as regras leem. as duas
leituras cabem na frase.

2, "a compact card with the input on top and the counts below":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical sm; padding sm, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): (as mesmas ligações)
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.counts (container, stack horizontal sm)
    todo_panel.active (stat, "active"): value <- todo.counter:active
    todo_panel.done (stat, "done"): value <- todo.counter:done
```

"compact" virou padding e gap `sm`; o filtro saiu, porque a frase não o
pede.

3, "show the filter as chips and hide the counts":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): (as mesmas ligações)
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.filter (container, "filter", stack horizontal xs)
    todo_panel.filter.all (chip, "All"): activate -> todo.filter[all] select
    todo_panel.filter.active (chip, "Active"): activate -> todo.filter[active] select
    todo_panel.filter.completed (chip, "Completed"): activate -> todo.filter[completed] select
```

nas três do todo, cada nó cuja capacidade continua ficou com o id da
superfície compacta (`todo_panel.input`, `todo_panel.add`,
`todo_panel.filter.*`, `todo_panel.active`, `todo_panel.done`), e o id da
superfície também: a página transformaria no lugar.

4, "review what goes into the commit: ...", sobre o workbench do ide:

```text
surface commit_review "Commit Review"
commit_review.root (container, stack vertical md)
  commit_review.changes (card, "Changes", stack vertical sm)
    commit_review.changes.list (list, "Changed files")
      changes.file[src/lexer.rs] (container, stack horizontal sm)
        changes.file[src/lexer.rs].name (text, "src/lexer.rs")
        changes.file[src/lexer.rs].status (badge, "src/lexer.rs status"): text <- changes.file[src/lexer.rs]:status
        changes.file[src/lexer.rs].stage_btn (button, "Stage src/lexer.rs"): activate -> changes.file[src/lexer.rs] stage
        changes.file[src/lexer.rs].unstage_btn (button, "Unstage src/lexer.rs"): activate -> changes.file[src/lexer.rs] unstage
      changes.file[notes.md] (...o mesmo, para notes.md)
      changes.file[src/parser.rs] (...o mesmo, para src/parser.rs)
  commit_review.commit (card, "Commit", stack vertical sm)
    commit_review.commit.message (text_field, "Commit message"): text <- commit.message:text, set_value -> commit.message set_value
    commit_review.commit.submit (button, "Commit"): activate -> commit.submit submit
```

duas coisas desse IR importam para depois. primeiro, o Claude escreveu os
três arquivos um a um, e não um `repeat` sobre `changes:files` (a lista
existe no registro, e a fixture `ide-review-commit.ir.json` repete sobre
ela): é a mesma leitura dos valores de agora que a sonda mostrou nos
itens do todo, e um arquivo que mudar depois não aparece. segundo, os nós
dos arquivos tomam o id do nó do app (`changes.file[src/lexer.rs]`) e os
filhos nascem embaixo dele. o verifier aceita (regra identity: um nó com
o id de um nó do app só manda as ações dele), e o host da superfície
trata esse nó como do app, o caso da tela fixa escrita em IR
(`rust/screen/src/project.rs`): não o projeta de novo, e os filhos
novos ficam sob o nó do app na projeção. se é isso que uma superfície
gerada ao lado do app deve fazer, a 3b-ii decide. nenhuma das duas é
deste crate; estão no handoff.

## handoff da fase 3b-i (2026-09-24)

goal: task-69 fase 3b-i, o Claude como modelo do proponente: um crate
novo implementa `propose::Model` chamando o `claude -p` como processo, e
um exemplo mostra uma intenção em texto virando um `VerifiedIr` sobre a
projeção real do todo e do ide, pelo `propose::Llm` e pelo
`propose_verified`.

status: implementado. `cargo test -p claude -p offers` verdes,
`./tool/tasks/fullcheck.sh` verde com a perna urnaui, sobre a `main` @ ce9acee. a
`main` andou para 9299f2d depois (só o journal); o rebase é do
orquestrador.

### commits

branch `task/69d-claude-model`, na ordem do briefing:

- b771ead task-69d: a sonda da fase 3b-i no arquivo da task
- 31ff396 task-69d: claude-model, the Claude behind propose::Model through claude -p
- 524332f task-69d: the propose example, and the Claude run for real on the todo and the ide
- e o commit do ADR e deste handoff

### a sonda

acima ("sonda da fase 3b-i"). em uma linha: o prompt do `propose` para
"only what is active and the add field" sobre o golden do todo, 17,7 KB,
pelo `claude -p` sem ferramentas, sem sessão salva, sem MCP e sem as
configurações do usuário, voltou só com o json e passou pelo
`propose_verified` de primeira nas duas chamadas (40,1 s com o system
prompt do Claude Code, 24,5 s com o do crate); o `claude` não usou
ferramenta nem arquivo do repositório (`tools: []`, `num_turns: 1`,
`permission_denials: []`); a terceira chamada mostrou que um orçamento
estourado sai com exit 1 e o motivo no json da stdout. o `propose::prompt`
não mudou. nenhuma condição de parada disparou.

### o crate

`rust/claude` (pacote `claude-model`, lib `claude_model`).
depende de `propose` (o trait) e de `serde_json` (o resultado do
filho); nunca de `uir`, `sem`, engine, comps, showcase, ide ou de um
cliente HTTP. `uir`, `sem` e `tempfile` só nos testes e no exemplo.

o `serde_json` não estava na lista do briefing (`propose`, a std e
`thiserror`). entrou porque a saída em json é o que separa resposta de
erro: um orçamento estourado sai com exit 1, nada no stderr e o motivo no
objeto da stdout; em texto, o erro e a resposta seriam o mesmo texto. o
`propose` já depende dele, então nada novo compila. o `thiserror` não
foi preciso: todo erro é o `ModelError` do `propose`.

a API:

- `ClaudeCli { program, model, timeout, budget_usd, dir }`, campos
  públicos, e o registro das chamadas, privado. `ClaudeCli::new()` (e
  `Default`): `PLEV_CLAUDE` ou `claude`, `sonnet`, 180 s, 0,5 USD, o
  diretório temporário do sistema;
- `impl propose::Model for ClaudeCli`: roda o programa uma vez por
  `complete`;
- `args()`: os argumentos exatos (os testes comparam com o que o
  `claude` falso recebeu); `calls()`: cada chamada como `Call { took,
  cost_usd, turns, reply: Result<usize, ModelError> }`; `spent_usd()`;
- as constantes `PROGRAM`, `PROGRAM_ENV` (`PLEV_CLAUDE`), `MODEL`,
  `TIMEOUT`, `BUDGET_USD`, `SYSTEM` e `SESSION_VARS`.

três módulos, todos pequenos: `lib.rs` (a configuração, os argumentos, o
`Model`, 263 linhas com os testes), `run.rs` (o processo: stdin numa
thread, stdout e stderr em threads, o prazo, o kill; 129) e `reply.rs`
(o json do resultado virando resposta ou `ModelError`; 177 com os
testes).

### as flags do `claude` e o motivo de cada uma

| flag | por quê |
| --- | --- |
| `-p` | uma resposta e o processo sai; o prompt vem da stdin, sem argumento posicional |
| `--output-format json` | um objeto só: `result`, ou `is_error` com `subtype` e `errors`, mais `total_cost_usd` e `num_turns`. é o que separa resposta de erro (a sonda: orçamento estourado sai com exit 1 e o motivo na stdout) |
| `--tools ""` | nenhuma ferramenta: o filho responde texto, não lê arquivo, não roda comando, e o turno é um só |
| `--no-session-persistence` | nada salvo em disco para retomar |
| `--setting-sources ""` | nenhum arquivo de configuração (usuário, projeto, local): sem hooks, sem permissões, sem a memória do usuário; o log da sonda diz "project memory is off" e 0 hooks |
| `--strict-mcp-config` | só os servidores MCP do `--mcp-config`, que não é passado: nenhum (`mcp_servers: []`) |
| `--disable-slash-commands` | nenhuma skill listada no prompt (`skills: []`) |
| `--permission-prompts none` | o que pediria permissão é negado sozinho: nada espera uma pessoa |
| `--system-prompt <SYSTEM>` | duas frases no lugar do system prompt do Claude Code, que fala de ferramentas, diretório e git que o filho não tem; uns 4.200 tokens de entrada a menos (11.150 para 6.983 na sonda) |
| `--model <model>` | `sonnet` por padrão: a sonda verificou de primeira em 24 a 40 s; `opus` ou um nome completo pelo campo |
| `--max-budget-usd <budget>` | 0,5 por padrão, umas dez vezes uma chamada do todo. teto de sessão: o filho confere depois do turno, não corta uma chamada no meio |

e fora das flags: o filho roda em `dir` (o diretório temporário do
sistema por padrão, nenhum projeto do app é lido dali) e sem as onze
variáveis que uma sessão do Claude Code põe nos processos dela
(`SESSION_VARS`: `CLAUDECODE`, `CLAUDE_CODE_ENTRYPOINT`,
`CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_CHILD_SESSION`,
`CLAUDE_CODE_SESSION_ATTENDED`, `CLAUDE_CODE_MESSAGING_SOCKET`,
`CLAUDE_CODE_MESSAGING_TOKEN`, `CLAUDE_CODE_EXECPATH`, `CLAUDE_PID`,
`CLAUDE_EFFORT`, `AI_AGENT`). um app rodado de dentro de uma sessão (como
o exemplo, rodado por um agente) passaria essas variáveis ao filho, e o
`CLAUDE_EFFORT=xhigh` desta sessão mudaria o esforço dele. o resto do
ambiente passa (PATH, HOME, e o que o usuário configurou para o login).

não usadas: `--bare` (lê só `ANTHROPIC_API_KEY`, nunca o login OAuth da
conta), `--json-schema` (decodificação restrita pelo `uir::schema()`; não
testada, porque o prompt já leva o schema e o verifier é quem decide, e
fica como opção para a 3b-ii), `--safe-mode` (o `--setting-sources ""`
com as outras flags já deixou tudo vazio na sonda), `--effort` (o padrão
do modelo verificou de primeira).

### o tempo máximo

180 s por chamada (`TIMEOUT`), o dobro da chamada mais lenta medida (89
s, o workbench do ide, 5,6 KB de IR); as do todo levaram de 4,5 a 40 s.
começou em 120 s (três vezes a sonda) e subiu depois da corrida do ide.
uma proposta faz até três chamadas (`Llm::ATTEMPTS`), então espera no
máximo 9 minutos; a página (3b-ii) não deve esperar isso sem mostrar e
sem deixar cancelar.

ao estourar, `Child::kill` (SIGKILL no pid do filho), `wait` para colher,
e `ModelError("<programa> did not answer in 180 s: killed (pid N)")`; o
`Llm` para na hora. o filho é consultado a cada 10 ms (`try_wait`).
depois que ele sai, o fim da saída é esperado no máximo 1 s: um neto que
segura o pipe não segura a chamada, e o que o filho escreveu fica (teste
`a_grandchild_holding_the_pipes_does_not_hold_the_call`). o neto não é
morto: só o pid do filho.

### as corridas reais

a tabela e os IRs estão acima ("corridas reais do exemplo"). resumo:

| app | intenção | as regras | tentativas | tempo |
| --- | --- | --- | --- | --- |
| todo | the add field and the counts in two columns | entendem | 1 | 4,5 s |
| todo | a compact card with the input on top and the counts below | não entendem | 1 | 5,6 s |
| todo | show the filter as chips and hide the counts | não entendem | 1 | 6,3 s (8,7 s de novo, com o exemplo final) |
| ide | review what goes into the commit: each changed file with stage and unstage, then the message and the commit button | não entendem | 1 | 88,9 s |

as quatro verificaram de primeira; nas do todo, os ids do que ficou são
os da superfície compacta. o custo que o `claude` mostra: uns 0,034 USD
por chamada no todo, 0,111 no ide, um terço com o prompt em cache.

### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ ce9acee (`./tool/tasks/fullcheck.sh` neste worktree, antes do primeiro código) | 2372 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2454 |
| depois, esta branch @ 524332f (`./tool/tasks/fullcheck.sh`, saída 0) | 2385 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2467 |

13 testes novos, todos no `claude-model`: 6 unitários e 7 com o `claude`
falso. a contagem não cai. clippy `-D warnings`, `cargo fmt --check` e o
check wasm do showcase verdes, dentro do gate. `df -h /` antes do gate:
14 GB livres.

### testes

- `src/lib.rs` (2): os argumentos são os da sonda, cada flag uma vez,
  as listas vazias como argumento vazio, o modelo e o orçamento dos
  campos; os padrões;
- `src/reply.rs` (4): um resultado é a resposta com o custo e os
  turnos; um resultado de erro é o erro com o tipo e o motivo (o do
  orçamento da sonda); um resultado só é resposta quando tudo diz que é
  (exit 0, `is_error: false`, `result` presente); saída que não é
  resultado vira um erro curto, numa linha;
- `tests/cli.rs` (7), com `tests/fake-claude` (um script sh que responde
  de arquivos no diretório que o `ClaudeCli` dá a ele, e guarda os
  argumentos, o ambiente, cada prompt e o pid): uma resposta boa é uma
  superfície verificada, e o filho recebeu exatamente `args()`, o prompt
  inteiro pela stdin (17 KB, mais do que um pipe segura) e nenhuma das
  variáveis da sessão (o gate rodou dentro de uma, então o teste
  conferiu de verdade); uma resposta ruim é consertada pela segunda
  chamada, que leva a resposta e os erros; um filho além do tempo morre
  (o pid dele não existe mais) e a proposta para em 1 s, não em 30; um
  neto que segura os pipes não segura a chamada; um programa que falha
  é o erro na hora; um orçamento estourado é o erro com o custo; um
  programa que não existe é o erro.

nenhum teste chama o `claude` de verdade: todos apontam `program` para o
script. os testes de processo são `#[cfg(unix)]`.

### validação executada

- `cargo test -p claude -p offers`;
- `cargo clippy -p claude --all-targets -- -D warnings`,
  `cargo fmt -p claude --check`;
- `cargo test -p engine --test arc_sync_guard`;
- `./tool/tasks/fullcheck.sh`, saída 0, com a perna urnaui;
- o exemplo com o `claude` de verdade: as quatro corridas e a repetição
  acima, e com `PLEV_CLAUDE=/nonexistent/claude` (erro "could not
  start", exit 1) e `--app nope` (uso, exit 2).

para reproduzir sem rede: `cargo test -p claude`. com rede, uma
chamada: `cargo run -p intent -- --app todo
"show the filter as chips and hide the counts"`.

### seção de ADR

[`0052-proposers-propose-the-runtime-verifies.md`](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md), seção nova no fim,
"addendum, 2026-09-24: the Claude as the model (task-69 phase 3b-i,
proposed)": contexto (a decisão 13), decisão (o crate, o processo, cada
flag com o motivo, o filho fora da sessão, o prazo, os erros, o prompt
que não muda, os testes sem rede, o exemplo), custo (a sonda e as
corridas), consequências (o que ele lê que as regras não leem, valores
de agora em vez de listas, os ids tomados do app, o teto de orçamento, o
que o `claude` escreve, o que a 3b-ii decide) e o que evitar.

para o orquestrador (o [`adr/`](../../adr/adr.md) é proibido aqui): no fim da
linha do índice desse ADR, algo como "; addendum 3b-i (proposed): crate
claude-model, `ClaudeCli` implementa `Model` pelo `claude -p` como
processo (json, sem ferramentas, sem sessão salva, sem MCP, sem
configurações, system prompt próprio, fora da sessão do Claude Code,
180 s e kill pelo pid), testes com um `claude` falso".

### decisões

- saída em json e `serde_json`. contexto: a sonda mostrou que um erro
  sai com exit 1 e o motivo na stdout. decisão: ler o objeto de
  resultado. razão: só ele separa resposta de erro, e dá custo e turnos.
  consequência: uma dependência a mais no crate (já compilada pelo
  `propose`);
- system prompt próprio. razão: o do Claude Code descreve ferramentas
  que o filho não tem e o diretório e o git dele; tirar poupa 40% da
  entrada. consequência: a instrução de responder só o json continua no
  prompt do `propose`, e o do crate só diz que quem chama é um programa;
- o filho fora da sessão do Claude Code (as onze variáveis) e num
  diretório dele. razão: um agente roda o exemplo de dentro de uma
  sessão, e o app também pode ser rodado assim;
- `sonnet` por padrão, 180 s, 0,5 USD; tudo campo;
- `PLEV_CLAUDE` só para o programa. os testes não usam a variável
  (mudar o ambiente num teste é `unsafe` na edição 2024); apontam
  `program` direto;
- o registro das chamadas no `ClaudeCli` (tempo, custo, turnos), para
  a página mostrar e o exemplo imprimir;
- o `propose` não mudou em nada.

### limitações

- síncrono: `complete` bloqueia a thread de quem chama de segundos a um
  minuto e meio por tentativa. rodar fora da thread do app é da 3b-ii;
- cancelar só pelo prazo: não há como matar o filho de fora antes dele;
- o Claude lê valores de agora, não listas (os itens do todo, os
  arquivos do ide), e responde diferente a cada chamada (as duas da
  sonda diferem no gap e nos ids dos itens);
- `SESSION_VARS` é a lista da versão 2.1.281; uma variável nova de uma
  versão futura passa;
- o custo mostrado é o de lista. as chamadas contam no plano do brenner,
  o mesmo das sessões dos agentes (a sonda viu `rate_limit_event` com a
  janela de cinco horas em 20%);
- o `--max-budget-usd` não corta uma chamada; o prazo é que corta;
- o `claude` escreve o próprio `~/.claude.json` e busca a lista de
  conectores do claude.ai ao iniciar, como em qualquer uso;
- um neto que segura um pipe continua vivo depois da chamada;
- os testes de processo são só unix (o `claude` falso é um script sh).

### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  [`changelog.md`](../../CHANGELOG.md) (crate novo e o exemplo); no `docs/ATLAS.toml`,
  a linha `claude-model = ["propose"]` em `[examples]` (só a entrada do
  crate era permitida; o exemplo está citado nela); no `AGENTS.md`,
  "## running", o comando do exemplo, se quiser;
- rebase sobre a `main` @ 9299f2d (só o journal; sem conflito
  esperado);
- o exemplo chama o `claude` de verdade e custa uma chamada ou mais;
  nada no gate o roda (é exemplo, compila no clippy, não executa);
- o `tests/fake-claude` precisa do bit de execução (está como 100755 no
  git).

### o que a 3b-ii (a página) precisa

- rodar fora da thread do app: o `propose` da página responde
  `Execution::Started` e termina com `Dispatcher::complete` ou `fail`
  quando o Claude responde. o `ClaudeCli` é `Clone` e `Send` e pode ir
  inteiro para um trabalhador; o `Ask` empresta, então o trabalhador
  leva o snapshot, o registro e o `VerifiedIr` atual clonados e monta o
  `Ask` lá dentro;
- cancelar: hoje só o prazo mata o filho. um cancelamento de fora pede
  uma mudança pequena no `run` (um sinal conferido a cada 10 ms, junto
  do prazo), e é a forma de a página abandonar uma proposta;
- o que a página mostra durante 5 a 90 s por tentativa, até três; o
  `calls()` dá tempo e custo de cada uma;
- qual proponente: as regras primeiro (0,16 ms) e o Claude quando elas
  não entendem, ou só o Claude. as duas leem "only what is active"
  diferente (o chip "Active" contra os itens não concluídos), então
  "regras primeiro" muda o sentido de uma intenção que as duas leem.
  decisão do orquestrador ou do brenner;
- o prompt: pedir um `repeat` quando uma lista mostra cada membro (o
  Claude desenrolou os itens e os arquivos), agora que a task-95 projeta
  os itens do todo como lista; e se um id tomado de um nó do app cabe
  numa superfície gerada ao lado dele. as duas mexem no `propose` e
  pedem uma sonda nova;
- continuidade: se uma proposta que troca o id de uma capacidade que
  continua volta ao modelo como erro brando (nas corridas, o Claude
  manteve todos);
- custo e limites: cada proposta custa uma a três chamadas do plano do
  brenner; o cache do prompt (uma hora) barateia a segunda proposta
  sobre o mesmo app.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0052: proposers propose, the runtime verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md)
- [task-69: ephemeral and generated surfaces](task-69-ephemeral-generative-surfaces.md)
<!-- /related -->
