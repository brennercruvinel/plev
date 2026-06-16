---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# reference analysis: tui patterns

data: 2026-03-11
status: completo

## escopo

padroes de UX extraidos de aplicacoes tui em rust que se traduzem para frameworks de UI baseados em GPU como φ. o foco e em navegacao, composicao de componentes, atualizacao de dados em tempo real, busca/filtro e layout responsivo, nao em detalhes de renderizacao terminal.

---

## repositorios analisados

### yazi (sxyazi/yazi), 33.7k stars

gerenciador de arquivos terminal com i/o assincrono via tokio. layout tres-colunas (diretorio pai, atual, preview). suporte a tabs multiplas com selecao cross-diretorio. sistema de plugins lua 5.5 com cinco tipos especializados (fetchers, spotters, preloaders, previewers, functional). event loop batcha ate 50 eventos por iteracao com throttle de 10ms para renders. scheduler com filas de prioridade dual (micro tasks para metadata, macro tasks para operacoes pesadas). input routing hierarquico: keybindings -> actions -> eventos, sem acoplamento direto. data distribution service para comunicacao cross-instancia e persistencia de estado. suporte a mouse integrado ao sistema de input.

### gitui (gitui-org/gitui), 21.5k stars

cliente git tui com foco em operacoes que sao dolorosas no shell (staging de hunks/lines, stashing). arquitetura modular com crates internas separadas: `asyncgit` (operacoes git assincronas), `filetreelist` (exibicao de arvore), `git2-hooks` (pre-commit, commit-msg). UI em tabs/paineis com keybindings contextuais, help e baseado no contexto atual, nao requer memorizacao de atalhos. staging granular em tres niveis: arquivo, hunk, linha. async git API demonstra parse de 900k commits em 24 segundos sem freeze da UI.

### gitu (altsem/gitu), 2.6k stars

cliente git tui inspirado no magit do emacs. keybindings mimetizam magit mantendo compatibilidade vim-like. interface modal com comandos hierarquicos: operacoes principais (staging, branching, committing) ramificam em sub-operacoes. help menu acessivel via `h` ou exibicao persistente configuravel. configuracao via TOML com paths platform-specific. integracao com editor externo respeitando `VISUAL`, `EDITOR`, `GIT_EDITOR` em ordem de preferencia.

### bottom (clementtsang/bottom), 13k stars

monitor de sistema cross-platform com widgets configuráveis. layout definido em TOML com hierarquia `[[row]]` -> `[[row.child]]` -> `[[row.child.child]]`, onde cada nivel pode ser coluna ou widget. sistema de ratios proporcionais para dimensionamento (default 1:1). sete tipos de widget (cpu, mem, proc, net, temp, disk, batt) mais `empty` para espacamento. modo de expansao para focar em widget unico. modo basico htop-inspired como alternativa. zoom temporal in/out no intervalo de tempo exibido. process widget com tree mode e busca/filtro inline. temas customizaveis via config file (gruvbox incluso como exemplo).

### trippy (fujiapple852/trippy), 6.7k stars

ferramenta de diagnostico de rede combinando traceroute e ping. multiplos modos de saida (tui interativo, stream, tabela, markdown, csv, JSON, graphviz dot, flows, silent). tui com visualizacao hop-by-hop com charts em tempo real. tabs numeradas (1-7) para configuracoes especificas (tui, trace, dns, geoip, bindings, theme, columns). toggle granular de elementos visuais: chart (`c`), mapa geoip (`m`), flows (`f`), detalhes de hop (`d`), info as (`z`). zoom de chart via `=`/`-`. contrai/expande hosts por hop via `[`/`]`. freeze de display via `ctrl+f`. keybindings customizaveis via config file com suporte a modificadores (shift, ctrl, alt, super, hyper, meta).

### television (alexpasmantier/television), 4.4k stars

