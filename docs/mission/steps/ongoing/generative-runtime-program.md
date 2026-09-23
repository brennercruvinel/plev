---
project: plev
audience: [ai-agents, contributors]
type: program
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
base: plevrev b8ba096 (gate green 2026-09-23)
tags: [program, goals, dependencies, contracts, ownership, waves]
---

# programa: de GPU UI engine a generative interaction runtime

a tese está em `generative-runtime-thesis.md`. as regras de trabalho estão
em `generative-runtime-guidelines.md` e são obrigatórias para o
orquestrador e para todo agente. este arquivo é o mapa executável: o goal,
o estado real do código, o grafo de dependências, a ordem de integração,
os contratos entre tasks, o ownership das superfícies compartilhadas e os
pontos de decisão.

## 1. goal

transformar o plev de GPU UI engine em generative interaction runtime.

1. a interface deixa de ser uma coleção fixa de telas e passa a ser
   materializada a partir de intenção, estado, capacidades e restrições.
2. a interface permanece semanticamente compreensível durante todo o
   runtime. todo controle possui identidade persistente, significado,
   estado, ações e constraints, independentemente de como esteja sendo
   renderizado.
3. existe uma representação intermediária própria e verificável, o plev
   IR. humanos, LLMs, importers e ferramentas produzem IR; o plev valida e
   materializa. o modelo não ganha acesso arbitrário ao runtime.
4. humanos, agentes, accessibility e testes operam o mesmo contrato de
   aplicação. não existem quatro interfaces paralelas para a mesma coisa.
5. a colaboração humano-agente é segura em presença de estado concorrente.
   observações podem envelhecer; ações carregam precondições, são validadas
   contra o estado atual e produzem resultados verificáveis.
6. toda interação é inspecionável e causalmente rastreável: quem observou,
   quem agiu, sobre qual estado, com quais condições e qual foi o
   resultado. o monster como flight recorder entra aqui.
7. o runtime nativo é realmente adequado a interfaces dinâmicas:
   GPU-first, incremental, headless, multiplataforma, com texto,
   accessibility e integrações de sistema sólidas o suficiente para
   aplicações reais.
8. interfaces são construídas, transformadas e recompostas em runtime. o
   sistema gera uma superfície nova para uma intenção específica ou
   modifica uma existente sem perder identidade semântica.
9. o plev é compiler target para interfaces. LLM, Figma, React, outras DSLs
   ou ferramentas convergem para plev IR sem obrigar o runtime a carregar
   os modelos de execução de origem.
10. provar que esse modelo produz software qualitativamente diferente. não
    apenas benchmarks: demonstrar interfaces efêmeras, human-agent handoff,
    stale-state rejection, inspectability e UI generativa em aplicações
    reais.

## 2. mantra para todos os agentes

> o plev deve permitir que uma capacidade mantenha identidade e semântica
> enquanto sua representação visual pode nascer, desaparecer ou se
> transformar em tempo real.

> criar um sistema de dependências incremental independente da
> representação visual, capaz de propagar invalidação mínima entre estado,
> semântica, layout, scene construction, hit testing e rendering.

todo briefing começa com essas duas frases. toda entrega é julgada contra
elas.

## 3. estado real do código (diagnóstico, 2026-09-23)

base: branch `plevrev` em `b8ba096`, dois commits à frente de `main`
(`c89d11e`), com o hot reload. gate completo verde nesta base (test,
clippy -D warnings, fmt, wasm check, perna urnaui).

o que a tese precisa e o que existe hoje:

| tese | o que existe | onde | lacuna |
|---|---|---|---|
| identidade lógica persistente | `ViewId(pub u64)` gerado por contador que volta a zero a cada frame, na ordem de emissão | `crates/engine/src/input/mod.rs:50-61`, `crates/engine/src/input/types.rs:134` | identidade é posicional; não existe identidade lógica em lugar nenhum |
| identidade temporária de render e posição | `HitRegion { view_id, x, y, w, h, focusable, layer_visible, layer_opacity }` | `crates/engine/src/input/types.rs:137-146` | sem ligação com significado; comps e apps fazem hit test próprio por `Rect::contains` (ex.: `crates/ide/src/views/commit_form.rs`), então `InputState` nem é o ponto de verdade do hit test |
| significado, papel, estado | nada no engine; cada widget guarda estado privado | `crates/comps/src/*` | nenhum widget declara o que é |
| accessibility do mesmo contrato | árvore AccessKit derivada de hit regions: role chutada por `focusable`, label sempre `None` | `crates/engine/src/window/render.rs:99-125` | geometria no lugar de significado |
| ação vinda da accessibility | `PlevActionHandler::do_action` só faz log | `crates/engine/src/window/mod.rs:221` | nenhuma ação chega ao app |
| accessibility nos apps que shipam | o caminho acima vive em `engine::window::App`; showcase, ide e urnaui usam o próprio `App` e o `present_frame` compartilhado | `crates/showcase/src/app.rs:367`, `crates/ide/src/main.rs:466`, `crates/urnaui/src/app.rs:432` | nenhum app que shipa expõe árvore de accessibility |
| ações nomeadas | registry estilo Zed, `"namespace::Name"`, keymap em JSON | `crates/engine/src/actions/mod.rs`, `crates/ide/src/actions.rs` (`commit::Submit`) | ligadas a tecla, não a capacidade com estado e precondição |
| ação com precondição | o commit do ide vai do clique ao git sem conferir nada | `crates/ide/src/views/workspace/input.rs:47` e `:259-267` (`submit_commit`), `crates/ide/src/views/workspace/mod.rs:115-123` (`UiRequest`), `crates/ide/src/session.rs:76` (`GitCommand::Commit`), `crates/git/src/repo.rs:187` (`git commit --message`) | o cenário da tese acontece hoje: o index é commitado como estiver no instante em que o worker executa |
| estado semântico versus visual no ide | `FileEntry.staged` (vai no commit) e `UnassignedView.selected_idx` (linha em destaque, não vai) | `crates/ide/src/views/unassigned_view.rs:51-61` | os dois são campos soltos; nada diz que o primeiro invalida um commit e o segundo não |
| reconciliação otimista | o ide atualiza a view e recarrega tudo depois de cada mutação | `crates/ide/src/session.rs:118-138` | a view pode divergir do index por um instante; a precondição precisa ser checada também no ponto de verdade (o worker) |
| flight recorder | monster: codec com IR próprio desacoplado de `SceneNode`, container v1 congelado com sha256 por seção, discover (modo B), player determinístico | `crates/monster/src/lib.rs`, `docs/ADR/monster-format-v0.md` | grava animação, não sessão: faltam eventos, semântica, recursos, ações, origem e resultados |
| compiler target | parser parse-resolve-emit com IR próprio (`ParserNode`) que emite código builder e droplist com arquivo:linha | `crates/parser/src/ir.rs`, `crates/parser/src/lib.rs` | emite código Rust, não IR de runtime; sem semântica; marcado experimental |
| pixel para código | hot reload de literal: skeleton + lista de literais por arquivo via proc-macro2 | `crates/engine/src/hot_reload/literals.rs`, `crates/engine/Cargo.toml:84` | sem posição (proc-macro2 sem `span-locations`); só no sentido código para pixel |
| headless | `text_probe`: wgpu offscreen para png | `crates/engine/src/text/probe.rs`, `crates/engine/examples/text_probe` | só texto; não generalizado para página, tema, viewport ou frame de monster |
| testes estruturais | hash por layer para dirty tracking | `crates/engine/src/compositor/layer/mod.rs:43` | hash não é exportado nem nomeado para regressão |
| incrementalidade | cena reconstruída por frame; layer limpa custa zero GPU; `TextMeasurer` com cache | `docs/arc/arc.toml` (frame-flow), `crates/engine/src/compositor/layer/mod.rs` | layout, shaping, hit test, scene build e style resolution refeitos por frame; task-39 (memoização) parada desde 2023 |
| texto | cosmic-text 0.18, fontes embutidas só Inclusive Sans, Inter e codicons; rope com transações, multi-cursor e history; editor em comps; `ImeState` básico | `crates/engine/src/text/fonts.rs:38-52`, `crates/rope/src`, `crates/comps/src/editor`, `crates/engine/src/platform/ime.rs` | árabe, CJK e emoji caem em fallback não embutido; IME e bidi sem cobertura de teste |
| integração de plataforma | clipboard (arboard no desktop, local no mobile e wasm), diálogos nativos e drop de arquivo no urnaui, lifecycle, safe areas | `crates/engine/src/clipboard.rs`, `crates/urnaui/src/app.rs`, `crates/engine/src/platform` | sem menus, tray, notificações, múltiplas janelas, device loss, deep links, câmera, microfone, impressão |
| determinismo | contratos srgb, raster de glifo determinístico, fontes embutidas, web igual ao desktop | `README.md` (contracts), `docs/ADR/glyph-raster-identity-and-atlas-isolation.md`, tasks 18, 19 e 20 em `ongoing/` | linux e windows pendentes; sem harness que classifique divergência |
| padrão de app | estado em structs, widgets retained, invalidação explícita; builder suportado para protótipo; view, component, signal e narrate experimentais | `docs/ADR/official-app-pattern.md` | superfície materializada a partir de IR não é nenhum dos padrões atuais: precisa de ADR |

