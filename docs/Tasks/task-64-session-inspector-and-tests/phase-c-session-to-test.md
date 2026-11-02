---
id: task-64/phase-c-session-to-test
title: "task-64 phase c: session to test, and export"
project: plev
type: section
parent: task-64
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [recording, determinism, agents]
relations: [adr-0050, task-63, task-68, task-69]
evidence: [edf6420, ed82720, bb0a99f, 901710c, d611fa4]
source: docs/log/steps/task-64-session-inspector-and-tests.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase c: session to test and export

This phase took an excerpt of a recorded session to a regression test: the probe re-applied the nine requests of the session a2ppa against a new repository and got the same verdicts, the `rerun` harness re-runs the acts of an excerpt against an ide fixture, and `plev-inspect --export` writes the excerpt as a bug report. The integration was at `d611fa4`; the generated test stayed green at the gate, and the regression proof, with the `commit.submit` precondition broken on purpose, made it fail with the divergent verdict.

## fase c: sonda (2026-09-24)

antes de implementar, a pergunta da sonda: os requests de uma sessão real,
reaplicados na ordem gravada contra uma fixture nova do mesmo repositório,
dão os mesmos vereditos?

### os requests gravados

sessão `target/agents/exp3/a2ppa/session.monster` (terceira rodada, o
`ide-lab` do build `96c20a0`, agente LLM `claude-a2ppa`, pessoa
`unstage-before-submit:src/session.rs`), lida com o `plev-inspect` do
target da task (`--timeline` e `--why` de cada request). origem 0, um
dispatcher só:

```text
n  +t        ator                alvo                               ação       observed (os reads, como o ator viu)                           veredito terminal
1  +41.011s  agent:claude-a2ppa  changes.file[README.md]            unstage    staged=true, unstaged=false                                    accepted, completed
2  +42.343s  agent:claude-a2ppa  changes.file[src/session.rs]       stage      staged=false, unstaged=true                                    accepted, completed
3  +43.703s  agent:claude-a2ppa  changes.file[tests/login_test.rs]  stage      staged=false, unstaged=true                                    accepted, completed
4  +56.210s  agent:claude-a2ppa  commit.form                        open       commit.form:open=false                                         accepted, completed
5  +62.426s  agent:claude-a2ppa  commit.message                     set_value  commit.message:text=""  (param "fix login session timeout")    accepted, completed
6  +70.679s  human               changes.file[src/session.rs]       unstage    staged=true, unstaged=false                                    accepted, completed (+70.693s)
7  +70.679s  agent:claude-a2ppa  commit.submit                      submit     changes:staged={src/login.rs, src/session.rs,                  rejected: stale, changes:staged
                                                                               tests/login_test.rs}, commit.message:text="fix login           removed src/session.rs
                                                                               session timeout"
8  +95.083s  agent:claude-a2ppa  changes.file[src/session.rs]       stage      staged=false, unstaged=true                                    accepted, completed
9  +110.643s agent:claude-a2ppa  commit.submit                      submit     changes:staged={src/login.rs, src/session.rs,                  accepted, completed: commit bf680c84,
                                                                               tests/login_test.rs}, commit.message:text="fix login           files [src/login.rs, src/session.rs,
                                                                               session timeout"                                               tests/login_test.rs]
```

o request 6 é a pessoa roteirizada: o laboratório o despacha logo antes do
submit 7 chegar ao dispatcher, e o 6 ainda estava `executing` quando o 7 foi
recusado (o worker respondeu 14 ms depois).

### a fixture e como ela alcança o repositório

`Repo::build` morava no bin do laboratório (`src/bin/lab/repo.rs`), fora do
alcance de um teste. a fixture o alcança sem duplicar: o arquivo foi movido
com `git mv` para a lib, `apps/idestudio/src/repository/eval_repo.rs` (`pub mod lab_repo`,
conteúdo igual), e o bin do laboratório passou a importá-lo com `use
ide::lab_repo as repo;`, então todo `crate::repo::...` do laboratório
continua como estava. a fixture é a mesma montagem do `Bench::open` do
laboratório e do `Rig` dos testes: `Repo::build()` num tempdir, um
`git::GitClient` nele, `Workbench::new(1280.0, 800.0)`, `connect`, e os
eventos do worker aplicados até o primeiro `Branches`.

### o rerun

