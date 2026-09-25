---
project: plev
audience: [orchestrator, ai-agents]
type: learnings
tags: [learnings, hypotheses, process]
---

# learnings

o que experimentos e implementações ensinaram, e o que mudou no programa por
causa disso. só aprendizado concreto, com a fonte. o mais novo em cima.

## produto e tese

### 2026-09-24, checkpoint b, primeira rodada

- **o mínimo de um texto num flex era um glifo:** o taffy toma o
  min-content como mínimo de um item, e o `TextMeasurer` mede o
  min-content com largura zero. qualquer linha apertada do IR espremia o
  texto em vez de quebrar. fonte: handoff das correções, sonda (a),
  `crates/surface/src/layout.rs` (`longest_word`).
- **a tela gerada nunca tinha sido vista inteira na janela padrão:** os
  testes da task-69 provavam a projeção, as ações e os frames, e o
  regress só tinha a Silences no estado inicial. a primeira vez que
  alguém olhou a revisão gerada a 1200x800, a linha do corte estava
  quebrada. regra: toda tela gerada que a tese usa tem imagem de
  referência no regress no estado em que a pessoa a usa.
- **o espelho da task-79 virou a ferramenta do checkpoint:** o
  orquestrador operou pelo contrato e fotografou pelo espelho, sem
  tocar a tela do brenner. é o jeito de repetir o checkpoint igual.

### 2026-09-24, task-76 item 10

- **um frame por imagem enchia o atlas em 16 segundos:** o depósito de
  imagens do engine só crescia e guardava os pixels de cada envio, o que
  serve a ícones e fotos e quebra com vídeo. a imagem de fluxo (um lugar
  fixo, os pixels trocados no lugar, reenviados uma vez por atlas) é o
  caminho de qualquer fonte ao vivo: câmera, vídeo, canvas de terceiros.
- **o ffmpeg não sabe de permissão:** quem decide é o macOS, pelo
  processo que lançou (o terminal ou o `.app`). a recusa chega como texto
  de erro genérico do AVFoundation, então a classificação em "permissão
  negada" só se confirma na máquina de uma pessoa.

### 2026-09-24, task-76 item 9

- **a página vai para o papel pelo mesmo caminho da tela:** o render sem
  janela a 300 dpi mantém o layout e rasteriza o texto na escala física
  (a tinta de um texto cresce na mesma razão da escala), então imprimir
  não precisa de um segundo renderer. o svg do mesmo frame já tem a
  geometria de um PDF vetorial; falta só o escritor. fonte:
  `task-76-platform-integration-done.md`, handoff do item 9.

### 2026-09-24, task-76 item 7

- **a janela coberta desenhava para ninguém:** o `request_redraw` do
  winit entrega `RedrawRequested` mesmo com a janela minimizada ou
  coberta, e `Occluded` não era lido por nenhum shell; uma página que
  anima gastava GPU sem tela, e a textura guardava o frame velho
  enquanto o estado andava. agora um host não acordado retém o pedido e
  acorda com o estado de agora. o sleep do sistema não vem pelo winit
  (é `NSWorkspace`, atrás de feature que ninguém liga): fica para o
  bring-up. fonte: `task-76-platform-integration-done.md`, handoff do item 7.

### 2026-09-24, task-76 itens 4, 5 e 8

- **fora da janela, o macOS cobra o bundle:** menu de barra, diálogo e
  tray funcionam num binário de `cargo run`; notificação do sistema
  (`UNUserNotificationCenter`) e esquema de URL exigem um `.app` com
  `CFBundleIdentifier`, senão o processo estoura. o engine responde
  `NeedsBundle` e o bundle vira item do bring-up de plataforma. criar um
  tray mostra o ícone na hora, então nenhum teste cria um real: o falso
  é o único caminho de teste, como nos diálogos.
- **o diálogo nativo bloqueia o laço:** o `act` de um agente pela porta
  só volta quando a pessoa fecha o diálogo, por isso o modal desenhado
  continua o padrão e o nativo é opção. fonte:
  `task-76-platform-integration-done.md`, handoffs dos itens 4, 5 e 8.

### 2026-09-24, task-76 itens 2 e 3

- **a segunda janela fechava a primeira:** no shell antigo do showcase,
  um segundo `create_window` sobrescrevia o `Arc<Window>`, o `Drop` do
  macOS fechava a primeira janela e os eventos dela passavam a ser
  roteados como da segunda. o `WindowId` que o winit entrega em todo
  evento era ignorado por todos os shells. agora cada janela é um host
  com o próprio contexto e o app roteia por id.
- **o drop de arquivo não tem posição em plataforma nenhuma:** o winit
  0.30 entrega `HoveredFile` e `DroppedFile` sem coordenadas e não manda
  `CursorMoved` durante o arraste; a posição fica opcional até ele expor.
  fonte: `task-76-platform-integration-done.md`, handoffs dos itens 2 e 3.

### 2026-09-24, task-76 fase 1

- **o device perdido era um buraco silencioso:** o wgpu engole todo erro
  `DeviceLost` antes do handler fatal, então um `draw` num device morto
  "termina" sem pânico e o compositor marca as layers como limpas em cima
  de nada; só o readback estourava. `destroy` sozinho não avisa: o
  callback chega no primeiro `poll`. a recuperação no lugar refaz o que
  mora no device e o frame seguinte é byte a byte o anterior. a surface
  sobrevive (é do `Instance`) e só precisa ser reconfigurada. fonte:
  `task-76-platform-integration-done.md`, "handoff da fase 1".

