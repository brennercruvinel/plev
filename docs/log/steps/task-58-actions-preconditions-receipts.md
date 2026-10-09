---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-23
domain: generative-runtime
task: task-58
wave: 2
depends: [task-57]
branch: task/58-sem-dispatch
integrated: 20c601e
---

# task-58: ação com precondição, dispatch e receipts

## fechamento (2026-09-23)

integrada na `main` em `20c601e` (dez commits, de `83562b9` a `20c601e`),
validada pelo orquestrador: diff inteiro lido, `./tool/tasks/fullcheck.sh` verde na
branch (54 suítes, 1608 testes no workspace, 63 a mais que a task-57), 141
testes do `sem` com serde e 128 sem, os testes 1 a 10 da spec, goldens
JSON de `Request`, `Receipt` e `Snapshot`. `cargo run -p sem --example
commit` roda o cenário da tese. ADR: `docs/adr/semantics/0033-actions-carry-observed-state.md`.

a entrega voltou uma vez ao agente, com duas correções de contrato feitas
antes de haver consumidor:

- uma pessoa confirma pela tela ou por tecnologia assistiva:
  `Dispatcher::confirm` aceita `Actor::Human` e `Actor::Assistive` e recusa
  agente e teste (`SemError::ConfirmByMachine`). a regra antiga ("só
  `Actor::Human`") era um erro da spec do orquestrador e quebrava o
  contrato de accessibility.
- `Operable::execute(&mut self, id: RequestId, req: &Request)`: a execução
  assíncrona sabe o próprio id, sem canal paralelo (a task-62 usa isso no
  worker do git).

o que ficou diferente da spec, aceito como contrato:

- `Receipt.validated_against` é `Option<Revision>`.
- `Rejection::InvalidProjection { errors }`, só do dispatcher, quando a
  projeção do app não valida.
- `SemError` ganhou `UnknownAction`, `UnknownRequest`, `IllegalTransition`
  e `ConfirmByMachine`.
- `confirm` checa de novo contra o estado vivo antes de executar.
- regras de parâmetro: sem `ParamSpec` não aceita parâmetro; com spec, o
  parâmetro é obrigatório e do tipo declarado; com `constrained`, aplica
  editable, required, range, max_len e options, nessa ordem; `step` não é
  validado.
- `RequestId` trafega como texto decimal; `stages` como pares
  `[estágio, millis]`.

limitações que seguem:

- não há recusa nem expiração de confirmação; receipts abertos não têm
  limite.
- o parâmetro trafega cru no request e no receipt: quem manda receipt para
  fora do processo (61, 63) redige parâmetro de campo secreto.
- o range de um `Int` é comparado como f64 (acima de 2^53 arredonda).
- o `Dispatcher` não é `Send` (sinks são `Box<dyn ReceiptSink>`).
- `Constraints.editable` é `false` por padrão: um nó com parâmetro
  `constrained` declara `editable: true`.

## objetivo

toda ação sobre uma capacidade passa a carregar o estado que o ator
observou, é validada contra o estado atual no momento da execução, e
produz um receipt com o ciclo explícito (requested, accepted ou rejected,
executing, completed ou failed, result). um único `Dispatcher` serve
humano, agente, accessibility e teste: não existem quatro caminhos.

## ligação com a tese

seções 8 (agir sobre estado conhecido), 9 (action receipts, invalidação
semântica), 10 (humano e agente compartilham o software), 30 (quais
condições tornam as ações válidas, quem está autorizado, o que aconteceu).

## estado atual no código

- o commit do ide sai do clique direto para o git, sem conferir nada:
  `apps/idestudio/src/interfaces/panels/layout/input.rs:47` chama `submit_commit`
  (`:259-267`), que empurra `UiRequest::Commit { message }`
  (`apps/idestudio/src/interfaces/panels/layout/mod.rs:115-123`), que vira
  `GitCommand::Commit` (`apps/idestudio/src/repository/git.rs:76`), que roda
  `git commit --message` (`rust/gitdrv/src/repo.rs:187`). o index é
  commitado como estiver no instante em que o worker executa.
- a execução é assíncrona: o resultado volta como `GitEvent::Committed`
  (`apps/idestudio/src/repository/git.rs:132`). o receipt precisa suportar execução
  que começa agora e termina depois.
- o action handler da accessibility só faz log
  (`rust/engine/src/runtime/window/mod.rs:221`).
- depois desta task nada disso muda ainda: a ligação com o ide é da
  task-62 e a com a accessibility é da task-60. esta task entrega o
  contrato e o dispatcher, provados com um app de brinquedo nos testes.

## o que a task-57 entregou e muda aqui (revisão do orquestrador, 2026-09-23)

a task-57 está na `main` (`860b029`); o fechamento está em
`docs/log/steps/task-57-semantic-contract.md`. leia antes
de começar. o que isso muda nesta task:

- `Change` já existe no `sem` (doze variantes, `Snapshot::diff`). o delta
  de `Rejection::Stale` é feito de `Change::StateChanged`, um por `read`
  cujo valor observado difere do atual, com `before` = o observado,
  `after` = o atual, e `added`/`removed` calculados como o `diff` já
  calcula (reuse a função interna do módulo `diff`; não escreva uma
  segunda).
- `Policy.actors` é `Actors { human, agent, assistive, test }`. o
  mapeamento de `Actor` é direto: `Human` -> `human`, `Agent { .. }` ->
  `agent`, `Assistive` -> `assistive`, `Test { .. }` -> `test`.
- `Confirm::ForAgents` pede confirmação só para `Actor::Agent`;
  `Confirm::Always` pede para qualquer ator. só `Actor::Human` confirma.
- `Disabled` vale quando o nó (`Node.enabled`) ou a ação
  (`ActionDecl.enabled`) está desabilitado.
- a revisão (`Observation.revision`) é registrada no request e no receipt
  para rastreio; ela nunca decide `Stale`. a revisão muda com qualquer
  estado projetado (outro item da lista), e só as `reads` da ação contam.
- `Request::observe` aceita alvo e nome de ação tipados ou como texto, do
  mesmo jeito que o builder (`impl TryInto<SemId, Error: Into<SemError>>`),
  e devolve `Err` se a ação não existe no snapshot visto ou se uma `read`
  não tem valor lá.
- campo `secret`: o snapshot guarda só o tamanho, então uma ação que lê um
  campo secreto só detecta mudança de tamanho. é a escolha certa para
  privacidade (nenhum hash de senha sai do processo); fica escrito no ADR
  e no doc da função.
- `serde_json` (já em `[workspace.dependencies]`) pode entrar como
  dev-dependency do `sem`, para um golden da forma JSON de `Request`,
  `Receipt` e `Snapshot`. é a única dependência nova permitida.

## escopo

### tipos

```rust
pub enum Actor {
    Human,
    Agent { name: String },
    Assistive,
    Test { name: String },
}

pub struct Observation {
    pub revision: Revision,
    pub values: Vec<(StateRef, Option<Value>)>,
}

pub struct Request {
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
    pub observed: Observation,
}

impl Request {
    /// lê as `reads` da ação no snapshot que o ator viu e guarda os valores
    pub fn observe(
        seen: &Snapshot,
        actor: Actor,
        target: SemId,
        action: ActionName,
        param: Option<Value>,
    ) -> Result<Request, SemError>;
}

pub enum Verdict {
    Accepted,
    NeedsConfirmation,
    Rejected(Rejection),
}

pub enum Rejection {
    NotFound { target: SemId },
    UnknownAction { target: SemId, action: ActionName },
    NotAuthorized { actor: Actor },
    Stale { changes: Vec<Change> },
    Inert { target: SemId },
    Disabled { target: SemId, action: ActionName },
    InvalidParam { reason: String },
}

pub fn check(req: &Request, now: &Snapshot) -> Verdict;
```

### ordem de validação (parte do contrato, testada)

1. `NotFound`: o alvo não existe no snapshot atual.
2. `UnknownAction`: o alvo não declara a ação.
3. `NotAuthorized`: a `Policy` não permite esse ator.
4. `Stale`: algum valor em `observed.values` difere do valor atual da
   mesma `StateRef`. o delta usa o `Change` da task-57 e contém só as
   `reads` da ação. vem antes de inert e disabled porque o delta explica
   por que o resto mudou.
5. `Inert`: o nó está inerte (modal aberto, bloqueado).
6. `Disabled`: o nó ou a ação estão desabilitados.
7. `InvalidParam`: o parâmetro viola as `Constraints` (tipo, range,
   `max_len`, `options`, `required`, `editable`).
8. `NeedsConfirmation`: a policy exige confirmação humana para esse ator.
9. `Accepted`.

a comparação de estado olha só as `reads` da ação. uma chave que a ação
não lê nunca causa `Stale`: essa é a invalidação semântica.

### receipt

```rust
pub struct RequestId(u64);

pub enum Stage {
    Requested,
    Accepted,
    AwaitingConfirmation,
    Confirmed { by: Actor },
    Rejected(Rejection),
    Executing,
    Completed(Outcome),
    Failed { error: String },
}

pub struct Outcome {
    pub result: IndexMap<String, Value>,
}

pub struct Receipt {
    pub id: RequestId,
    pub request: Request,
    pub validated_against: Revision,
    pub stages: Vec<(Stage, Millis)>,
}
```

transições legais:

```text
Requested -> Accepted | Rejected | AwaitingConfirmation
AwaitingConfirmation -> Confirmed | Rejected
Accepted | Confirmed -> Executing
Executing -> Completed | Failed | Rejected   (rejeição no ponto de verdade)
terminais: Rejected, Completed, Failed
```

transição ilegal é `SemError::IllegalTransition`, nunca panic. o tempo
(`Millis`, u64 monotônico) é injetado por quem chama: `sem` nunca lê o
relógio, o que mantém teste e replay determinísticos.

### o caminho único

```rust
pub trait Operable: Semantics {
    fn execute(&mut self, req: &Request) -> Execution;
}

pub enum Execution {
    Done(Outcome),
    Started,                 // assíncrono; o app chama complete/fail depois
    Failed(String),
    Rejected(Rejection),     // o ponto de verdade discordou (ex.: o index do git)
}

pub trait ReceiptSink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage);
}

impl Dispatcher {
    pub fn new() -> Self;
    pub fn dispatch<A: Operable>(&mut self, app: &mut A, req: Request, now: Millis) -> RequestId;
    pub fn confirm<A: Operable>(&mut self, app: &mut A, id: RequestId, by: Actor, now: Millis) -> Result<(), SemError>;
    pub fn complete(&mut self, id: RequestId, outcome: Outcome, now: Millis) -> Result<(), SemError>;
    pub fn fail(&mut self, id: RequestId, error: String, now: Millis) -> Result<(), SemError>;
    pub fn reject_at_truth(&mut self, id: RequestId, rejection: Rejection, now: Millis) -> Result<(), SemError>;
    pub fn receipt(&self, id: RequestId) -> Option<&Receipt>;
    pub fn add_sink(&mut self, sink: Box<dyn ReceiptSink>);
}
```

`dispatch` projeta um snapshot novo do app naquele instante
(`app.project`), roda `check` contra ele, e só então chama `execute`. a
validação é sempre contra o estado atual, nunca contra o último frame
desenhado.

validação em dois pontos: quando a verdade mora fora do processo de UI (o
index do git, um servidor), `execute` leva a precondição até lá e o ponto
de verdade confere de novo antes do efeito. se discordar, o receipt vai de
`Executing` para `Rejected(Stale)` com o delta visto lá. a task-62 usa isso
no worker do git.

pedido humano usa o mesmo caminho: o clique vira `Request::observe` sobre
o snapshot que o humano viu (o do frame corrente) com `Actor::Human`.

os receipts abertos ficam no dispatcher; os terminados saem para os sinks
e são descartados depois de um limite configurável (sem crescimento
ilimitado).

## fora de escopo

ligar o dispatcher em comps ou apps (task-59), accessibility (task-60),
socket (task-61), ide (task-62), gravação (task-63), ação composta
(task-70).

## dependências

task-57 integrada na `main`.

## superfície sob responsabilidade

`rust/accord/**` (módulos novos: `request`, `check`, `receipt`,
`dispatch`). mudança nos tipos da task-57 só com aprovação do orquestrador,
e reportada no handoff.

## interfaces que consome

`SemId`, `Snapshot`, `Revision`, `Change`, `ActionDecl`, `Policy`,
`Constraints`, `Semantics` (task-57).

## interfaces que pode alterar

apenas as novas desta task.

## contrato exposto

tipos e ordem de validação acima.

invariantes:

1. `check` é pura: mesma entrada, mesmo veredito.
2. mudança numa chave fora de `reads` nunca produz `Stale` (propriedade
   testada com proptest).
3. `Stale.changes` contém só `StateRef`s de `reads`.
4. o receipt só percorre transições legais; o tempo de cada estágio é o
   que o chamador passou.
5. `execute` só é chamado depois de `Accepted` ou `Confirmed`.
6. um `Request` de replay não existe: nenhum `Actor` representa o player,
   e a task-63 nunca cria request.

ownership: o dispatcher é dono dos receipts abertos; o app é dono do
estado e da execução.

error model: `Verdict` para decisão de negócio; `SemError` para uso
errado da API (id desconhecido, transição ilegal).

compatibilidade: a representação serde de `Request`, `Receipt` e
`Rejection` é contrato com as tasks 61 e 63.

## critérios de aceitação

- os testes abaixo, `cargo test -p sem`, clippy, wasm check do crate,
  `./tool/tasks/fullcheck.sh`.
- um exemplo (`demo/commit/src/main.rs` ou doc test) que roda o
  cenário da tese e imprime os receipts.
- ADR proposto: `docs/adr/semantics/0033-actions-carry-observed-state.md`.

## testes esperados

com um app de brinquedo em memória (estado estilo git: arquivos staged,
mensagem, commits feitos):

1. o cenário da tese: o agente observa `{A,B,C}` e "fix login"; o humano
   tira B pelo dispatcher (`Actor::Human`, aceito); o pedido do agente
   volta `Rejected(Stale)` com `removed: [B]`; nenhum commit acontece.
2. o agente observa de novo, pede de novo, é aceito, e o receipt termina
   em `Completed` com `commit` e `files` no resultado.
3. mudança numa chave que a ação não lê (o cursor da lista): aceito.
4. `NotFound`, `UnknownAction`, `NotAuthorized`, `Inert`, `Disabled`,
   `InvalidParam`, cada um isolado, e a ordem quando dois valem juntos.
5. `NeedsConfirmation` para agente em ação destrutiva; confirmação humana
   leva a `Executing`; confirmação por agente é recusada.
6. execução assíncrona: `Started`, depois `complete`; e `fail`.
7. rejeição no ponto de verdade: `Execution::Rejected(Stale)` leva o
   receipt de `Executing` a `Rejected`.
8. transições ilegais retornam erro.
9. proptest da invariante 2.
10. um `ReceiptSink` de teste recebe todos os estágios em ordem.

## riscos e perguntas abertas

- policy por ator é grossa (humano, agente, assistive, teste). identidade
  de agente mais fina (qual agente, qual escopo de permissão) fica
  registrada como melhoria, não entra aqui.
- confirmação humana precisa de UI. aqui é só estado do receipt; a UI
  entra na task-62 e na task-69.

## commits esperados

1. `task-58: Actor, Observation e Request::observe`
2. `task-58: check com a ordem de validação e o delta Stale`
3. `task-58: Receipt e as transições legais`
4. `task-58: Operable, Dispatcher e ReceiptSink`
5. `task-58: exemplo do cenário da tese`

## pare e reporte se

- a ordem de validação parecer errada num caso real;
- `Operable` não couber em app com estado distribuído (widgets donos de
  parte do estado);
- for preciso mudar qualquer tipo da task-57.