cada request gravado, inteiro (ator, alvo, ação, parâmetro, `observed`),
passou por `Workbench::dispatch` na ordem gravada, e o worker foi drenado
entre um e outro: os eventos aplicados até o receipt terminar, depois um
`Workbench::refresh()` como barreira e os eventos aplicados até chegar cada
`Branches` devido (um por resposta de mutação, mais o da barreira; o worker
responde na ordem em que recebe). um teste descartável, fora do commit.

resultado: os nove vereditos iguais, e o caminho inteiro de estágios igual
em cada request (`requested, accepted, executing, completed` em 1 a 6, 8 e
9; `requested, rejected` em 7). a recusa do 7 com o mesmo `read` mudado e os
mesmos itens (`changes:staged`, removido `src/session.rs`); o commit do 9
com os mesmos três arquivos. a mesma sonda nas outras sessões da terceira
rodada: `a1pp` (7 requests, o submit 6 recusado stale em
`commit.message:text`), `a2ppc` (8) e `a2ppb` (7, o submit 7 recusado stale
em `changes:staged`), todos os vereditos iguais.

### o que difere sem mexer em veredito

- a revisão. o único nó diferente no estado de partida é
  `stacks.commit[<sha>]` do commit inicial (`0d3be3cc...` gravado,
  `a17675e2...` no rerun): o `git commit` do `Repo::build` carimba a hora do
  relógio, e o sha muda a cada montagem. por isso toda revisão difere
  (`0e3579d7ca054a45` gravada no início, `8790fa32701b4f56` no rerun). o
  `check` nunca compara revisão, só os `reads`.
- o `acted` da recusa do 7. gravado: a pessoa, request 6 (a porta sabia do
  olhar do agente). no rerun: ninguém, porque nenhum sink sabe do olhar e,
  sem ele, a regra parte de um receipt validado na revisão observada, que o
  rerun nunca tem. o harness não compara o `acted`; compara os `reads`
  mudados, como o briefing pede.
- a sobreposição do 6 com o 7. na gravação o 6 ainda executava quando o 7
  chegou; na sonda o worker foi drenado entre eles, e o 6 já tinha
  terminado. o veredito é o mesmo porque a view mostra o unstage assim que
  ele sai para o worker.
- os tempos: o workbench carimba os receipts no próprio relógio (`Instant`),
  e o sha do commit do 9 (`85f2f632...` no rerun) carrega a hora do git.
  nenhum dos dois é comparado.

nenhum veredito divergiu, e reaplicar não tocou nada fora do tempdir nem a
rede: o worker roda `git` no repositório temporário. a implementação segue.

## fase c: handoff (2026-09-24)

```text
Goal: task-64 fase c, um trecho de sessão gravada vira teste de regressão no gate, e a CLI exporta um trecho como bug report
Status: pronto para revisão (branch task/64c-session-to-test, rebaseada na main @ 51394a9; o briefing partiu de 41449e1)
```

### commits

- `edf6420` task-64c: the probe, a real session's requests rerun on a fresh laboratory repository
- `ed82720` task-64c: the session-to-test harness and a toy fixture
- `bb0a99f` task-64c: the ide's fixture and a test generated from a real session
- `901710c` task-64c: plev-inspect --export, a stretch of a session as a bug report
- este: task-64c: the ADR and the handoff

todos assinados. a main andou durante a fase (a task-68, o crate `uir`, e
a abertura da task-69); a branch entrou nela sem conflito e foi rebaseada
antes do pedido de integração.

### a sonda

na seção "fase c: sonda", acima: os nove requests da a2ppa, reaplicados na
ordem gravada sobre um repositório novo do laboratório, deram os mesmos
vereditos e o mesmo caminho de estágios; a1pp, a2ppb e a2ppc também. o que
difere (revisão, pelo sha do commit inicial; `acted`; a sobreposição do
unstage da pessoa com o submit; relógios) não mexe em veredito.

### o harness

onde: `apps/idestudio/tests/rerun/`, um binário de teste (`cargo test -p idestudio
--test rerun`). pelo ADR da 63, não no `rec` (o crate não executa nada, o
`Session` não segura app, e o ADR evita qualquer api que receba um
dispatcher ou um app no lado da leitura), nem no
`inspect` (só lê; o teste que confere as fontes continua verde). um crate
novo pede `arc.toml` e README, fora desta fase, e uma fixture só não faz
reuso: quando o showcase tiver a dele, o harness vira crate de teste.