### 2026-09-24, task-74 fase 1

- **o texto do plev hoje é latino:** com as 9 faces embutidas, árabe,
  hebraico, devanágari, chinês, japonês e coreano viram tofu, e o tofu
  (9,9 px em 16 px) erra a largura medida nos dois sentidos, então até a
  quebra de linha sai errada. o que já funciona sem fonte: a ordem bidi
  (inclusive dígitos em nível 2), as marcas combinantes (o shaper compõe)
  e o kerning. o que não existe: `liga` nas duas faces e o caminho de cor
  no atlas (emoji tem largura e nenhuma tinta, mesmo com Apple Color
  Emoji). embutido e desktop são dois produtos: a mesma frase mede 251 px
  embutida e 151 px com a fonte do sistema. a fase 2 (embutir subsets
  Noto, fallback de sistema, ou os dois) é decisão do brenner, com a
  estimativa de tamanho no relatório. fonte:
  `task-74-text-and-editing.md`, "relatório da fase 1".

### 2026-09-24, task-69 fase 3a

- **gerar sem template funcionou pela estrutura do registro:** o
  proponente por regras monta a superfície a partir de roles, tipos de
  valor, parâmetros e parents, e as palavras da intenção só escolhem
  entre as do app. as mesmas regras, sem mudança, leem o todo do
  showcase e o workbench do ide. o limite apareceu onde o registro é
  pobre: "os itens ativos" não sai dele porque o todo não projeta os
  itens como lista (P2). o que falta a um proponente é informação no
  contrato, não regra no proponente.
- **o autor da proposta é quem pediu:** a página verifica a proposta sob
  a policy de quem pediu, então um agente só gera superfície com o que
  agentes podem pedir. a verificação usa o mesmo contrato que já decide
  cada ato.

### 2026-09-24, checkpoint a, quinta rodada

- **o contrato ensina uma capacidade quando a mostra na hora em que ela
  serve:** com o mesmo briefing da quarta rodada, e o `plev` apontando
  para o `--ask` só na recusa que nomeia uma pessoa (task-94), os dois
  agentes perguntaram, e a resposta da pessoa decidiu (commit com
  `allow`, parada com `decline`). a documentação (`--help`) não tinha
  bastado: nenhum agente a abriu. vale para o resto do contrato: o que
  um agente precisa decidir deve vir junto do estado ou da recusa que
  pede a decisão. fonte: `task-62-ide-human-agent-proof.md`, "resultado
  da quinta rodada".

### 2026-09-24, checkpoint a, quarta rodada (o handoff)

- **o agente usa o que o contrato mostra na hora da decisão:** com a
  pessoa tirando um arquivo antes do submit, nenhum dos quatro agentes
  sem dica perguntou. dois pararam dizendo que não havia canal, dois
  passaram por cima dela. nenhum abriu o `plev act --help`, onde o
  `--ask` estava. com uma frase no briefing, os dois perguntaram na
  primeira recusa, e a resposta da pessoa decidiu (commit com `allow`,
  parada com `decline`). a segurança não dependeu disso: nas seis, a
  recusa veio antes do efeito e nomeou a pessoa. o que muda no programa:
  a task-94 põe a pergunta na recusa que nomeia uma pessoa, e uma quinta
  rodada curta mede se basta. fonte: `task-62-ide-human-agent-proof.md`,
  "resultado da quarta rodada".

### 2026-09-24, task-69 fase 2

- **o mantra funciona numa superfície gerada:** a mesma capacidade
  mantém a instância e o estado (foco, caret, arraste) enquanto a
  representação muda de forma. identidade por `SemId` foi o que
  bastou: nada no comps ou no engine precisou mudar. a continuidade
  depende de o proponente (fase 3) manter os ids das capacidades que
  continuam.

### 2026-09-24, task-92

- **o handoff ganhou a peça que faltava:** um agente que encontra a
  tarefa em conflito com o que a pessoa fez pode perguntar a ela, pelo
  mesmo modal que o `discard` já usava. a pessoa responde, e o agente
  recebe a resposta como estágio. ainda sem experimento: o laboratório
  precisa de uma pessoa roteirizada que responda.

### 2026-09-24, task-69 fase 1

- **uma interface que nunca foi escrita como tela funciona pelo mesmo
  contrato:** a superfície do todo, materializada de um JSON verificado,
  opera o mesmo estado que o todo escrito à mão, pelo mesmo dispatcher, e
  um agente a observa e age nela pela porta como em qualquer página. do
  IR verificado à cena do primeiro frame leva menos de um milissegundo em
  debug; o que pesa é o texto e a GPU, como em qualquer frame.
- **o IR prometia mais que o widget cumpre:** dois props passam no
  verifier e o widget ignora. o verifier deve recusar o que a
  materialização não honra, senão o "só o verificado existe" vira "o
  verificado existe, em parte".

### 2026-09-24, task-64 fase c

- **uma sessão real virou teste que pega regressão:** a sessão de um
  agente LLM com a pessoa agindo antes do submit, reaplicada num
  repositório novo, deu os mesmos vereditos. com a precondição do commit
  quebrada, o teste nomeou o request que mudou de veredito. o contrato
  torna a reexecução determinística onde importa: o request carrega o
  que o ator viu, e o veredito depende só disso e do estado.