## 4. tasks

| task | nome | onda | depende de |
|---|---|---|---|
| task-57 | contrato semântico (crate `sem`) | 1 | nenhuma |
| task-58 | ação com precondição, dispatch e receipts | 2 | 57 |
| task-59 | ligação semântica com o render e um caminho operacional | 3 | 57, 58 |
| task-60 | accessibility como projeção do contrato | 4 | 57, 58, 59 |
| task-61 | superfície do agente (socket local, observe, act) | 4 | 57, 58, 59 |
| task-62 | prova no ide: stale-state rejection e handoff | 4 | 57, 58, 59 (fase c: 61, 63) |
| task-63 | flight recorder de sessão | 4 | 57, 58 |
| task-64 | inspector de sessão, bug report e sessão para teste | 5 | 63, 66 (link de código: 65) |
| task-65 | pixel para código e inspector de desenvolvimento | 5 | 57, 59 |
| task-66 | headless como primeira classe | 3 | nenhuma (saída semântica: 57) |
| task-67 | regressão estrutural antes de pixel | 5 | 59, 66 |
| task-68 | plev IR e verifier | 6 | 57, 58 |
| task-69 | superfícies efêmeras e geradas | 6 | 68, 59 (fase 0 em refs/ sem dependência) |
| task-70 | transformações adaptativas | 7 | 68, 69, 63 |
| task-71 | plev como compiler target (importers para IR) | 7 | 68 |
| task-72 | UI asset compiler | trilha | monster (existente); espera 63 liberar o container |
| task-73 | sistema de dependências incremental | trilha (medição na onda 3) | medição: nenhuma; desenho: 57, 59 |
| task-74 | texto e edição | trilha | independente; accessibility de texto: 60 |
| task-75 | accessibility de plataforma production-ready | trilha | 60 |
| task-76 | integração de plataforma | trilha | independente; itens como capability: 58 |
| task-77 | determinismo visual | trilha | 66 |
| task-78 | aplicações AI-native | trilha | 73 (streaming), 66 |
| task-79 | cena por socket | trilha | 61 (transporte), 66 |
| task-80 | prova de software qualitativamente diferente | 8 | 62, 64, 69, 70 |

## 5. grafo de dependências

```text
57 contrato semântico
 └─ 58 precondição + dispatch + receipts
     ├─ 59 ligação com render + caminho operacional único
     │   ├─ 60 accessibility ──────────────── 75 a11y de plataforma
     │   ├─ 61 superfície do agente ────────── 79 cena por socket
     │   ├─ 62 prova no ide ◄── 61, 63 (fase c)
     │   ├─ 65 pixel para código
     │   └─ 67 regressão estrutural ◄── 66
     ├─ 63 flight recorder
     │   └─ 64 inspector + sessão para teste ◄── 66, 65
     └─ 68 plev IR + verifier
         ├─ 69 superfícies efêmeras e geradas
         │   └─ 70 transformações adaptativas ◄── 63
         └─ 71 compiler target

66 headless (independente) ── 67, 64, 77, 78, 79
73 incrementalidade (medição independente; desenho depois de 57 e 59)
72 asset compiler (espera o container do monster ficar livre de 63)
74 texto, 76 plataforma (trilhas longas, independentes)
80 prova final ◄── 62, 64, 69, 70
```