fuzzy finder modular baseado em "canais" (channels) como abstracoes de fontes de dados. cada canal responde a queries do usuario e retorna lista de entries. canais built-in: files, text content, git repos, env vars, docker containers. canais customizados via TOML em `~/.config/television/cable/` especificando comando de fonte, logica de preview, keybindings e acoes. transicoes entre canais na mesma sessao (ex: git repos -> arquivos -> conteudo). fuzzy matching via nucleo (mesmo usado pelo editor helix). async search via tokio sem bloquear input do usuario. preview sincronizada com item selecionado. integracao shell (`tv init [shell]`) para autocomplete (ctrl+t) e historico (ctrl+r). plugins para neovim, vim, vscode, zed.

### oha (hatoo/oha), 10.1k stars

gerador de carga http com tui em tempo real via ratatui. charts de latencia e throughput atualizados continuamente durante execucao de requests. FPS configuravel via `--fps` (default 16). flag `--no-tui` para modo mais rapido sem overhead de coleta de dados em tempo real. requests executam assincronamente via tokio workers; eventos de conclusao alimentam agregador de metricas que calcula estatisticas; dados agregados fluem para camada de rendering a cada frame tick. suporte a http/1.1, http/2, http/3 (experimental). multiplos formatos de saida (text, JSON, csv). correcao de latencia para evitar coordinated omission problem.

### gobang (tako8ki/gobang), 3.3k stars, alpha

cliente de banco de dados tui com suporte a mysql, postgresql, sqlite. layout multi-painel: painel de conexoes (`c`), views tabuladas acessiveis por numeros 1-5 (records, columns, constraints, foreign keys, indexes). navegacao vim-style completa: `h,j,k,l` para scroll, `H,J,K,L` para estender selecao de celula, `y` para copiar valor, `g`/`G` para limites do documento, `Ctrl+U/D` para scroll rapido. filtro inline via `/`. pop-ups modais dismissiveis com escape. configuracao de conexoes via TOML com suporte a multiplas conexoes simultaneas.

### zenith (bvaisvil/zenith), 3k stars

monitor de sistema com charts zoom-aveis e persistencia de dados entre sessoes via database local (`~/.zenith`). charts de CPU, memoria, rede, disco com navegacao temporal: setas esquerda/direita movem no historico, backtick reseta para presente. zoom via `+`/`-`. tab alterna entre secoes (CPU, memory, network, disk, GPU). secoes expandem (`e`) e minimizam (`m`) individualmente. suporte nvidia GPU via feature flag. metricas por processo incluem uso de disco.

### basalt (erikjuhani/basalt), 1.1k stars

tui para gerenciamento de notas obsidian. interface com painel lateral para selecao de notas dentro de um vault e area principal com rendering wysiwyg de markdown. tres modos de interacao: select, normal, insert, seguindo paradigma modal de editores vim. barra inferior com informacoes contextuais (modo atual, contagem de palavras/caracteres). scroll position persistente entre navegacoes. arquitetura modular com crates separadas (basalt-core, basalt-widgets) sobre ratatui. keybindings customizaveis via TOML.

### oxker (mrjackwills/oxker), 1.6k stars

tui para docker com tres paineis principais: lista de containers, logs em tempo real, stats/inspect. navegacao entre paineis via tab/shift+tab ou click direto. intervalo de atualizacao configuravel via `-d [ms]` (default 1000ms). dois modos de filtragem: filtro de containers (f1 ou `/`) e busca em logs (`#`). vim-style bindings (`j`/`k` para scroll). teclas numericas (1-9) para ordenacao de colunas. altura de paineis ajustavel (`-`/`=`). logs exportaveis para arquivo local (`s`). config file para persistir keymaps, esquema de cores e preferencias.

### ratzilla (ratatui/ratzilla), 1.3k stars

framework para aplicacoes web com estetica terminal, usando rust e webassembly sobre ratatui. dois backends de rendering: dombackend (DOM elements) e canvasbackend (HTML canvas/webgl2). desenvolvedores escrevem com widgets familiares de ratatui (block, paragraph, layout) mas deploy e para browser. compilacao: rust -> WASM (wasm32-unknown-unknown) -> trunk -> browser. eventos de teclado capturados do ambiente web. metodo `draw_web` para rendering frame-based, habilitando animacoes via atualizacao de estado entre ciclos de render.

