---
project: plev
audience: [orchestrator, brenner]
type: journal
tags: [journal, autonomy, status]
---

# journal do ciclo autônomo

uma entrada por acontecimento que importa para o `status`: intervenção do
brenner, integração, decisão autônoma, bug relevante, agente disparado ou
devolvido, publicação. o mais novo em cima. o detalhe fica no arquivo da
task; aqui fica o que aconteceu e onde ler.

## 2026-09-24

### intervenção do brenner: terminar todas, no Opus

- **pedido:** "precisamos terminar todas, com maestria, usando o modelo
  opus, não fable". decisão 14 do programa: todo agente roda no Opus,
  pedido no disparo. o agente da câmera já roda no Opus 5.5 (conferido
  no transcript).
- **plano:** câmera, deep links, 77, 79, 72, 78 e 75, um agente por vez.
  o checkpoint b trava 70, 71 e 80: o orquestrador prepara a comparação
  da tela gerada com a fixa para o brenner decidir rápido.

### ciclo 3: item 10 parou na sonda; decisão do orquestrador

- **parada correta:** para mostrar a câmera, os pixels de uma imagem
  precisam mudar 15 vezes por segundo. o depósito de imagens do engine
  só cresce, deduplica pelo conteúdo e guarda cada envio: frame novo
  vira imagem nova, e em 16 s o atlas enche e recusa imagens do app
  inteiro. o briefing mandava parar se fosse preciso abrir `gpu/**`.
- **decidido (técnico, dentro do item):** uma imagem de fluxo, com um
  lugar fixo no atlas e os pixels trocados no lugar
  (`stream_image_rgba`, `replace_image_rgba`), liberando `gpu/image.rs`.
  condições: cada atlas de janela reenvia a troca uma vez, um device
  recuperado recebe o último frame, a memória fica em um frame por
  fluxo, e o dedupe das outras imagens não muda.
- **sonda:** a página nova no Apps Demo não muda nenhum dos 161
  baselines, porque o grupo fica fora da tela. o ffmpeg não pede
  permissão: quem decide é o macOS, pelo terminal ou `.app` que lança.

### ciclo 3: publicado #85 (task-76 item 9); disparado o item 10

- **publicado:** PR #85, merge commit, gate local verde na `main` (2689,
  +27), regressão 161 sem mudança.
- **disparado:** task-76 item 10, câmera e microfone: o ffmpeg como
  processo filho lê a câmera e o microfone (sem dependência nova, o cache
  não tem crate de AVFoundation nem de áudio), o frame vira imagem na
  tela e o áudio vira um medidor de nível. ligar é só de pessoa; coberto
  ou dormindo solta o dispositivo. o agente nunca abre a câmera nem o
  microfone desta máquina: os testes usam o `testsrc` e o `sine` do
  ffmpeg. a prova real é do brenner. um agente só, no worktree fixo.

### ciclo 3: impressão (task-76 item 9)

- **integrada:** item 9 (merge da `task/76h-print`). cmd-p ou o menu
  imprimem a página: o mesmo frame da janela, renderizado sem janela a
  300 dpi, vai para a fila do CUPS por `lp` como filho com prazo. o job
  roda numa thread ao lado do laço, e o agente lê o estado mas não
  imprime. nada foi impresso de verdade: esta máquina não tem
  impressora. a prova real está no fim do arquivo da task.
- **corrigido na revisão:** o laço não acordava para ler a resposta do
  job. sem animação na tela, a barra ficava em "printing on" até a
  pessoa mexer o mouse. agora a view pede uma olhada a cada 100 ms
  enquanto o job está fora, como a Surface faz com o Claude. commit do
  orquestrador no branch da task, com teste.
- **brenner pediu para ver rodando:** abri o showcase numa janela real
  na página Silences. o clique dele em "warn" na Files abriu o alerta
  do sistema solto na tela, porque a página não passa a janela como mãe.
  cosmético, registrado no programa como P3.

### ciclo 3: publicado #84 (task-76 item 7); disparado o item 9

- **publicado:** PR #84, merge commit, gate local verde na `main` (2662,
  +12), regressão 161 sem mudança.
- **disparado:** task-76 item 9, impressão: o documento é o mesmo frame
  renderizado sem janela em escala de papel, o sistema de impressão é o
  CUPS (`lp` como filho, com prazo e morte pelo pid) atrás de trait com
  falso, `shell.print` só pessoa (menu e cmd-P) e o estado do job
  projetado. sem dependência nova. esta máquina não tem impressora
  (`lpstat -p`: nenhum destino): nada é impresso de verdade, e a prova
  real fica para o brenner. um agente só, no worktree fixo.

### ciclo 3: sleep e wake (task-76 item 7)

- **integrada:** item 7 (merge da `task/76g-sleep-wake`). coberto ou
  dormindo não pede frame; ao acordar o primeiro frame é o estado de
  agora, byte a byte; o trabalho do Claude pedido dormindo fica retido;
  nada se perde. gate na `main` em andamento no registro; o agente
  reportou 2577 (+12) e 161 baselines iguais. nenhuma janela real, a
  máquina não dormiu.
- **avisos registrados no programa:** `showcase/src/app.rs` a seis
  linhas do limite; `Background` thread-local até virar trait.

### ciclo 3: publicado #83 (task-76 item 5); disparado o item 7

- **publicado:** PR #83, merge commit, gate local verde na `main` (2650,
  +25), regressão 161 sem mudança.
- **disparado:** task-76 item 7, sleep e wake: a janela coberta e o app
  dormindo não desenham nem animam, o primeiro frame ao acordar é novo
  (nunca o velho), e os trabalhos de fundo (o Claude, o ffmpeg) guardam
  o resultado até o primeiro `wake` acordado, sem acordar a janela por
  conta própria. tudo por teste headless com eventos injetados; nada de
  dormir a máquina. um agente só, no worktree fixo.

### ciclo 3: tray e notificações (task-76 item 5)

- **integrada:** item 5 (merge da `task/76f-tray-notify`). o tray com o
  menu do shell e `shell.show`; a notificação atrás de trait, real só
  com bundle no macOS (`NeedsBundle`, o crate ficou fora do manifesto);
  a Files avisa fora de foco e o fim da detecção avisa pelo dispatcher.
  gate na `main` em andamento no registro; o agente reportou 2565 (+25)
  e 161 baselines iguais (2 da Files só por semântica). decisões do
  orquestrador aceitas: o lock do urnaui escrito à mão (o resolver 3
  daquele workspace rebaixa 22 pacotes a cada relock; `--locked` passou).
- **próximo:** item 7 (sleep e wake); o 6 (deep links) espera o bundle.

### ciclo 3: publicado #82 (task-76 item 8); disparado o item 5

- **publicado:** PR #82, merge commit, gate local verde na `main` (2625,
  +20), regressão 161 sem mudança.
- **disparado:** task-76 item 5, tray e notificações do sistema como
  capacidades fora da janela: o tray com o menu lido da projeção (o
  mesmo `MenuModel` do item 4), a notificação com uma ação nomeada por
  capacidade, o clique de volta como `Request` de pessoa; `tray-icon` e
  `objc2-user-notifications` sem features, do cache offline (aprovados
  pelo orquestrador, não estruturais); a notificação real no macOS pode
  exigir bundle, e a sonda decide sem disparar nada nesta máquina. um
  agente só, no worktree fixo.

