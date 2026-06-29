+++
authors = ["Brenner Cruvinel"]
title = "Composição visual semântica: a convergência que ninguém construiu"
description = "Panorama de pesquisa conectando geração de UI por LLM, renderização GPU, dataflow reativo, gramáticas de visualização e protocolos de descrição de cena."
date = 2025-05-12
[taxonomies]
tags = ["PLEV", "Composição Visual", "LLM", "GPU"]
+++

A convergência que ninguém construiu: panorama de pesquisa para composição visual semântica

Cinco campos que sempre caminharam separados estão convergindo ao mesmo tempo: geração de interfaces por LLM, renderização por GPU compute, sistemas de dataflow reativo, gramáticas de visualização e protocolos de descrição de cena. Ninguém construiu o sistema que conecta todos eles. A tentativa mais próxima é o A2UI (https://a2ui.org/) do Google (dez/2025), mas ele opera no nível errado, descreve widgets, não semântica visual. As peças já existem na academia: o scenegraph relacional do Bluefish (https://github.com/bluefishjs/bluefish) (UIST 2024), o pipeline GPU do Vello (https://github.com/linebender/vello), o substrato reativo da proposta TC39 Signals (https://github.com/tc39/proposal-signals) e a álgebra de composição do OpenUSD (https://github.com/PixarAnimationStudios/OpenUSD). Falta o tecido conectivo, um protocolo que seja ao mesmo tempo acessível para LLMs, nativo de GPU, incrementalmente reativo e capaz de zoom semântico. Este relatório mapeia cada peça, identifica as lacunas e traça a estratégia de publicação e adoção.

A crise de representação intermediária em UI generativa

A descoberta central da literatura recente é simples: não existe consenso sobre qual representação intermediária (IR) deve ficar entre a intenção do LLM e os pixels na tela. Todo projeto relevante de 2024-2026 inventou a sua, e o espectro é revelador.

Num extremo, o artigo Generative UI do Google (https://generativeui.github.io/static/pdfs/paper.pdf) (nov/2025, Yaniv Leviathan (https://yanivle.github.io/) et al.) mostrou que o Gemini 3 Pro gera interfaces HTML/CSS/JS completas direto, sem IR nenhuma. Usuários preferiram essas interfaces a sites tradicionais 90% das vezes. Mas a abordagem é frágil: não permite diffing semântico, atualização incremental nem reuso entre plataformas. No extremo oposto, o Jelly (https://dl.acm.org/doi/10.1145/3706598.3713285) (Cao, Jiang e Haijun Xia (https://hci.ucsd.edu/haijunxia), CHI 2025, UCSD) gera primeiro um modelo de dados orientado a tarefas, um esquema objeto-relacional com grafo de dependências, que fundamenta a interface antes de qualquer renderização. Esse modelo é a IR e evolui tanto por linguagem natural quanto por manipulação direta.

Entre os dois polos:

- SpecifyUI (https://arxiv.org/abs/2509.07334) (Xiang 'Anthony' Chen (https://hci.prof/) et al., arXiv set/2025) propõe o SPEC, uma codificação hierárquica parametrizada de layout, estilo e semântica de componentes.
- O GenUI do SALT-NLP (https://arxiv.org/abs/2508.19227) (Chen, Zhang et al., Stanford, arXiv ago/2025, grupo de Diyi Yang (https://cs.stanford.edu/~diyiy/)) usa fluxos de interação em dois níveis com máquinas de estado finito.
- Graphologue (https://dl.acm.org/doi/10.1145/3586183.3606737) (Jiang e Rayan, UIST 2023, UCSD) embute anotações estruturadas de entidades inline no texto do LLM para construir diagramas em tempo real.

O caso mais instrutivo para o Phi é o DynaVis (https://dl.acm.org/doi/10.1145/3613904.3642639) (Vaithilingam et al., CHI 2024 Best Paper, Harvard / Microsoft Research, grupo de Elena Glassman (https://glassmanlab.seas.harvard.edu/glassman.html)). Ele usa especificações Vega-Lite (https://vega.github.io/vega-lite/) como IR de visualização e gera widgets interativos em HTML/JS para refinamento, combinando gramática visual declarativa com GUI sintetizada dinamicamente. É o único sistema na literatura que usa uma gramática de visualização como representação intermediária em vez de um modelo de componentes de UI. Ganhou Best Paper por isso.

Padronização em andamento

O cenário competitivo está convergindo em três protocolos complementares:

| Protocolo | O que faz | Status |
|-----------|-----------|--------|
| A2UI (https://a2ui.org/) (Google) | Árvore de componentes JSON com referências por ID, bindings e gerenciamento de superfícies. | v0.8, Apache 2.0. Em produção no Gemini Enterprise e Flutter GenUI SDK. Renderers para Angular, Flutter, Lit, Web Components. |
| AG-UI (https://docs.ag-ui.com/) (CopilotKit) | Camada de transporte, 16 tipos de evento via SSE/WebSockets para streaming, sync de estado, tool calls e human-in-the-loop. | Adotado por AWS, Microsoft, Google, LangChain, Oracle. |
| MCP Apps (https://github.com/modelcontextprotocol/ext-apps) (Anthropic + OpenAI) | Capacidades de UI no Model Context Protocol via iframes isolados. | Em evolução. |

AG-UI transporta payloads A2UI, são complementares.

A lacuna crítica: os três protocolos descrevem componentes de UI, não semântica visual. O A2UI diz "renderize um Button com label 'Submit'". Não consegue dizer "renderize uma codificação proporcional desses três valores com zoom semântico no nível 3". Não existe protocolo que conecte o mundo da Grammar of Graphics (Vega-Lite (https://github.com/vega/vega-lite), Bluefish (https://bluefishjs.org/)) com o mundo de geração por LLM (A2UI, AG-UI). Essa é a oportunidade do Phi.

Pesquisadores centrais

| Quem | Onde | Trabalho principal |
|------|------|--------------------|
| Haijun Xia (https://hci.ucsd.edu/haijunxia) | UCSD | Jelly, Graphologue |
| Elena Glassman (https://glassmanlab.seas.harvard.edu/glassman.html) | Harvard | DynaVis |
| Diyi Yang (https://cs.stanford.edu/~diyiy/) | Stanford SALT-NLP | GenUI |
| Titus Barik (https://www.barik.net/) / Jeffrey Nichols (http://www.jeffreynichols.com/) | Apple ML Research | BISCUIT (https://arxiv.org/abs/2404.07387), Misty (https://arxiv.org/abs/2409.13900) |
| Xiang 'Anthony' Chen (https://hci.prof/) | UCLA | GenUI Study |
| Yaniv Leviathan (https://yanivle.github.io/) | Google Research | Generative UI |

Venues principais: CHI e UIST dominam. DIS está emergindo para teoria de design. O workshop "What does Generative UI mean for HCI Practice?" (https://genuimeetshci.github.io/chi26-workshop/) no CHI 2026 sinaliza que o campo está cristalizando.

Renderização GPU compute e a fronteira de memória limitada

O Vello (https://github.com/linebender/vello) não é apenas um renderer, é um programa de pesquisa testando a hipótese de que gráficos 2D devem ser renderizados inteiramente em GPU compute shaders. Entender sua arquitetura e seus problemas em aberto é essencial para o Phi: qualquer protocolo de composição visual nativo de GPU precisa construir sobre o Vello ou resolver os mesmos problemas do zero.

Arquitetura

A cena é codificada na CPU como formato binário simplificado. Um pipeline de compute shaders cuida de parsing, geometria, sorting/binning e geração de comandos por tile. O estágio final faz rasterização fina interpretando um programa bytecode por tile de 16×16 pixels.

A técnica central é o prefix sum (parallel scan), permite que operações inerentemente sequenciais (ordenação, alocação, travessia de árvore) rodem em paralelo em milhares de threads GPU. O algoritmo Stack Monoid de Raph Levien (https://raphlinus.github.io/) (arXiv:2205.11659 (https://arxiv.org/abs/2205.11659), 2022) é particularmente novo: resolve o problema de matching de parênteses em cenas com estrutura de árvore (hierarquias de clip/blend) usando redução por stack monoid e busca binária por bicyclic semigroup, processamento paralelo de composição hierárquica na GPU.

O artigo GPU-friendly Stroke Expansion (https://dl.acm.org/doi/10.1145/3675390) (Levien e Arman Uguray (https://github.com/armansito), HPG 2024) apresenta outra primitiva fundamental: expansão de stroke totalmente paralela usando aproximações por Euler spiral em compute shaders. Junto com o prefix sum, esses dois trabalhos formam a base teórica da renderização 2D compute-native.

O problema de bounded memory é a fronteira

O ensaio "I want a good parallel computer" (https://raphlinus.github.io/gpu/2025/03/21/good-parallel-computer.html) de Levien (mar/2025) é o documento mais importante deste espaço. Cada estágio do pipeline produz estruturas intermediárias de tamanho imprevisível, mudar uma única transformação pode alterar drasticamente o plano de renderização. Buffers de GPU precisam ser alocados pela CPU antes do pipeline rodar, mas os tamanhos necessários só ficam claros depois. As soluções atuais são todas ruins: estimar na CPU (caro e impreciso) ou tentar-renderizar-detectar-repetir (o readback GPU→CPU mata a performance).

A análise de Levien dos D3D12 Work Graphs (spec (https://microsoft.github.io/DirectX-Specs/d3d/WorkGraphs.html)) encontrou três limitações fatais para 2D: sem joins (o Vello precisa de entrada sincronizada de duas filas), sem garantia de ordenação (2D exige ordem estrita de draw) e sem elementos de tamanho variável. O modelo ideal seria estágios conectados por filas com buffers limitados, basicamente o que as dataflow machines prometeram nos anos 1970.

O futuro: sparse strips

O sparse strips renderer (https://github.com/linebender/vello/issues/670) é o futuro arquitetural. Em vez do pipeline monolítico, sparse strips produz uma IR bem definida: caminhos renderizados com compressão run-length e compressão de regiões sólidas. Desacopla estágios, melhora modularidade e reduz requisitos de memória dependentes de dados.

Três implementações existem até final de 2025:

| Variante | Descrição | Status |
|----------|-----------|--------|
| Vello CPU | Software puro, otimizado com SIMD. | Competitivo com Skia, mais rápido que Cairo em ARM. |
| Vello Hybrid | Pré-processamento CPU + fragment shaders GPU. | Funciona em WebGL2. |
| Vello GPU | Pipeline compute completo. | Em andamento. |

No benchmark Blend2D em Apple M1 Pro (jul/2025), Vello CPU ficou em segundo lugar geral, atrás apenas do Blend2D (que usa pipelines JIT). Benchmarks do Vello GPU estão incompletos, pendente f16 math, subgroup operations e troca de device atomics por monoid-based aggregation.

Implicação estratégica para o Phi: a IR de sparse strips é o alvo natural para o backend de renderização. É um formato intermediário streamable que pode ser gerado por CPU ou GPU compute. Um protocolo de composição visual semântica compilaria para sparse strips da mesma forma que Vega-Lite (https://github.com/vega/vega-lite) compila para Vega, que compila para chamadas Canvas/SVG.

Pessoas-chave

| Quem | Onde | Contribuição |
|------|------|-------------|
| Raph Levien (https://raphlinus.github.io/) | Canva / Linebender (https://xi.zulipchat.com/) | Criador do Vello. Na Canva desde jan/2026, continuando o Linebender. |
| Arman Uguray (https://github.com/armansito) | | GPU stroke expansion |
| Chad Brokaw (https://github.com/dfrg) | | Fontes/texto (autor de swash e zeno) |
| Laurenz Stampfl (https://github.com/LaurenzV) | | Vello CPU, tempo integral desde mar/2025 |

A ida de Levien para a Canva é estrategicamente significativa, a Canva está investindo no Vello, com engenheiros Alex Gemberg e Taj Pereira contribuindo no Vello Hybrid.

Gramáticas de visualização estão virando gramáticas de composição universais

O desenvolvimento mais subestimado aqui é a extensão do paradigma Grammar of Graphics para além de gráficos estatísticos, rumo a um framework de composição visual universal. Três artigos definem essa trajetória.

Bluefish, o artigo pivotal

Bluefish (https://dl.acm.org/doi/10.1145/3654777.3676465) (Josh Pollock (https://joshmpollock.com/), Mei, Huang, Evans, Jackson e Arvind Satyanarayan (https://arvindsatya.com/), UIST 2024, MIT CSAIL, código (https://github.com/bluefishjs/bluefish))

Estende a Grammar of Graphics para domínios que ela nunca alcançou: diagramas moleculares, geometria euclidiana, fórmulas matemáticas, visualizações de estado de programa. A inovação central é o scenegraph relacional, um grafo composto (não só árvore) onde elementos participam de múltiplas relações espaciais ao mesmo tempo via referências declarativas. Scenegraphs tradicionais são árvores com um pai por nó. O de Bluefish é um grafo direcionado onde elementos Ref criam adjacências, permitindo que um elemento seja posicionado por várias relações de layout simultaneamente. Tempo de layout escala linearmente com o tamanho do scenegraph (14.000 nós em menos de 5s; o Penrose (https://penrose.cs.cmu.edu/) passa de 27.000ms com apenas 3.000 nós).

A biblioteca padrão formaliza primitivas de agrupamento perceptual Gestalt como operações composíveis de primeira classe:

- Distribute, espaçamento uniforme
- Align, alinhamento espacial
- Background, contenção
- Arrow/Line, conectividade
- Stack, alinhamento + densidade

O insight da tese de Pollock: "perceptual groups visualize discrete relations." Isso fornece a maquinaria teórica para descrever UI, diagramas e visualizações de dados num vocabulário unificado.

GoFish, estendendo para primitivas puras

GoFish (https://vis.csail.mit.edu/pubs/gofish/) (Pollock (https://joshmpollock.com/) e Satyanarayan (https://arvindsatya.com/), IEEE TVCG/VIS 2026 (https://ieeexplore.ieee.org/document/11274318/))

Substitui composite marks (bar, dot) das implementações GoG existentes por formas primitivas (Ellipse, Rect) + operadores gráficos explícitos baseados em princípios Gestalt. Os operadores aninham recursivamente, gerando um espaço de design infinito que borra a fronteira entre gramáticas de alto nível e primitivas de desenho. Gráficos como mosaics, waffles e ribbons, que ficam fora das implementações GoG, emergem naturalmente dessa decomposição.

Penrose, a fundação

Penrose (https://dl.acm.org/doi/10.1145/3386569.3392375) (Katherine Ye (https://katherineye.com/), Wode Ni (https://wodenimoni.com/), Krieger et al., SIGGRAPH 2020, CMU, código (https://github.com/penrose/penrose), orientadores Joshua Sunshine (https://www.cs.cmu.edu/~jssunshi/) e Keenan Crane (https://www.cs.cmu.edu/~kmcrane/))

Separa em três linguagens: Domain (tipos e operações), Substance (o que desenhar, análogo a HTML), Style (como desenhar, análogo a CSS). O mesmo conteúdo matemático pode ser visualizado em geometria euclidiana, esférica ou hiperbólica trocando o programa Style. Trabalho recente de Wode Ni (https://wodenimoni.com/) (Diagrams 2024 (https://doi.org/10.1007/978-3-031-71291-3_37)) explora primitivas de layout composíveis via signed distance functions. O framework Rose autodiff (https://arxiv.org/abs/2402.17743) (Estep et al., ECOOP 2024) emergiu do ecossistema Penrose para otimização web interativa.

Raízes teóricas

A formalização tem raízes profundas em teoria de categorias:

- Algebraic Visualization Design (https://doi.org/10.1109/TVCG.2014.2346325) (Gordon Kindlmann (http://people.cs.uchicago.edu/~glk/) e Carlos Scheidegger (https://cscheid.net/), IEEE VIS 2014, Honorable Mention), define três princípios usando categorias: representação inequívoca, correspondência visual-dados, invariância de representação.
- Vickers, Faith e Rossiter (https://doi.org/10.1109/TVCG.2012.294) (IEEE TVCG 2013), combinam semiótica com teoria de categorias, modelando o pipeline de visualização inteiro como morfismos em uma categoria.
- Seven Sketches in Compositionality (https://arxiv.org/abs/1803.05316) (Fong e Spivak, 2018, MIT), vocabulário matemático de funtores e transformações naturais para raciocinar sobre sistemas composicionais.

Para o Phi: a maquinaria formal já existe na literatura. O scenegraph relacional + operadores Gestalt do Bluefish fornecem a arquitetura prática. O framework de Kindlmann e Scheidegger fornece os critérios formais de correção. Falta: (1) conectar à geração por LLM (Bluefish não tem integração com LLM), (2) conectar à renderização GPU (Bluefish compila para SVG/Canvas), e (3) fazer do zoom semântico uma primitiva de primeira classe.

Zoom semântico: uma primitiva órfã

Apesar de 30 anos de história, nenhum framework moderno formalizou zoom semântico como primitiva de primeira classe. É uma lacuna surpreendente.

Fundações

- Pad++ (https://dl.acm.org/doi/10.1145/192426.192435) (Bederson e Hollan, UIST 1994), introduziu o conceito: objetos mudam de representação visual conforme o nível de zoom, em vez de simplesmente escalar geometricamente. Um documento aparece como ponto colorido; amplie e vira miniatura; amplie mais e vira texto.
- A linhagem Pad++ (https://dl.acm.org/doi/10.1145/192426.192435) → Jazz (https://dl.acm.org/doi/10.1145/354401.354754) (UIST 2000) → Piccolo (https://doi.org/10.1109/TSE.2004.44) (IEEE TSE 2004) explorou designs polilíticos e monolíticos de toolkits para interfaces zoomáveis.
- A Degree of Interest function (https://dl.acm.org/doi/10.1145/22627.22342) de Furnas (CHI 1986) é a espinha dorsal formal: DOI(x, .) = API(x) - D(., x), onde API é importância a priori e D é distância do foco atual. Um único formalismo que unifica fisheye views, overview+detail, focus+context e zoom semântico, todos computam relevância por elemento e mapeiam para propriedades visuais. A retrospectiva de Furnas em 2006 (https://dl.acm.org/doi/10.1145/1124772.1124921) identificou explicitamente essa unificação.
- O hyperbolic tree browser (https://dl.acm.org/doi/10.1145/223904.223956) de Lamping e Rao (CHI 1995) dispõe hierarquias no plano hiperbólico para distorção natural de foco+contexto.
- Trabalhos modernos incluem a abordagem de data cubes de Stolte, Tang e Hanrahan (https://ieeexplore.ieee.org/document/1196005) (Stanford) e o mapeamento multi-escala de grafos de Jonker et al. (https://doi.org/10.1177/1473871616661195) (2017).

A lacuna

Nenhum framework reativo, nenhuma gramática de visualização e nenhum renderer GPU trata zoom semântico como primitiva composível. No Vega-Lite, dá pra especificar interações (seleções, bindings) mas não regras de codificação visual que mudam por escala. No Bluefish, dá pra compor diagramas com relações Gestalt mas não definir como essas composições variam entre níveis de zoom. No CSS, @container queries se aproximam, mas são grosseiras e não composicionais.
Leptos (ou Dioxus) compilando pra WASM como frontend. O crate clust ou reqwest direto fazendo chamadas pra Anthropic API com streaming via tokio. SurrealDB com SDK Rust nativo pro teu brain system. Tudo em Rust, end-to-end. Zero JavaScript. O bundle WASM final seria menor que qualquer SvelteKit ou Next.js. E o mais importante: quando tu quiser migrar pra desktop nativo, com Dioxus é literalmente trocar o target de compilação. Com Leptos tu wrapa em Tauri.

O bottleneck técnico real numa interface de chat nunca é a renderização do UI. 

É latência de rede e velocidade de streaming de tokens. Então o argumento clássico "WASM é mais rápido que JS" não tem tanto impacto aqui na experiência percebida. 

Onde WASM brilha nesse contexto é em processamento local: 

parsing de markdown/LaTeX em tempo real
syntax highlighting de blocos de código
manipulação de grafos 
criptografia client-side. 
Formalmente, zoom semântico é uma codificação visual condicional, o mapeamento dados→representação muda em função da escala. No framework de Kindlmann e Scheidegger, é uma família parametrizada de funções de visualização indexadas por nível de zoom. Nos termos do Bluefish, é seleção de operadores Gestalt dependente de escala. Em GPU, é LOD para conteúdo 2D.

A oportunidade mais distintiva do Phi: fazer zoom semântico ser uma primitiva de primeira classe no protocolo, para que um LLM possa dizer "mostre esses dados como glyph resumido de longe, sparkline a média distância, gráfico interativo completo de perto", e o renderer GPU cuida das transições.

Signals, dataflow e renderização GPU-reativa

A convergência do ecossistema JavaScript em reatividade baseada em signals é a tendência de consenso mais forte da engenharia frontend. Angular (v16+), Vue (Composition API + Vapor Mode), SolidJS (https://github.com/solidjs/solid), Svelte 5 (https://github.com/sveltejs/svelte) (Runes), Qwik, Preact e Leptos (https://github.com/leptos-rs/leptos) (Rust) adotaram ou estão adotando reatividade fina híbrida push-pull como mecanismo central de atualização. A proposta TC39 Signals (https://github.com/tc39/proposal-signals) (Stage 1, abr/2024, liderada por Daniel Ehrenberg (https://github.com/littledan) e Rob Eisenberg (https://github.com/eisenbergeffect), com input de praticamente todo framework) quer padronizar isso na plataforma web. Rich Harris (https://github.com/Rich-Harris) resumiu o clima: "Like every other framework, we've come to the realisation that Knockout was right all along."

Como funciona

O algoritmo consensual é hybrid push-pull: propaga dirty flags das fontes pelo grafo, depois avalia preguiçosamente os valores reais só quando um effect demanda (update do DOM, request de rede). Elimina tanto o diamond problem (algoritmos eager avaliam nós duas vezes) quanto o equality check problem (algoritmos lazy podem pular avaliações necessárias).

A linhagem acadêmica:
- Conal Elliott (https://dl.acm.org/doi/10.1145/258948.258973), FRP fundacional (ICFP 1997)
- Umut Acar (https://www.cs.cmu.edu/~rwh/students/acar.pdf), Self-Adjusting Computation (CMU, 2005)
- Milo M (https://milomg.dev/2022-12-01/reactivity), "Super-Charging Fine-Grained Reactive Performance" (2022), cujo algoritmo Reactively foi adotado pelo Leptos (https://leptos.dev/)

Performance

Os dados são inequívocos. No benchmark js-framework-benchmark de krausest, frameworks baseados em signals consistentemente superam virtual DOM. SolidJS lidera; Svelte 5 segue de perto com bundle de 15KB versus 45KB do React. Um estudo controlado de 2025 com Puppeteer comparou dashboards React e Solid.js idênticas: Solid teve quase zero mutações DOM desnecessárias, usou menos de um terço da memória e produziu significativamente menos long tasks.

O insight profundo

Grafos de signals são grafos de dataflow. Sources são nós de entrada, computeds são transformações, effects são saídas. Propagação segue ordenação topológica. Detecção de mudanças usa push de dirty flags + avaliação pull sob demanda.

Essa é exatamente a arquitetura do Reactive Vega (https://doi.org/10.1109/TVCG.2015.2467091) (Satyanarayan (https://arvindsatya.com/), Russell, Hoffswell e Jeff Heer (https://homes.cs.washington.edu/~jheer/), IEEE TVCG 2016): constrói um grafo de dataflow streaming a partir de uma spec declarativa de visualização, onde dados, elementos de scene graph e eventos de interação são fontes de streaming de primeira classe. O único sink é o renderer. O Reactive Vega inclusive reescreve seu próprio grafo em runtime (operadores Facet criam ramos para dados hierárquicos).

Os GPU Work Graphs (https://devblogs.microsoft.com/directx/d3d12-work-graphs/) (D3D12, mar/2024) implementam o mesmo padrão no hardware: threads de shader (produtores) solicitam dinamicamente outro trabalho (consumidores), formando um grafo acíclico. A Epic Games confirma que Nanite e Lumen estão atingindo os limites do paradigma atual de compute shaders; work graphs endereçam diretamente o problema de expansão dinâmica de trabalho na GPU.

O paralelo formal entre propagação de signals na CPU e dispatch de work graphs na GPU aponta para uma arquitetura de renderização reativa unificada: estado de UI gerenciado por signals, mudanças propagando para GPU via work graphs, a GPU gerenciando o pipeline de renderização como seu próprio dataflow reativo, computação incremental minimizando trabalho em todo nível. Ninguém construiu isso. As peças existem separadas, Reactive Vega no lado CPU, GPU work graphs no hardware, mas a ponte não existe.

Para o Phi: o substrato reativo deve ser projetado desde o início para suportar tanto propagação de signals na CPU quanto dispatch de compute na GPU, via abstração unificada de dataflow. O grafo de signals é o grafo de renderização.

O ecossistema GUI em Rust está convergindo para infraestrutura compartilhada

O cenário de GUI em Rust em 2026 amadureceu de experimentação fragmentada para um ecossistema mais consolidado. A tendência estrutural mais relevante é a convergência de infraestrutura em torno dos crates do Linebender.

Os seis frameworks principais

| Framework | Stars | Arquitetura | Destaque |
|-----------|-------|-------------|----------|
| egui (https://github.com/emilk/egui) | ~28.6K | Immediate mode | Patrocinado pela Rerun. Domina tooling e prototipagem. |
| Dioxus (https://github.com/DioxusLabs/dioxus) | ~25K | Estilo React | YC S23. Cobertura mais ampla (web + desktop + mobile + server). |
| Iced (https://github.com/iced-rs/iced) | ~25K | Arquitetura Elm | |
| Slint (https://github.com/slint-ui/slint) | ~18K | DSL compilada AOT | SixtyFPS GmbH. Único com API estável 1.0. Foco embedded (<300KB RAM, roda em RPi Pico). |
| GPUI (https://github.com/zed-industries/zed/tree/main/crates/gpui) (Zed (https://github.com/zed-industries/zed)) | ~55K (repo) | Híbrido immediate/retained | $32M Sequoia. Startup 0.12s (vs 1.2s VS Code), 142MB RAM (vs 730MB), 120 FPS via SDF shaders. Acoplado ao Zed, pre-1.0. |
| Xilem (https://github.com/linebender/xilem) | ~4.6K | Inspirado em SwiftUI | Framework reativo do Linebender. Ainda alpha. |

O sinal de convergência é mais forte que o de competição

- egui 0.34 (https://github.com/emilk/egui) (mar/2026, Emil Ernerfeldt (https://github.com/emilk)) trocou renderização de fontes de ab_glyph para skrifa + vello_cpu. PR da comunidade em andamento para adotar Parley (https://github.com/linebender/parley) para text layout.
- Servo (https://github.com/servo/servo) integrou Vello CPU como backend de canvas.
- Bevy (https://github.com/bevyengine/bevy) usa bevy_vello (https://github.com/linebender/bevy_vello) para renderização.
- AccessKit (https://github.com/AccessKit/accesskit) (Matt Campbell (https://github.com/mwcampbell)) fornece acessibilidade cross-platform para egui, Masonry/Xilem, Bevy e Slint.
- Taffy (https://github.com/DioxusLabs/taffy) (DioxusLabs) fornece CSS Flexbox e Grid para Dioxus, Bevy, Zed (fork) e egui (via egui_taffy).

A stack do Linebender, Vello (https://github.com/linebender/vello) (renderização) → Parley (https://github.com/linebender/parley)/Fontique (texto) → Masonry (widgets) → Xilem (https://github.com/linebender/xilem) (reatividade), é o esforço mais coerente e líder em pesquisa, mesmo com adoção modesta do Xilem. A stack é modular por design: dá pra usar Vello sem Xilem, Parley sem Masonry, AccessKit sem nenhum deles. Essa modularidade explica a adoção cross-framework dos componentes.

Para o Phi: Vello + Parley + AccessKit são a infraestrutura de renderização natural. Construir sobre o Linebender evita a "roleta de frameworks" que matou o XAML e aproveita a tendência de convergência. A ida de Levien para a Canva e a base crescente de contribuidores sugerem manutenção a longo prazo.

Ninguém construiu a camada de composição visual universal

A análise competitiva revela que cada grande player está construindo dentro do seu próprio paradigma sem resolver a lacuna de protocolo entre semântica visual e renderização.

Panorama competitivo

| Player | Estratégia | Limitação |
|--------|-----------|-----------|
| Google | Full-stack: Gemini 3 (geração) + A2UI (https://a2ui.org/) (componentes) + AG-UI (https://docs.ag-ui.com/) (transporte) + Stitch/ex-Galileo AI (design-to-code). | A2UI descreve widgets (Button, Card), não semântica visual. Não expressa uma spec Vega-Lite ou diagrama Bluefish. |
| Vercel | Recuou de RSC-based GenUI (AI SDK RSC "pausado"). v0.dev virou gerador agentivo de código (4M+ users). Vercel AI SDK 6.0 com ToolLoopAgent. | Nenhum formato visual universal. Acoplado a React/Next.js. |
| Anthropic | Artifacts evoluindo de preview para plataforma de mini-apps com storage, MCP e IA embarcada. | Compete em "prompt-to-app", não em padronização de protocolo. |
| OpenAI | Canvas foca em edição/colaboração. | Não é composição de UI generativa. |
| Thesys (https://www.thesys.dev/) | API C1, substituto drop-in de LLM que retorna UI estruturada, com suporte a Vega-Lite. | O mais próximo da visão do Phi em produção, mas é API hospedada, não protocolo aberto. |
| Builder.io Mitosis (https://github.com/BuilderIO/mitosis) | Compilador que gera output multi-framework a partir de uma definição única de componente. | Resolve renderização cross-framework, não descrição visual semântica. |
| Figma Make (Config 2025) | Gera protótipos de alta fidelidade a partir de prompts, com MCP Server para agentes IA. | Centrado em ferramenta de design, não em protocolo. |

As cinco lacunas que definem a oportunidade do Phi

1. Codificação visual semântica na camada de protocolo. A2UI descreve widgets. Vega-Lite descreve gráficos estatísticos. Bluefish descreve diagramas. Nenhum protocolo abrange os três com primitivas visuais e operadores de composição unificados.

2. Renderização nativa de GPU a partir de specs declarativas. Todo sistema atual compila para HTML/CSS/DOM ou Canvas/SVG. Nenhum compila specs visuais declarativas direto para pipelines de GPU compute.

3. Zoom semântico como primitiva de protocolo. Nenhum sistema de UI generativa, gramática de visualização ou protocolo de renderização trata transformação visual dependente de escala como operação composível de primeira classe.

4. Dataflow reativo conectando signals da CPU com compute da GPU. A ponte grafo-de-signals→grafo-de-renderização é só conceitual. Nenhuma implementação propaga mudanças de estado reativo até o dispatch de trabalho na GPU.

5. Descrição visual cross-modal. Nenhum protocolo cobre o espectro completo: labels de texto → sparklines → widgets interativos → layouts completos → transições de zoom semântico, tudo numa única spec.

Janela de oportunidade

O tempo está se comprimindo. O A2UI chegou a v0.8 em dez/2025; v1.0 deve sair em 2026. AG-UI já foi adotado por AWS, Microsoft e Oracle. MCP Apps avança. Até meados de 2027, a camada de protocolo agente→UI provavelmente estará padronizada de facto em torno de A2UI + AG-UI, e a janela para uma alternativa nessa camada vai fechar.

Mas a oportunidade do Phi não está na mesma camada do A2UI. A2UI descreve componentes para renderização cross-framework. Phi descreveria semântica visual para composição universal, abstração mais baixa e mais fundamental. O posicionamento ideal: A2UI poderia ser compilado a partir das descrições semânticas do Phi, como Vega-Lite compila para Vega. Phi não é concorrente do A2UI, é a camada abaixo.

Padrões históricos preveem sucesso e fracasso

O histórico de padrões visuais é brutalmente claro.

Fracassos

| Padrão | Por que falhou |
|--------|---------------|
| VRML (1994-99) | "A technology in search of a problem" (Clay Shirky (http://www.shirky.com/writings/quake.html), ACM 1998). Sem implementação de qualidade única. Destruído quando a Platinum Technologies comprou as duas maiores empresas VRML e demitiu toda a divisão. |
| XForms (W3C 2003, descontinuado 2015) | Nenhum browser implementou. O Google escolheu AngularJS. |
| XAML | Microsoft fragmentou seu próprio ecossistema: WPF → Silverlight → WinRT → UWP → WinUI → MAUI, todos com XAML mas modelos de objetos incompatíveis. |

Sucessos

| Padrão | Por que funcionou |
|--------|-------------------|
| PostScript (1982) | Resolveu problema urgente (conectar qualquer app a qualquer impressora), teve champion forte (Adobe), entregou valor comercial imediato (Apple LaserWriter). |
| PDF | Adobe distribuiu Acrobat Reader grátis. Declarativo (sem controle de fluxo). Independência real de plataforma. |
| OpenUSD (https://openusd.org/) | Open-sourced da ferramenta battle-tested interna da Pixar. Coalizão (AOUSD (https://aousd.org/): Pixar + Adobe + Apple + Autodesk + NVIDIA). Core Spec 1.0 em dez/2025, 9 anos do open-source à spec formal. Composition arcs LIVERPS (Local → Inherits → VariantSets → Relocates → References → Payloads → Specializes), a álgebra de composição não-destrutiva mais sofisticada em produção. Core explicitamente agnóstico de domínio. |
| Vega-Lite (https://doi.org/10.1109/TVCG.2016.2599030) | Publicação peer-reviewed (IEEE InfoVis 2017 Best Paper). Spec JSON limpa. Arquitetura de compilação (Vega-Lite → Vega). Bindings matadoras (Python Altair). |

O padrão que prevê sucesso

1. Resolver problema urgente e específico primeiro, não "composição visual universal" (ambição estilo VRML). Em vez disso: "LLMs gerando visualizações interativas a 120fps em qualquer dispositivo."
2. Ter ao menos uma implementação excelente com champion comprometido.
3. Ser gratuito com licença permissiva e barreira de adoção baixa.
4. Formato declarativo portátil, JSON, não DSL (lição do Vega-Lite).
5. Arquitetura de compilação, o protocolo deve mirar múltiplos renderers.
6. Crescer a partir de workflows existentes, não exigir substituição total.
7. Publicar a gramática em venue acadêmico de topo para credibilidade intelectual.

Estratégia de publicação: UIST 2026 e IEEE VIS 2026

As oportunidades mais imediatas compartilham uma janela de deadline em torno de 31 de março de 2026.

Venues

| Venue | Data | Deadline | Oportunidade |
|-------|------|----------|-------------|
| ACM UIST 2026 (https://uist.acm.org/2026/) | Detroit, 2-5 nov | Abstract 24/mar, paper 31/mar. Demos abrem 10/jul. | Venue ideal. Co-patrocinado por SIGCHI e SIGGRAPH. Paper de sistemas com demo funcional seria altamente competitivo. |
| IEEE VIS 2026 (https://ieeevis.org/year/2026/info/call-participation/call-for-participation/) | ~São Francisco, out | Abstract 21/mar, paper 31/mar. | Arvind Satyanarayan (https://arvindsatya.com/) (co-criador Vega-Lite, orientador Bluefish) é General Chair. Forte receptividade para extensões de gramáticas de visualização. |
| SIGGRAPH 2026 (https://s2026.siggraph.org/) | Los Angeles, 19-23 jul | Poster 21/abr. Technical Workshops aceitam Sketch. | Visibilidade na comunidade de computação gráfica. |
| Onward! 2026 (https://2026.splashcon.org/) (SPLASH) | Oakland, out | ~verão | Excelente para artigo de visão/ensaio. Acolhe propostas de novos paradigmas. |

Sequenciamento recomendado

Espelha a abordagem bem-sucedida do Vega-Lite:

1. Preprint no arXiv imediatamente, estabelecer prioridade. Categoria primária cs.GR, cross-listings cs.HC + cs.PL.
2. Paper completo no UIST 2026 (31/mar), publicação de sistemas principal.
3. Short paper ou workshop no VIS 2026 (mesma janela), credibilidade na comunidade de visualização.
4. Poster no SIGGRAPH 2026 (21/abr), visibilidade em computação gráfica.
5. Artigo de visão no Onward! 2026 (~verão), articular a visão de padrão de protocolo.
6. Paper completo no CHI 2027 (deadline set/2026), avaliação de fatores humanos da integração com LLM.

Engajamento em padrões

- W3C Community Group para "Semantic Visual Composition", sem taxas, dá proteção de IP.
- AOUSD (https://aousd.org/) como Contributor gratuito → participar dos Interest Groups, especialmente o Web Interest Group (formado mar/2025, explorando USD para web).
- Khronos Group (https://www.khronos.org/), submeter New Initiative Proposal para "semantic 2D scene description for GPU rendering", complementar a OpenVG e ANARI.

As 30 pessoas mais importantes

Este trabalho fica na interseção de domínios onde poucos têm expertise em todos. Agrupados por área:

Gramáticas e sistemas de visualização

| Quem | Onde | Contribuição |
|------|------|-------------|
| Jeff Heer (https://homes.cs.washington.edu/~jheer/) (GitHub (https://github.com/jheer)) | UW | Criador de Vega e D3 (https://github.com/d3/d3). A pessoa mais influente em visualização declarativa. |
| Arvind Satyanarayan (https://arvindsatya.com/) (Scholar (https://scholar.google.com/citations?user=y-CX0aQAAAAJ)) | MIT | Co-criador Vega-Lite. General Chair VIS 2026. |
| Dominik Moritz (https://www.domoritz.de/) (GitHub (https://github.com/domoritz)) | CMU / Apple | Vega-Lite, AI4VIS. |
| Mike Bostock (https://bost.ocks.org/mike/) (GitHub (https://github.com/mbostock)) | Observable | Criador do D3. |
| Josh Pollock (https://joshmpollock.com/) (GitHub (https://github.com/joshpoll)) | MIT | Bluefish, GoFish, estendendo GoG para diagramas universais. |

Renderização 2D em GPU e infraestrutura Rust

| Quem | Onde | Contribuição |
|------|------|-------------|
| Raph Levien (https://raphlinus.github.io/) (GitHub (https://github.com/raphlinus)) | Canva / Linebender | Criador do Vello. A pessoa mais importante para renderização 2D nativa de GPU. |
| Chad Brokaw (https://github.com/dfrg) | | Fontes/texto (swash, zeno). |
| Patrick Walton (https://pcwalton.github.io/) (GitHub (https://github.com/pcwalton)) | Meta | Pathfinder (https://github.com/servo/pathfinder) GPU renderer. |
| Matt Campbell (https://github.com/mwcampbell) | | AccessKit (https://accesskit.dev/). |

LLM + geração de UI

| Quem | Onde | Contribuição |
|------|------|-------------|
| Haijun Xia (https://hci.ucsd.edu/haijunxia) | UCSD | Jelly, Graphologue. |
| Elena Glassman (https://glassmanlab.seas.harvard.edu/glassman.html) (Scholar (https://scholar.google.com/citations?user=C_r8d0AAAAAJ)) | Harvard | DynaVis. |
| Diyi Yang (https://cs.stanford.edu/~diyiy/) (Scholar (https://scholar.google.com/citations?user=j9jhYqQAAAAJ)) | Stanford | SALT-NLP, GenUI. |
| Titus Barik (https://www.barik.net/) (Scholar (https://scholar.google.com/citations?user=o5Q3xxoAAAAJ)) | Apple ML Research | BISCUIT. |
| Yaniv Leviathan (https://yanivle.github.io/) | Google Research | Generative UI paper. |

Diagramas e sistemas visuais formais

| Quem | Onde | Contribuição |
|------|------|-------------|
| Katherine Ye (https://katherineye.com/) | CMU | Penrose. |
| Joshua Sunshine (https://www.cs.cmu.edu/~jssunshi/) (Scholar (https://scholar.google.com/citations?user=V1texCUAAAAJ)) | CMU | Penrose. |
| Wode Ni (https://wodenimoni.com/) | CMU | Penrose PhD, layout composível via SDFs. |
| Keenan Crane (https://www.cs.cmu.edu/~kmcrane/) (Scholar (https://scholar.google.com/citations?user=Qs9FzFUAAAAJ)) | CMU | Penrose co-PI. |
| Ravi Chugh (https://people.cs.uchicago.edu/~rchugh/) (GitHub (https://github.com/ravichugh)) | UChicago | Sketch-n-Sketch, programação bidirecional para SVG. |

Compiladores e DSLs para computação visual

| Quem | Onde | Contribuição |
|------|------|-------------|
| Jonathan Ragan-Kelley (https://people.csail.mit.edu/jrk/) (Scholar (https://scholar.google.com/citations?user=nBcay4oAAAAJ)) | MIT | Halide (https://github.com/halide/Halide), a pessoa mais relevante para compilar specs visuais para código GPU. |
| Gordon Kindlmann (http://people.cs.uchicago.edu/~glk/) (Scholar (https://scholar.google.com/citations?user=Ky1op0MAAAAJ)) | UChicago | Algebraic Visualization Design. |
| Carlos Scheidegger (https://cscheid.net/) (GitHub (https://github.com/cscheid)) | | Algebraic Visualization Design. |

Padrões e indústria

| Quem | Onde | Contribuição |
|------|------|-------------|
| Neil Trevett (https://www.khronos.org/about/directors-officers/) | Khronos Group (https://www.khronos.org/) / NVIDIA | Presidente do Khronos. |
| Steve May | Pixar | CTO, Chairperson AOUSD (https://aousd.org/). |
| Guy Martin (https://blogs.nvidia.com/blog/author/guymartin/) | NVIDIA | AOUSD. |

Comunidades para engajar já

- Linebender Zulip (https://xi.zulipchat.com/), comunidade técnica mais relevante
- Observable community (talk.observablehq.com)
- Vega GitHub Discussions (vega-lite (https://github.com/vega/vega-lite/discussions))
- r/rust e r/graphicsprogramming no Reddit
- Hacker News, posts do Linebender regularmente ganham tração
- Comunidade IEEE VIS centrada no MIT (Satyanarayan (https://arvindsatya.com/)) e UW (Heer (https://homes.cs.washington.edu/~jheer/))
- Workshop GenUI meets HCI (https://genuimeetshci.github.io/chi26-workshop/)

Cinco padrões de convergência e a tese que sugerem

Afastando dos domínios individuais, cinco padrões emergem que juntos formam a base teórica do Phi:

Padrão 1, O imperativo da representação intermediária

Todo domínio está convergindo independentemente para IRs estruturadas. UI generativa precisa de IR entre intenção do LLM e renderização (modelo do Jelly, SPEC do SpecifyUI, specs Vega-Lite do DynaVis). Renderização GPU precisa de IR entre descrição de cena e pixels (sparse strips do Vello). Visualização precisa de IR entre dados e codificação visual (camadas GoG). O sinal é claro: a próxima camada de plataforma é uma IR que abrange as três.

Padrão 2, Composition arcs são universais

Ordenação LIVERPS do USD, especificidade de cascata do CSS, grafos de dependência de signals, composição de camadas GoG, scenegraph relacional do Bluefish, todos resolvem o mesmo problema: como combinar peças modulares e independentes num todo coerente com resolução determinística de conflitos? O framework matemático (teoria de categorias, conforme Vickers et al. e Fong/Spivak) é o mesmo. Um protocolo universal formalizaria essa álgebra compartilhada.

Padrão 3, Computação incremental é o modelo de performance unificador

Signals pulam reavaliação quando nada mudou. GPU work graphs pulam dispatch sem records. Reactive Vega poda ramos sem novas tuplas. Differential dataflow rastreia deltas em vez de recomputar tudo. Princípio idêntico: recomputar apenas o que mudou, na granularidade mais fina possível. Um sistema que mantém isso da interação do usuário até a renderização GPU seria fundamentalmente mais rápido que qualquer abordagem atual.

Padrão 4, Princípios Gestalt como primitivas composíveis

Biblioteca do Bluefish (Distribute, Align, Background, Arrow, Stack), operadores do GoFish, variáveis retinais de Bertin (https://www.esri.com/en-us/esri-press/browse/semiology-of-graphics-diagrams-networks-maps), taxonomia marks-and-channels de Munzner, todos convergem no insight de que um pequeno conjunto de operações perceptuais gera o espaço completo de representações visuais. Não são heurísticas de layout, são primitivas composicionais formalmente especificáveis e mecanicamente combináveis.

Padrão 5, O pipeline gramática→compilador

Vega-Lite (https://github.com/vega/vega-lite) compila para Vega. Vega compila para Canvas/SVG. Halide (https://github.com/halide/Halide) compila DSL de processamento de imagem para GPU/CPU. Svelte (https://github.com/sveltejs/svelte) compila sintaxe tipo JSX para operações DOM diretas. Penrose (https://github.com/penrose/penrose) compila Domain+Substance+Style para layouts otimizados. Os sistemas visuais mais bem-sucedidos não são interpretadores, são compiladores com separação clara de níveis. Phi deve ser um compilador de descrições visuais semânticas para dispatch de GPU compute.

A tese

Um protocolo de composição visual semântica deve:

1. Definir primitivas visuais composíveis baseadas em operações perceptuais Gestalt (à la Bluefish)
2. Ter uma álgebra de composição baseada em overrides em camadas (à la USD)
3. Usar um modelo de execução de dataflow reativo que conecte signals da CPU com compute da GPU
4. Tratar zoom semântico como primitiva de codificação dependente de escala de primeira classe
5. Ter um pipeline de compilação que mira sparse strips do Vello (ou equivalente) para renderização nativa de GPU
6. Usar formato JSON, acessível para LLMs (à la Vega-Lite)

A primeira demonstração matadora: um LLM gerando specs visuais semânticas que renderizam a 120fps com atualizações reativas e transições de zoom semântico, algo que nenhum sistema existente faz.

Riscos e modos de falha

| Risco | Descrição | Mitigação |
|-------|-----------|-----------|
| Armadilha VRML | "Protocolo visual universal" vira tecnologia sem problema. | Focar num caso de uso primeiro: visualizações interativas por LLM com renderização GPU. Expandir só depois de adoção. |
| Armadilha XForms | Spec bonita que ninguém implementa. | Entregar com implementação de referência excelente (Rust/Vello) e bindings imediatamente úteis (TypeScript + Python). |
| Armadilha XAML | Lock-in de plataforma destrói confiança. | Open-source dia 1, Apache 2.0. Múltiplos alvos: GPU compute, CPU software, WebGL2, Canvas fallback. |
| Risco de timing | A2UI chega a 1.0 e vira padrão de facto antes do Phi. | Posicionar Phi como camada abaixo do A2UI, não concorrente. A2UI diz "renderize um gráfico", Phi diz como compor o gráfico a partir de primitivas. |
| Risco de complexidade | Protocolo tenta cobrir tudo (2D + 3D + viz + diagramas + docs) e fica impossível de implementar. | Começar na interseção de viz de dados com UI interativa (onde GoG já é tratável). Expandir para diagramas depois. Deixar 3D pra depois ou nunca. |
| Risco de bounded memory | Problema de memória GPU do Vello bloqueia deploy em produção. | Sparse strips + híbrido CPU/GPU são alternativas viáveis. Protocolo deve ser agnóstico de renderer. |

A janela está aberta mas se estreitando. A pesquisa existe espalhada em comunidades que não se falam. O primeiro sistema que conectar o scenegraph relacional do Bluefish (https://github.com/bluefishjs/bluefish), o pipeline GPU do Vello (https://github.com/linebender/vello), o substrato reativo do TC39 Signals (https://github.com/tc39/proposal-signals) e o formato JSON do Vega-Lite (https://github.com/vega/vega-lite) num todo coerente vai definir a próxima camada de plataforma para computação visual.
