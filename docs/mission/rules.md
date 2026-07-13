# Regras de arquitetura do plev

- **id:** idx-rules
- **typ:** index
- **sts:** reference
- **dom:** architecture-rule
- **dat:** 2022-11-14

As 15 regras-principio (antes mantras + rules, fundidos em um no por principio) que governam a arquitetura do plev. Cada no `rul-nn` e uma regra com seu corpo cru. As arestas `lnk` ligam regras relacionadas: rul-07 -> rul-11 (side effects e persistencia via trait), rul-12 -> rul-10 (i18n depende de text layout).

Ordem: 01 fronteira app/engine, 02 estado de dominio fora do plev, 03 fluxo unidirecional, 04 composicao sobre heranca, 05 layout declarativo, 06 navegacao como enum, 07 side effects isolados, 08 theming como struct, 09 acessibilidade via accesskit, 10 text layout via parley, 11 persistencia via trait, 12 internacionalizacao, 13 error handling tipado, 14 props minimas, 15 testabilidade por camada.

| Regra | Dominio | Titulo |
|-------|---------|--------|
| [rul-01](#rul-01--fronteira-app-engine) | boundary | fronteira app engine |
| [rul-02](#rul-02--estado-de-dominio-fora-do-plev) | state | estado de dominio fora do plev |
| [rul-03](#rul-03--fluxo-unidirecional) | data-flow | fluxo unidirecional |
| [rul-04](#rul-04--composicao-sobre-heranca) | composition | composicao sobre heranca |
| [rul-05](#rul-05--layout-declarativo-nunca-manual) | layout | layout declarativo nunca manual |
| [rul-06](#rul-06--navegacao-como-enum) | navigation | navegacao como enum |
| [rul-07](#rul-07--side-effects-isolados-com-abstracao-de-runtime) | side-effects | side effects isolados com abstracao de runtime |
| [rul-08](#rul-08--theming-como-struct-com-dimensoes-comportamentais) | theming | theming como struct com dimensoes comportamentais |
| [rul-09](#rul-09--acessibilidade-como-constraint-via-accesskit) | accessibility | acessibilidade como constraint via accesskit |
| [rul-10](#rul-10--text-layout-via-parley-com-suporte-bidi) | text | text layout via parley com suporte bidi |
| [rul-11](#rul-11--persistencia-via-trait-com-migracao-versionada) | persistence | persistencia via trait com migracao versionada |
| [rul-12](#rul-12--internacionalizacao-alem-de-text-layout) | i18n | internacionalizacao alem de text layout |
| [rul-13](#rul-13--error-handling-tipado-e-visivel) | errors | error handling tipado e visivel |
| [rul-14](#rul-14--props-minimas-com-contexto-explicito) | components | props minimas com contexto explicito |
| [rul-15](#rul-15--testabilidade-por-camada-sem-gpu) | testing | testabilidade por camada sem gpu |

---

## rul-01 — fronteira app engine

- **dom:** boundary
- **dat:** 2022-11-14
- **lnk:** idx-rules

Codigo de aplicacao nunca importa wgpu, scenenode, gpuvec, compositor, nem qualquer tipo do rendering pipeline. A app fala com plev exclusivamente via `builder.rs` (elements) e `signal.rs` (reatividade).

Acoplamento direto com internals do renderer significa que qualquer refactor no plev, trocar packing do atlas, mudar pipeline de blur, adicionar backend metal ou vulkan, quebra codigo de app com blast radius imprevisivel. A fronteira forca que mudancas na engine sejam invisiveis para consumers. A mesma decisao esta documentada no architecture do xilem como separacao entre view e widget, a camada view e descartavel entre ciclos, a camada widget (masonry) e retained. A app nunca toca masonry diretamente.

---

## rul-02 — estado de dominio fora do plev

- **dom:** state
- **dat:** 2022-11-14
- **lnk:** idx-rules

Signals do plev armazenam exclusivamente estado de ui: scroll position, painel aberto, hover state, selecao ativa, modo de edicao. Dados de dominio, usuario autenticado, lista de entidades, sessao, configuracao persistida, vivem em structs rust puros, owned pela app, sem dependencia de plev.

Estado de dominio dentro de signals cria dependencia circular: testar logica de negocio exige instanciar o sistema reativo do plev, que exige event loop, que exige window. Zero testes unitarios na pratica porque o setup e proibitivo.

Para estado hibrido, dados de dominio que a ui precisa transformar localmente, como lista filtrada ou ordenada, o protocolo padrao e derivacao pura: o dominio expoe o dado bruto via referencia imutavel, a ui computa a view como funcao pura no momento do render, sem armazenar o resultado derivado como estado.

Excecao obrigatoria para memoizacao: quando a transformacao opera sobre colecoes grandes ou tem custo computacional mensuravel, usar no memoize equivalente ao do xilem, que prune a view tree quando as dependencias nao mudaram entre ciclos. Memoizacao e cache controlado com invalidacao explicita por dependencia, nao e estado de dominio, nao viola a separacao. Derivacao e funcao nao cache e o padrao; memoizacao com dependencias declaradas e a excecao justificada por profile, nao por preferencia.

---

## rul-03 — fluxo unidirecional

- **dom:** data-flow
- **dat:** 2022-11-14
- **lnk:** idx-rules

Toda mutacao de estado segue: userinput -> action (enum tipado) -> handler centralizado -> estado mutado -> re-render. Callbacks de componentes emitem actions via `actionqueue.emit()`, nunca mutam estado diretamente.

Mutacao espalhada em callbacks cria estado inconsistente, componente a muta x, callback de b le x no estado antigo, render mostra dado stale. Com fluxo unidirecional, toda mutacao e rastreavel via log do action stream, reproduzivel via replay de actions, e o estado e sempre consistente no momento do render porque mutacoes sao batch-processadas entre frames. O modelo e equivalente ao the elm architecture, que demonstrou escalar para aplicacoes de producao desde 2012. A diferenca em plev e que o enum de actions e tipado em rust com exaustividade garantida em compile time.

---

## rul-04 — composicao sobre heranca

- **dom:** composition
- **dat:** 2022-11-14
- **lnk:** idx-rules

Componentes sao funcoes `fn(props) -> element`. Sem trait objects de widget como interface publica na camada de app, sem hierarquia de tipos, sem dyn widget em composicao estatica de ui. Componente complexo e composicao de componentes simples via `child()`.

Para listas com tipos heterogeneos de item, o padrao correto e enum com variantes: `listitem::text(textitem)`, `listitem::image(imageitem)`, `listitem::action(actionitem)` com match exaustivo no render. Exaustividade em compile time, zero dispatch dinamico.

Para sistemas de plugin onde o tipo e genuinamente desconhecido em compile time, `box<dyn component>` e permitido exclusivamente no pluginregistry, modulo isolado em `src/plugins/registry.rs`, com interface publica que expoe apenas `fn registered_components() -> vec<componentdescriptor>`. Componentdescriptor e struct serializable com metadata estatico. O `box<dyn component>` nunca escapa do registry para o codigo de composicao de ui. Sem essa fronteira arquitetural explicita, camada de plugins expande ate contaminar a composicao estatica.

---

## rul-05 — layout declarativo nunca manual

- **dom:** layout
- **dat:** 2022-11-14
- **lnk:** idx-rules

Posicionamento usa exclusivamente as primitivas de layout do plev via taffy: col, row, gap, p, w, h, grow, shrink, basis. Zero coordenadas absolutas calculadas manualmente. Zero offsets hardcoded.

Plev executa em 6 plataformas com densidades de pixel radicalmente diferentes, retina 2x, android mdpi ate xxxhdpi, browser zoom, hidpi linux. Coordenadas manuais quebram em variacoes de tela, dpi, orientacao e resize. Taffy resolve constraints via flexbox e grid automaticamente com o mesmo algoritmo do browser mas sem dom. Posicao manual cria bugs que so aparecem em devices especificos, os mais caros de diagnosticar porque nao sao reproduziveis em desenvolvimento.

---

## rul-06 — navegacao como enum

- **dom:** navigation
- **dat:** 2022-11-14
- **lnk:** idx-rules

Telas e rotas da app sao variantes de um enum rust. Transicao e mutar o valor do enum no estado. O render faz match exaustivo no enum para decidir o que renderizar. Zero string matching, zero router framework.

Rust garante exaustividade no match, adicionar uma tela nova e esquecer de trata-la e erro de compilacao, nao bug em producao. Strings sao frageis: typo em `/setings` compila e mostra tela branca. Enums com dados associados, `screen::userprofile { id: userid }`, `screen::documenteditor { doc_id: docid, mode: editmode }`, carregam parametros type-safe sem runtime de routing, sem regex de path matching, verificaveis em compile time.

---

## rul-07 — side effects isolados com abstracao de runtime

- **dom:** side-effects
- **dat:** 2022-11-14
- **lnk:** idx-rules, rul-11

Nenhum io, network, filesystem, ipc, timers longos, acontece dentro de `render()`, dentro de callbacks de componente, ou de forma sincrona dentro de handlers de action. Io dispara via spawn assincrono e retorna como nova action no fluxo normal.

O spawn de tasks usa tokio no nativo e wasm-bindgen-futures no browser. Essa divergencia nao pode vazar para o codigo de dominio como `cfg(target_arch)` inline, isso criaria exatamente o acoplamento de plataforma que rul-11 proibe. O padrao correto e definir `trait taskspawner { fn spawn(&self, fut: impl future<output = action> + static); }` com implementacoes concretas por plataforma injetadas na inicializacao, junto com o trait storage. O codigo de dominio chama `spawner.spawn(...)` sem saber o runtime subjacente. Platform-awareness fica confinada ao ponto de inicializacao do app.

---

## rul-08 — theming como struct com dimensoes comportamentais

- **dom:** theming
- **dat:** 2022-11-14
- **lnk:** idx-rules

Definir struct theme com todas as dimensoes de design como tokens de primeira classe. Cores, escala tipografica, escala de espacamento e border radius sao a camada visual. Motion physics, mass, tension, friction como parametros globais do sistema cinetico, e intent tokens, intent: destructive, constructive, neutral, informational como dado estrutural que propaga para cor, motion e aria simultaneamente, sao camadas comportamentais obrigatorias. Componentes recebem `&theme` e leem tokens dele. Zero valores visuais ou comportamentais hardcoded.

Com struct, dark mode e `theme::dark()`, rebranding e um novo theme, e a sensacao fisica do produto, leveza ou solidez, e controlavel via `theme.motion.mass` e `theme.motion.tension` propagando coerentemente para todos os comportamentos cineticos. E a unica arquitetura onde coerencia fisica global e uma propriedade de design em vez de animacoes por-componente sem relacao sistemica. Nao existe equivalente publico em nenhum framework rust atualmente.

---

## rul-09 — acessibilidade como constraint via accesskit

- **dom:** accessibility
- **dat:** 2022-11-14
- **lnk:** idx-rules

Toda arvore de elementos do plev mantem uma arvore de acessibilidade paralela via accesskit. Nao e feature opcional, e parte do contrato de cada componente desde a primeira implementacao.

Plev nao tem dom. O browser nao constroi a arvore de acessibilidade automaticamente porque nao ha html. Em rendering custom via skia ou wgpu, a arvore precisa ser construida explicitamente ou o produto e inacessivel para screen readers em todas as plataformas. Accesskit fornece adapters portaveis, at-spi no linux, nsaccessibility no macos, ui automation no windows, sem implementacao manual por plataforma. Sem essa regra como constraint arquitetural, acessibilidade sera postergada ate producao, onde o custo de retrofit e uma ordem de magnitude maior.

---

## rul-10 — text layout via parley com suporte bidi

- **dom:** text
- **dat:** 2022-11-14
- **lnk:** idx-rules

Todo rendering de texto passa por parley (linebender). Zero implementacao manual de text layout. Bidi, scripts complexos (devanagari, tailandes, arabe, hebraico) e features opentype sao suportados por construcao via harfrust.

Text layout correto e um dos problemas computacionalmente mais complexos em ui, line breaking, word wrapping, ligatures, kerning, bidi reordering, combining characters. Implementacao manual produz resultado que parece correto em ingles e quebra silenciosamente em qualquer outro script. Parley resolve isso com o mesmo rigor do browser, em rust, sem dependencia de sistema operacional.

---

## rul-11 — persistencia via trait com migracao versionada

- **dom:** persistence
- **dat:** 2022-11-14
- **lnk:** idx-rules, rul-07

Definir `trait storage { fn load(...) -> result<t>; fn save(...) -> result<()>; }`. Implementacoes concretas, rusqlite, sled, indexeddb, filesystem, ficam em modulos separados injetados na inicializacao. Dominio e ui dependem do trait, nunca da implementacao.

Alem do trait de acesso, definir `trait migration { fn version() -> u32; fn up(db: &mut dyn storage) -> result<()>; }` com vetor de migrations aplicadas em ordem na inicializacao. Schema de dados persistidos versiona junto com o codigo, toda mudanca de estrutura e uma nova migration registrada. App em producao com dados reais nao pode assumir que o schema no device do usuario corresponde ao schema atual do codigo. Sem migrations, atualizacoes de app ou corrompem dados silenciosamente ou exigem reset forcado, ambos sao falhas de produto.

---

## rul-12 — internacionalizacao alem de text layout

- **dom:** i18n
- **dat:** 2022-11-14
- **lnk:** idx-rules, rul-10

Strings visiveis ao usuario vivem exclusivamente em arquivos de localizacao, formato fluent (project fluent da mozilla) por ser o unico sistema que resolve pluralizacao, genero gramatical e variacoes contextuais como dado, nao como logica condicional no codigo. Zero string literal em portugues, ingles ou qualquer idioma dentro de componentes. Zero formatacao manual de data, numero, moeda ou unidade, usar icu4x como unica fonte de formatacao locale-aware.

Rtl layout e consequencia de locale, nao de configuracao manual. Taffy suporta direction rtl como propriedade de layout, ativar globalmente quando o locale detectado e rtl. Suporte a scripts (rul-10) e suporte a localizacao sao problemas ortogonais: parley garante que arabe renderiza corretamente; esta regra garante que o texto correto em arabe esta disponivel e formatado corretamente para o contexto.

---

## rul-13 — error handling tipado e visivel

- **dom:** errors
- **dat:** 2022-11-14
- **lnk:** idx-rules

Definir enum apperror com variantes semanticas: networktimeout, storagecorrupted, `invalidinput { field: static str, reason: string }`, authexpired, `ratelimited { retry_after: duration }`. Todo result na app usa apperror. Erros se tornam estado visivel, inline message, toast, retry button, via action no fluxo normal. Zero `unwrap()` em codigo de producao. Zero erro silencioso.

`unwrap()` e panic em producao. Erros silenciosos criam dados corrompidos que aparecem horas depois. Erros como strings genericas impedem handling granular: retry faz sentido para networktimeout, fallback para storagecorrupted, validacao inline para invalidinput. Enum tipado com match exaustivo garante que todo erro tem tratamento explicito decidido em compile time.

---

## rul-14 — props minimas com contexto explicito

- **dom:** components
- **dat:** 2022-11-14
- **lnk:** idx-rules

Componente com mais de 5 props deve ser quebrado em componentes menores ou receber struct de configuracao. Dado que o componente nao usa diretamente, so repassa para filhos, vai via contexto ou e passado diretamente ao filho que precisa.

Contexto em plev segue o modelo do xilem env, dado disponivel para toda a subarvore sem passar explicitamente em cada nivel, com tipagem estatica garantindo que o dado existe no contexto antes de ser consumido. Prop drilling profundo cria acoplamento vertical: mudar um campo no nivel 5 exige editar os niveis 1 a 4 que so repassam o valor.

---

## rul-15 — testabilidade por camada sem gpu

- **dom:** testing
- **dat:** 2022-11-14
- **lnk:** idx-rules

Tres niveis obrigatorios. Dominio: unit tests puros com test, sem plev, sem window, em milissegundos. Componentes: snapshot do element tree retornado, element e struct rust inspecionavel sem necessidade de render. Integracao: headless render para screenshot diff apenas em critical paths com baseline versionado no repositorio.

Para snapshot de componentes, element expoe apenas campos deterministicamente comparaveis: tipo, props estruturais, filhos, intent tokens. Closures de callback sao opacas em teste, representadas por identificador de tipo, nao por conteudo. Igualdade estrutural do element tree nao inclui callbacks. Testes de componente verificam estrutura e semantica, nao identidade de funcao. Testes que precisam de gpu sao lentos, flaky por diferencas de driver e impossiveis em ci sem hardware dedicado. A separacao em camadas nao e preferencia, e a condicao para que testes sejam executados com frequencia suficiente para ter valor.