### ohmyzsh (ohmyzsh/ohmyzsh), 180k+ stars

framework de extensibilidade para shell zsh (nao e rust, mas os patterns de plugin sao universais). plugins sao opcionais e independentes, core nao depende de nenhum plugin. lifecycle de carregamento em quatro estagios: library files (alfabeticamente) -> custom user code -> plugins (pre-loaded via fpath) -> theme. convencao sobre configuracao: resolucao de paths via `${0:h}`, naming `ZSH_THEME_*` para theming. degradacao graciosa: funcoes dummy (`XXX_prompt_info()`) permitem que temas usem funcoes de plugins incondicionalmente, retornando string vazia se plugin nao carregado. hooks composiveis via `chpwd_functions` e `precmd_functions` (listas, nao variaveis unicas), permitindo multiplas extensoes simultaneas sem conflito.

---

## padroes extraidos

### navegacao

**1. vim-like como lingua franca.** todas as 12 aplicacoes tui analisadas usam `h,j,k,l` ou `Up/Down/Left/Right` como base de navegacao, com 10 delas oferecendo explicitamente keybindings vim-style. isso indica que usuarios de ferramentas tecnicas esperam esse vocabulario de movimento.

**2. keybindings contextuais superam memorizacao.** gitui enfatiza "context based help (no need to memorize tons of hot-keys)". trippy oferece `h` ou `?` para toggle de help. gitu mostra help persistente configuravel. o padrao e: mostrar atalhos relevantes ao contexto atual, nao exigir memorizacao de tabela completa.

**3. tabs numericas para acesso direto.** trippy usa teclas 1-7 para tabs de configuracao. gobang usa 1-5 para views de tabela. bottom usa tabs para widgets. o acesso via numero e mais rapido que tab cycling sequencial.

**4. modal editing para contextos distintos.** basalt implementa tres modos (select, normal, insert). gitu usa interface modal hierarquica. o padrao modal permite que as mesmas teclas signifiquem coisas diferentes por contexto, aumentando a densidade de comandos por tecla.

**5. expansao/foco de widget.** bottom permite expandir um widget para tela cheia. zenith permite expandir/minimizar secoes individuais. trippy faz toggle de chart, mapa, flows independentemente. o padrao e: cada area visual pode ser maximizada para foco temporario.

**6. mouse como complemento, nao substituto.** yazi, oxker e gitui suportam mouse (click em paineis, scroll) mas nao dependem dele. o mouse e atalho para operacoes que o teclado ja cobre.

### composicao de componentes

**1. widget trait como contrato de rendering.** ratatui (usado por 8 das 12 apps analisadas) define widget trait onde cada elemento visual implementa `render()` recebendo area (`Rect`) e buffer. statefulwidget separa widget de seu estado, permitindo que a aplicacao gerencie estado independentemente.

**2. separacao modular via crates internas.** gitui separa `asyncgit`, `filetreelist`, `git2-hooks`. yazi tem 30+ crates (yazi-core, yazi-fm, yazi-vfs). basalt separa `basalt-core` e `basalt-widgets`. o padrao e: logica de dominio em crate independente da UI.

**3. component architecture com estado local.** a arquitetura de componentes do ratatui define: `init()`, `handle_events()`, `handle_key_events()`, `handle_mouse_events()`, `update()`, `render()`. cada componente encapsula estado proprio, event handlers e logica de rendering. actions servem como mecanismo de passagem de mensagens entre componentes.

**4. canais como abstracoes de fonte de dados.** television implementa "channels" onde cada canal responde a queries e retorna entries. canais sao definidos declarativamente via TOML. o padrao e: a UI nao conhece a fonte de dados, apenas consome uma interface de canal.

**5. degradacao graciosa.** oh-my-zsh implementa funcoes dummy para plugins nao carregados. o padrao e: componentes opcionais devem ter fallback que nao quebra o sistema. temas podem referenciar funcoes de plugins sem verificar se estao presentes.

### dados em tempo real