### 2026-09-24, task-68

- **o IR é pequeno porque a projeção já declara tudo:** o registro de
  capacidades sai do `Snapshot` que o app já projeta para agentes e
  leitor de tela. o IR só pode referenciar o que existe, sem uma segunda
  declaração que envelheça. o contrato semântico virou o espaço formal
  da geração, como a tese pedia.
- **o vocabulário do comps não é todo semântico:** 9 dos 21 componentes
  não têm `project`. para a interface gerada ser operável pelo mesmo
  contrato, a materialização (69) precisa que cada componente se projete.

### 2026-09-24, task-91

- **a âncora certa de "quem agiu" é o instante do olhar, não uma
  revisão.** revisão é hash de conteúdo e muda sem receipt (uma seleção,
  um refresh do git). o dispatcher numera os requests na ordem, e o port
  marca cada observação nessa ordem. com isso a cena comum do ide nomeia
  quem tirou o arquivo, o que duas regras por revisão (83 e 90) não
  conseguiam. e o agente recusado pela própria escrita agora lê que foi
  ele.

### 2026-09-24, checkpoint a, terceira rodada (a regra da task-89)

- **a segurança se sustentou com a regra nova:** quatro corridas com
  agentes LLM, nenhum commit errado, nenhuma recusa do protocolo, todo ato
  com `--from-revision`. com a pessoa agindo entre a decisão e o submit, a
  recusa veio antes do efeito e nomeou a pessoa nas duas corridas.
- **um agente escolheu não passar por cima da pessoa:** com a recusa e o
  aviso dizendo que a pessoa tirou o arquivo, um agente recolocou o
  arquivo e commitou; o outro parou e explicou por quê. o contrato deu a
  informação para os dois; o que falta é o agente poder perguntar à
  pessoa. é o primeiro sinal concreto de handoff, e a lacuna é de
  capacidade, não de segurança.
- **a recusa pela própria escrita confunde:** o agente que encadeou
  depois de escrever foi recusado, se recuperou, mas explicou a causa
  errado. a regra está certa; a recusa não diz que a mudança foi dele.
- **os agentes não economizaram:** mesmo com o estágio trazendo a
  revisão, eles observam antes de decidir, e o controle gastou mais bytes
  que na segunda rodada. o ganho da task-87 aparece no roteirizado, não
  num agente LLM que prefere olhar.

### 2026-09-23, task-88 (sonda)

- **com a regra da task-89, escrever e encadear exige olhar:** um agente
  que escreve a mensagem (`set_value`) e submete encadeado na revisão do
  estágio é recusado pela própria escrita. um estágio terminal não mostra
  estado, e a mensagem em que ele se apoia vem da última observação, que
  estava vazia. o `acted` sai vazio, porque ninguém mais agiu. é
  conservador e está escrito no `plev act --help`. o próximo experimento
  com LLM precisa medir quanto isso custa em idas e voltas.
- **revisão é hash de conteúdo:** um estado que volta exatamente ao que
  era tem a mesma revisão. a medição de stale do laboratório, que achava
  o ato pela revisão, deu "válida" numa rodada em que a pessoa nunca
  exerceu a checagem. medir pelo valor que o request carrega e pelo valor
  vivo no dispatch, nunca pela revisão.

### 2026-09-23, task-89

- **o furo fechou e ficou reproduzível:** no laboratório, olhar
  `changes` (três arquivos), a pessoa tirar um, olhar só o formulário e
  submeter agora é recusado stale, com o arquivo que saiu. antes, o
  commit saía com dois arquivos e o agente achava que eram três.
- **a identidade do agente é o nome, não a conexão:** "o que foi mostrado
  àquela conexão" recusaria todo agente pelo cli, que abre uma conexão
  por comando. o nome do `hello` já era a identidade dos receipts e do
  `acted`, e virou também a do que foi mostrado. o custo é que dois
  processos com o mesmo nome contam como um agente só.
- **`keys` também corta o que foi mostrado:** um agente que observa com
  `--keys staged` não viu o `unstaged` que o `stage` lê, e o ato sai
  `not_observed`. as `reads` de cada ação estão na observação, então o
  agente sabe o que precisa ver.

### 2026-09-23, task-90 (sonda)

- **estado projetado que muda fora do dispatcher:** no ide, a seleção
  por clique ou teclado muda `changes:selected` direto na view, e a
  revisão muda sem receipt; o mesmo vale para a digitação na mensagem, o
  foco, o toast saindo e as linhas do diff. o `check` não sofre (as ações
  leem o que muda com elas), mas atribuição e inspeção sim: a recusa não
  tem onde ancorar "desde que o agente olhou", e o inspector vê a mudança
  sem ator. é a mesma classe da dívida "digitação fora do contrato".
  a correção de atribuição é a task-91 (o dispatcher sabe do olhar).

### 2026-09-23, task-86: o cenário da tese numa janela de verdade

- **a tese saiu do laboratório:** no `ide` com janela, um agente pelo
  `plev` e uma pessoa com cliques reais operaram o mesmo workbench; a
  recusa stale nomeou a pessoa e o texto do toast é o nó que o agente
  lê. antes isso só existia no `ide-lab`, sem janela e com a pessoa
  roteirizada.
