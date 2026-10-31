---
id: task-61
title: "task-61: superfície do agente (socket local, observe, act)"
project: plev
type: tarefa
status: concluido
date: 2026-09-23
date_basis: cabecalho
topics: [agentes, semantica]
relations: [task-57, task-57b, task-58, task-59, task-60, task-62, task-63, task-79, adr-0026, adr-0039, tese-runtime]
evidence: [0ec72f9]
source: docs/log/steps/task-61-agent-surface.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-61: superfície do agente (socket local, observe, act)

Abre ao agente um canal de fora para dentro de um app plev: um socket local, crate `wire` (hoje `socket`), expõe `observe` e `act` sobre o contrato semântico, sem screenshot nem coordenada, e cada ação passa pelo mesmo `Dispatcher` da interação humana. A observação sai por escopo e por delta, e um `act` feito sobre uma revisão antiga volta rejeitado com o delta. Integrada em `0ec72f9` (doze commits), com gate verde (1920 testes), servidor desligado por padrão (`PLEV_AGENT=1` em debug) e o todo do showcase exposto pelo socket. A forma JSON verbosa que ela devolveu foi enxugada na task-57b.

## objetivo

um agente de fora do processo observa e opera um app plev pelo contrato
semântico, sem screenshot, sem coordenada, sem acesso direto ao backend do
app. a ação dele passa pelo mesmo `Dispatcher` da interação humana. a
observação é por escopo e por delta, para gastar pouco dado e pouco token.

## ligação com a tese

seções 2 (o agente trabalha sobre a estrutura que a engine usa), 8 e 10
(stale-state rejection, handoff), 25 (o agente não precisa de framebuffer).
do arquivo de concepção, as ideias de implementação: socket local
expondo cena, hit regions, identidades, estados e ações disponíveis em
formato estável; `plev act <id> <evento>` passando pelo mesmo caminho
lógico da interação humana; observações por escopo; deltas; revisão
semântica independente da animação.

## estado atual no código

- nenhum canal de fora para dentro de um app plev.
- o padrão de thread de fundo que acorda o loop existe no urnaui (worker
  com `EventLoopProxy`, ADR `worker-results-wake-the-event-loop`).
- depois das tasks 57, 58 e 59: `Snapshot`, `Revision`, `Change`,
  `subtree`, `Request`, `Receipt`, `Dispatcher`, bindings com bounds.

## fechamento (2026-09-23)

integrada na `main` em `0ec72f9` (doze commits), validada: gate verde
(1920 testes), testes de socket sem falha em 15 execuções seguidas do
orquestrador. crate `wire`: protocolo v1 em JSON lines (observe, act,
confirm, decline; erros tipados), anel de revisões servidas, `AgentPort`
(socket unix 0600 num diretório 0700, desligado por padrão, `PLEV_AGENT=1`
em debug), `plev observe` e `plev act`, e o showcase servindo o todo pelo
mesmo gancho da accessibility, renomeado para `act`. validado pelo agente
contra o showcase em janela: observe, act concluído e o `submit` de uma
revisão velha recusado com o delta. ADR:
[`0039-agents-act-through-the-semantic-contract.md`](../decisoes/semantica/0039-agents-act-through-the-semantic-contract.md).

a entrega voltou uma vez. o gate do orquestrador pegou um teste instável
(`find_names_the_one_live_socket`): no macOS o std marca o socket como
`FD_CLOEXEC` depois de criá-lo, e um teste paralelo que lança processo
herdava o listener "morto". a correção isolou os testes que lançam
processo num binário próprio e, no caminho, achou um defeito real em
`port.rs`: ao terminar a leitura, a conexão era cortada antes do escritor
mandar o que estava na fila, e um cliente que manda e fecha o lado dele
perdia a resposta. corrigido na origem (a conexão é esquecida e o
escritor a encerra depois da última linha; `WRITE_STALL` de 10 s para quem
não lê).

aceitos: dois arquivos fora da lista (`TodoPage::add_sink`, porque o
dispatcher do todo é privado ao módulo, e o rename nos testes
assistivos). limitações que seguem: o nome do ator é declarado, não
autenticado (qualquer processo do mesmo usuário conecta com qualquer
nome); um processo morto por sinal deixa o arquivo do socket; e o
mecanismo de herança de descritor existe em produção, de forma benigna
(documentado no ADR). a forma JSON verbosa e a redação duplicada viraram
a task-57b.

## revisão do orquestrador (2026-09-23, antes do briefing)

- nome do crate: `wire`. os tipos do protocolo são puros (compilam em
  wasm); o servidor (`AgentPort`) e o cliente ficam atrás de `cfg(unix)`;
  o binário `plev` é um bin do mesmo crate (`cargo run -p socket --bin
  plev`).
- o shell do showcase já tem, da task-60, os ganchos da página ativa:
  `ShowcaseView::semantics()` (projeta) e `assist(req)` (despacha um
  `Request` no dispatcher da página). o agente usa os mesmos ganchos, com
  `Actor::Agent`; o gancho ganha um nome neutro (serve qualquer ator), sem
  segundo caminho.
- estágios que terminam depois (execução assíncrona, como o commit do
  ide) chegam à conexão por um `ReceiptSink` que a porta registra no
  dispatcher; a porta encaminha só os estágios dos requests que ela mesma
  emitiu. o `Workbench` do ide não é `Send`: tudo que toca dispatcher
  roda na thread da UI, no `drain`.
- o ide abre a porta na task-62 fase c, não aqui.
- a gravação (`rec`) ainda não está ligada no showcase; a fase d da 63
  entra depois desta task.

## decisão de arquitetura (recomendada; vira ADR)