- `harness.rs`: o `Fixture` e o executor `rerun`.
- `marks.rs`: o formato das marcas.
- `report.rs`: o relatório, request por request e marca por marca.
- `toy.rs`: o harness contra uma fixture de brinquedo (6 testes).
- `lab.rs`: a fixture do ide e o teste gerado da a2ppa.

a interface que o app fornece:

```rust
pub trait Fixture {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId;
    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn settle(&mut self) -> Result<(), String>;
    fn project(&self) -> Result<Snapshot, String>;
    fn receipt(&self, id: RequestId) -> Option<Receipt>;
}

pub fn rerun<F: Fixture>(
    excerpt: &Session,
    marks: &[Mark],
    start: impl FnOnce() -> Result<F, String>,
) -> Result<Report, String>;
```

`start` é o "partir do estado inicial num ambiente isolado"; `&mut F`
também é fixture, para o teste olhar a fixture depois. as regras do rerun:

- cada ato gravado é refeito na ordem: o `requested` vira um dispatch do
  request exatamente como foi gravado (ator, alvo, ação, parâmetro e os
  valores observados, que são a precondição); `confirmed` e `declined` de
  uma pessoa viram a mesma resposta. os outros estágios são a resposta do
  app, produzida pelo rerun, nunca reproduzida da gravação.
- antes de cada ato o app assenta (`settle`), a menos que a gravação
  tivesse uma execução ainda rodando naquele ponto: aí o ato entra com ela
  em voo, como entrou. isso diverge do "drenando o worker entre eles" do
  briefing de propósito, só onde a gravação mostra sobreposição: na a2ppa,
  o unstage da pessoa (6) ainda executava quando o submit (7) foi recusado,
  e o rerun reproduz isso. o teste do brinquedo confere a ordem das
  chamadas (nenhum `settle` no meio da sobreposição) e que o commit lento
  leva só o arquivo que levou na gravação; assentando no meio, o unstage
  chegaria depois do commit e ele levaria os dois.
- um request que o trecho tem sem o primeiro estágio (começou antes do
  corte) recusa o rerun antes de a fixture ser montada.
- no fim, veredito (o estágio depois de `requested`) e estágio terminal de
  cada request contra os gravados: mesmo estágio, numa recusa o mesmo tipo,
  numa recusa stale os mesmos `reads` mudados. um request aberto no fim do
  trecho não tem terminal a comparar. não compara tempo, revisão, `acted`
  nem valor de outcome (o sha do commit); uma marca confere valor de
  outcome quando importa.

a fixture do ide (`lab.rs`) é a mesma partida do laboratório
(`Bench::open`): `ide::lab_repo::Repo::build()` num tempdir, um
`git::GitClient` real nele, `Workbench::new(1280.0, 800.0)`, `connect` e
os eventos aplicados até carregar. o `settle` pede um `refresh` como
barreira e aplica os eventos do worker até chegar cada `Branches` devido
(um por refresh pedido, mais um por mutação respondida, que o
`GitSession::apply` sempre segue de um refresh). o worker só roda `git`
no tempdir; nada de rede.

o `Repo::build` saiu do bin sem duplicar: `git mv` de
`src/bin/lab/repo.rs` para `src/lab_repo.rs` (conteúdo idêntico), `pub mod
lab_repo;` no `lib.rs` (com uma linha de doc), e o bin do laboratório
troca `mod repo;` por `use ide::lab_repo as repo;`, então todo
`crate::repo::...` dele continua valendo. o teste do repositório só mudou
de alvo (do bin `ide-lab` para a lib), a contagem não muda. o `tests/lab.rs`,
que roda o `ide-lab` inteiro, continua verde.

### o formato das asserções

um arquivo de texto ao lado do trecho (`<nome>.marks`), uma marca por
linha, `#` comenta. só o que foi marcado é conferido, nunca o estado
inteiro:

```text
<âncora>  <o quê> = <valor>

âncora  after request <id> [origin <n>]   quando o request acabou na gravação
        at <ms> | at +<s>s                tempo desde o começo do trecho, como o plev-inspect imprime e o --at recebe
o quê   <id>:<chave>                      um valor da projeção do app
        outcome.<chave>                   um valor com que o request da âncora terminou (só com after request)
        stale                             os reads que a recusa stale do request nomeia, como conjunto (só com after request)
valor   true | false | 12 | 0.5 | "texto" | ["a", "b"] (lista) | {"a", "b"} (conjunto) | absent
```