- **o limite escrito do `acted` cai no uso normal:** a regra da task-83
  (nomear quem agiu na revisão exata que o autor viu) foi aceita como
  limite documentado. na janela, a ordem natural de uma pessoa (clicar
  na linha, depois tirar do index) faz a recusa nomear a seleção e
  esconder o unstage. limite aceito por ser raro virou limite comum
  quando uma pessoa de verdade entrou: task-90.

### 2026-09-23, task-64 fase a e o buraco que ela achou

- **a inspeção pagou na primeira leitura:** lendo a sessão do A2 da
  primeira rodada, o inspector mostrou um submit apoiado numa observação
  só de `commit.form`, com `changes:staged` fora do escopo, e o servidor
  dizendo que o agente tinha visto aquele valor. a promessa escrita no
  ADR ("the stale check compares what the agent saw") não valia com
  observação por escopo. nenhum teste, nenhuma revisão de diff e nenhum
  dos dois experimentos tinha pego isso: o experimento passou porque a
  pessoa agiu depois da observação com escopo; se tivesse agido antes, o
  commit errado teria saído. virou a task-89.
- **pressuposto errado:** "o anel guarda o que foi servido" foi lido como
  "guarda o que o agente viu". o anel guarda o snapshot inteiro; o que a
  conexão viu é um recorte dele, e ninguém registrava o recorte.
- **a pergunta do núcleo começa a ter resposta:** a CLI respondeu, sobre
  as nove sessões gravadas, qual controle, quem observou o quê, o que
  mudou no intervalo e quem mudou, o veredito e o resultado. a interface
  visual (fase b) ainda falta; o checkpoint c pergunta se isso basta para
  confiar numa delegação.

### 2026-09-23, task-67

- **o contrato serve para verificar, não só para operar:** a regressão
  do Todo diz `todo.item[2] semantics changed: label ...` com a região,
  em vez de "727 pixels diferentes". só o Todo e o ide projetam; nas
  outras páginas a regressão cai para layer e pixel. a cobertura
  semântica fina (task-59 e a auditoria) é o limite prático dela.