1. dois pedaços: um crate puro de protocolo (tipos de mensagem, versão,
   codec JSON lines; nome sugerido `wire`) e um servidor embutível que o
   shell do app segura (`AgentPort`), mais o binário de linha de comando
   `plev` (`plev observe`, `plev act`).
2. transporte: unix domain socket no desktop (macOS e linux), em
   `$TMPDIR/plev/<app>-<pid>.sock`, permissão 0600. named pipe no Windows
   fica para quando o alvo existir. nada em wasm e mobile nesta task.
3. desligado por padrão. liga por variável de ambiente (`PLEV_AGENT=1`)
   ou por flag do app; em build release só se o app optar explicitamente.
   só local, sem rede, sem pareamento remoto (cena remota é da task-79).
4. o agente nunca recebe acesso arbitrário ao runtime: só observe e act
   sobre o contrato. não existe mensagem para rodar código, ler arquivo ou
   chamar backend.
5. threading: a thread do socket só enfileira e acorda o loop; a
   observação (projeção) e o dispatch acontecem na thread principal, no
   próximo tick, e a resposta volta pela fila.

## escopo

### protocolo v1 (JSON lines, cada mensagem com `"v": 1` e `"id"`)

- `observe { scope?: SemId, keys?: [StateKey], since?: Revision }` ->
  `snapshot { revision, nodes }` ou `delta { base, revision, changes }`
  quando `since` é conhecido; `bounds` opcionais das ligações.
- `act { target, action, param?, observed }` onde `observed` é explícito
  (valores das `reads`) ou `act { ..., from_revision: R }`: o servidor
  guarda um anel das últimas revisões servidas e monta o `observed` a
  partir da revisão R. revisão fora do anel volta `UnknownRevision` e o
  agente observa de novo.
- resposta de `act`: o stream de estágios do receipt (`accepted`,
  `rejected` com delta, `executing`, `completed` com resultado, `failed`),
  cada estágio como uma mensagem com o mesmo `id`.
- `confirm { request }` só aceito de `Actor::Human`: o agente não confirma
  a própria ação.
- erros de protocolo tipados (`BadVersion`, `BadMessage`,
  `UnknownRevision`, `NotEnabled`).

### CLI

- `plev observe [--scope commit] [--since <rev>] [--app <sock>]`
- `plev act commit.submit submit [--from-revision <rev>] [--param json]`
- saída JSON, legível por agente e por humano; código de saída diferente
  para aceito, rejeitado e erro.

### servidor embutível

`AgentPort::open(app_name) -> Result<AgentPort, ...>`,
`port.drain(&mut app, &mut dispatcher, now)` chamado no tick do shell.
identidade do ator: `Actor::Agent { name }` com o nome que o cliente
declara na conexão.

### prova

o todo app do showcase (ligado na task-59) exposto pelo socket:
observar, agir, ver rejeição por estado velho quando o estado muda entre
observe e act.

## fora de escopo

o ide (task-62 liga o próprio shell no socket), gravação (63), cena e
pixels pelo socket (79), autenticação de agente além do ator declarado.

## dependências

57, 58, 59 integradas.

## superfície sob responsabilidade

- crate novo de protocolo e o binário `plev` (nomes aprovados no
  briefing)
- `apps/uigallery/src/shell/mod.rs`: só a abertura do `AgentPort` e o `drain`
  no tick. atenção: a task-60 também toca esse arquivo na mesma onda; o
  orquestrador sequencia os dois commits ou define os blocos de cada um no
  briefing
- entradas em `Cargo.toml`, `docs/ATLAS.toml`, `README.md`
- proposta de ADR [`0039-agents-act-through-the-semantic-contract.md`](../decisoes/semantica/0039-agents-act-through-the-semantic-contract.md)

## interfaces que consome

`Snapshot`, `Revision`, `Change`, `subtree`, `Request`, `Receipt`,
`Dispatcher`, `ReceiptSink` (57, 58); `Compositor::bindings` (59).

## interfaces que pode alterar

as próprias, novas. o protocolo v1 é contrato público: mudança exige ADR
e nova versão.

## critérios de aceitação

- com o showcase rodando em debug com `PLEV_AGENT=1`: `plev observe
  --scope todo` devolve o snapshot; `plev act todo.add activate
  --from-revision R` adiciona o item; mudar o input na janela entre o
  observe e o act faz o act voltar rejeitado com o delta.
- observe com `since` devolve só o delta.
- medir e registrar: bytes por observação completa e por delta na página
  do todo; tempo de ida e volta de um act.
- sem o env var, nenhum socket é criado (teste).
- `./tool/tasks/fullcheck.sh` verde; o crate de protocolo compila em wasm (é puro),
  o servidor e o CLI só no desktop.

## testes esperados

codec (roundtrip de cada mensagem, versão errada, JSON inválido), anel de
revisões (hit, evicção, `UnknownRevision`), servidor com um app de
brinquedo em processo (sem socket real) e um teste de integração com
socket real num tempdir.

## riscos e perguntas abertas

- JSON lines contra binário: JSON primeiro porque é o que agente lê; o
  custo em bytes é medido e o ADR registra.
- várias conexões ao mesmo tempo (dois agentes): permitido, cada uma com
  o próprio nome de ator; o dispatcher é um só.

## commits esperados

1. `task-61: crate de protocolo com mensagens v1 e codec`
2. `task-61: anel de revisões e observed a partir de revisão`
3. `task-61: AgentPort com thread de socket e drain no tick`
4. `task-61: binário plev observe e act`
5. `task-61: showcase expõe o todo pelo socket`

## pare e reporte se

- o formato de `Snapshot` em JSON sair grande demais para uso real;
- for preciso mexer em `sem`;
- aparecer necessidade de ação que não cabe em observe e act.