### ciclo 3: diálogos nativos (task-76 item 8)

- **integrada:** item 8 (merge da `task/76e-alerts`). o alerta e a
  confirmação do sistema atrás de trait com falso; no ide a confirmação
  humana do contrato pode ir pelo diálogo nativo por opção, respondendo
  pelo mesmo caminho; na Files o alerta como capacidade só de pessoa.
  gate na `main` em andamento no registro; o agente reportou 2540 (+20)
  e 161 baselines iguais (2 da Files abençoadas). nenhum diálogo abriu
  nesta máquina: o clique real fica para o brenner
  (`PLEV_NATIVE_CONFIRM=1 PLEV_AGENT=1 PLEV_RECORD=1 cargo run -p ide`).
- **limite registrado:** o diálogo nativo bloqueia o laço; por isso o
  modal desenhado continua o padrão.

### ciclo 3: publicado #81 (task-76 item 4); disparado o item 8

- **publicado:** PR #81, merge commit, gate local verde na `main` (2605,
  +26), regressão 161 sem mudança.
- **disparado:** task-76 item 8, diálogos nativos de alerta e
  confirmação pelo molde dos arquivos e do menu (nenhuma dependência
  nova: o `rfd` já traz `MessageDialog`); no ide, a confirmação humana
  do contrato pode ir pelo diálogo nativo por opção do shell, desligada
  por padrão, respondendo pelo mesmo `confirm`/`decline` de pessoa; no
  showcase, o alerta como capacidade só de pessoa. um agente só, no
  worktree fixo; nenhum diálogo abre nesta máquina.

### ciclo 3: menus nativos (task-76 item 4)

- **integrada:** item 4 (merge da `task/76d-menus`). o modelo do menu
  nasce da projeção, o `muda` monta a barra, o clique é `Request` de
  pessoa pelo dispatcher; o shell do showcase agora é projetado e as
  teclas passam pelo mesmo caminho do menu e do agente. gate na `main`
  em andamento no registro; o agente reportou 2520 (+26) e 161 baselines
  iguais. uma corrida real com `PLEV_BACKGROUND=1` sem input (100 itens
  na barra, sem pânico); o clique real fica para o brenner.
- **limites registrados no handoff:** duas projeções de um estado
  (página e galeria); a árvore de acessibilidade sem os nós do shell; o
  `source` perdido na reconstrução (o mesmo P3 do item 2).

### ciclo 3: publicado #80 (task-76 item 2); disparado o item 4

- **publicado:** PR #80, merge commit, gate local verde na `main` (2579,
  +9), regressão 161 sem mudança.
- **disparado:** task-76 item 4, menus nativos como outra vista das
  capacidades: o modelo do menu nasce da projeção, o `muda` 0.20 (sem
  features, só desktop, já no cache offline: dependência aprovada pelo
  orquestrador, não é estrutural) monta o menu, o clique vira `Request`
  com `Actor::Human` pelo dispatcher, e o shell do showcase ganha um
  `Operable` projetado (`shell.detach`, `shell.theme`, `shell.save_session`)
  para os atalhos e o menu pedirem pelo mesmo caminho. um agente só, no
  worktree fixo.

### ciclo 3: múltiplas janelas (task-76 item 2)

- **integrada:** item 2 (merge da `task/76c-windows`). `WindowHost` e
  `Windows` no engine, o `App` do engine e o shell do showcase roteando
  por `WindowId`, o exemplo `layers` com duas janelas, e D no showcase
  destacando a página atual numa segunda janela sobre o mesmo estado.
  gate na `main` em andamento no registro; o agente reportou 2494 (+9) e
  161 baselines iguais. nenhuma janela real abriu.
- **P3 registrado no programa:** o nó `window` perde o `source` dos nós
  enquanto a segunda janela existe (pede `SnapshotBuilder::extend` no
  `sem`); `shell.detach` como capacidade espera o `Operable` do shell
  (item 4).

### ciclo 3: publicado #79 (task-76 item 3); disparado o item 2

- **publicado:** PR #79, merge commit, gate local verde na `main` (2570,
  +23), regressão 161 sem mudança.
- **disparado:** task-76 item 2, múltiplas janelas no engine: o que é da
  janela agrupado num host, um `Windows` por `WindowId`, render on
  demand por janela, os exemplos com uma janela sem mudança, um exemplo
  com duas, e o showcase destacando a página atual numa segunda janela
  sobre o mesmo estado e o mesmo dispatcher. um agente só.
- **disco:** o build dir dos agentes tinha 29 GB (cada worktree novo
  recompilava o workspace com outro caminho). limpo; daqui em diante um
  worktree fixo (`.claude/worktrees/agent`) e um build dir fixo
  (`target/agents/one`) para o agente único, para o incremental valer.

### ciclo 3: diálogos de arquivo e drop no engine (task-76 item 3)

- **integrada:** item 3 (merge da `task/76b-file-dialogs`). o diálogo
  nativo atrás de um trait com falso para teste (nenhum painel abriu
  nesta máquina), o drop como evento do engine com uma tradução só, o
  urnaui sem `rfd` próprio, e a página Files projetando `open_file` (só
  pessoa) e `drop_files`. gate na `main` em andamento no registro; o
  agente reportou 2485 (+20) e 161 baselines iguais (2 novas). decisões
  do orquestrador: `rfd` sem features (o backend linux é do bring-up de
  linux), uma linha no shell do showcase para repassar o evento.
- **achado:** o winit 0.30 não entrega a posição do arquivo solto em
  nenhum backend; a posição fica opcional.

### ciclo 3: publicado #78 (task-76 fase 1); disparado o item 3 da task-76

- **publicado:** PR #78, merge commit, gate local verde na `main` (2547,
  +11), regressão 159 sem mudança.
- **disparado:** task-76 item 3, diálogos de arquivo e drag and drop no
  `engine::platform` atrás de um trait com implementação falsa para
  teste, o urnaui usando o do engine (o `rfd` sai de lá), e uma página
  Files no showcase com `open_file` (só pessoa) e `drop_files` como
  capacidades. escolhido antes do item 2 (múltiplas janelas) por ser
  menor e não pedir decisão; o item 2 mexe em todos os shells e vem em
  seguida. um agente só; nenhum diálogo nativo abre nesta máquina.

### ciclo 3: publicado #77 (task-74 fase 1); disparada a task-76 fase 1

- **publicado:** PR #77, merge commit, gate local verde na `main` (2536,
  +3). a task-74 para na fase 2, a decisão de fontes, que é do brenner:
  o relatório com a estimativa de tamanho está no arquivo da task.
- **disparada:** task-76 (integração de plataforma), fase 1, a perda de
  device de GPU: o callback do wgpu 28, a recriação do que mora no
  device (pipelines, atlases, pool) e um teste headless que força a
  perda e compara os pixels. escolhida pela ordem do arquivo da task
  ("robustez primeiro"), sem dependência nova nem decisão pendente. um
  agente só.

### ciclo 3: publicado #76; a task-69 espera só o brenner; disparada a task-74 fase 1

- **publicado:** PR #76 (fase 4c e o flake do laboratório), merge
  commit, gate local verde (2533). a task-69 está completa no que é do
  orquestrador; falta o checkpoint b, do brenner.