a marca é lida logo antes do próximo ato que a gravação tem, ou no fim,
com o app assentado (menos no meio de uma sobreposição). linha que não lê
é erro com o número; marca num request que o trecho não tem recusa o
rerun. o `" = "` primeiro da linha separa o quê do valor.

### o teste gerado e a prova de que pega regressão

`apps/idestudio/tests/fixtures/sessions/a2ppa.monster` é a sessão da terceira
rodada exportada do começo até o commit e a projeção que ele deixa
(`plev-inspect target/agents/exp3/a2ppa/session.monster --export 0 110688
a2ppa.monster`; 184042 bytes, 65 eventos, 12 frames; o sha256 da sessão de
origem, `f07f34fc...`, igual antes e depois). `a2ppa.marks` tem seis
marcas: o estado de partida, a recusa stale do primeiro submit em
`changes:staged` com o staged e a mensagem depois dela, e o commit com os
três arquivos da tarefa e o staged vazio.

verde no gate:

```text
rerun of an excerpt recorded by rec 0.1.0, built from 96c20a0: 9 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:claude-a2ppa: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:claude-a2ppa: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:claude-a2ppa: completed
  same      request 5 (origin 0, rerun as 5): set_value on commit.message by agent:claude-a2ppa: completed
  same      request 6 (origin 0, rerun as 6): unstage on changes.file[src/session.rs] by human: completed
  same      request 7 (origin 0, rerun as 7): submit on commit.submit by agent:claude-a2ppa: rejected: stale, changes:staged (removed src/session.rs)
  same      request 8 (origin 0, rerun as 8): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 9 (origin 0, rerun as 9): submit on commit.submit by agent:claude-a2ppa: completed
  held      line 11: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 15: after request 7  stale = {"changes:staged"}
  held      line 16: after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
  held      line 17: after request 7  commit.message:text = "fix login session timeout"
  held      line 21: after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
  held      line 22: after request 9  changes:staged = {}
nothing diverged
```

o teste também confere, no repositório temporário, que o `HEAD` do rerun
é o commit "fix login session timeout" com os três arquivos.

a prova: `.reads(format!("{}:staged", ids.changes))` comentada em
`apps/idestudio/src/capability/project.rs` (a precondição do `commit.submit`
quebrada de propósito), `cargo test -p idestudio --test rerun -- lab`, e o
arquivo restaurado com `git checkout` logo depois (nenhum diff no
workbench, nada disso commitado). a saída:

```text
test lab::the_a2ppa_session_reruns_as_it_was_recorded ... FAILED
---- lab::the_a2ppa_session_reruns_as_it_was_recorded stdout ----
thread 'lab::the_a2ppa_session_reruns_as_it_was_recorded' panicked at apps/idestudio/tests/rerun/eval.rs:171:5:
rerun of an excerpt recorded by rec 0.1.0, built from 96c20a0: 9 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:claude-a2ppa: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:claude-a2ppa: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:claude-a2ppa: completed
  same      request 5 (origin 0, rerun as 5): set_value on commit.message by agent:claude-a2ppa: completed
  same      request 6 (origin 0, rerun as 6): unstage on changes.file[src/session.rs] by human: completed
  DIVERGED  request 7 (origin 0, rerun as 7): submit on commit.submit by agent:claude-a2ppa
            verdict   recorded  rejected: stale, changes:staged (removed src/session.rs)
                      rerun     accepted  <- differs
            terminal  recorded  rejected: stale, changes:staged (removed src/session.rs)
                      rerun     rejected: stale, changes:staged (removed src/session.rs)
  same      request 8 (origin 0, rerun as 8): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 9 (origin 0, rerun as 9): submit on commit.submit by agent:claude-a2ppa: completed
  held      line 11: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 15: after request 7  stale = {"changes:staged"}
  held      line 16: after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
  held      line 17: after request 7  commit.message:text = "fix login session timeout"
  held      line 21: after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
  held      line 22: after request 9  changes:staged = {}
the rerun diverged: 1 request(s), 0 mark(s)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.38s
```

o veredito do 7 diverge (aceito no check em vez de recusado stale) e o
teste falha dizendo onde. o terminal fica igual porque o ponto de verdade
ainda segura: o `CommitIf` do worker confere o índice real e recusa com o
mesmo delta. ou seja, com a regra do app quebrada o índice continua
protegido, e o harness pega a regressão pelo veredito, não pelo
resultado. no gate, o teste do brinquedo
(`a_broken_precondition_diverges_and_the_report_says_where`) guarda o
mesmo caso de forma automática, com o texto da mensagem conferido:

```text
  DIVERGED  request 2 (origin 0, rerun as 2): submit on commit.submit by agent:toy
            verdict   recorded  rejected: stale, changes:staged (removed B)
                      rerun     accepted  <- differs
            terminal  recorded  rejected: stale, changes:staged (removed B)
                      rerun     completed  <- differs
  ...
  DIVERGED  line 4: after request 2  stale = {"changes:staged"}
            found     absent
  DIVERGED  line 5: after request 3  outcome.files = ["A", "C"]
            found     the request did not complete in the rerun: rejected: disabled
the rerun diverged: 2 request(s), 2 mark(s)
```

### o export

`plev-inspect <sessão> --export <t0> <t1> <saída>`: t0 e t1 em ms desde o
começo da sessão (o mesmo tempo do `--at`), a saída pelo
`Session::excerpt` do `rec` (do keyframe de cada trilha que governa t0 até
t1, chunks copiados como estão, só os assets que eles usam). mantive a
forma do briefing. vai sozinho (com `--timeline`, `--why`, `--control` ou
`--json` é exit 2), nunca por cima do arquivo que lê (exit 2), e uma
escrita que falha é exit 4, novo, no `--help`. a lib do `inspect` não
mudou; só o bin.

testes (`rust/replay/tests/queries/export.rs`): uma sessão com frames e
ligações de um compositor e keyframes frequentes; o trecho do meio reabre
abrindo nos keyframes das duas trilhas antes de t0, com os mesmos eventos
em [t0, t1], o mesmo estado, frame, tempo de frame e ligações a cada 10 ms,
e os mesmos requests inteiros (a recusa stale e o commit); o arquivo de
origem com os mesmos bytes; o resumo do trecho pelo próprio
`plev-inspect`. e os erros: t1 antes de t0, junto com `--why`, por cima do
arquivo lido (sem escrever nada), diretório que não existe (exit 4),
sessão que não lê (exit 3). o parse ganhou os casos no teste do bin.

### validação

- `cargo test -p replay -p record -p idestudio`: verde.
- `./tool/tasks/fullcheck.sh` com a perna urnaui (o `worktrees/urna` aponta para o
  checkout irmão), antes e depois, no target da task:
  - antes (a main, com o `git mv` já feito, que não muda contagem): 2277
    passaram (2195 do workspace, 82 da urnaui), 0 falharam, 12 ignorados.
  - depois (os quatro commits de código, ainda sobre 41449e1): 2288
    passaram (2206 do workspace, 82 da urnaui), 0 falharam, 12 ignorados.
  - os 11 a mais: 9 do `rerun` (2 do formato das marcas, 6 do brinquedo, 1
    da a2ppa) e 2 do export.
  - rebaseada na main @ 51394a9: 2349 passaram (2267 do workspace, 82 da
    urnaui), 0 falharam, 12 ignorados, com os 9 do `rerun` e os 35 do
    modelo do `inspect` (eram 33) dentro. a main sozinha, por diferença e
    sem rodar à parte, fica em 2338.
- cada commit de código conferido sozinho (o resto guardado em stash):
  clippy `-D warnings`, fmt e os testes do crate.
- o guard das 639 linhas passa: o bin do `plev-inspect` tem 440; os
  arquivos do harness são de teste.

### arquivos fora do óbvio

- `apps/idestudio/Cargo.toml`: só o comentário do `[dependencies]` (dizia que
  a lib não usa o `tempfile`; agora usa, pelo `lab_repo`). o briefing
  permitia esse arquivo para dev-dependencies; nenhuma dependência mudou.
- `rust/replay/Cargo.toml`: `engine` como dev-dependency, para o teste
  do export gravar frames com ligações de um compositor (o `rec` já
  depende do `engine`); no `Cargo.lock`, essa linha.
- nada em `sem`, `wire`, `workbench`, `views`, `rec`, no formato `MONS`, na
  lib do `inspect`, no `arc.toml`, no README ou no índice de ADRs.

### linha proposta para o índice de ADRs