- **o hash é estável fora desta máquina:** o primeiro job `structural
  regression` no macOS do GitHub (PR #53) passou com as 157 baselines
  gravadas aqui; o medo dos bits de `sin`/`cos` do sistema não se
  confirmou nesse runner.
- **o hash da cena pega mudança que o pixel não vê:** o Button no tema
  hoff mudou de hash com 0 pixels diferentes quando um token mudou. o
  check falha por isso e diz 0 px; se isso é ruído ou sinal, o uso vai
  dizer.

### 2026-09-23, task-82 fase b

- **a promessa do README era falsa num lugar que ninguém mediu:** o ide
  parado num repositório comum desenhava para sempre, porque o `git
  status` criava o lock opcional do index, o watcher via, e o refresh
  rodava `git status` de novo. só apareceu quando alguém contou frames e
  processos numa janela de verdade com o app parado. dois cortes
  independentes fecharam: leitura sem lock opcional, e o watcher ignora o
  lock (outro `git` na máquina ainda cria).
- **render on demand pede que o tempo alcance o evento:** com o shell
  dormindo até o prazo, o primeiro tick depois de um evento recebia o
  intervalo inteiro de ócio, e toda mola iniciada por um evento pulava
  para o fim. um tick de alcance quando o loop acorda resolve para todos
  os widgets de uma vez; a correção por widget era um sintoma.

### 2026-09-23, task-87

- **o custo de idas e voltas era o protocolo, não o contrato:** com a
  revisão no estágio terminal, o agente roteirizado com intervenção caiu
  de 24 para 17 mensagens e de 91,5 para 45,9 KB, sem perder a recusa
  stale nem a recuperação. a derrota do plev em bytes e idas e voltas
  (H4) encolheu pela metade no laboratório; contra o `git` direto (1,5 KB
  em 20 invocações) continua perdendo em bytes, e isso é a observação
  inteira do ide, que é grande.
- **a revisão do estágio nunca enganou:** nas 25 vezes em que um ato foi
  seguido por uma observação, ela serviu a mesma revisão. só é verdade
  porque a task-84 e a 85 fizeram o ide mostrar o efeito no passo em que
  o ato termina: uma correção de invariante habilitou uma otimização.

### 2026-09-23, checkpoint a, segunda rodada

fonte: `steps/ongoing/task-62-ide-human-agent-proof.md`, "resultado da
segunda rodada"; saídas em `target/agents/exp2/`.

- **confirmado de novo, com o app corrigido:** três agentes LLM
  (`claude-fable-5-1`, o mesmo briefing da primeira rodada) fizeram a
  tarefa só pelo contrato; as duas intervenções foram recusadas antes do
  efeito e recuperadas; receipt igual ao git nas três.
- **as correções aparecem no comportamento:** os três confirmaram o
  próprio commit numa observação por delta que já trazia o nó com
  `files`; os dois com intervenção citaram o `acted` (a pessoa, request 6)
  ao explicar a recuperação. tokens de saída caíram 24 a 26%.
- **bytes, não:** com intervenção, 14 a 16% mais bytes que antes, com as
  mesmas 30 mensagens. o que sobra de custo de idas e voltas é uma
  observação depois de cada ato só para saber a revisão nova; os três
  agentes pediram a revisão no `completed` (task-87).
- **texto projetado não pode supor quem lê:** o aviso dizia "you unstaged
  src/session.rs" para a pessoa na tela, e o agente leu o mesmo nó achando
  que o "you" era ele. um contrato único para humano e agente quer frases
  que nomeiam o ator; corrigido ("the person").

### 2026-09-23, task-85

- **o receipt e a projeção podem se contradizer sem ninguém mentir:** o
  receipt dizia "commitado" e a projeção dizia "ainda staged, sem commit"
  até o refresh. cada um estava certo no seu tempo; o agente que
  confere logo depois via um mundo que contradizia o receipt que acabou
  de receber. regra que ficou: o app mostra o efeito no passo em que o
  receipt conclui.
- **duas fontes para o mesmo fato divergem:** os arquivos do commit
  vinham de `git show --name-only` no receipt e do diff de árvores no
  log; concordavam até o refresh e discordavam depois, numa renomeação.
  uma definição só, lida por uma função só.
- **o custo de uma observação inteira do ide é o log:** neste repositório
  a observação inteira tem uns 43 KB, 97% dela o `stacks` com 100 commits.
  pôr `files` em todos triplicaria. um agente que observa por escopo não
  paga isso; o limite (`files` nos 10 mais recentes) mantém o aumento em
  3,8%. quantos commits a projeção carrega é uma pergunta que ainda não
  foi feita.

### 2026-09-23, task-83

- **confirmado no laboratório:** a regra exata do `acted` (ações de
  outros atores checadas na mesma revisão que o autor viu e que rodaram)
  nomeia no cenário da intervenção exatamente a pessoa que tirou o
  arquivo do index, e ninguém mais. a atribuição útil saiu barata porque
  o dispatcher já guardava `validated_against` em cada receipt.
- **bytes, pela primeira vez a favor:** o commit nascendo em `stacks`
  custava ao agente 172 B de delta mais 402 B de uma segunda observação;
  com o nó no delta, 427 B numa ida. o delta de um item novo no todo
  subiu de 423 para 647 B, ainda 28% da observação inteira com 3 itens.
- **limite que continua:** uma mudança de fora do app e uma digitação
  (sem request) nunca aparecem em `acted`; a recusa diz o que mudou e
  fica calada sobre quem.

### 2026-09-23, task-84 e auditoria de verdade

fonte: `steps/checked/task-84-ide-refresh-never-goes-back-done.md`,
`state/truth.md`.

- **pressuposto errado:** "a view é otimista e reconcilia com um refresh"
  era tratado como inofensivo. não era: um refresh lido antes de uma
  mutação pousava por cima dela, e a projeção mostrava por um instante um
  estado que não era nem o do git nem o que o ide aceitou. o agente
  roteirizado sem a espera que o contornava deu errado nas dez corridas
  com intervenção na base (nove recusas à toa ou decisões erradas, e um
  commit de dois arquivos em vez de três).
- **o que isso ensina sobre o contrato:** a checagem de stale protege
  contra o mundo que mudou, não contra o app que mostra um mundo errado.
  o contrato ficou coerente o tempo todo; quem mentiu foi a projeção.
  projeção verdadeira é uma invariante de cada `Operable`, que o `sem`
  não tem como garantir: cada app precisa de teste que prove que a
  projeção nunca fica mais velha que uma mutação aceita e converge para
  a verdade.
- **o checkpoint a teve sorte que não mediu:** os dois agentes LLM
  escaparam da corrida por tempo. H1 e H2 continuam de pé, mas a
  confiabilidade do resultado dependia de algo que o experimento não
  controlava. a nova rodada do experimento, depois das tasks 83 a 85, é
  necessária também por isso.
- **o cenário da tese ainda não rodou num app de verdade:** o binário
  `ide` não abre a porta do agente; o checkpoint a rodou no `ide-lab`,
  com o workbench sem janela e uma pessoa roteirizada. "uma pessoa usa o
  app enquanto um agente o opera" nunca aconteceu num app que alguém usa.
- **digitação fora do contrato:** no todo e no ide, digitar muda o texto
  projetado sem request nem receipt (o ADR de ligação isenta de
  propósito). consequência: nem o `acted` da task-83 nem o recorder sabem
  dizer quem digitou. fica como limite registrado até a task-64
  (inspector) precisar.
- **cobertura semântica fina:** dos 19 widgets com `project`, só
  `TextField` e `Toast` têm chamador de produção; só o todo e o ide
  projetam. para superfícies geradas (69) isso vai importar.

### 2026-09-23, checkpoint a (task-62 fase c)

fonte: `steps/ongoing/task-62-ide-human-agent-proof.md`, "resultado da fase
c"; saídas em `target/agents/exp/`.

- **confirmado (H1, H2, H3, H5):** um agente LLM sem contexto do repositório
  opera o ide só pelo contrato (`plev observe`/`act`), sem screenshot e sem
  `git`; a ação sobre estado velho é recusada antes do efeito, com o delta,
  e o HEAD não se move; o agente relê, repara e conclui; o receipt gravado
  na sessão bate com o git real.
- **onde o plev ganha (H4):** a recusa vem antes do efeito e nomeia o que
  mudou. no braço `git` com a mesma intervenção, o commit errado existiu, o
  git não avisou (exit 0), e o conserto foi `commit --amend`, que só
  funciona em commit local.
- **onde não ganha (H4):** volume e idas e voltas. 22 KB em 30 mensagens
  (plev) contra 1,5 KB em 20 invocações (`git`). tokens comparáveis. uma
  conexão por comando do `plev` pesa nas idas e voltas.
- **o que mudou no programa:** as três lacunas de contrato que os agentes
  apontaram viraram a task-83 (delta com o nó, recusa com autor, regra de
  stale escrita) e a corrida do `Refresh` virou a task-84, antes da onda 5
  (decisão do brenner: "correções primeiro"). a onda 5 espera o
  experimento rodar de novo com as correções.
- **a medir, não a supor:** se a conexão persistente (um processo `plev`
  por sessão de agente) muda o placar de idas e voltas. é ferramenta, não
  contrato; entra só se o re-run mostrar que o custo importa.

### 2026-09-23, forma enxuta (task-57b)

- a observação inteira do todo caiu de 6419 para 2039 B (3 itens) e de 87
  para 26 KB (103 itens) só tirando `source` e campos no default. o custo de
  token do contrato era forma, não conteúdo.

## processo

### 2026-09-24, um canal novo herda as promessas dos antigos

- **o canal de cena reabriu o que o item 10 fechou:** o briefing da
  task-79 falou de segurança de rede (localhost, pareamento), mas não
  lembrou que todo cliente do port é agente e que a câmera tinha
  promessa própria (pixels nunca vão para agente). o agente seguiu o
  briefing e o teste dele provou o vazamento. regra para briefing de
  canal ou de saída nova: listar as promessas de privacidade já
  aceitas nos ADRs (câmera, redação do `sem`, só pessoa) e exigir um
  teste de cada uma contra o canal novo.

### 2026-09-24, dois checkouts num target dir só

- **o cargo rodou um `regress` velho:** um script de captura com o
  `REPO_ROOT` errado compilou o exemplo do checkout principal em
  `target/agents/one`; o `cargo run` seguinte no worktree executou esse
  binário, porque o cargo não recopia o executável quando o build local
  está em dia. regra: um target dir serve a um checkout só; quem precisar
  compilar o checkout principal usa outro dir, e antes de um gate que
  valida um exemplo o orquestrador toca a fonte do exemplo para forçar o
  rebuild.

### 2026-09-24, `trash` não libera disco

- **o orquestrador mandou um build dir de 3 GB para a Lixeira:** o
  `/usr/bin/trash` move para a Lixeira do mesmo volume, então o espaço
  não volta, e esta sessão não tem permissão de ler a Lixeira para
  desfazer. esvaziar a Lixeira é do brenner (apaga as coisas dele
  também). regra: cache de build se limpa com `cargo clean` (com
  `--target-dir` ou `--profile`, e antes um `--dry-run` que mostra o que
  sai); `trash` fica para arquivo que alguém pode querer de volta.

### 2026-09-24, documentação de biblioteca não é fonte

- **o briefing do item 6 citou a doc do winit como fato:** a doc dizia
  que o winit não registra delegado de aplicação no macOS; a fonte da
  versão travada no lock registra, e trocar o delegado derruba o laço. o
  agente leu a fonte na sonda e parou. regra para o briefing: afirmação
  sobre biblioteca vem da fonte da versão do lock, com arquivo e linha,
  não da doc; e uma frase de handoff anterior ("o engine já tem X")
  se confere no manifesto antes de virar premissa.

### 2026-09-24, a lição do item 9 chegou ao item 10

- **o briefing citou o `print_wake`, e o agente aplicou:** o estado
  "começando" da câmera conta como vivo e pede prazo ao laço, e o frame
  entra só no wake visível. escrever o achado da revisão anterior no
  briefing seguinte evitou o mesmo defeito.

### 2026-09-24, trabalho de fundo precisa acordar o laço

- **o teste chamava o `settle` direto e não via o laço dormindo:** no
  item 9, os testes do job de impressão liam a resposta chamando
  `settle_print` num loop, então passavam. na janela real, com render on
  demand e nada animando, ninguém chamava o `settle` até o próximo
  input. a revisão do diff achou pela ausência de um `Wake::after` onde
  a Surface e a Silences têm um. regra para briefing e revisão: todo
  trabalho que responde por canal ao lado do laço pede um prazo no
  `Wake` enquanto está fora, e o teste prova o prazo, não só a resposta.

### 2026-09-24, um milissegundo que derruba o gate

- **o gate caiu num teste que não era da task:** na integração da fase 4c,
  `the_agent_asks_the_person_who_declines_and_it_stops` (task-93) caiu
  uma vez em 20 corridas sob carga e nenhuma sozinha. a causa: o
  laboratório lia o relógio para gravar o clique da pessoa e o dispatcher
  lia de novo dentro do clique para carimbar o estágio; num limite de
  milissegundo os dois divergem por 1, e o teste exige o mesmo instante.
  correção na fonte, não no teste: o clique é gravado no instante do
  estágio que ele causou (o mesmo `answered_at` do relatório). regra: um
  fato gravado em dois lugares vem de uma leitura só do relógio.
- **o orquestrador publicou antes de ler o gate:** o comando que
  commitava e abria o PR estava encadeado depois do `tail` do gate, sem
  condição. o PR #76 ficou aberto com o gate vermelho até a correção.
  regra: a publicação só entra no mesmo comando quando está condicionada
  ao `exit 0` do gate (`&&` sobre o resultado, não sobre o `tail`).

### 2026-09-24, briefing com premissa velha (fase 4a)

- **"congelado" tinha deixado de valer:** o briefing da fase 4a chamou o
  `view/dispatch.rs` de congelado e proibiu tocá-lo; ele saiu da lista
  `FROZEN` na fase 1 e tem 338 linhas. o agente parou certo e perdeu uma
  rodada. quarto briefing do ciclo com premissa que o código já não
  sustentava (83, 86, 90, 4a). regra: toda proibição de arquivo num
  briefing vem com o motivo conferido na hora (`grep` no guard, `wc -l`),
  não copiado do briefing anterior.

### 2026-09-24, o auditor de transcrição

- **o auditor contava errado um `;` entre aspas:** os dois `--ask` da
  quinta rodada tinham `;` no texto, e o auditor partia a linha ali e
  contava uma palavra da pergunta como programa. o erro só soma (nunca
  esconde um programa), então as contagens de zero das rodadas
  anteriores valem. o partidor agora respeita aspas, e um `--ask` com
  `;` e `|` virou caso de teste dele.

### 2026-09-24

- **o CI parou por cobrança, e o merge espera:** às 06:22 UTC todo job
  do PR #68 e o da `main` depois do #67 falharam em segundos, sem
  runner. a anotação do github diz que os jobs não começaram porque um
  pagamento da conta falhou ou o limite de gasto precisa subir. é
  serviço pago (condição de parada da autonomia): o orquestrador não
  mergeia sem o CI, continua integrando na `main` local com o gate, e a
  publicação espera o brenner (ele decidiu: mergear com o gate local
  verde enquanto o CI estiver fora, journal). uma falha de todos os jobs em segundos
  se lê pelas anotações antes de se ler como código quebrado.

### 2026-09-23

- **teste do caminho inteiro pega o que teste de peça não pega:** a
  recusa de confirmação (58b) foi testada no `sem` e no ide, e o porto foi
  testado com atos que terminam pelo git. ninguém testou a recusa pela
  porta real com o shell servindo só no wake. a sonda da 92, que
  atravessou tudo, achou o estágio parado. um contrato que atravessa
  crates pede um teste pelo caminho inteiro, porta real e host real.

- **briefing que proíbe o que o gate exige:** o briefing da 68 proibiu
  `README.md` e `docs/arc/arc.toml`, e o `arc_sync_guard` exige de todo
  crate novo uma entrada nos dois. o agente parou com o gate vermelho
  num teste só. regra: crate novo sempre leva, nos arquivos permitidos, a
  própria linha no README e a entrada no `arc.toml`, como a seção 8 do
  programa já dizia. o agente achou também que o guard passava por acaso
  para o `uir` no `arc.toml` (a palavra "acquire" contém "uir"): o guard
  procura texto, não entrada.

- **a guarda de merge nunca guardou:** o orquestrador decidia o merge
  com `if ! grep -qv pass`, e o `grep` desta shell é uma função de
  snapshot que devolveu 1 com uma linha `fail` no arquivo. todo PR até o
  #58 estava verde, então nada entrou vermelho por isso. o PR #59 entrou
  vermelho por outro caminho (merge feito na conta do github às 00:24, com
  o teste do macOS já falhando), e a guarda também não teria segurado.
  regra: a decisão de merge lê as colunas com `awk -F'\t' '$2!="pass"'` e
  só mergeia com zero linhas; e o teste que falhou era de tempo (task-91),
  corrigido em seguida (PR #60).

- **não roubar o foco dá mais trabalho do que parece no macOS:** só
  não ativar o app não segurou o foco quando o app da frente era o que
  lançou o processo (o Zed). o macOS 14 deixa esse app ceder a ativação
  por cooperação. o que segurou foi a política de ativação proibida com
  a janela sempre por baixo (`PLEV_BACKGROUND=1`). a prova em janela real
  com o brenner na máquina usa isso, e a janela sobe sem que ninguém
  possa usá-la.

- **a sonda antes do briefing pagou na primeira vez:** o briefing da
  88 marcou o diagnóstico como hipótese e pediu a sonda primeiro. ela
  mostrou que o gatilho proposto para a pessoa quebraria o teste do gate
  e que a medição proposta dava falso válido. nenhuma linha de correção
  foi escrita em cima da premissa errada.

- **limitação "lida no código, não rodada" não vira cenário de task sem
  sonda:** o handoff da 86 descreveu a limitação do `acted` lendo o
  código, e eu escrevi a task-90 com esse cenário. a sonda do agente, pelo
  clique real, mostrou outro mecanismo (a seleção muda a revisão fora do
  dispatcher) e o `acted` vazio, não nomeando a seleção. terceiro briefing
  do ciclo com premissa errada (83, 86, 90). em todos, o agente conferiu
  antes de implementar, porque o briefing mandava confirmar e parar. regra
  a partir daqui: o cenário que motiva uma task nova é reproduzido por
  teste ou sonda antes do briefing, ou o briefing o marca como hipótese e
  pede a sonda como primeiro passo.

- **prova em janela real na máquina do brenner:** a primeira tentativa
  da prova da task-86 postou um clique sintético que caiu no navegador
  dele, que estava na frente; na reprodução do orquestrador, a janela
  nova do ide tomou o foco e recebeu o que ele fazia. regra: o harness
  confere a janela no ponto antes de cada clique (a 86 passou a fazer);
  com o brenner na máquina, a reprodução do orquestrador usa o teste de
  porta sem janela e deixa a janela para a prova do agente; um processo
  morto pelo pid deixa o socket, que sai com `trash`.

- **premissa do briefing conferida no código, de novo:** o briefing da
  86 dizia que `AgentPort::open` respeita `PLEV_AGENT=1` e debug; quem
  respeita é `from_env`. o agente seguiu o contrato ("release fechado")
  e apontou a divergência. é o segundo briefing do ciclo com premissa
  errada (a primeira foi a da 83); ao citar o comportamento de uma
  função no briefing, ler o doc dela antes.

- **regra sem gate não vale:** o limite de 639 linhas de produção por
  arquivo estava na convenção e nenhum gate conferia; três entregas do
  ciclo passaram dele sem aviso, e o próprio orquestrador contou errado
  uma vez (contou o arquivo inteiro, não a produção). agora o gate conta
  (`crates/engine/tests/file_hygiene_guard.rs`), com a regra exata e os
  seis arquivos que já passavam congelados no tamanho de hoje; um
  arquivo de prova com 700 linhas fez o guard falhar com o nome dele.

- **o auditor de transcrição dava falso positivo:** um agente guardou o
  caminho do `plev` numa variável (`P=...; $P observe`) e o auditor, que
  olhava o começo da linha, marcou 12 comandos como proibidos. conferido
  à mão antes de concluir; o script agora resolve a variável. regra:
  número de auditoria que acusa o agente se confere na transcrição antes
  de entrar no resultado.

- **dois workspaces num target dir:** com a perna urnaui rodando nos
  worktrees, o workspace raiz e o do urnaui passaram a gravar no mesmo
  `CARGO_TARGET_DIR`; o engine, com crate-type cdylib e staticlib, grava
  `libengine.rlib` sem hash no nome, e um workspace pegava o arquivo do
  outro ("multiple different versions of crate `sem`"). a classe:
  artefato sem hash compartilhado entre workspaces. o gate agora dá à
  perna urnaui um subdiretório próprio (`1bf6606`). achado pelo agente da
  task-83, que o contornou com `cargo clean -p engine` e reportou em vez
  de mexer no gate.

- **premissa de compatibilidade errada no briefing:** o briefing da
  task-83 afirmou que nenhum arquivo gravado continha `node_added`,
  conferido só no `Kind::Changes` do recorder. o `Kind::Snapshot` guarda o
  diff do passo, com `node_added`, e as sessões do checkpoint a deixariam
  de abrir. o agente achou abrindo as sessões de verdade. regra: toda
  afirmação de compatibilidade num briefing se confere contra os
  artefatos gravados de verdade (abrir os arquivos), não só lendo o
  código do escritor.

- **contorno de harness escondeu defeito de produto:** o agente
  roteirizado esperava o index "parar" (até 200 observações), e isso
  escondia a corrida do `Refresh`. regra: um harness ou agente que
  espera o app "assentar" antes de agir é sintoma; a espera vira bug do
  app, registrado, nunca contorno silencioso.
- **a perna urnaui do gate pulava em todo worktree:** o gate procura
  `../urna` a partir do worktree, que não existe. o symlink
  `.claude/worktrees/urna` fez a perna rodar (a primeira vez foi no gate
  da task-84).
- **32 divergências doc x código acumuladas:** os fechamentos escreviam
  o doc novo e não corrigiam os docs que a mudança contradizia (a 58b
  trouxe `decline` e o ADR commit-checks continuou dizendo que não
  existia). regra nova do fechamento: procurar em `docs/` e no README os
  nomes do contrato que mudou e corrigir o que ficou falso
  (`docs/how-to/orchestrate-coding-agents-on-this-repo.md`).

- **o gate local divergia do CI:** a 66b deixou `ShapedGlyph` e
  `shaped_glyphs` como código morto no wasm32 (o único leitor, o export
  svg, é só nativo). o CI (`RUSTFLAGS=-D warnings`) reprovou os jobs wasm e
  urnaui do PR #45; o gate local passou porque a perna wasm rodava sem `-D
  warnings` e nunca checava o urnaui no wasm32. a classe: uma perna do
  gate mais frouxa que a do CI deixa passar o que o CI reprova. o gate
  agora roda as duas pernas wasm como o CI (`c07a7f0`); a falha reproduziu
  localmente antes da correção e sumiu depois. regra: toda perna do CI tem
  a mesma perna, com as mesmas flags, em `script/gate`.

- **graça do laboratório:** o laboratório fechava 1 s depois do commit e
  derrubou a verificação final de dois agentes (A2, B2). a classe: um
  harness que encerra no primeiro sinal de sucesso mede o sucesso e apaga a
  verificação. o `--grace` passou a ser parte do protocolo (180 s nas
  corridas refeitas). regra para os próximos experimentos: o harness só
  encerra depois que o agente disse que terminou, ou no timeout.
- **agentes parados:** o brenner viu agentes 16 minutos parados numa task.
  o ciclo autônomo agora volta a cada 10 min com agente rodando e confere
  commits e arquivos alterados; parado, pergunta; parado de novo,
  rebriefing (`autonomy.md`, "máquina").
- **disco:** três worktrees compilando em paralelo encheram o disco. um
  target dir por agente, incremental desligado, debug só com line tables;
  no máximo dois ou três agentes compilando ao mesmo tempo com 30 GB
  livres.