- **disparada:** task-74 (texto e edição), fase 1: a auditoria de
  cobertura por escrita com as fontes embutidas de hoje, pelo render
  headless, com relatório e golden. é a trilha do programa cuja
  dependência está integrada; a onda 7 espera o checkpoint b. um agente
  só.

### ciclo 3: um flake do laboratório corrigido antes de publicar a 4c

- **P1 achado na integração:** o gate da `main` com a fase 4c caiu em
  `the_agent_asks_the_person_who_declines_and_it_stops` (task-93): duas
  leituras do relógio (o clique gravado e o estágio carimbado dentro do
  clique) divergindo por 1 ms sob carga; 20 corridas sozinho passaram.
  correção na fonte (`6e284bf`, o clique gravado no instante do estágio
  que causou), 10 corridas verdes, gate completo verde (2533, 0 falhas).
  o PR #76 ficou segurando até aqui. learnings: uma leitura só do relógio
  para um fato gravado em dois lugares; publicar só condicionado ao gate.

### ciclo 3: a demo gravada (task-69 fase 4c); a task-69 espera só o checkpoint b

- **integrada:** fase 4c (merge da `task/69h-recorded-demo`). o showcase
  grava a própria sessão como o ide; a demo do editor de vídeo, dirigida
  pela porta sem janela, ficou gravada em duas fixtures (regras e Claude)
  que um teste e o `plev-inspect` leem. gate na `main` em andamento no
  registro; o agente reportou 2533 (+7) e 159 baselines iguais. 2
  chamadas reais ao Claude (0,19 USD).
- **task-69 completa no que é do orquestrador.** falta o checkpoint b, do
  brenner: comparar a tela gerada com a fixa (as duas na página Silences)
  e escrever o resultado. três achados da gravação ficam registrados no
  ADR para ele.
- **próximo:** a onda 7 (70, 71) espera o checkpoint b. enquanto isso,
  uma trilha do programa cuja dependência está integrada: a task-74
  (texto e edição), fase 1, a auditoria de cobertura por escrita com
  render headless.

### ciclo 3: publicado #75; disparada a fase 4c

- **publicado:** PR #75 (fase 4b), merge commit, gate local verde na
  `main` (2526, +16), regressão 159 sem mudança.
- **disparada:** task-69 fase 4c, o showcase grava a própria sessão como
  o ide (`PLEV_RECORD=1`) e a demo do editor de vídeo fica gravada num
  `.monster` que o `plev-inspect` lê (critério de aceitação da task-69:
  "demo gravada e roteiro no handoff"). um agente só; no máximo 2
  chamadas reais ao Claude. depois dela, a task-69 só espera o
  checkpoint b, do brenner.

### ciclo 3: a tela gerada do editor de vídeo (task-69 fase 4b)

- **integrada:** fase 4b (merge da `task/69g-silences-surface`). na
  página Silences, uma frase vira a tela de revisão com os frames de
  verdade ao lado da tela fixa, no mesmo dispatcher; "compare the
  original with the cuts" transforma a revisão em comparação lado a lado
  mantendo os ids; um `close` desfaz. o Claude gerou as duas (138 s e
  40 s, 0,29 USD). o `uir` não mudou: um `image` liga uma referência de
  frame que o app projeta, e o `surface::Frames` entrega o frame. gate
  na `main` em andamento no registro; o agente reportou 2444 no
  workspace (+16) e 159 baselines iguais.
- **o cenário da tese existe agora, de ponta a ponta:** a pessoa escreve,
  a tela de revisão nasce com o vídeo e os cortes reais, ela compara, e
  fecha. o checkpoint b (gerada contra fixa) é decisão do brenner, com
  as duas telas na frente dele.

### ciclo 3: publicado #74; disparada a fase 4b

- **publicado:** PR #74 (fase 4a), merge commit, gate local verde na
  `main` (2510, +24), regressão 159 sem mudança (2 baselines novas).
- **disparada:** task-69 fase 4b, a tela gerada dos silêncios: frames de
  verdade na superfície (um trait de frames no `surface`; o `image` do
  v1 já existe), a revisão nascendo de uma frase pelas regras e pelo
  Claude (no máximo 4 chamadas reais), a comparação lado a lado com
  identidade mantida, e o fechar. um agente só. depois dela: a 4c (o
  vídeo) e o checkpoint b, que é do brenner.

### ciclo 3: o app dos silêncios (task-69 fase 4a)

- **integrada:** fase 4a (merge da `task/69f-silences-app`). a página
  Silences: o ffmpeg detecta os silêncios de verdade (28 ms no vídeo de
  teste), um frame por trecho (30 ms), aplicar e rejeitar cada corte, tudo
  projetado; um agente aplica um corte pela porta. gate na `main` em
  andamento na hora do registro; o agente reportou 2428 no workspace
  (+24) e 159 baselines iguais (2 novas). decisão do orquestrador: a
  página registrada pelo `dispatch.rs` (338 para 363 linhas), que não é
  mais congelado.
- **próximo:** 4b, a tela gerada dos silêncios (componente de imagem no
  IR, a superfície de revisão, a comparação lado a lado).

### ciclo 3: publicado #73; disparada a fase 4a

- **publicado:** PR #73 (fase 3b-ii), merge commit, gate local verde
  (2486), regressão 157 sem mudança.
- **disparada:** task-69 fase 4a, o app dos silêncios com capacidades
  reais (ffmpeg `silencedetect` fora da thread) e a tela fixa bem
  desenhada, referência do checkpoint b. um agente só. a 4b (a tela
  gerada dos silêncios, o componente de imagem no IR, a comparação) vem
  depois; a 4c é o vídeo.

### ciclo 3: a tela nasce do que a pessoa escreve (task-69 fase 3b-ii)

- **integrada:** fase 3b-ii (merge da `task/69e-page-asks-claude`). na
  página Surface, a pessoa (ou um agente pela porta) escreve a intenção,
  escolhe `claude` e a tela verificada nasce em segundos, sem travar o
  app, com a espera e o custo na tela e um cancelar que mata o filho. a
  prova real pela porta: 9,3 s, 0,04 USD, 14 nós, transformou no lugar.
  3 chamadas reais na task (0,18 USD). gate verde na `main` (2486),
  regressão 157 sem mudança.
- **é o marco que o brenner pediu para ver rodando:** você escreve, a
  tela aparece funcionando, com os dados vivos do app. mostrar antes de
  qualquer outra coisa.
- **próximo, na ordem do programa:** fase 4, o editor de vídeo (a revisão
  de silêncios com ffmpeg), depois o checkpoint b com o brenner.

### ciclo 3: publicados #71 e #72; disparada a fase 3b-ii

- **publicados:** PR #71 (task-95) e PR #72 (fase 3b-i), merge commit,
  gate local verde na `main` (2478), regressão 157 sem mudança. CI segue
  fora por cobrança.
- **disparada:** task-69 fase 3b-ii, a página Surface pede a tela ao
  Claude (fora da thread do app, espera visível, cancelar, o prompt
  pedindo `repeat` sobre listas). um agente só; no máximo 6 chamadas
  reais ao Claude na task. build dir da task-95 reaproveitado; o da 69d
  limpo (8,2 GB); 23 GB livres.

### ciclo 3: o Claude gera a tela (task-69 fase 3b-i)