```text
| 2026-09-24 | [sessions-become-regression-tests](sessions-become-regression-tests.md) | proposed | um trecho de sessão gravada vira teste de regressão: o harness (apps/idestudio/tests/rerun, reexecução controlada, só em teste; nunca no rec nem no inspect) refaz cada ato do trecho na ordem gravada contra uma fixture do app (dispatch, confirm, decline, settle, project, receipt), assenta antes de cada ato menos onde a gravação tinha uma execução em voo, e compara veredito e estágio terminal (numa recusa stale, os reads mudados) e as marcas do desenvolvedor num arquivo de texto ao lado; a fixture do ide sobe o repositório do laboratório num tempdir com o worker do git real; plev-inspect --export grava um trecho como bug report (task-64 fase c) |
```

### para a integração (o que é do orquestrador)

- `arc.toml`, entrada do ide: o repositório do laboratório mora na lib
  (`lab_repo`), e `apps/idestudio/tests/rerun` guarda o harness e as sessões
  que viraram teste.
- README: a linha do `plev-inspect` pode ganhar o `--export`.
- manual: nenhum `docs/ref` trata do inspector; se um passar a tratar,
  o formato das marcas está no ADR.

### limitações

- relógio: o workbench carimba no próprio `Instant` e não recebe relógio;
  injetar pede mexer no workbench, proibido aqui. nada que o harness
  compara lê esse relógio, e os timers dos toasts só andam pelo
  `Workbench::wake`, que a fixture nunca chama. na fixture de brinquedo o
  relógio é o do harness.
- a prova de regressão no ide foi à mão, porque a regra mora no workbench;
  no gate, quem guarda é o brinquedo.
- o que fica frágil está no ADR: ids que carregam valor da execução
  (`stacks.commit[<sha>]`, `notices[<n>]`) viram `not_found` no rerun; o
  estado de partida da fixture tem de ser o do começo do trecho (o harness
  recusa um request cortado, mas não confere o estado); o `settle` do ide
  depende de cada mutação respondida pedir um refresh; parâmetro secreto
  volta como o comprimento; sessão com vários dispatchers passa pelo único
  da fixture; frames não são comparados; o `git` lê a configuração global
  do usuário (a local do repositório fixa identidade e desliga
  assinatura), sem escrever fora do tempdir.
- `acted` e revisão ficam de fora da comparação: a revisão muda com o sha
  do commit inicial (o `Repo::build` carimba a hora), e o olhar do agente
  não está na gravação. fixar a data do commit inicial no `Repo::build`
  deixaria o estado de partida com a mesma revisão numa gravação futura;
  não mudei porque as sessões já gravadas têm o sha antigo.

### o que falta para o showcase

- o showcase não grava sessão: não depende do `rec` (a fase d da task-63
  ligou só o ide). sem sessão da página do todo não há trecho para
  rerodar.
- a fixture da página do todo: o dispatcher da página (os cliques já
  passam pelo `sem`) com o relógio do harness, um `settle` que avance o que
  a página anima, e o `project` dela.
- com a segunda fixture, o harness sai de `apps/idestudio/tests/rerun` para um
  crate de teste (com `arc.toml` e README).

### o que falta para a fase b

- marcar na UI e escrever o arquivo de marcas neste formato (o `marks.rs`
  tem o `render` que escreve um valor de volta).
- exportar da UI pelo mesmo `Session::excerpt` que a CLI usa; o modelo não
  precisa de nada novo para isso.
- mostrar o relatório do rerun ao lado da cadeia causal, quando a fase b
  quiser; hoje ele é texto de teste.

## fechamento da fase c (orquestrador, 2026-09-24)

integrada na `main` em `d611fa4` (cinco commits, rebaseados sem
conflito). o ADR sessions-become-regression-tests foi aceito.

validação do orquestrador:
- diff lido (o harness, a fixture do ide, as marcas, o export, o
  `lab_repo` movido para a lib sem mudar o arquivo);
- gate verde no diretório da task: 2349 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- o teste gerado da sessão a2ppa rodou dez vezes seguidas, verde nas dez;
- a prova de que ele pega regressão foi repetida pelo orquestrador: com
  o `changes:staged` tirado dos `reads` do `submit` em `project.rs`, o
  teste falhou com "DIVERGED request 7 ... recorded rejected: stale,
  changes:staged ... rerun accepted", e o arquivo foi restaurado.

severidade do que ficou:
- P3: o `lab_repo` é módulo público da lib do ide (código do
  laboratório no caminho de produção); o relógio do workbench não é
  injetado (nada que o harness compara lê esse relógio);
- P2: o showcase não grava sessão nem tem fixture; com a segunda fixture
  o harness vira crate próprio; a fase b (UI) marca valores e escreve o
  `.marks`.