## 6. ondas de integração

uma onda só começa quando a anterior está integrada na `main` e o gate
está verde depois do merge. dentro de uma onda, tasks em paralelo só
quando as superfícies (seção 8) são disjuntas.

- onda 0, orquestrador, sem agente: integrar `plevrev` na `main`;
  reconstruir os `docs/how-to/` a partir da nova missão; aplicar em
  `.contracts/.agents/AGENTS.md` e no rust-conventions a regra de commit
  por branch de task; registrar estes documentos.
- onda 1: task-57, sozinha. é o contrato de que tudo depende.
- onda 2: task-58, sozinha. estende o mesmo crate.
- onda 3: task-59 (fase a) em paralelo com task-66 (superfícies
  disjuntas: tabela de ligação no compositor + três widgets do comps + o
  todo do showcase; construtor headless do `GpuContext` + `window/frame.rs`
  + módulo headless + probe + exemplos + CLI do showcase). a fase de
  medição da task-73 roda logo depois das duas, porque a instrumentação
  toca os arquivos que elas possuem nesta onda (decisão do orquestrador,
  2026-09-23). a fase b da 59 roda depois da validação da fase a.
- onda 4: task-62 (fases a e b), task-60, task-61 e task-63 em paralelo,
  com as superfícies da seção 8. task-60 liga a accessibility primeiro no
  showcase; o ide entra depois da integração da 62.
- checkpoint a (seção 9) depois da fase c da task-62.
- onda 5: task-64, task-65, task-67.
- onda 6: task-68, depois task-69. a fase 0 da task-69 (spike descartável
  em `refs/`) pode rodar a partir da onda 4, porque não integra nada.
- checkpoint b depois da task-69.
- onda 7: task-70, task-71.
- onda 8: task-80.
- trilhas (72, 74, 75, 76, 77, 78, 79): liberadas pelo orquestrador quando
  a dependência está integrada e a superfície está livre. são longas e não
  bloqueiam o núcleo.

## 7. contratos entre tasks

cada linha é uma interface que um goal expõe para outro. o consumidor
depende do contrato escrito na task produtora, nunca da implementação
interna. mudança em qualquer um deles passa pelo orquestrador.

| contrato | produtor | consumidores | onde fica |
|---|---|---|---|
| `SemId`, `Role`, `Value`, `StateRef`, `Node`, `ActionDecl`, `Policy`, `Constraints`, `Snapshot`, `Revision`, `SnapshotBuilder`, `Semantics`, `Change` (diff) | 57 | 58 a 71, 73 | `crates/sem` |
| `Actor`, `Request`, `Observation`, `Verdict`, `Rejection`, `Receipt`, `Stage`, `Outcome`, `check`, `Operable`, `Dispatcher` | 58 | 59, 60, 61, 62, 63, 68, 69, 70 | `crates/sem` |
| ligação por frame `SemId -> (ViewId, bounds, layer, clip)` e o caminho único de dispatch em comps e apps | 59 | 60, 61, 62, 65, 67, 69 | `crates/engine` (compositor e input), `crates/comps` |
| projeção AccessKit a partir de `Snapshot` + ligação | 60 | 75, 74 | `crates/engine/src/accessibility` |
| protocolo de observação e ação (versionado) | 61 | 62, 79, 80 | crate novo (nome na task) |
| formato de sessão e API de gravação | 63 | 64, 70, 80 | `crates/monster` (perfil de sessão) ou crate irmão, decidido em ADR |
| API de render headless | 66 | 64, 67, 77, 78, 79 | `crates/engine` |
| plev IR + `VerifiedIr` + registro de capabilities | 68 | 69, 70, 71 | crate novo (nome na task) |
| host de superfícies materializadas | 69 | 70, 80 | `crates/comps` ou crate novo, decidido em ADR |