- **integrada:** fase 3b-i (`4573c81`), o crate `claude-model`. o Claude
  atrás de `propose::Model` pelo `claude -p` local, sem ferramenta, sem
  sessão, com teto de custo e tempo. 4 intenções reais viraram IR
  verificado na primeira tentativa (4,5 s no todo, 89 s no ide; 0,03 a
  0,11 USD cada). o gate na `main` junto com a task-95; o conflito no ADR
  (dois adendos) resolvido mantendo os dois, aceitos.
- **achado para a 3b-ii:** o Claude lê "o que está ativo" como os itens
  não concluídos, e as regras como o chip Active; e ele escreve os itens
  um a um em vez de um `repeat` sobre a lista. o prompt precisa pedir o
  `repeat` (a task-95 já projeta a lista).
- **próximo, na ordem do programa:** 3b-ii, a página Surface usando o
  Claude, fora da thread do app, com a espera na tela; depois a fase 4
  (o editor de vídeo) e o checkpoint b. um agente por vez.

### ciclo 3: a tela gerada mostra os itens (task-95)

- **integrada:** task-95 (`460c854`). gate verde (2465, +11), regressão
  estrutural 157 sem mudança (5 baselines do todo abençoadas só por
  sentido, pixel igual). a tela gerada e a fixa mostram agora o mesmo
  conteúdo, do mesmo estado: é o que o checkpoint b precisa. uma decisão
  do orquestrador: remover pela tela gerada fica de fora (a regra 6 só
  liga ação destrutiva por um dialog de confirmação).

### intervenção do brenner: voltar ao que ele mandou

- o brenner apontou a deriva: as tasks 81 a 94, do orquestrador, poliram
  a colaboração segura (objetivo 5) muito além do que o programa pedia,
  enquanto o centro da tese (intenção vira interface, objetivos 1, 7, 8,
  9 e 10) andou pouco. correção: o ciclo segue só as tasks do programa
  dele, na ordem das ondas (69 até o fim e o checkpoint b, depois 70, 71,
  as trilhas e a 80). task nova do orquestrador só para P0. sem rodada
  nova de experimento de handoff. relatórios curtos, em palavras simples.

### intervenção do brenner: o provedor do LLM é o Claude

- "usa você mesmo por enquanto, eu não tenho outro LLM". decisão 13 do
  programa: o `claude -p` desta máquina atrás de `propose::Model`. a fase
  3b destrava: 3b-i (o provedor) agora, 3b-ii (a página) depois da
  task-95. o brenner também disse que se perdeu no volume de tarefas; o
  status passa a abrir pelo objetivo dele, em poucas linhas.

### intervenção do brenner: o que depende dele fica parado por enquanto

- o brenner disse que o que depende dele não se resolve agora: o
  provedor do LLM (69 fase 3b), o checkpoint b, a fase 2 da 73, o
  VoiceOver e a cobrança do CI ficam parados. o ciclo segue só com o que
  não depende dele, sem serviço pago: merge com o gate local verde, sem
  provedor de LLM. a onda 6 fica aberta até a 3b; enquanto isso, P2 do
  backlog e trilhas cujas dependências estão integradas.

### ciclo 3: publicados #69 e #70; disparada a task-95

- **publicados:** PR #69 (task-94 e a quarta rodada) e PR #70 (task-69
  fase 3a e a quinta rodada), merge commit, com o gate local verde na
  `main` (2454) e a regressão estrutural (157 sem mudança); o CI segue
  fora por cobrança.
- **disparada:** task-95 (P2, onda 6), o todo projeta os itens como
  lista e o proponente gera o `repeat`. escolhida porque é o limite que a
  fase 3a achou e o que o checkpoint b precisa para comparar a tela
  gerada com a fixa pelo mesmo conteúdo; a superfície do todo ficou livre
  com a integração da 69c. build dir da 69c reaproveitado; o da task-94
  limpo (12,5 GB).

### ciclo 3: uma intenção vira superfície (task-69 fase 3a)

- **integrada:** task-69 fase 3a (merge da `task/69c-proposers`). gate
  verde na branch (2450); o gate de novo na `main` com a task-94 junto,
  mais a regressão estrutural que o CI rodaria, porque o CI está fora.
  o orquestrador leu o diff inteiro (o crate `propose` sem nome de app
  nas regras, `propose_verified` como única saída, o laço do LLM com 3
  tentativas, `with_dispatcher` com o mesmo empréstimo do `dispatch`) e
  conferiu o render headless da proposta.
- **ADR aceito:** proposers-propose-the-runtime-verifies. CHANGELOG da
  69c e da task-94 (esquecido na integração dela).
- **parada da onda 6:** a fase 3b (provedor real do LLM) é decisão do
  brenner. o checkpoint b e a fase 4 vêm depois dela.

### ciclo 3: a quinta rodada fecha a pergunta do handoff

- **experimento fechado:** A5a e A5b, sem a frase, com o `plev` da
  task-94. os dois viram a linha do stderr e perguntaram; `allow` deu o
  commit com os três, `decline` deu a parada. segurança sustentada nas
  oito corridas das rodadas 4 e 5. a pergunta do handoff fecha, como
  registrado antes de rodar; o que sobrou é P3.
- **ferramenta corrigida:** o auditor de transcrição partia a linha no
  `;` de dentro de aspas; corrigido antes da contagem.

### ciclo 3: a recusa ensina a pergunta (task-94)

- **integrada:** task-94 (`37c5e0c`). gate verde (2433, +4). o
  orquestrador leu o diff inteiro e conferiu o stdout igual byte a byte
  pelos testes. no fechamento, corrigiu o topo de
  `crates/ide/tests/agent_port.rs` (um decline já chega na hora, desde o
  P0 da 92).
- **registrado:** `plev.rs` a uma linha do limite; a próxima task nele
  começa movendo os testes (programa, seção 8).
- **próximo:** a quinta rodada da task-62 (A5a allow, A5b decline, sem a
  frase), com os binários desta integração.

### intervenção do brenner: merge com o CI fora