**1. async runtime separado da UI.** oha executa requests via tokio workers; agregador de metricas calcula estatisticas; dados fluem para rendering a cada frame tick. yazi usa tokio com localset, batch de ate 50 eventos por iteracao, throttle de 10ms. gitui usa async git API. o padrao universal e: i/o nunca bloqueia a thread de rendering.

**2. FPS configuravel.** oha permite `--fps` (default 16). oxker usa `-d [ms]` (default 1000ms) para intervalo de atualizacao. zenith permite configurar refresh rate. o padrao e: a taxa de atualizacao visual e independente da taxa de coleta de dados.

**3. persistencia temporal.** zenith armazena metricas em database local para navegacao historica (setas esquerda/direita no tempo). bottom permite zoom temporal no intervalo exibido. o padrao e: dados em tempo real nao precisam ser efemeros, historico habilita analise.

**4. freeze de display.** trippy implementa `ctrl+f` para congelar a visualizacao sem parar a coleta de dados. o padrao e: separar captura de dados de exibicao permite que o usuario pause para analisar sem perder dados.

**5. filas de prioridade para tasks.** yazi separa micro tasks (metadata, mime detection) de macro tasks (transferencia de arquivos) com workers dedicados. macro workers fazem load balancing processando ambos tipos. o padrao e: nem todas as operacoes async tem a mesma prioridade.

### busca e filtro

**1. fuzzy matching como expectativa.** television usa nucleo (mesmo do helix) para fuzzy matching em tempo real. yazi integra com fzf, ripgrep, fd. o padrao e: busca exata e insuficiente, usuarios esperam matching tolerante a erros.

**2. filtro inline sem modal.** bottom permite filtrar no process widget inline. gobang usa `/` para filtro. oxker tem filtro de containers via `F1` ou `/`. o padrao e: filtro deve ser acionavel com uma tecla e aplicado incrementalmente.

**3. busca contextual por painel.** oxker separa filtro de containers (f1/`/`) de busca em logs (`#`). o padrao e: o contexto determina o escopo da busca, busca global e rara.

**4. transicoes entre escopos.** television permite transicionar entre canais (git repos -> arquivos -> conteudo textual) refinando a busca progressivamente. o padrao e: resultados de uma busca alimentam o escopo da proxima.

### layout responsivo

**1. hierarquia row/column/widget.** bottom define layout via TOML: `[[row]]` contem `[[row.child]]` que contem widgets ou sub-colunas. ratios proporcionais controlam dimensionamento (default 1). o padrao e: layout declarativo hierarquico com pesos relativos.

**2. split panes com resize.** yazi usa tres colunas (pai, atual, preview). gobang separa conexoes, tabela, detalhes. oxker permite ajustar altura de paineis via `-`/`=`. o padrao e: paineis divididos com proporcoes ajustaveis pelo usuario em runtime.