## 8. superfícies compartilhadas e ownership

| superfície | dono enquanto a task roda | regra |
|---|---|---|
| `crates/sem` | 57, depois 58 | depois da onda 2, só com aprovação do orquestrador |
| `crates/engine/src/input`, `crates/engine/src/compositor` (tabela de ligação) | 59 | ninguém mais toca na onda 3 |
| `crates/comps` (declaração semântica dos widgets) | 59 | 69 consome, não altera sem aprovação |
| `crates/engine/src/accessibility`, `crates/engine/src/window/{mod,render,lifecycle}.rs` | 60 | o caminho antigo derivado de hit region é substituído, não mantido em paralelo |
| `crates/ide`, `crates/git` | 62 | 60 só liga o ide depois da integração da 62 |
| `crates/monster/src/container.rs` e formato | 63 | 72 espera; o golden v1 (`fixtures/golden_v0_minimal.monster`) não muda |
| `crates/engine/src/hot_reload` | 65 | só `#[cfg(hot_reload)]` |
| `crates/engine/src/theme` | ninguém sem aprovação | tocado por 65 (leitura de token) só depois de proposta aprovada |
| `crates/engine/src/text` | 74 | 66 toca só `probe.rs` e o novo módulo headless |
| `crates/parser` | 71 | |
| `Cargo.toml` (members e workspace deps), `docs/arc/arc.toml`, `README.md` | orquestrador | agente que cria crate adiciona a própria entrada (o `arc_sync_guard` exige) e o orquestrador revisa |
| `.contracts/`, `docs/ADR/README.md`, `docs/mission/` | orquestrador | agente propõe no handoff |

## 9. checkpoints e critérios de parada

o plano é desenhado para falhar barato. cada checkpoint é uma decisão
explícita do brenner com o orquestrador, registrada neste arquivo.

- checkpoint a, depois da task-62 fase c: um agente de verdade opera o ide
  pela superfície da task-61 (sem screenshot, sem git direto) enquanto o
  brenner usa o app ao mesmo tempo. pergunta: isso é qualitativamente
  diferente de agente clicando em pixel ou lendo um snapshot de
  accessibility de terceiros? se não for, parar e repensar antes de
  investir no IR.
- checkpoint b, depois da task-69: uma superfície gerada (revisão de
  silêncios) contra uma tela fixa bem desenhada para a mesma tarefa. se a
  gerada for pior para quem usa, a geração volta a ser hipótese e o
  investimento vai para transformação adaptativa (70) sobre telas fixas.
- checkpoint c, task-80: a pergunta do núcleo. quando o brenner abrir o
  primeiro `.monster` de uma delegação real, ele consegue descobrir o que
  precisa para confiar nela?

## 10. métricas que o programa carrega

nenhum número de orçamento é inventado antes da medição. cada task que
mede registra a baseline no próprio arquivo e no ADR.

- experimento do ide (62, 80): sucesso, tempo, volume de dados e tokens
  enviados ao agente, recuperação de erro, custo da gravação na engine.
  comparação contra screenshot e contra uma alternativa estruturada
  existente (snapshot de acessibilidade no estilo do Playwright MCP); se
  não houver comparação justa, a limitação é registrada, não disfarçada.
- recorder (63): custo por frame com gravação ligada, memória do ring
  buffer por segundo gravado, tamanho do trecho exportado.
- incrementalidade (73): trabalho por estágio para "mudei 1 caractere".
- AI-native (78): memória, startup, latência de token até pixel, tempo de
  frame sob streaming, contra um equivalente Electron.

## 11. decisões

cada uma é do brenner; o orquestrador traz recomendação.

decididas em 2026-09-23:

1. `plevrev` integrada na `main` antes da onda 1: sim.
2. os `docs/how-to/*.md` desatualizados foram reconstruídos a partir da
   nova missão (manual do engine com a camada do contrato semântico,
   validação por pixel, alvo web, orquestração de agentes), e as
   referências voltaram a apontar para arquivo que existe.