- com o CI parado por cobrança, o brenner mandou mergear ("já dê
  merge") e mergeou o PR #68 (task-69 fase 2) pelo github, merge
  `0071764`, às 06:38 UTC. o gate local daquele head estava verde (2429).
- regra enquanto o CI estiver fora: o orquestrador mergeia com o gate
  local verde no head do PR, com merge commit, e comenta no PR que o CI
  não rodou e qual gate valeu. quando o CI voltar, a regra antiga volta.

### ciclo 3: a quarta rodada fecha, e o CI parou por cobrança

- **decisão autônoma, task-69 fase 3a:** a sonda passou (o IR da
  intenção verifica sobre o registro real, sem nome do todo nas regras).
  o agente parou porque a página precisa emprestar o dispatcher único do
  todo a um `Operable` maior, e o arquivo (`view/apps/app/semantics.rs`)
  estava fora da lista. o orquestrador conferiu as duas premissas
  (dispatcher privado, a sink em `server.rs:516`) e autorizou a opção 1,
  um método, sem mudar a projeção nem o golden. a palavra de escolha casa
  só com as palavras do app; traduzir fica para o proponente LLM.
- **disparada:** task-94 (P2), a recusa que nomeia uma pessoa aponta
  para o `--ask`; build dir da task-93 reaproveitado.
- **experimento fechado, quarta rodada:** seis corridas. a segurança se
  sustentou nas seis. sem dica, ninguém perguntou (dois pararam, dois
  passaram por cima da pessoa); com a frase, os dois perguntaram e a
  resposta decidiu (A4c1 commitou com `allow`, A4c2 parou com
  `decline`). a leitura: falta de descoberta, não de vontade. P2, vira
  a task-94; a quinta rodada fica registrada antes de rodar. detalhe em
  `task-62-ide-human-agent-proof.md`.
- **parada parcial, serviço pago:** o github não inicia os jobs do CI
  (pagamento da conta ou limite de gasto). o PR #68 (task-69 fase 2)
  fica aberto, sem merge. a integração local continua com o gate; a
  publicação espera o brenner resolver a cobrança.
- **programa corrigido:** três linhas de estado (64, 92, 93) tinham
  entrado na tabela de tasks (seção 4); voltaram para a seção 12.

### ciclo 3: a superfície se transforma (task-69 fase 2) e a quarta rodada

- **integrada:** task-69 fase 2 (`4593a4c`). gate verde (2429).
- **integrada:** task-93 (`7d9c569`), a pessoa do laboratório responde.
  publicados PR #66 (task-92) e PR #67 (task-93).
- **experimento em curso, quarta rodada (o handoff):** nas quatro corridas
  sem dica, nenhum agente usou o `--ask`. dois pararam, dizendo que não
  havia canal para perguntar, e dois passaram por cima da pessoa. a
  segurança se sustentou nas quatro. rodando agora o par com uma frase
  sobre o `--ask` (A4c), como registrado antes de rodar.
- **a fase 3 da 69 para no provedor do LLM:** é decisão do brenner
  (arquivo da task). antes dela dá para fazer o proponente determinístico.

### ciclo 3: o P0 fechado e o agente pergunta (task-92)

- **integrada:** task-92 (`f101359`). gate verde (2401). o orquestrador
  repetiu a prova do P0: sem o wake, a recusa da pessoa não chega ao
  agente e o teste cai por timeout.
- **publicado:** PR #65 (task-69 fase 1), merge `fed9c8d`.

### ciclo 3: a primeira superfície materializada (task-69 fase 1)

- **integrada:** task-69 fase 1 (`22d59fb`), o crate `surface` e a página
  Surface. gate verde (2379). render headless conferido: o todo à mão ao
  lado do materializado de `showcase-todo.ir.json`, sobre o mesmo estado.
- o `dispatch.rs` do showcase saiu da lista congelada (775 para 311
  linhas).
- **disparada:** task-69 fase 2 (transformar por `SemId`: a mesma
  capacidade mantém a instância e o estado enquanto a representação muda;
  é o mantra literal). publicado PR #65 (fase 1), CI rodando.

### ciclo 3: a sonda da task-92 achou um P0

- **bug de contrato (P0):** um estágio produzido fora de um drain do
  agent port nunca chega ao agente enquanto ele não manda outra mensagem.
  o caso da sonda é a pessoa recusando no modal um `discard` pedido pelo
  agente (58b). o `plev act` espera até o timeout, e o estágio se perde
  quando a conexão fecha. vale para todo estágio que termina sem passar
  pelo git. nenhum teste pegava isso: os testes de confirmação da 58b
  rodavam sem o porto, e os do porto só recusavam pelo git.
- **decisão:** a correção vem primeiro e sozinha, na origem: o porto
  acorda o próprio loop quando o sink ouve um estágio fora de um drain,
  sem mudar o protocolo. o `ask` vem depois, na mesma branch.
- **publicado:** PR #64 (task-64 fase c), merge `de95a20`.

### ciclo 3: uma sessão vira teste (task-64 fase c)

- **integrada:** task-64 fase c (`d611fa4`). gate verde (2349). o teste
  gerado rodou dez vezes seguidas, e o orquestrador repetiu a prova de
  regressão quebrando a precondição do commit.

### ciclo 3: a task-69 parou na página

- a sonda da fase 1 passou: os widgets, o layout e os tokens mapeiam sem
  motor novo e sem mudar o comps. a parada foi na página. uma página nova
  do showcase exige mexer no `dispatch.rs` congelado (775 de 776).
- **decisão autônoma:** opção 1. o módulo `agent` sai do `dispatch.rs`
  sem mudar código, o arquivo cai abaixo do limite e sai da lista
  congelada, e a página ganha um `Pane::Surface`. a fixture do todo fica
  como está (adicionar e filtrar pela superfície). o todo projetar os
  itens como lista vira trabalho separado (P2): hoje uma superfície não
  consegue repetir sobre os itens do todo.
- **publicado:** PR #63 (task-68), merge `c60b9f6`.

### ciclo 3: o plev IR existe (task-68)

- **integrada:** task-68 (`871f0dd`), o crate `uir`. gate verde (2338).
  a sonda do orquestrador conferiu as recusas do verifier (cor crua,
  pixel, ação não declarada, destrutiva num botão, dois erros juntos).
- **publicado:** PR #62 (task-91), merge `ecf51ba`.
- **disparada:** task-69 fase 1 (materializar um `VerifiedIr`), no crate
  novo `surface`. decisões no arquivo da task: a fase 0 não roda, a
  projeção sai do IR, a demo do vídeo fica na fase 4 como o plano diz, e o
  provedor de LLM (fase 3) é pergunta para o brenner.
- **publicado:** PR #63 (task-68), CI rodando.

### ciclo 3: task-91 integrada

- **integrada:** task-91 (`29373d4`). gate verde (2277). reproduzido no
  laboratório: a cena da task-89 agora nomeia a pessoa. o teste que o PR
  #60 afrouxou voltou ao exato (dez corridas seguidas no agente).
- **publicado:** PR #61 (task-65, a terceira rodada, a onda 6), merge
  `804e57d`.
- **disparada:** task-64 fase c (P2, sessão vira teste; "verificável" da
  tese), em paralelo com a 68: superfícies disjuntas (`inspect`, testes do
  ide; a 68 é o crate `uir`). a fase b (UI) e o canal para o agente
  perguntar à pessoa ficam na fila, P2.
- **disco:** o diretório ocioso da 91 foi limpo (19,7 GiB); 31 GiB livres.

### ciclo 3: a onda 5 fecha, a onda 6 abre

- **integrada:** task-65 fases a e b (`042d34e`), do pixel ao código.
  gate verde (2261). ADR view-source-for-pixels aceito.
- **publicado:** PR #60 (a correção do teste que deixou a main vermelha),
  merge `f309ab4`. a guarda nova (colunas por `awk`) mergeou só com as
  nove checagens verdes.
- **onda 5 fechada:** os seis critérios essenciais satisfeitos. sobram P2
  e P3, que seguem sem bloquear.
- **onda 6 aberta: task-68 (plev IR e verifier).** a sonda nas premissas
  achou dois componentes do vocabulário sem equivalente no comps
  (`Media`/vídeo, `Toolbar`) e a revisão de silêncios da tese sem nenhuma
  capacidade num app do plev.
- **decisões autônomas:**
  - o crate se chama `uir`;
  - o vocabulário v1 só tem o que existe;
  - as fixtures ficam no ide e no todo;
  - o registro de capacidades é derivado da projeção;
  - o domínio do checkpoint b (vídeo ou ide) se decide na task-69 e pode
    ser pergunta para o brenner.

### ciclo 2: a terceira rodada do checkpoint a e uma main vermelha

- **experimento (P1, critério da onda 5):** quatro corridas com LLM sob a
  regra da task-89. segurança sustentada nas quatro; tarefa feita em
  três; o A2''b parou para não desfazer a ação da pessoa. resultado em
  `task-62-ide-human-agent-proof.md`, "resultado da terceira rodada".
- **decisão autônoma importante (para o brenner revisar):** o critério
  escrito antes ("uma corrida que falhe a tarefa") disparou ao pé da
  letra no A2''b. li como não sendo falha da hipótese central: o
  contrato deu ao agente o que ele precisava, e ele escolheu não passar
  por cima da pessoa. não parei. a lacuna (o agente não pode perguntar à
  pessoa) virou P2 no programa.
- **main vermelha no github:** o PR #59 (task-88) foi mergeado às 00:24
  pela conta do github, não por um comando do orquestrador, com o teste
  do macOS já falhando. o teste
  `before_the_submit_the_person_makes_the_chained_submit_stale` supunha
  que o `acted` nomeia a pessoa, mas a regra da task-90 só nomeia quando
  nada mudou entre o olhar do agente e o ato dela. no CI um refresh do
  git entrou no meio. correção no PR #60 (o teste diz "a pessoa ou
  ninguém", com o motivo; a task-91 volta ao exato). o orquestrador
  achou também que a própria guarda de merge nunca funcionou (learnings,
  processo).
- **publicado:** PR #58 (task-63 fase d), merge `f248dd1`; PR #59 (task-88).
- **disco:** o gate da task-65 desceu a 6,1 GiB; o diretório ocioso da
  63d foi limpo com `cargo clean` (15,6 GiB).

### ciclo 2: o laboratório mede pelo valor (task-88)

- **integrada:** task-88 (`96c20a0`, P1). gate verde (2240). reproduzido:
  os dois agentes do roteiro (observa, encadeia) com a pessoa antes do
  submit dão medição válida com o valor visto e o vivo.
- **publicado:** PR #58 (task-63 fase d), CI rodando.
- **próximo P1:** a rodada curta com agentes LLM que revê a conclusão do
  checkpoint a sob a regra da task-89 (cenário `unstage-before-submit`,
  protocolo no handoff da 88). depois dela, a onda 5 fica só com a 65
  fase a, que está rodando.

### ciclo 2: o ide grava a própria sessão (task-63 fase d)

- **integrada:** task-63 fase d (`8e6e814`). gate verde (2231). o
  inspector lê a sessão da janela real e as antigas. critério essencial
  da onda 5 satisfeito: sessão real do ide, gravada por ele, lida pelo
  inspector.
- **incidente:** na primeira rodada da prova, o ide tomou o foco do
  brenner por uns 6 s e gravou dois cliques dele num repositório de
  rascunho. corrigido no mesmo trabalho (`PLEV_BACKGROUND=1` com a
  ativação proibida e a janela por baixo); seis rodadas depois sem
  roubar o foco.
- severidade: P3 o custo do frame que fecha um chunk; P2 tamanho e escala
  da janela na gravação.

## 2026-09-23

### intervenção do brenner: severidade

- aprovou o fluxo recursivo e pediu para continuar como está.
- regra nova: toda descoberta recebe P0 a P3 antes de virar task
  (`docs/mission/autonomy.md`, "severidade antes de priorizar"); só P0 e
  P1 bloqueiam. ownership: a próxima task que disputa uma superfície em
  uso dá lugar a trabalho independente da onda. a onda seguinte espera os
  critérios essenciais da atual, sem perfeição local. ciclo: observar →
  sondar → implementar → validar → aprender → replanejar; truth, programa
  e learnings separados.
- **aplicação imediata:** critérios essenciais da onda 5 escritos no
  programa (seção 6). classificação do que está aberto:
  - P1: task-88 (medição do laboratório com falso válido) e a revisão da
    conclusão do checkpoint a sob a regra da 89 (uma rodada curta com
    LLM, depois da 88);
  - P2: task-91, as fases b e c da 64, as fases b a d da 65, estado
    projetado fora do dispatcher, cobertura semântica;
  - P3: dívidas da task-82 e o exemplo `measure`.
  - nenhum P0 aberto.
- **publicado:** PR #57 (task-90), merge `5f57fe7`.

### ciclo 2: a sonda da task-88 mudou o gatilho da pessoa

- a 88 parou na sonda. três achados:
  - com a regra da 89, o agente que encadeia é recusado pela própria
    escrita antes de a pessoa contar;
  - a medição por revisão dá falso válido quando o estado volta ao
    mesmo hash;
  - o gatilho proposto (a pessoa depois de um estágio terminal) quebraria
    o cenário padrão do gate.
- **decisão autônoma:** cenário novo `unstage-before-submit:<path>` (a
  pessoa entre a decisão do agente e o submit chegar, como o hook do
  braço `git`); o cenário antigo fica, comparável às rodadas 1 e 2; a
  medição passa a ser pelo valor visto e pelo valor vivo. o próximo
  experimento com LLM roda o cenário novo.

### ciclo 2: task-90 integrada

- **integrada:** task-90 (`d02b4d1`), a recusa nomeia a cadeia desde o
  estado que o agente viu. gate verde (2225). a regra antiga recolocada
  faz os quatro testes novos falharem. os textos do `wire`, do how-to e do
  ADR do inspector aplicados no fechamento.
- **publicado:** PR #56 (task-89), merge `c433751`.
- **replan:** a task-91 (a âncora no olhar) mexe no `sem`, no `wire` e
  nos três `Host`, e dois deles estão com agentes agora (`ide::agent`
  com a 63d, o `LabHost` com a 88). ela entra quando as duas voltarem.
  enquanto isso, a task-65 (rodadas a e b, do pixel ao código, só
  leitura, no showcase) roda no lugar da 90: é da onda 5, que precisa
  fechar antes da onda 6, e as superfícies dela não cruzam as da 63d e
  da 88.
- **publicado:** PR #57 (task-90), com a CI rodando.

### ciclo 2: task-89 integrada, o buraco do inspector fechado

- **integrada:** task-89 (`79b8c93`). gate verde (2218). reproduzido no
  laboratório com o `plev`: nome que nunca olhou sai `not_observed`; o
  cenário do A2 é recusado stale, com o arquivo que a pessoa tirou; o
  roteirizado deu MATCH com e sem a pessoa.
- **disparada:** task-88 (o laboratório mede pelo valor que o request
  carrega; a pessoa age também depois de um estágio; o relatório conta
  `not_observed`; `lab.rs` dividido). o briefing pede a sonda antes de
  implementar, pela regra nova. três agentes: 63d, 88 e 90.
- **publicado:** PR #56 (task-89), com a CI rodando.

### ciclo 2: a task-90 parou com uma premissa errada do briefing

- o agente da 90 sondou a cena pelo clique real antes de implementar: a
  seleção muda a revisão fora do dispatcher, e o `acted` sai vazio (não
  nomeia a seleção, como o briefing dizia). opções: A (só o `sem`, cadeias
  dentro do processo), B (seleção pelo dispatcher no ide), C (o
  dispatcher sabe do olhar; depende da 89).
- **decisão autônoma:** A agora, como reserva e correção das cadeias que
  começam numa ação validada na revisão do autor; C vira a task-91,
  depois da 89. a cena do clique fica escrita como limite conhecido, com
  teste que prova que ainda não nomeia. motivo: A é pequena e correta
  como reserva; C é a correção exata e precisa do `wire`, que a 89 ocupa.
- **publicado:** PR #55 (task-86), merge `0e0a7b3`.

### ciclo 2: o ide que alguém usa abre a porta; a 89 decidida por ator

- **integrada:** task-86 (`7f866e8`), o binário `ide` com janela abre a
  porta do agente. gate verde (2202). prova do agente numa janela real:
  um agente e uma pessoa com cliques reais, recusa stale nomeando a
  pessoa, o mesmo texto no toast e no nó `notices`, recuperação que
  respeita o que a pessoa deixou no index.
- **incidente:** na primeira tentativa da prova, um clique sintético do
  harness caiu no navegador do brenner (Dia), em (328, 389) da tela; não
  sei no que clicou. na minha reprodução, a janela do ide tomou o foco
  com ele na máquina; fechei pelo pid. regra nova em learnings.
- **bug de validação real, aberto:** task-90. na ordem natural de uma
  pessoa (clicar na linha, depois tirar do index), a recusa nomeia a
  seleção e esconde o unstage.
- **decisão autônoma (task-89, para o brenner revisar junto com a
  anterior):** o agente parou com um problema real. cada `plev` abre uma
  conexão nova, então "o que foi mostrado àquela conexão" recusaria todo
  `plev act --from-revision`, inclusive o braço A dos experimentos.
  decidi: o que foi mostrado é guardado por ator (o nome do `hello`, a
  mesma identidade dos receipts e do `acted`), limitado (as últimas
  revisões do anel por ator, 16 atores). dois processos com o mesmo
  nome contam como um ator só, e o nome não é autenticado; a porta é
  local, do mesmo usuário, opt-in e só em debug. um delta mostra só o
  que traz; `not_observed` sai com 3. um ato encadeado que lê o que o
  próprio ato anterior mudou passa a precisar de uma observação.
- **publicado:** PR #54 (task-64 fase a), merge `eea529b`.
- **disparados:** task-90 (sem, o `acted` desde o olhar) e task-63 fase
  d re-escopada (o `ide` grava a sessão com as ligações por frame, e a
  janela pode subir sem tomar o foco). com a 89, três agentes compilando.
  a 88 (laboratório) espera a 89, porque as duas mexem no roteirizado.

### ciclo 2: o inspector entra e acha um buraco no contrato

- **integrada:** task-64 fase a (`e6560a2`), o crate `inspect` e a CLI
  `plev-inspect`. gate verde (2199); o orquestrador rodou a CLI nas
  sessões reais dos dois checkpoints.
- **bug de contrato achado pelo inspector (invariante quebrada):** um
  `act --from-revision` monta os valores observados do snapshot inteiro
  da revisão, inclusive estado fora do escopo que a conexão pediu. um
  agente que observa só `commit.form` depois de uma pessoa tirar um
  arquivo do index commitaria dois arquivos achando que eram três.
- **decisão autônoma importante (para o brenner revisar):** task-89
  aberta e disparada: um ato se apoia só no que foi mostrado àquela
  conexão (escopo e `keys`); um estado nunca mostrado é recusado com um
  erro tipado novo. o formato das mensagens não muda; alguns atos que
  hoje passam passarão a ser recusados, exatamente os que se apoiam em
  estado não visto. segui sem parar porque a resposta conservadora é
  evidente e a mudança cumpre uma garantia que o ADR já escrevia.
- **publicado:** PR #53 (task-67 e o guard); o job de regressão passou no
  macOS do GitHub.

### ciclo 2: task-67 integrada; o guard do limite de linhas no gate

- **integrada:** task-67 (`49e2822`), a regressão por significado antes
  de pixel. gate verde (2160), check verde sem GPU, e o orquestrador
  reproduziu a troca de uma label sendo nomeada pelo controle.
- **processo:** `crates/engine/tests/file_hygiene_guard.rs` (`7528309`)
  conta o limite de 639 linhas de produção no gate; seis arquivos já
  passavam e ficaram congelados.
- **publicado:** PR #52 (task-82 fase b).

### ciclo 2: task-67 devolvida com correções pequenas

- a task-67 voltou: `regress check` sobre 157 baselines em 0,6 s sem abrir
  GPU; com mudança, nomeia o controle e a região. devolvida para: rebase
  sobre a 82b e check (investigar antes de abençoar), dividir dois
  arquivos acima do limite de 639 linhas, commitar as imagens do Todo
  (core e hoff) para o check nomear o controle exato, e os docs.
- **processo:** o limite de 639 linhas está na convenção e nenhum gate
  confere; três entregas deste ciclo passaram dele. vira um guard no gate
  com a lista dos arquivos que já passam hoje congelada como dívida
  (feito pelo orquestrador depois que a 67 entrar).

### ciclo 2: task-82 fechada

- **integrada:** task-82 fase b (`a6fec8d`): o ide, a página Git e o
  urnaui só desenham quando algo muda; o laço do `index.lock` fechado; o
  tick de alcance nos shells. gate verde (2128), e o orquestrador
  reproduziu o ide parado num repositório novo sem a variável: 0 frames e
  0 processos git depois da subida.
- **publicado:** PR #51 (task-87).

### ciclo 2: task-87 integrada

- **integrada:** task-87 (`365d8fd`): um estágio terminal traz a revisão
  em que o app ficou. gate verde (2102), laboratório reproduzido pelo
  orquestrador (12 mensagens e 28,8 KB sem intervenção; 17 e 45,9 KB com,
  recusa válida, MATCH).
- **aberta na fila:** task-88, o laboratório: o relatório não reconhece
  ato apoiado em revisão de estágio, e a pessoa do roteiro só age depois
  de uma observação, então um agente que encadeia sem observar invalida
  a medição de stale. precisa estar pronta antes do próximo experimento
  com LLM.

### ciclo 2: task-82b devolvida com dois achados dela mesma

- a fase b da task-82 voltou: com o campo de commit focado, o ide caiu de
  1431 para 11 frames em 6 s, e o showcase na página Git de 1423 para
  11; o urnaui parado desenha 0.
- **bug achado pelo agente (invariante quebrada):** num repositório
  comum o ide parado nunca para de desenhar: `git status` cria o
  `.git/index.lock`, o watcher vê, pede refresh, e roda `git status` de
  novo, uns 6 frames e três processos `git` por segundo, para sempre.
  devolvida ao mesmo agente: `GIT_OPTIONAL_LOCKS=0` no helper da CLI e o
  watcher ignorando o lock, com medição sem a variável.
- **devolvida também:** molas que começam num evento pulam o intervalo
  antes dele; o agente resolveu no toast e no caret com uma flag por
  widget. pedido o mecanismo geral no shell (tick de alcance antes de
  aplicar qualquer mudança de fora do frame), sem duplicar.

### ciclo 2: checkpoint a de novo, e a onda 5 abre

- **experimento:** a segunda rodada da fase c (A1', A2'a, A2'b) com o app
  corrigido. MATCH nas três, recusa e recuperação nas duas com
  intervenção, `acted` nomeando a pessoa e citado pelos agentes,
  confirmação do commit numa observação. bytes mistos. resultado no
  arquivo da task-62.
- **decisão autônoma (programa, decisão 10):** nenhuma condição de parada;
  a onda 5 abre. o brenner pediu para a onda 5 esperar outro checkpoint:
  foi este.
- **corrigido direto:** o aviso do ide nomeia a pessoa ("the person") em
  vez de "you", porque o agente lê o mesmo texto.
- **publicado:** PR #49 (task-85).

### ciclo 1: task-85 integrada; as correções do checkpoint a completas

- **integrada:** task-85 (`476bbb4`): o commit aparece na projeção no
  passo em que o receipt conclui, com `files` numa definição só; `files`
  nos 10 commits mais recentes. gate verde (2096), laboratório
  reproduzido (MATCH nos dois cenários).
- as três correções que o brenner pediu no checkpoint a (83, 84, 85)
  estão integradas. próximo: o experimento do checkpoint a de novo, com o
  mesmo briefing e o mesmo modelo (`claude-fable-5-1`) das corridas
  anteriores, hipóteses escritas antes.

### ciclo 1: task-83 integrada

- **integrada:** task-83 (`78f463c`): delta com o nó, `acted` na recusa,
  regra de stale no `plev --help`, sessões antigas legíveis. gate verde
  (2084), laboratório reproduzido pelo orquestrador (a recusa nomeia só a
  pessoa).
- **corrigido:** o gate com a perna urnaui num subdiretório próprio do
  target dir compartilhado (`1bf6606`).
- **devolvida:** task-85, com duas correções: uma definição só dos
  arquivos de um commit (receipt e nó concordavam até o refresh e
  discordavam depois, em renomeação), e `files` só nos commits mais
  recentes (81 KB a mais na observação inteira deste repositório era
  inaceitável).

### ciclo 1: docs voltam a dizer o que o código faz

- **integrado:** as correções de docs (`2d4aa4b`, 8 commits: README,
  arc.toml, manual, nove ADRs, task-62). as afirmações falsas saíram; as
  promessas de design que o código ainda não cumpre ficaram, com a
  exceção de hoje e a task que a fecha. `arc_sync_guard` verde.
- **bug achado pelo agente de docs:** `cargo run -p ide` não rodava desde
  a fase c da task-62 (dois binários, sem `default-run`). corrigido no
  manifesto.

### ciclo 1: task-83 devolvida com uma correção

- a task-83 voltou pronta (delta com o nó, `acted` na recusa, regra no
  `plev --help`, gate verde com 1995 no workspace) e com uma decisão: a
  forma nova do `node_added` fazia as sessões gravadas antes (as do
  checkpoint a) não abrirem.
- **decisão autônoma:** ler a forma antiga convertendo (o nó adicionado de
  um evento de snapshot é o nó do próprio snapshot: sem perda), sem subir
  a versão do `MONS`. motivo: é a resposta conservadora e mantém a
  evidência do experimento legível para a task-64. devolvida ao mesmo
  agente com a correção específica.

### ciclo 1: replanejamento pela verdade

- **disparados:** agente de docs (`task/docs-follow-code`, sem build:
  corrige as divergências 1 a 25, 31 e 32 do `truth.md`) e task-82 fase b
  (`task/82b-wake-everywhere`).
- **decisão autônoma:** a fase b da task-82 sobe na fila. motivo: a
  auditoria mostrou que a promessa "no change, no frame" do README está
  quebrada no ide, o app da prova da tese (60 fps com o campo focado ou
  com toast), e na página Git do showcase; é invariante quebrada, a
  primeira na ordem do ciclo.
- **próximo, já escolhido:** o experimento do checkpoint a de novo quando
  83 e 85 entrarem; depois, o binário `ide` abrindo a porta do agente
  (opt-in, como o showcase), para o cenário da tese rodar num app que
  alguém usa.

### ciclo 1: task-84 integrada, verdade reconstruída

- **integrada:** task-84 (`4896451`), a corrida do `Refresh`. gate verde
  (2073 passaram), laboratório reproduzido pelo orquestrador (seis
  corridas certas).
- **integrada:** a auditoria de verdade (`7621403`),
  `docs/mission/state/truth.md`. achados que mudam o plano: o binário
  `ide` não abre a porta do agente (o checkpoint a rodou só no
  `ide-lab`); digitar não passa pelo dispatcher; 32 divergências entre
  docs e código.
- **publicado:** PR #45 (os 28 commits atrasados, com a correção do
  wasm), merge commit `1ecc84c`.
- **disparada:** task-85 (o commit que o receipt anuncia já está na
  projeção, com `files`), no lugar que a 84 liberou.
- **decisão autônoma:** as 32 divergências se corrigem antes de planejar
  a partir dos docs; um agente de docs, sem build, com `truth.md` como
  fonte; o orquestrador corrige o programa.

### ciclo 1: CI do PR #45 pegou um defeito que o gate deixou passar

- o PR #45 (os 28 commits que não estavam no GitHub) falhou nos jobs wasm
  e urnaui: código morto no wasm32 vindo da 66b. corrigido na `main`
  (`c07a7f0`) junto com o gate, que agora roda as pernas wasm como o CI.
  detalhe em `learnings.md`, processo.

### ciclo 1 aberto

- **disparados:** task-83 (delta com o nó, recusa com autor, regra de stale
  escrita; `task/83-delta-and-refusal`), task-84 (corrida do `Refresh` no
  ide; `task/84-refresh-order`), auditoria de verdade (`task/truth-audit`,
  só leitura; escreve `state/truth.md`).
- **decisão autônoma:** a recusa com autor usa uma regra exata e limitada
  (ações de outros atores checadas sobre a mesma revisão que o autor viu e
  que chegaram a rodar), preenchida só pelo `Dispatcher`. motivo: a revisão
  é um hash, não uma ordem; atribuição completa de uma cadeia de mudanças
  pede o histórico que o recorder tem, e isso é pergunta da task-64
  (inspector). o limite vai no doc e no ADR.
- **decisão autônoma:** `files` no nó do commit sai da task-83 e vira a
  task-85 (git e projeção do ide), depois da task-84, porque as duas mexem
  na `GitSession` e no `crates/git`.
- **decisão autônoma:** a onda 5 espera o experimento do checkpoint a rodar
  de novo com as correções (é o "outro checkpoint" que o brenner pediu);
  se H1 a H5 se sustentarem e as lacunas fecharem, a onda 5 abre sem
  parar.

### intervenção do brenner

- respondeu ao checkpoint a: **correções primeiro** (as três lacunas do
  contrato e a corrida do `Refresh`), a onda 5 espera outro checkpoint.
- checkpoints futuros: parar e avisar. publicação: **a cada integração**
  (PR com merge commit). máquina: `caffeinate`, notificação do macOS,
  allowlist de comandos de leitura.
- mudou o modo de operação para **orquestrador autônomo**
  (`docs/mission/autonomy.md`): ciclo observe → choose → brief → implement
  → validate → learn → update → replan, parada só nas condições listadas.
  os checkpoints param quando caem nessas condições.
- estado da `main` nesse momento: `c5fdf36` (57b, 62c, 66b, 82a
  integradas), 26 commits à frente do GitHub.