**3. toggle de visibilidade por componente.** trippy permite toggle individual de chart, mapa, flows, detalhes. oxker permite toggle de paineis com `\`. o padrao e: cada area visual pode ser ocultada para dar mais espaco as demais, redistribuindo automaticamente.

**4. widget duplicavel.** bottom permite widgets duplicados no layout (ex: dois CPU widgets com configuracoes diferentes). o padrao e: componentes sao instanciaveis, nao singletons.

**5. dois backends de rendering.** ratzilla oferece dombackend (semantico) e canvasbackend (performante) com a mesma API de widgets. o padrao e: abstracoes de layout devem ser independentes do backend de rendering.

---

## implicacoes para φ

### navegacao

o sistema de input de φ (`input/mod.rs`, `input/touch.rs`, `input/gesture.rs`) ja suporta event queue e hit-testing. os padroes tui indicam que:

- **keybindings contextuais** sao mais usaveis que atalhos globais. o gesturerecognizer de 6 estados de φ poderia ser estendido com um sistema de contexto que altera o significado de gestos conforme o componente focado.
- **focus management** (qual componente recebe input) e prerequisito para qualquer sistema de navegacao. os tuis resolvem isso com tab cycling e click-to-focus. φ precisara de um focus tree integrado ao hit-testing existente.
- **help contextual** (mostrar atalhos disponiveis) e expectativa de usuarios. um componente de overlay que mostra keybindings do contexto atual seria equivalente ao `?`/`h` dos tuis.

### composicao de componentes

o sistema view/component de φ (`view.rs`, `component.rs`, `builder.rs`) ja define lifecycle (mount/update/unmount). os padroes tui referencam:

- **estado local por componente** (nao global) e o padrao dominante. o sistema de signals de φ (`signal.rs`) ja suporta isso via `create_signal()` com readsignal/writesignal por componente.
- **actions como mensagens** entre componentes (ratatui component architecture) e analogo ao sistema de signals push-pull de φ. a diferenca e que signals sao reativos (push automatico) enquanto actions sao imperativas (dispatch explicito). φ pode oferecer ambos.
- **canais/fontes de dados abstratas** (television) sugerem que φ deveria separar a interface de dados (trait datasource ou similar) da UI que consome esses dados. isso permitiria que o mesmo componente visual renderize dados de qualquer fonte sem acoplamento.

### dados em tempo real

- **separacao i/o da thread de rendering** e universal nos tuis e ja e pratica em φ (GPU init async, compositor com dirty tracking). o padrao de oha (tokio workers -> agregador -> render tick) e modelo direto para dashboards sobre φ.
- **FPS configuravel** e relevante para φ. o render loop de φ poderia expor um knob de FPS target, permitindo que apps de monitoramento operem a 16 FPS enquanto apps interativos rodem a 60 FPS, economizando GPU.
- **historico temporal** (zenith) requer que o sistema de dados mantenha buffer circular ou persistencia. isso e responsabilidade da app, nao do engine, mas φ poderia oferecer primitivas (ring buffer signal, time-series data source) que facilitam implementacao.

### busca e filtro

- **fuzzy matching** (nucleo, fzf) e expectativa em qualquer campo de busca moderno. φ nao precisa implementar fuzzy matching no engine, isso e responsabilidade da app. porem, um componente de texto input com hook para filtro customizado (via closure ou signal) facilitaria integracao com libs como nucleo.
- **filtro incremental** (cada tecla refina resultados) requer que o rendering seja rapido o suficiente para re-layout a cada keystroke. o dirty tracking do compositor φ (fxhasher, unchanged layer = zero GPU work) ja viabiliza isso.

### layout responsivo

- **layout declarativo hierarquico** e o que φ ja tem via taffy 0.9 flexbox (`layout.rs`). os padroes tui confirmam que row/column/ratio e suficiente para a maioria dos layouts.
- **toggle de visibilidade** com redistribuicao automatica de espaco e nativo em flexbox (display:none colapsa o node). φ ja suporta isso via `set_layer_visible()`.
- **resize interativo de paineis** (arrastar divisor) requer gestures sobre divisores virtuais. o sistema de input de φ (touch.rs, gesture.rs) ja tem o gesturerecognizer necessario, falta apenas o componente de divisor que traduz drag em mudanca de flex-basis.
- **backend-agnostic widgets** (ratzilla dombackend vs canvasbackend) validam a decisao de φ de manter render loop, shaders e scene graph sem branches de plataforma. widgets φ devem emitir scenenodes sem saber se o backend e metal, vulkan, dx12 ou webgpu.

### plugin/extensibilidade

- **degradacao graciosa** (ohmyzsh) e aplicavel ao sistema de componentes de φ. componentes opcionais devem ter fallback (render vazio, noop handlers) em vez de panic quando dependencias estao ausentes.
- **lifecycle de carregamento ordenado** (ohmyzsh: libs -> custom -> plugins -> theme) e relevante para inicializacao de apps φ. o mount order de componentes deveria ser determinístico e documentado.
- **plugin system via scripting** (yazi/lua) nao e prioridade para φ engine, mas apps construidas sobre φ poderiam adotar o pattern de yazi: runtime lua embutido com API globals tipadas para acesso controlado ao estado da aplicacao.