3. regra de commit: a regra "agente não commita" saiu do AGENTS.md e do
   rust-conventions. o agente commita só na branch da própria task; o
   orquestrador integra na `main`.
4. modo de execução: o brenner autorizou o ciclo contínuo até terminar
   todas as tasks. o orquestrador integra cada task que ele validou sem
   pedir ok a cada uma, e para só nas decisões reservadas ao brenner
   (itens abertos abaixo, checkpoints da seção 9, escolhas de provedor,
   de fonte e de biblioteca de inferência, e testes manuais que só ele
   pode fazer). nada é enviado para o remoto sem pedido dele.

abertas:

5. task-39 (memoização, pendente desde 2023) é absorvida pela task-73.
   recomendação: mover a task-39 para `checked/` como superseded quando a
   73 abrir.
6. nome dos crates novos: `sem` fechado na task-57; protocolo do agente e
   IR fechados pelo orquestrador antes do briefing da task.
7. provedor do proponente LLM (task-69): local ou API. decidir na task-69,
   com o proponente atrás de um trait e um proponente determinístico para
   os testes.

## 12. estado

atualizado em cada integração.

| task | estado | na `main` | notas |
|---|---|---|---|
| onda 0 | integrada | `1d384d4` | how-to reconstruído, regra de commit, docs do programa |
| task-57 | integrada | `860b029` | crate `sem`; `Change` ficou na 57 com doze variantes; detalhes em `checked/task-57-semantic-contract-done.md` |
| task-58 | integrada | `20c601e` | dispatcher, receipts, check; devolvida uma vez (confirmação por tecnologia assistiva, id no `execute`); detalhes em `checked/task-58-actions-preconditions-receipts-done.md` |
| task-59 | integrada | `f71dc20`, `f5b7cee` | ligação no compositor e todo pelo dispatcher (fase a); os 16 widgets restantes com `SemId` e projeção, regra "toda ação lê o que muda", defeito do `Select` corrigido (fase b); rolagem do `Select` virou a task-81 |
| task-66 | integrada | `c6738be` | headless pelo caminho da janela; SVG e `--semantics` foram para a task-66b (pendente) |
| task-73 | fases 1 e 1b integradas; fase 2 (ADR de desenho) espera decisão do brenner | `ed814da`, `91fdb33` | limitado pela GPU mesmo sem a latência de envio (4 a 12 vezes a CPU); pass de layer segue os pixels; composite em todo frame; o caret a 60 fps é a task-82 |
| task-62 | fases a e b integradas; fase c (checkpoint a) desenhada, briefing depois da 57b | `8978602` | `Workbench` no ide, commit conferido no dispatcher e no index real; recusa de confirmação virou a task-58b; o desenho do experimento está no arquivo da task |
| task-63 | fases a, b e c integradas; fase d na fila | `c62a05d` | arquivo `.monster` de sessão (magic `MONS`); fase d = retenção de pixels opt-in e ligação no shell do showcase, depois da 61 |
| task-60 | integrada | `3ba5b04` | árvore AccessKit do snapshot, leitor de tela pelo dispatcher, caminho antigo removido; VoiceOver manual pendente com o brenner (roteiro no arquivo da task) |
| task-61 | integrada | `0ec72f9` | crate `wire`, `plev observe`/`act`, porta no showcase; um defeito real de fechamento de conexão corrigido no caminho; forma enxuta e redação única viraram a task-57b |
| task-58b | integrada | `dc6fde2` | recusa de confirmação no `sem` e no ide |
| task-81 | integrada | `7b3e48f` | painel do `Select` rola; um `Window` para desenho, ligação e clique |
| task-57b | integrada | `a7a6735` | forma serde enxuta (todo com 3 itens: 6419 para 2039 B; 103 itens: 87 para 26 KB), `source` opt-in no wire (`plev observe --source`), redação de segredo uma vez só em `sem::redact`; ADR lean-semantic-form-and-one-redaction |
| task-82 | em andamento | | caret como prazo (`WaitUntil`), sem 60 fps num campo parado; comps e showcase agora, ide depois da 62c |
| task-66b | em andamento | | SVG do frame headless com relatório de aproximação; `showcase --render --semantics` |
