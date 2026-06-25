+++
title = "parte 2, livros e recursos de rust + webassembly (mundo + brasil)"
date = 2026-01-01
draft = true
+++

\

|nome|url|categoria|subdomínio|stars (aprox.)|maturidade|por que é relevante|
|---|---|---|---|---|---|---|
|rust-lang/rust|github.com/rust-lang/rust|compilador / linguagem|toolchain|~113k|produção consolidada|implementação oficial, governança forte, foco em segurança de memória e performance|
|denoland/deno|github.com/denoland/deno|runtime js/ts|server-side / cli|~107k|produção recente|runtime seguro por padrão, permissions model e tooling integrado em rust|
|tauri-apps/tauri|github.com/tauri-apps/tauri|apps desktop|webview / desktop|~107k|produção consolidada|apps desktop com front-end web e backend rust, footprint pequeno vs electron|
|oven-sh/bun|github.com/oven-sh/bun|runtime js|fullstack (runtime, bundler, test)|~93k|produção recente|runtime js muito rápido, bundler e test runner integrados, parte em zig e rust|
|rustdesk/rustdesk|github.com/rustdesk/rustdesk|app desktop|remote desktop|~116k|produção consolidada|alternativa self-hosted ao teamviewer, cliente e servidor em rust|
|cc-switch/cc-switch|github.com/cc-switch/cc-switch|devtools / ai|gerenciador de agentes|~100k|produção recente|desktop all-in-one para gerenciar agentes de código (claude code, codex etc.)|
|codex/codex|github.com/codex-team/codex|devtools / ai|coding agent cli|~90k|beta avançado|agente de código em terminal, foco em fluxos de dev modernos|
|astral-sh/uv|github.com/astral-sh/uv|gestor de pacotes|python tooling|~86k|produção recente|substituto rápido para pip/pipenv/poetry, rust como infra padrão de tooling python|
|zed-industries/zed|github.com/zed-industries/zed|editor de código|multiplayer / nativo|~85k|produção recente|editor nativo multiplayer, arquitetura client-server em rust, baixa latência|
|BurntSushi/ripgrep|github.com/BurntSushi/ripgrep|devtools|code search / grep|~65k|produção consolidada|busca de código padrão-de-fato, 5 a 8x mais rápida que grep/ag em vários cenários|
|alacritty/alacritty|github.com/alacritty/alacritty|terminal|gpu terminal|~64k|produção consolidada|terminal opengl multiplataforma, base de comparação em performance|
|rust-lang/rustlings|github.com/rust-lang/rustlings|educação|exercícios|~63k|produção consolidada|curso de entrada de fato para rust no open source|
|dani-garcia/vaultwarden|github.com/dani-garcia/vaultwarden|segurança|password manager server|~62k|produção consolidada|servidor compatível com bitwarden, leve, muito usado em self-hosting|
|sharkdp/fd|github.com/sharkdp/fd|devtools|find alternativo|~43k|produção consolidada|alternativa rápida e ergonômica ao find, paralela por padrão|
|sharkdp/bat|github.com/sharkdp/bat|devtools|cat melhorado|~59k|produção consolidada|cat com syntax highlight e integração git, substitui cat/less|
|starship/starship|github.com/starship/starship|shell prompt|prompt multi-shell|~58k|produção consolidada|prompt único para zsh/bash/fish, rápido e customizável|
|meilisearch/meilisearch|github.com/meilisearch/meilisearch|banco / busca|search engine|~58k|produção consolidada|engine de busca http, relevância e baixa latência, usada em saas|
|qdrant/qdrant|github.com/qdrant/qdrant|banco vetorial|ai infra / vector db|~32k|produção consolidada|vector db de alta performance, hnsw, sharding, cloud, muito usado em rag|
|influxdata/influxdb|github.com/influxdata/influxdb|time-series db|observabilidade / métricas|~31k|produção consolidada|time-series de referência, componentes em rust nas versões recentes|
|vectordotdev/vector|github.com/vectordotdev/vector|observabilidade|logs & metrics pipeline|~15k+|produção consolidada|pipeline de observabilidade, até 10x mais throughput que logstash com menos cpu/mem|
|firecracker-microvm/firecracker|github.com/firecracker-microvm/firecracker|virtualização|microvm / serverless|~34k|produção consolidada|microvms usadas em aws lambda e fargate, isolamento e baixa latência|
|tokio-rs/tokio|github.com/tokio-rs/tokio|runtime async|networking / io|~32k|produção consolidada|runtime async dominante, base de servidores e proxies de alta performance|
|actix/actix-web|github.com/actix/actix-web|web framework|apis http|~24k|produção consolidada|foco em performance, frequentemente no topo de benchmarks http|
|tokio-rs/axum|github.com/tokio-rs/axum|web framework|apis http|~26k|produção recente|sobre tokio/tower, ergonomia moderna, adoção crescente em backends cloud-native|
|seanmonstar/warp|github.com/seanmonstar/warp|web framework|apis http|~61k*|produção consolidada|estilo funcional com filters, base de serviços de alto throughput (*valor do ranking, parece inflado)|
|SergioBenitez/Rocket|github.com/SergioBenitez/Rocket|web framework|apis / sites|~25k+|produção consolidada|foco em type-safety e ergonomia, popular em web monolítica|
|bevyengine/bevy|github.com/bevyengine/bevy|game engine|2d/3d ecs|~46k|beta avançado|engine data-driven com ecs, principal aposta rust em engines generalistas|
|helix-editor/helix|github.com/helix-editor/helix|editor de código|modal / terminal|~44k|produção recente|editor modal pós-vim, lsp integrado, forte em performance|
|nushell/nushell|github.com/nushell/nushell|shell|data-oriented shell|~39k|produção recente|pipelines estruturados e tipados, integra json/yaml etc.|
|sxyazi/yazi|github.com/sxyazi/yazi|terminal / tui|gerenciador de arquivos|~39k|produção recente|file manager tui muito rápido, async io|
|zellij-org/zellij|github.com/zellij-org/zellij|terminal|multiplexer|~33k|produção recente|multiplexador com layout declarativo e plugins em rust|
|wez/wezterm|github.com/wez/wezterm|terminal|gpu terminal / multiplexer|~26k|produção consolidada|terminal gpu com multiplexing, wayland e ssh integrado|
|astral-sh/ruff|github.com/astral-sh/ruff|devtools|linter/formatter python|~47k|produção consolidada|linter/formatter 10 a 100x mais rápido que toolchains python tradicionais|
|pola-rs/polars|github.com/pola-rs/polars|data / analytics|dataframes|~38k|produção consolidada|engine de dataframe columnar, bindings python/r, substitui pandas em cargas pesadas|
|surrealdb/surrealdb|github.com/surrealdb/surrealdb|banco|document-graph db|~32k|beta avançado|multi-modelo documento + grafo, query unificada, foco realtime|
|typst/typst|github.com/typst/typst|tipografia|typesetting|~54k|produção recente|composição baseada em markup, alternativa moderna ao latex|
|sharkdp/hyperfine|github.com/sharkdp/hyperfine|devtools|benchmark cli|~28k|produção consolidada|benchmark de linha de comando, harness padrão em artigos de performance|
|cloudflare/pingora|github.com/cloudflare/pingora|networking|http proxy / framework|~26k+|produção consolidada|framework de proxy http da cloudflare, base de edge, evidência forte em infra crítica|
|nautechsystems/nautilus_trader|github.com/nautechsystems/nautilus_trader|fintech|trading engine|~23k+|produção recente|engine de trading determinístico, rust-native, baixa latência previsível|

## tabelas por domínio

### devtools (busca de código, cli, benchmark, tooling python)

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|BurntSushi/ripgrep|code search|~65k|simd, memmap, .gitignore por padrão; até 8x mais rápido que grep e ~5x que ag no kernel linux|
|sharkdp/fd|find alternativo|~43k|sintaxe minimalista, paralelo, padrão em dotfiles modernos|
|sharkdp/bat|cat com highlight|~59k|syntax highlight, integração git, paging automático|
|sharkdp/hyperfine|benchmark cli|~28k|benchmarks reprodutíveis com estatística, muito usado como harness|
|astral-sh/ruff|linter python|~47k|10 a 100x mais rápido que flake8/black em grandes codebases|
|astral-sh/uv|gestor de pacotes python|~86k|reimplementa packaging python focado em velocidade|
|dandavison/delta|pager de diff|~31k|syntax highlight para git diff, grep, rg --json|
|casey/just|task runner|~34k|"make para recipes", sintaxe simples, multimódulo|

### runtimes, plataformas e empacotadores

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|denoland/deno|runtime js/ts|~107k|permission model, bundling, test runner, ffi; supera node em rps em alguns cenários|
|oven-sh/bun|runtime js|~93k|foco em throughput; 2 a 4x o throughput de node dependendo do cenário|
|tauri-apps/tauri|desktop apps|~107k|webview nativo + rust, binários menores e menos memória que electron|
|pnpm/pnpm|gestor de pacotes js|~35k+|origem node, partes críticas reescritas em rust, reuso de armazenamento|
|vercel/turborepo|build system monorepo|~30k|builds incrementais para monorepos; vercel migrou componentes de go para rust|

### infra, observabilidade, bancos de dados e search

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|vectordotdev/vector|observability pipeline|~15k+|coletor/roteador de logs e métricas; até 10x throughput com menos cpu vs logstash|
|meilisearch/meilisearch|search engine|~58k|api de busca http batteries-included, indexação rápida, relevância configurável|
|qdrant/qdrant|vector db|~32k|hnsw, sharding, replicação, oferta cloud, muito usado em rag|
|influxdata/influxdb|time-series db|~31k|referência em observabilidade, partes de performance em rust|
|tikv/tikv|kv distribuído|~14k+|backend de storage do tidb, baixa latência e forte consistência|
|MaterializeInc/materialize|streaming sql|~10k+|materialized views incrementais sobre streams, núcleo em rust|
|surrealdb/surrealdb|document-graph db|~32k|híbrido documento + grafo, query própria, realtime|
|pola-rs/polars|dataframes|~38k|substitui pandas em cargas pesadas, engine columnar, múltiplas bindings|
|clockworklabs/SpacetimeDB|db para jogos/tempo real|~24k+|stateful serverless para apps interativas, engine em rust, baixa latência|

### terminais, shells e editores

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|alacritty/alacritty|terminal gpu|~64k|referência em performance e rendering, cross-platform|
|wez/wezterm|terminal + multiplexer|~26k|terminal gpu com multiplexing, wayland, ssh integrado|
|zellij-org/zellij|multiplexer|~33k|tmux reinterpretado, layout declarativo, plugins em rust|
|nushell/nushell|shell|~39k|pipelines estruturados, tipos fortes|
|starship/starship|prompt|~58k|prompt único multi-shell, rápido, customizável|
|helix-editor/helix|editor|~44k|editor modal pós-vim, lsp embutido|
|zed-industries/zed|editor|~85k|editor nativo multiplayer, arquitetura cliente-servidor em rust|
|lapce/lapce|editor|~38k|editor rápido e nativo, alternativa leve ao vscode|

### web e apis

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|actix/actix-web|web framework|~24k|benchmarks http muito altos, base de várias apis em produção|
|tokio-rs/axum|web framework|~26k|ergonomia sobre tower/tokio, boa história de middlewares|
|seanmonstar/warp|web framework|~61k*|estilo funcional com filters, microserviços e protótipos http|
|SergioBenitez/Rocket|web framework|~25k|type-safety e ergonomia, apis monolíticas|
|poem-web/poem|web framework|~7k+|full-featured, modular, suporte a openapi|

### segurança, criptografia, password managers

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|dani-garcia/vaultwarden|password manager server|~62k|alternativa leve ao servidor bitwarden, muito deployada em self-hosting|
|cloudflare/boringtun|vpn wireguard userspace|~5k+|wireguard userspace da cloudflare, usada em vpn comercial e edge|
|build-trust/ockam|security / messaging|~4k+|comunicação segura entre dispositivos e cloud, foco iot/edge|

### ai/ml infra e tooling

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|qdrant/qdrant|vector db|~32k|infra central em rag, cloud, alta performance de busca vetorial|
|meilisearch/meilisearch|hybrid search|~58k|full-text + filtros estruturados, features híbridas para ai search|
|pola-rs/polars|dataframes|~38k|backbone de pipelines de dados e notebooks python em cargas intensivas|
|chroma-core/chroma|ai search infra|~28k|vector store e apis de indexação para aplicações llm|

### embedded / sistemas de baixo nível

|nome|subdomínio|stars|por que é foda|
|---|---|---|---|
|redox-os/redox|sistema operacional (microkernel)|~22k+|so em rust buscando segurança de memória em baixo nível|
|tock/tock|so embarcado (cortex-m)|~4k+|so seguro para microcontroladores, pesquisa e produtos específicos|
|firecracker-microvm/firecracker|microvm / serverless|~34k|base de aws lambda/fargate, um dos ambientes mais exigentes de isolamento|

## benchmarks por projeto

### ripgrep vs grep, ag e outros

fonte: página de benchmarks do ripgrep (ripgrep.dev/benchmarks) e comparações adicionais. medições com hyperfine, mediana de várias execuções. diferenças são mais dramáticas em árvores grandes ou padrões com muitos matches.

|benchmark / cenário|métrica|ripgrep|concorrente|resultado concorrente|vantagem|
|---|---|---|---|---|---|
|regex no kernel linux (~25M loc, ~900 mb), com .gitignore|tempo (s)|0,082|ag|0,443|~5,4x vs ag|
|mesmo cenário kernel linux|tempo (s)|0,082|grep (gnu)|0,671|~8,2x vs grep|
|arquivo único 13,5 gb, busca literal|tempo (s)|6,73|grep (gnu)|9,20|~1,37x vs grep|
|arquivo único grande, mesma busca|tempo (s)|6,73|ag (mmap do arquivo inteiro)|34,60|~5,1x vs ag|

### runtimes js/ts: deno, bun, node.js

benchmarks de fontes diferentes não são diretamente comparáveis entre si. cada cenário fica separado. a própria comunidade deno alerta que benchmarks http sintéticos ignoram tls, http/2, compressão e roteamento real, e que cenários distintos podem inverter resultados.

fonte denosaurs/bench (frameworks otimizados, máquina dedicada):

|projeto|métrica|resultado|concorrente|resultado concorrente|vantagem|
|---|---|---|---|---|---|
|bun|req/s (média)|~73.612|deno|~58.534|bun ~1,26x vs deno|
|deno|req/s|~58.534|node + framework rápido (hyperexpress)|até ~69.429|node+hyperexpress ~1,19x vs deno|

fonte artigo "Bun vs Deno vs Node.js in 2026" (http com express equivalente, mesma máquina):

|projeto|métrica|resultado|concorrente|resultado concorrente|vantagem|
|---|---|---|---|---|---|
|bun|req/s|~52.000|node.js|~14.000|bun ~3,7x vs node|
|deno|req/s|~29.000|node.js|~14.000|deno ~2,1x vs node|

fonte artigo "Deno vs Bun vs Node.js: Performance & Benchmarks" (servidor http custom):

|projeto|métrica|resultado|concorrente|resultado concorrente|vantagem|
|---|---|---|---|---|---|
|bun|rps|~58.000|node.js 22|~30.000|bun ~1,93x vs node|
|deno|rps|~38.000|node.js 22|~30.000|deno ~1,27x vs node|

### vector vs logstash

os posts de lançamento da vector relatam "até 10x mais rápido" e "redução significativa de cpu/memória" vs logstash, sem números brutos públicos completos. tratar o "10x" como ordem de grandeza, não como sla replicável.

para a maioria dos demais projetos (meilisearch, qdrant, axum, actix-web), não há benchmark público padronizado e reprodutível para comparação direta. classificados como "sem benchmarks públicos confiáveis" nesse sentido.

## uso em produção e empresas

o repositório rust-in-production lista dezenas de empresas, várias ligadas diretamente a projetos da tabela.

github usa BurntSushi/ripgrep no pipeline de indexação e pesquisa de code search, combinado com go em outras camadas. a microsoft usa ripgrep dentro do vs code search para acelerar buscas em grandes workspaces.

cloudflare usa cloudflare/pingora como base da nova infra de proxy http de alta escala, no core de produtos de edge e cdn. (pingora foi aberto pela cloudflare em 2024.)

aws usa firecracker-microvm/firecracker em lambda e fargate, microvms leves isolando funções serverless. é um showcase de rust em produção extrema.

vercel migrou partes do pipeline de build (turborepo e o sucessor turbopack) de go para rust, por performance e controle de recursos.

em observabilidade, timber/mezmo promovem vectordotdev/vector como alternativa de alta performance a agentes tradicionais, com adoção em larga escala.

em ai/ml e analytics, pola-rs/polars substitui pandas em cargas pesadas, e qdrant/qdrant aparece como vector store principal em várias arquiteturas rag.

em produtividade e desktop, tauri está na base de vários apps desktop modernos com front-end web, e rustdesk é muito usado como alternativa self-hosted ao teamviewer, inclusive por equipes de suporte internas.

## análise qualitativa do ecossistema

distribuição por tipo: forte concentração em quatro blocos. devtools/cli (ripgrep, fd, bat, hyperfine, just, ruff, uv), runtimes/plataformas (deno, bun, tauri, pnpm), infra/observabilidade/databases (vector, meilisearch, qdrant, influxdb, tikv, surrealdb, polars, spacetimedb) e terminais/shells/editores (alacritty, wezterm, zellij, nushell, starship, helix, lapce, zed). parcela significativa do top 100 é devtools, coerente com o perfil dos primeiros adotantes: devs escrevendo ferramentas para si mesmos, onde performance, consumo de recursos e ux de cli são decisivos.

vantagem comparativa de rust: clis rápidas (ripgrep, fd, bat, ruff, uv), runtimes de alto desempenho (deno, bun) e bancos/infra com requisitos agressivos de latência e recursos (vector, qdrant, polars, firecracker).

padrões técnicos recorrentes: servidores e infra usam async/await sobre tokio-rs/tokio (axum, warp, pingora, vector). observabilidade interna com crates como tracing, env_logger e opentelemetry, com ênfase em structured logging. arquitetura predominante de binário único com config via arquivo ou cli (vector, meilisearch, qdrant, surrealdb, nushell), e plugins em zellij e wezterm para extensibilidade barata em runtime.

adoção corporativa por domínio: provas mais sólidas em navegadores e sistemas de alto risco (firefox, componentes de chrome/android, kernel windows), proxies http e cdns (cloudflare com pingora e oxy), pipelines de observabilidade (vector), bancos e search (tikv, meilisearch, qdrant, influxdb), build e empacotamento (turbopack/turborepo, uv, ruff) e serverless/vm (firecracker). comparado a go e node.js, rust ainda tem menos frameworks web full-stack dominantes em produção, mas forte presença em componentes de infra de alto impacto onde c/c++ eram onipresentes e estão sendo substituídos por segurança e manutenção.

benchmarks vs marketing: onde há benchmark sólido (ripgrep), a narrativa de "muito mais rápido" se sustenta (até 8x vs grep, ~5x vs ag). nos runtimes js, bun e deno ganham de node em http sintético, mas o benefício real depende do workload (tls, http/2, compressão, roteamento). em observabilidade, o "10x" da vector é crível como ordem de grandeza, não como sla universal. em muitos casos o ganho de performance é marginal vs c/go otimizado, e o que pesa é segurança de memória, expressividade do type system e ergonomia de tooling.

lacunas e oportunidades: poucos projetos rust maduros em frameworks web full-stack com ecossistema de plugins tipo django/laravel/spring; stacks completas de data engineering tipo airflow/spark/flink; bi/dashboards mainstream; e plataformas low-code dominantes. espaço evidente para infra de ai (runtimes para modelos, orquestração de agentes, feature pipelines para llms), data engineering pesado (etl de alto volume onde spark/flink dominam) e trading de ultra-baixa latência com toolchain moderno.


---

# parte 2, livros e recursos de rust + webassembly (mundo + brasil)


o recurso oficial "The Rust and WebAssembly Book" e toda a organização rustwasm no github foram arquivados em agosto/setembro de 2025. o repositório do livro diz explicitamente que não é mais onde a atividade acontece. logo, ele não é mais "atualizado continuamente", está congelado, e a recomendação de "começar por ele porque é o mais atual" caducou. ele ainda serve como introdução, mas com a ressalva de tooling possivelmente defasado. wasm-bindgen migrou para organização própria, docs em wasm-bindgen.github.io. wasm-pack, gloo e twiggy foram arquivados ou repassados a mantenedores individuais.

"The Rust Programming Language" está na 3ª edição (No Starch Press, 31 mar 2026, ISBN 978-1718504448), agora com Chris Krycho como co-autor, sobre a Rust 2024 Edition, com capítulo completo de async e seção de Miri para unsafe. a versão online em doc.rust-lang.org/book acompanha (assume rust 1.90+ e edition 2024).

"Programming Rust" está na 3ª edição (O'Reilly, jul 2026, ISBN 9781098176228), sobre a Rust 2024 Edition, ~690 páginas.

vários títulos "Rust + WebAssembly" que apareciam na pesquisa como livros comerciais legítimos ("Rust WebAssembly: A Hands-On Guide" 2025, "Rust for WebAssembly" 2026, "Rust and WebAssembly for Web Development", "Learning Rust for WebAssembly") são conteúdo de fazenda KDP gerado por IA. autores como "Nexus AI" (97 livros), "ALI. E. PACE", "Cliff S. Armstrong", zero avaliações, descrições genéricas, múltiplas edições no mesmo dia. marcados abaixo como não recomendados.

"Engenharia de Prompt para Devs" (relevante ao contexto brasil, parte 4) foi o #1 da casa do código em 2024, não "por dois anos consecutivos". nova edição ampliada lançada em nov 2025, 384 páginas.

## 1. livros-base de rust (core), pré-requisito para rust + wasm

| título                                | autor(es)                                    | ano (ed. atual)    | nível                     | temas                                                      | relevância p/ wasm                                | links                                                                                      |
| ------------------------------------- | -------------------------------------------- | ------------------ | ------------------------- | ---------------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| The Rust Programming Language (3ª ed) | Steve Klabnik, Carol Nichols, Chris Krycho   | 2026 (No Starch)   | iniciante a intermediário | ownership, borrowing, traits, async (cap. novo), rust 2024 | essencial, ponto de partida absoluto              | doc.rust-lang.org/book ; nostarch.com/rust-programming-language-3e                         |
| Programming Rust (3ª ed)              | Jim Blandy, Jason Orendorff, Leonora Tindall | 2026 (O'Reilly)    | intermediário a avançado  | ffi, concorrência, async, rust 2024                        | recomendado, ffi ajuda a entender wasm sob o capô | oreilly.com (ISBN 9781098176228)                                                           |
| Rust for Rustaceans                   | Jon Gjengset                                 | 2021 (No Starch)   | avançado                  | unsafe, trait coherence, async internals, no_std           | recomendado, "próximo passo" após o the book      | rust-for-rustaceans.com                                                                    |
| Rust in Action                        | Tim McNamara                                 | 2021 (Manning)     | intermediário             | sistemas práticos, concorrência, embedded, interop         | recomendado, casos práticos                       | manning.com/books/rust-in-action                                                           |
| Zero to Production in Rust            | Luca Palmieri                                | 2023               | intermediário             | backend, testes, observability, ci/cd, async               | opcional, arquitetura e padrões de produção       | zero2prod.com ; github.com/LukeMathWalker/zero-to-production                               |
| Command-Line Rust                     | Ken Youens-Clark                             | 2022 (atual. 2024) | iniciante a intermediário | cli, error handling, testing, file io, regex               | opcional, foco cli, menos relevante p/ web        | oreilly.com/library/view/command-line-rust/9781098109424                                   |
| Asynchronous Programming in Rust      | Carl Fredrik Samson                          | 2024 (Packt)       | intermediário a avançado  | futures, async/await, executors, epoll/kqueue/iocp         | recomendado, async crítico em apps wasm modernas  | packtpub.com (9781805128137) ; github.com/PacktPublishing/Asynchronous-Programming-in-Rust |
| Hands-On Concurrency with Rust        | Brian L. Troutwine                           | 2018 (Packt)       | intermediário a avançado  | threading, lock-free, ffi, memory model, tuning            | recomendado, concorrência p/ wasm complexo        | packtpub.com (9781788399975)                                                               |

## 2. livros específicos de rust + webassembly

| título                                                                                                                               | autor(es)                                             | tipo                                 | ano                 | nível                     | temas                                                           | escopo                     | links                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------- | ------------------------------------ | ------------------- | ------------------------- | --------------------------------------------------------------- | -------------------------- | ---------------------------------------------------------------------------------------------- |
| The Rust and WebAssembly Book                                                                                                        | Rust and WebAssembly Working Group                    | online gratuito (arquivado set 2025) | congelado           | iniciante a intermediário | game of life, wasm-bindgen, wasm-pack, web-sys, js-sys, deploy  | browser                    | rustwasm.github.io/book ; github.com/rustwasm/book                                             |
| Programming WebAssembly with Rust                                                                                                    | Kevin Hoffman                                         | comercial                            | 2019 (PragProg)     | intermediário             | fundamentos wasm, rust→wasm, integração js, host implementation | browser e hosts custom     | pragprog.com/titles/khrust                                                                     |
| Practical WebAssembly                                                                                                                | Sendil Kumar Nellaiyapen                              | comercial                            | 2022 (Packt, 232pp) | iniciante a intermediário | wasm-bindgen, web-sys, debugging, otimização, deploy            | browser                    | packtpub.com (9781838828004) ; github.com/PacktPublishing/Practical-WebAssembly                |
| WebAssembly with Rust                                                                                                                | sendilkumarn                                          | livro-curso online                   | atual. contínua     | intermediário             | wat, wabt, binaryen, toolchain rust→wasm, wasm-pack, wasi       | browser e wasi             | leanpub.com/webassembly                                                                        |
| Game Development with Rust and WebAssembly                                                                                           | Eric Smith                                            | comercial                            | 2022 (Packt)        | intermediário             | game loop, canvas, sprites, colisão, state machines, deploy     | browser (jogos 2d)         | packtpub.com/product/game-development-with-rust-and-webassembly/9781801070973                  |
| LEARN WebAssembly 2025 Edition                                                                                                       | Diego Rodrigues                                       | comercial (auto-publicado)           | 2025                | iniciante a intermediário | c/c++/rust→wasm, emscripten, wasi, segurança                    | browser e wasi             | atenção: autor associado a publicações KDP em volume, verificar qualidade                      |
| Building and Deploying WebAssembly Apps                                                                                              | autor não confirmado                                  | comercial                            | 2024                | intermediário a avançado  | assemblyscript, c/c++, rust→wasm, smart contracts, kubernetes   | browser, cloud, blockchain | abebooks (9789365898477)                                                                       |
| Rust WebAssembly: A Hands-On Guide / Rust for WebAssembly / Rust and WebAssembly for Web Development / Learning Rust for WebAssembly | "Nexus AI", "ALI. E. PACE", "Cliff S. Armstrong" etc. | KDP                                  | 2024 a 2026         | n/a                       | genéricos                                                       | browser                    | NÃO RECOMENDADOS, conteúdo de fazenda KDP gerado por IA, zero avaliações, descrições genéricas |

## 3. livros de webassembly geral relevantes para rust

|título|autor(es)|ano|linguagens foco|relevância p/ rust|links|
|---|---|---|---|---|---|
|WebAssembly: The Definitive Guide|Brian Sletten|2021 (O'Reilly)|genérico (c, c++, rust)|alta, fundamentos independentes de linguagem|oreilly.com/library/view/webassembly-the-definitive/9781492089834|
|Learn WebAssembly|Mike Rourke|2018 (Packt)|c, c++, rust|média, foca mais c/c++ e emscripten|packtpub.com/product/learn-webassembly/9781788997379|
|WebAssembly System Interface 入門|Asuka Yagi|2024|go, rust|alta, essencial para wasi com rust|impress nextpublishing (livrarias japonesas)|
|Server-Side WebAssembly|Danilo Chiarlone e outros|2025 (Manning)|genérico (rust citado)|alta, wasi server-side crítico p/ rust backend|manning.com/books/server-side-webassembly|
|WebAssembly: The Universal Runtime|Moment Tech|2026|genérico|alta (se legítimo), component model, wasi, wasmgc, edge|amazon kdp (verificar autoria)|
|AssemblyScript for WebAssembly Applications|William Smith|2025|assemblyscript|baixa, foca assemblyscript|google play books|

## 4. contexto brasil: traduções e disponibilidade

|título (pt-br)|original|autor(es)|editora br|e-book|foco|observações|
|---|---|---|---|---|---|---|
|Programação em Rust 2ª edição|Programming Rust (2nd)|Blandy, Orendorff, Tindall|Novatec|sim|rust geral|tradução da 2ª ed.; 3ª ainda não traduzida|
|Primeiros passos com a linguagem Rust|original pt-br|não informado|Novatec|sim|rust básico|introdução nacional|
|Desmistificando WebAssembly|original pt-br|Raphael Amorim|Casa do Código|sim|wasm com rust, segurança, performance, wasi|único livro brasileiro específico de wasm + rust; exemplos em rust; autor ex-spotify/globo, hoje viaplay|
|APRENDA WebAssembly Edição 2025|original pt-br|Diego Rodrigues|auto-publicado|sim|c/c++/rust→wasm, otimização|publicação independente, verificar qualidade|
|A linguagem de programação Rust (tradução comunitária)|The Rust Programming Language|Klabnik, Nichols|online|gratuito|rust geral|rust-br.github.io/rust-book-pt-br ; tradução comunitária, defasada|

## 5. recursos oficiais e books online

|recurso|tipo|organização|temas|links|licença|
|---|---|---|---|---|---|
|The Rust and WebAssembly Book|book oficial (arquivado)|rustwasm wg|tutorial completo, wasm-bindgen, wasm-pack, web-sys|rustwasm.github.io/book ; github.com/rustwasm/book|mit/apache-2.0|
|wasm-bindgen Guide|doc oficial|nova org wasm-bindgen|interop rust↔js, types|wasm-bindgen.github.io (url nova pós-migração)|mit/apache-2.0|
|wasm-pack Book|book/doc oficial (arquivado)|rustwasm|build/test/publish de pacotes wasm, npm|rustwasm.github.io/wasm-pack/book|mit/apache-2.0|
|Leptos Book|book oficial|leptos contributors|ui reativa, signals, ssr|book.leptos.dev ; github.com/leptos-rs/leptos|mit|
|Yew Documentation|doc oficial|yew contributors|componentes, hooks, routing|yew.rs ; github.com/yewstack/yew|mit/apache-2.0|
|Dioxus Guide|guia oficial|dioxus labs|ui cross-platform (web/desktop/mobile)|dioxuslabs.com ; github.com/DioxusLabs/dioxus|mit/apache-2.0|
|WASI Tutorial (Wasmtime)|tutorial oficial|bytecode alliance|wasi, runtime wasmtime, embeddings|docs.wasmtime.dev ; github.com/bytecodealliance/wasmtime|apache-2.0|
|MDN: Compiling Rust to WebAssembly|tutorial oficial|mozilla|setup, wasm-pack, browser|developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm|cc-by-sa|

## análise qualitativa do ecossistema de livros rust + wasm

pré-requisitos bem estabelecidos: The Rust Programming Language (3ª ed), Rust for Rustaceans e Programming Rust (3ª ed) formam a base. wasm exige ownership/borrowing (memória sem gc no browser), ffi/interop (rust↔js via wasm-bindgen), async/await (operações não-bloqueantes no browser e wasi) e no_std (muitos ambientes wasm não têm a std completa).

recurso principal rust + wasm: até 2025, "The Rust and WebAssembly Book" oficial era o ponto de partida. com o arquivamento, ele continua útil como introdução mas perde o status de "mais atual". docs vivas hoje: wasm-bindgen guide (nova url), documentação dos frameworks (leptos, yew, dioxus) e mdn.

livros comerciais de qualidade: pragprog (Hoffman) e packt (Nellaiyapen, Smith) têm títulos sólidos, mas convém checar data de publicação para evitar tooling defasado (wasm-pack/wasm-bindgen mudaram). cuidado com a enxurrada de títulos KDP gerados por IA pós-2023.

situação brasil: lacuna severa. apenas um livro brasileiro específico ("Desmistificando WebAssembly", Raphael Amorim, casa do código). tradução parcial e defasada do rust book. zero sobre frameworks modernos (yew, leptos, dioxus). dependência quase total de material em inglês e documentação oficial.

temas sub-representados: wasi 0.2 (preview 2) e component model; frameworks modernos de ui com comparação estruturada (state management, ssr/hydration); segurança e fuzzing de módulos wasm; casos além do browser (iot/embedded, plugins de apps como vscode/figma/shopify, edge computing como cloudflare workers/fastly, blockchain near/polkadot, ml com wasi-nn); e profiling/otimização de tamanho de binário .wasm (twiggy, devtools).

recomendação prática atualizada: começar por "The Rust Programming Language" (3ª ed), passar para a documentação viva de wasm-bindgen e dos frameworks (não mais só o book oficial congelado), complementar com "WebAssembly: The Definitive Guide" para fundamentos e "Server-Side WebAssembly" para wasi/edge.

---

# parte 3, pessoas e projetos de visualização em rust + wasm

mapeamento de 50+ pessoas relevantes e 50+ projetos de grafos/charts/visualização em wasm, mais conexões pessoa↔projeto. dado de contexto importante: a organização rustwasm está em sunset (arquivada set 2025), com repositórios migrando para a bytecode alliance, para a nova org wasm-bindgen ou para mantenedores individuais.

ressalva de confiabilidade: o arquivo original já marcava perfis não confirmados. alguns handles de github estão claramente errados ou conflitantes e foram sinalizados na nota ao fim desta parte. tratar links de perfil individual com cautela; os de organização e projeto são confiáveis.

## 1. pessoas: núcleo rust, wasm e runtimes

|nome|país|papel|projetos principais|links|
|---|---|---|---|---|
|Alex Crichton|eua|maintainer principal wasmtime, ex-top contributor rust|wasmtime, wasm-bindgen, cargo|github.com/alexcrichton|
|Nick Fitzgerald|eua|lead wasm wg, wasmtime, bindgen wg|wasmtime, wasm-bindgen, walrus|github.com/fitzgen ; fitzgeraldnick.com|
|Ashley Williams|eua|fundadora rust wasm wg, ex-npm cto|rustwasm, wasm-pack|github.com/ashleygwilliams ; @ag_dubs|
|Lin Clark|eua|engenheira mozilla/fastly, posts seminais wasm|lucet, component model|github.com/linclark ; code-cartoons.com|
|Till Schneidereit|alemanha|bytecode alliance, component model|wasmtime, wasi|github.com/tschneidereit|
|Luke Wagner|eua|co-criador do wasm e component model|component model, wasi|github.com/lukewagner|
|Peter Huene|eua|bytecode alliance, wasm-tools|wasm-tools, wit-bindgen|github.com/peterhuene|
|daxpedda|suíça|maintainer principal atual de wasm-bindgen|wasm-bindgen|github.com/daxpedda|
|Guy Bedford|reino unido|co-maintainer wasm-bindgen (cloudflare), criador systemjs|wasm-bindgen|github.com/guybedford|
|Saoirse / withoutboats|eua|rust core, async, autor técnico|fehler, async rust|github.com/withoutboats ; without.boats|
|Aaron Turon|eua|ex-rust core, tokio, async design|tokio, rfc process|github.com/aturon|
|Niko Matsakis|eua|rust core, borrow checker, async|rustc, polonius|github.com/nikomatsakis ; smallcultfollowing.com/babysteps|
|Steve Klabnik|eua|autor de The Rust Programming Language|the book, rust docs|github.com/steveklabnik ; steveklabnik.com|
|Carol Nichols|eua|co-autora de The Rust Programming Language|the book, crates.io|github.com/carols10cents|
|Syrus Akbary|eua|criador do wasmer|wasmer|github.com/syrusakbary ; wasmer.io|

## 2. pessoas: frameworks web rust, visualização e educadores

|nome|país|papel|projetos principais|links|
|---|---|---|---|---|
|Greg Johnston|eua|criador leptos|leptos|github.com/gbj ; leptos.dev|
|Jonathan Kelley|eua|criador dioxus|dioxus|github.com/jkelleyrtp ; dioxuslabs.com|
|Denis Kolodin|ucrânia|criador yew (original)|yew|github.com/DenisKolodin|
|Kaede Hoshikawa|japão|maintainer yew|yew|github.com/futursolo|
|Emil Ernerfeldt|suécia|criador egui/eframe, co-fundador rerun|egui, eframe, rerun|github.com/emilk ; ernerfeldt.com|
|Amos Wenger|frança|educador, autor fasterthanli.me|cargo-zigbuild, artigos|github.com/fasterthanlime ; fasterthanli.me|
|Jon Gjengset|eua/noruega|educador, youtuber, autor rust for rustaceans|livestreams, livro|github.com/jonhoo ; thesquareplanet.com|
|Luca Palmieri|itália|autor zero to production in rust|zero2prod, cargo-chef|github.com/LukeMathWalker ; lpalmieri.com|
|Pascal Hertleif|alemanha|educador, cli book, rustdoc|rust cli book, diesel|github.com/killercup ; deterministic.space|
|Richard Dodd|reino unido|tutoriais rust + wasm|wasm-pack, web-sys|github.com/richard-dodd|
|Dominik Nakamura|japão|contributor wasm tooling, educador|crates utilitários|github.com/dnaka91|
|Carter Anderson|eua|criador bevy|bevy|github.com/cart ; bevyengine.org|
|François Mockers|frança|core contributor bevy|bevy|github.com/mockersf|
|Hao Hou|china|criador plotters|plotters|github.com/38 (handle a confirmar)|
|blitzarx1|n/d|criador/maintainer egui_graphs|egui_graphs|github.com/blitzarx1|
|Grant Handy|eua|criador fdg (force-directed graph)|fdg|github.com/grantshandy|
|Jorge Aparicio (japaric)|origem br|rust embedded, cross, wasm targets|cross, embedded|github.com/japaric ; blog.japaric.io|
|David Tolnay|eua|criador serde, syn, quote, cxx|serde, syn, cxx|github.com/dtolnay|
|Sean McArthur|eua|criador hyper, reqwest|hyper, reqwest|github.com/seanmonstar ; seanmonstar.com|
|Yoshua Wuyts|holanda|rust async, wasm streams|async-std, wasm-streams|github.com/yoshuawuyts|
|Mattias Buelens|bélgica|streams wasm, web-sys contributor|wasm-streams|github.com/MattiasBuelens|
|Ivan Petkov|n/d|wasm tooling, crane|crane|github.com/ipetkov|
|Hans Larsen|canadá|angular cli, wasm-opt contributor|wasm-opt|github.com/hansl|
|Pat Shaughnessy|eua|autor de conteúdo rust internals|artigos|github.com/patshaughnessy ; patshaughnessy.net|
|Florian Gilcher|alemanha|rustfest organizer, educador|ferrous-systems training|github.com/skade ; yakshav.es|
|Jan-Erik Rediger|alemanha|mozilla, glean|glean|github.com/badboy ; fnordig.de|
|Jake Goulding|eua|rust playground, top contributor so|rust playground|github.com/shepmaster|
|Michael Gattozzi|eua|educador, posts wasm|wasm tooling|github.com/mgattozzi ; blog.mgattozzi.dev|
|Andrew Gallant (BurntSushi)|eua|criador ripgrep, regex engine|ripgrep, regex|github.com/BurntSushi ; blog.burntsushi.net|
|Surma|alemanha|google, educador wasm, threading|wasm threads, web workers|github.com/surma ; surma.dev|
|Ingvar Stepanyan (RReverser)|n/d|wasm-bindgen contributor, wabt bindings|wasm-bindgen|github.com/RReverser|
|Will Crichton|eua|tooling rust→wasm, educação, ide|rustc-dev-guide|github.com/willcrichton ; willcrichton.net|
|Michael Sproul|austrália|lighthouse (ethereum em rust), wasm|lighthouse|github.com/michaelsproul|
|Simon Sapin|frança|mozilla, servo, html/css em rust|servo|github.com/SimonSapin|
|Tobias Koppers (sokra)|alemanha|criador webpack, rspack (rust)|rspack|github.com/sokra|
|Evan Schwartz|eua|interledger rust, wasm finance|interledger-rs|github.com/emschwartz|
|Félix Saparelli|nova zelândia|watchexec, cargo tooling|watchexec|github.com/passcod ; passcod.name|

## 3. projetos de grafos, charts e visualização em wasm

|projeto|descrição|linguagem|tipo de viz|status|links|
|---|---|---|---|---|---|
|plotters|drawing de gráficos de alta qualidade, backend html5 canvas via wasm|rust|charts, scatter, histogramas, séries|ativo|github.com/plotters-rs/plotters ; plotters-rs.github.io|
|egui_graphs|widget de grafos interativos para egui, sobre petgraph; nativo e wasm|rust|grafos, redes direcionados|ativo|github.com/blitzarx1/egui_graphs|
|rerun|viz multimodal para dados científicos e ml, viewer em rust com target wasm|rust|nuvem de pontos, imagens, séries, 3d|ativo/maduro|github.com/rerun-io/rerun ; rerun.io|
|fdg|desenho de grafos por force-directed (fruchterman-reingold)|rust|force-directed, redes|ativo (inicial)|github.com/grantshandy/fdg|
|bevy|game engine ecs com suporte nativo a wasm, base de viz 2d/3d|rust|2d, 3d, sprites, shaders|ativo/maduro|github.com/bevyengine/bevy ; bevyengine.org|
|egui|gui imediata em rust puro, compila p/ wasm via eframe|rust|widgets, gráficos, painéis|ativo/maduro|github.com/emilk/egui ; egui.rs|
|colorgrad-js|gradientes de cor de alta performance, rust + wasm|rust + wasm|gradientes, colormaps|ativo|github.com/mazznoer/colorgrad-js|
|pathfinder|renderizador de fontes e paths vetoriais em gpu, webgl/wasm|rust|vetores, texto, 2d|manutenção|github.com/servo/pathfinder|
|vello|renderizador 2d gpu-accelerated, sucessor do piet, wasm via webgpu|rust|2d vetorial, texto, paths|ativo|github.com/linebender/vello|
|piet|abstração de rendering 2d com múltiplos backends incl. canvas web|rust|2d, shapes, texto|manutenção|github.com/linebender/piet|
|leptos|framework web full-stack em rust→wasm, base de dashboards/spas|rust|ui reativa, componentes|ativo/maduro|github.com/leptos-rs/leptos ; leptos.dev|
|yew|framework front-end rust/wasm inspirado em react/elm|rust|ui, componentes web|ativo/maduro|github.com/yewstack/yew ; yew.rs|
|dioxus|framework cross-platform (web/desktop/mobile) em rust com wasm|rust|ui, aplicações|ativo/maduro|github.com/dioxuslabs/dioxus ; dioxuslabs.com|
|perspective|engine de análise e viz interativa, núcleo c++/wasm, front-end ts|c++ + wasm|tables, charts, pivots|maduro|github.com/finos/perspective ; perspective.finos.org|
|apache arrow / datafusion|engine de query analítica em rust, suporte wasm crescente|rust|tabular, analytics|ativo|github.com/apache/arrow-datafusion ; arrow.apache.org|
|polars|dataframe em rust com bindings python e suporte parcial wasm|rust|tabular, séries|ativo/maduro|github.com/pola-rs/polars ; pola.rs|
|wgpu|implementação rust de webgpu/webgl com target wasm|rust|3d, gpu rendering|ativo/maduro|github.com/gfx-rs/wgpu ; wgpu.rs|
|three-d|renderizador 3d sobre wgpu, compila p/ wasm|rust|3d, meshes, materiais|ativo|github.com/asny/three-d|
|kiss3d|lib 3d simples em rust, wasm via webgl|rust|3d simples, formas|manutenção|github.com/sebcrozet/kiss3d|
|rapier|motor de física 2d/3d com bindings js/wasm|rust|física, simulações|ativo|github.com/dimforge/rapier ; rapier.rs|
|Mandelbrot.site|explorador de mandelbrot no browser, rust + wasm + ts|rust + wasm|fractais, zoom|ativo|github.com/rosslh/Mandelbrot.site ; mandelbrot.site|
|muze|viz de dados composável, wasm para cálculos pesados|js + wasm|charts compostos, crosstab|ativo|github.com/chartshq/muze|
|plotlars|integração polars + plotly em rust|rust|charts plotly, scatter, bar|ativo|github.com/alceal/plotlars|
|plotters-iced|backend iced para plotters, dentro de apps iced (wasm)|rust|charts em gui iced|ativo|github.com/Joylei/plotters-iced|
|ux-charts|charts responsivos zero-deps em rust/wasm via canvas|rust + wasm|charts, graphs|inicial|github.com/angular-rust/ux-charts|
|resvg|renderizador svg em rust puro com binding wasm|rust|svg, vetorial|maduro|github.com/RazrFalcon/resvg|
|resvg-js|binding js/wasm do resvg, svg de alta fidelidade no node/browser|rust + wasm|svg|maduro|github.com/yisibl/resvg-js|
|slint|toolkit de gui declarativo (rust/c++/js/python), compila p/ wasm|rust|ui declarativa|ativo/maduro|github.com/slint-ui/slint ; slint.dev|
|leptos-chartistry|biblioteca de charts para leptos|rust|charts: linhas, barras, scatter|ativo|github.com/feral-dot-io/leptos-chartistry|
|charming|viz em rust usando apache echarts como backend, ssr e wasm|rust|charts echarts, maps|ativo|github.com/yuankunzhang/charming|
|mermaid-rs-renderer|renderizador mermaid em rust puro, 23 tipos de diagrama|rust|diagramas, flowcharts, er|ativo|crates.io/crates/mermaid-rs-renderer|
|dagre-rs|port do dagre (layout sugiyama de grafos direcionados)|rust|grafos direcionados, dags|ativo|crates.io/crates/dagre|
|cytoscape.js|viz de grafos js, módulos de layout substituíveis por wasm|js (+ wasm)|grafos, redes|maduro|github.com/cytoscape/cytoscape.js ; js.cytoscape.org|
|sigma.js|renderizador webgl para grafos grandes, integrável com wasm|js + webgl|grafos grandes|maduro|github.com/jacomyal/sigma.js ; sigmajs.org|
|wasm-bindgen|interop rust/wasm com js, base de toda viz web em rust|rust|tooling|maduro|github.com/wasm-bindgen/wasm-bindgen (org nova pós-migração)|
|web-sys|bindings das web apis para rust/wasm (canvas, webgl, svg)|rust|tooling/bindings|maduro|docs.rs/web-sys|
|js-sys|bindings dos objetos js core para rust/wasm|rust|tooling/bindings|maduro|docs.rs/js-sys|
|wasmer|runtime wasm universal em rust (server, edge, browser)|rust|runtime|ativo/maduro|github.com/wasmerio/wasmer ; wasmer.io|
|wasmtime|runtime wasm de alta performance da bytecode alliance|rust|runtime|maduro|github.com/bytecodealliance/wasmtime ; wasmtime.dev|
|wasm-pack|compila/testa/publica rust como pacotes wasm p/ npm (arquivado)|rust|tooling|arquivado/manutenção|github.com/rustwasm/wasm-pack|
|wasm-opt-rs|binding rust do otimizador wasm-opt do binaryen|rust|tooling|ativo|github.com/brson/wasm-opt-rs|
|nalgebra|álgebra linear em rust com suporte wasm, base p/ layout de grafos|rust|computação linear|maduro|github.com/dimforge/nalgebra ; nalgebra.org|
|petgraph|estruturas de dados de grafos em rust, compilável p/ wasm|rust|grafos (estrutura)|maduro|github.com/petgraph/petgraph ; docs.rs/petgraph|
|veloren|game de mundo aberto em rust, rendering 3d, parte usa wasm|rust|3d, voxel|ativo|github.com/veloren/veloren ; veloren.net|
|chart-js-rs|bindings rust para chart.js via wasm|rust + wasm|charts (chart.js)|ativo|github.com/Billy-Sheppard/chart-js-rs|
|plotly (rust)|lib rust usando plotly.js como backend|rust|charts plotly|ativo|github.com/igiagkiozis/plotly|
|cosmograph|viz de grafos massivos com gpu, núcleo wasm/webgl|ts + webgl/wasm|grafos massivos, gpu|ativo|github.com/cosmograph-org/cosmograph ; cosmograph.app|
|WebCola|algoritmos de layout de grafos, aceleração wasm planejada|js + wasm exp.|grafos, layout|ativo|github.com/tgdwyer/WebCola|

## 4. conexão pessoa↔projeto (vínculos confirmados)

|pessoa|projeto|papel|comprovação|
|---|---|---|---|
|Alex Crichton|wasm-bindgen|criador original e maintainer histórico|issue #2337 rustwasm|
|Alex Crichton|wasmtime|maintainer principal atual|bytecode alliance|
|Nick Fitzgerald|wasmtime|core contributor e lead|rust governance|
|daxpedda|wasm-bindgen|maintainer principal atual|issue #4533 wasm-bindgen|
|Guy Bedford|wasm-bindgen|co-maintainer (cloudflare)|issue #4533 wasm-bindgen|
|Ashley Williams|wasm-pack|co-criadora, rustwasm wg lead|github.com/rustwasm/team|
|Emil Ernerfeldt|egui|criador e maintainer|github.com/emilk/egui|
|Emil Ernerfeldt|rerun|co-fundador (rerun.io)|rerun.io|
|blitzarx1|egui_graphs|criador e maintainer|github.com/blitzarx1/egui_graphs|
|Grant Handy|fdg|criador|github.com/grantshandy/fdg|
|Carter Anderson|bevy|criador e bdfl|bevyengine.org|
|Hao Hou|plotters|criador original|github.com/plotters-rs/plotters|
|Syrus Akbary|wasmer|criador e ceo wasmer|wasmer.io|
|Luke Wagner|wasmtime / component model|co-criador do wasm e component model|bytecode alliance|
|Lin Clark|component model / standards|engenheira de standards wasm|hacks.mozilla.org|
|Greg Johnston|leptos|criador e maintainer|leptos.dev|
|Jonathan Kelley|dioxus|criador e maintainer|dioxuslabs.com|
|Surma|wasm threads / web workers|educador e engenheiro google|surma.dev|

## 5. contexto brasil

participação brasileira identificável é limitada em maintainers de crates wasm populares, mas há contribuidores e educadores ativos. nenhum projeto brasileiro de destaque mundial em grafos ou viz em wasm + rust foi identificado com presença confirmada em repositórios top auditados.

|nome / projeto|papel|foco|links|
|---|---|---|---|
|Jorge Aparicio (japaric)|rust embedded lead, criador do cross|embedded, cross-compilation, wasm targets|github.com/japaric ; blog.japaric.io|
|Rustlang Brasil (comunidade)|telegram|rust geral|t.me/rustlangbr|
|Rust Brasil Discord|servidor comunitário|rust geral, ajuda|discord (rust-br)|
|Rust no TDC / RustLab BR|palestras em conferências|rust geral|thedevconf.com|

o caso mais documentado de impacto global por pessoa de origem brasileira no rust é Jorge Aparicio (japaric), especialmente em targets de compilação cruzada que incluem wasm-unknown-unknown.

## análise qualitativa

núcleos de contribuição: os centros mais densos são a bytecode alliance (wasmtime, wasm-tools, wasi, component model), a org rustwasm (wasm-bindgen, wasm-pack, web-sys, em sunset) e empresas como cloudflare, fastly e mozilla, que empregam os principais maintainers.

clusters de viz em wasm, quatro principais: rendering gpu (wgpu, vello, three-d, bevy) para gráficos e games; charts e dashboards (plotters, plotlars, charming, leptos-chartistry) integrados a frameworks web rust; visualização de grafos (egui_graphs, fdg, petgraph, dagre-rs, cosmograph), ainda incipiente em rust puro; e viz científica/multimodal (rerun, resvg, colorgrad-js) para ml, robótica e ciência.

perfis predominantes: engenheiros open source de crates de infra (Crichton, daxpedda, dtolnay), autores de conteúdo educacional de alto impacto (Gjengset, Wenger, Palmieri, Klabnik), criadores de frameworks web rust (gbj/leptos, jkelleyrtp/dioxus, DenisKolodin/yew) e criadores de engines gráficas (emilk/egui+rerun, cart/bevy). pesquisadores acadêmicos formais são minoria, refletindo o caráter prático da comunidade.

lacunas: falta tooling de alto nível para grafos complexos em rust + wasm, não há equivalente maduro e adotado ao networkx (python) ou d3-force (js) com binding wasm. fdg e egui_graphs cobrem o básico, faltam layouts avançados (hierárquicos, sugiyama, dimensionamento automático). falta documentação para iniciantes em português ligando rust + wasm a viz de dados. e escassez de demos interativas de qualidade para grafos dinâmicos em tempo real, onde js/ts ainda domina via sigma.js e cytoscape.js.

global vs brasil: a liderança é europeia e norte-americana, vinculada a grandes empresas e comunidades acadêmicas. a participação brasileira, embora presente em comunidades de discussão, ainda não produziu projetos de viz em rust + wasm com tração internacional mensurável. tendência de maior participação nos próximos anos via tdc e rust brasil, especialmente em viz para agronegócio e finanças.


---

# parte 4, mercado editorial de programação (global e brasil)

tópico mais amplo que os de rust/wasm: livros técnicos e autores, global e brasil, adulto e infantil. o mercado global é dominado por clássicos atemporais de princípios (clean code, design patterns, algoritmos); o brasileiro mostra forte crescimento em ia e python. metodologia: bestsellers de grandes livrarias (amazon, novatec, casa do código), rankings comunitários (goodreads, reddit, stack overflow) e metanálises.

## top 20 livros de programação no mundo

|#|título|autor(es)|ano|nível|temas|
|---|---|---|---|---|---|
|1|Clean Code|Robert C. Martin|2008|interm-avançado|qualidade, boas práticas, refatoração|
|2|The Pragmatic Programmer|Andrew Hunt, David Thomas|1999/2019|todos|práticas profissionais, carreira|
|3|Code Complete|Steve McConnell|1993/2004|intermediário|construção de software|
|4|Introduction to Algorithms (CLRS)|Cormen, Leiserson, Rivest, Stein|1990/2022|avançado|algoritmos, estruturas, complexidade|
|5|Design Patterns (gof)|Gamma, Helm, Johnson, Vlissides|1994|interm-avançado|padrões de projeto, oo|
|6|Refactoring|Martin Fowler|1999/2018|intermediário|refatoração, design|
|7|The Art of Computer Programming (1-4)|Donald Knuth|1968-|avançado|algoritmos fundamentais, análise matemática|
|8|Cracking the Coding Interview|Gayle Laakmann McDowell|2008/2015|intermediário|entrevistas, problemas algorítmicos|
|9|Domain-Driven Design|Eric Evans|2003|avançado|modelagem de domínio|
|10|The Mythical Man-Month|Frederick P. Brooks Jr.|1975/1995|todos|gestão de projetos de software|
|11|Designing Data-Intensive Applications|Martin Kleppmann|2017|avançado|sistemas distribuídos, escalabilidade|
|12|Working Effectively with Legacy Code|Michael Feathers|2004|interm-avançado|código legado, testes|
|13|SICP|Abelson, Sussman|1985/1996|interm-avançado|fundamentos cs, fp, lisp|
|14|Head First Design Patterns|Freeman, Robson|2004/2020|inic-interm|padrões, abordagem visual|
|15|The Clean Coder|Robert C. Martin|2011|todos|profissionalismo, ética|
|16|Python Crash Course|Eric Matthes|2015/2023|iniciante|python, projetos práticos|
|17|Patterns of Enterprise Application Architecture|Martin Fowler|2002|avançado|arquitetura empresarial|
|18|Clean Architecture|Robert C. Martin|2017|avançado|arquitetura, solid|
|19|Peopleware|DeMarco, Lister|1987/2013|todos|gestão de pessoas, cultura|
|20|Programming Pearls|Jon Bentley|1986/1999|interm-avançado|algoritmos, otimização|

padrões globais: domínio de princípios atemporais sobre tecnologias específicas. autoria concentrada (Robert C. Martin com 4 títulos da série clean; Martin Fowler com 3). longevidade extraordinária (taocp de 1968, mythical man-month de 1975 ainda essenciais). adição recente mais notável: designing data-intensive applications (2017).

## top 20 livros de programação no brasil

|#|título|autor(es)|ano|origem|editora|
|---|---|---|---|---|---|
|1|Engenharia de Prompt para Devs|Ricardo Pupo Larguesa|2024, nova ed. 2025|original pt-br|Casa do Código|
|2|Introdução à Programação com Python (4ª ed)|Nilo Ney Coutinho Menezes|2024|original pt-br|Novatec|
|3|Entendendo Algoritmos|Aditya Y. Bhargava|2017|tradução|Novatec|
|4|Inteligência Artificial e ChatGPT|Fabrício Carraro|2023-2024|original pt-br|Casa do Código|
|5|Código Limpo (Clean Code)|Robert C. Martin|2009 (trad)|tradução|Alta Books|
|6|Estruturas de Dados e Algoritmos com JavaScript|Loiane Groner|2019|original pt-br|Novatec|
|7|Lógica de Programação|Paulo Silveira|2014-2016|original pt-br|Casa do Código|
|8|Arquitetura Java|Paulo e Guilherme Silveira|2012-2018|original pt-br|Casa do Código|
|9|Use a Cabeça Java|Kathy Sierra, Bert Bates|2005 (trad)|tradução|Alta Books|
|10|Expressões Regulares: Uma Abordagem Divertida|Aurelio Marinho Jargas|2006-2016|original pt-br|Novatec|
|11|O Programador Apaixonado|Chad Fowler|2009 (trad)|tradução|Casa do Código|
|12|Engenharia de Software para Ciência de Dados|Aniche, Gerosa|2023|original pt-br|Casa do Código|
|13|A Linguagem de Programação C|Kernighan, Ritchie|1988 (trad)|tradução|Campus/Elsevier|
|14|Algoritmos Teoria e Prática (CLRS)|Cormen et al|2012 (trad)|tradução|Campus/Elsevier|
|15|Como ser um Programador Melhor|Pete Goodliffe|2015 (trad)|tradução|Novatec|
|16|Desenvolvendo Websites com PHP|Juliano Niederauer|2011-2017|original pt-br|Novatec|
|17|Java: Como Programar|Deitel & Deitel|2010 (trad)|tradução|Pearson BR|
|18|C: Completo e Total|Herbert Schildt|1997 (trad)|tradução|Pearson BR|
|19|Python para Desenvolvedores|Luiz Eduardo Borges|2014|original pt-br|Novatec|
|20|Testes de Software|Thiago Leite e Carvalho, João F. R. Viana|2023|original pt-br|Casa do Código|

características do mercado br: explosão de ia generativa (engenharia de prompt para devs foi #1 da casa do código em 2024, nova edição ampliada de 384 págs em nov 2025, com capítulo sobre agentes como cursor, copilot, claude code, codex e gemini cli). editoras nacionais especializadas (casa do código, do grupo alura, publica só autores brasileiros desde 2012; e novatec). python em ascensão como linguagem de entrada. tradução de clássicos via alta books, pearson e campus/elsevier. livros nacionais 30 a 50% mais baratos que traduções importadas.

## top 15 livros de programação infantil no mundo

|#|título|autor(es)|faixa|linguagem|país|
|---|---|---|---|---|---|
|1|Coding for Kids: Python and Scratch|CodaKid Team|6-12|python, scratch|eua|
|2|Hello Ruby: Adventures in Coding|Linda Liukas|4-7|conceitual|finlândia|
|3|Help Your Kids with Computer Coding|DK|6-14|scratch, python|reino unido|
|4|Scratch Coding for Beginners|David Dodge et al|6-12|scratch|eua|
|5|Coding Games in Scratch|Jon Woodcock|8-12|scratch|reino unido|
|6|Python for Kids|Jason Briggs|10-14|python|eua/austrália|
|7|How to Code a Sandcastle|Josh Funk|4-8|conceitual|eua|
|8|25 Scratch 3 Games for Kids|Max Wainewright|6-12|scratch 3|reino unido|
|9|Computer Coding for Kids|Carol Vorderman|6-14|scratch, python|reino unido|
|10|Lift The Flap Computers & Coding|Rosie Dickins|7-12|conceitual|reino unido|
|11|Creative Coding in Python|Sheena Vaidyanathan|8-11|python|eua|
|12|Lauren Ipsum|Carlos Bueno|8-14|conceitual|eua|
|13|HTML for Babies|John C Vanden-Heuvel Sr|3-5|html (conceitual)|eua|
|14|Coding for Kids Ages 8-12|Grapevine|8-12|scratch, blocos|eua|
|15|Programming for Kids Age 8-12|Various|8-12|scratch, python|eua|

tendências infantis globais: scratch como padrão de entrada (6-12 anos), python como segundo passo (10-12+), abordagem lúdica e visual (storytelling, ilustrações), distinção entre pensamento computacional "unplugged" e sintaxe de linguagem, e material complementar digital (github, vídeos).

## top 15 livros de programação infantil no brasil

|#|título|autor(es)|faixa|linguagem|origem/editora|
|---|---|---|---|---|---|
|1|Olá, Ruby: Uma Aventura pela Programação|Linda Liukas|4-7|conceitual|Companhia das Letrinhas (trad)|
|2|Aprenda a Programar com Scratch|Majed Marji|8-14|scratch|Novatec (trad)|
|3|Aprenda a Programar com Python|Leonardo Soares, Gabriel Fortes|10-16|python|Casa do Código|
|4|Programando com Scratch JR|vários|5-7|scratch jr|materiais educacionais|
|5|Lógica de Programação|Paulo Silveira|10-16|javascript, html|Casa do Código|
|6|Lauren Ipsum|Carlos Bueno|8-14|conceitual|Novatec (trad)|
|7|Computadores e Programação: Brincar e Aprender|Usborne|7-12|conceitual|Usborne Brasil (trad)|
|8|Meu Primeiro Livro de Programação|Nestor Burlamaqui|8-14|múltiplas|Letras & Cia|
|9|Programando com o Scratch|autores brasileiros|8-14|scratch|Clube de Autores|
|10|C# para Crianças|autor br|10-14|c#|Iberlibro|
|11|Aprenda a Programar com Minecraft|vários|8-14|python (minecraft)|Novatec (trad)|
|12|Matemática e Programação na Educação Básica|Marcos Galvão|10-16|scratch, python, portugol|Dialética|
|13|Computação e Eu (6º ano)|Bianca Santana et al|11-12|conceitual|material didático uefs|
|14|Kit CDR Kids: Robótica para Crianças|Casa da Robótica|8-14|programação visual|Casa da Robótica|
|15|Explorando Computação na Educação Infantil|vários|4-6|unplugged|e-book independente|

características do infantil br: traduções predominantes, crescimento de autoria nacional nos últimos 3-5 anos, foco escolar e bncc (pensamento computacional, atividades desplugadas), lacuna de impressos de alta qualidade, integração com robótica e materiais gratuitos do scratch brasil.

## top 15 autores brasileiros de programação

|#|nome|principais obras|temas|perfil|
|---|---|---|---|---|
|1|Nilo Ney Coutinho Menezes|Introdução à Programação com Python (4 ed)|python, iniciantes|autor do livro de python mais vendido no brasil ; python.nilo.pro.br|
|2|Paulo Silveira|Lógica de Programação, Arquitetura Java|java, arquitetura|cofundador caelum/alura/casa do código, mestre usp|
|3|Ricardo Pupo Larguesa|Engenharia de Prompt para Devs|ia, llms, prompts|#1 casa do código 2024, professor, fundador t2s|
|4|Loiane Groner|Estruturas de Dados e Algoritmos com JavaScript|js, estruturas, typescript|autora pela packt, instrutora ; loiane.com|
|5|Guilherme Silveira|Arquitetura Java, títulos casa do código|java, arquitetura|cofundador caelum, editor-chefe casa do código|
|6|Aurelio Marinho Jargas|Expressões Regulares: Uma Abordagem Divertida|regex, shell|pioneiro em regex acessível ; aurelio.net|
|7|Fabrício Carraro|Inteligência Artificial e ChatGPT|ia, chatgpt|entre os mais vendidos da casa do código 2024|
|8|Maurício Aniche|Engenharia de Software para Ciência de Dados|ml eng, qualidade, testes|professor tu delft, finalista jabuti|
|9|Thiago Leite e Carvalho|Testes de Software|testes, automação|especialista em qualidade|
|10|Juliano Niederauer|Desenvolvendo Websites com PHP|php, web|referência php br anos 2010|
|11|Leonardo Soares e Silva|Aprenda a Programar com Python|python, educação|professor ifpe, doutor coimbra|
|12|Gabriel Fortes|Aprenda a Programar com Python (coautor)|python, educação|doutor, pesquisador em ensino|
|13|Luiz Eduardo Borges|Python para Desenvolvedores|python prático|foco em aplicações reais|
|14|Bianca Leite Santana|Computação e Eu (didático)|fundamentos cs, ensino fundamental|materiais alinhados a k-12 cs standards|
|15|Nestor Burlamaqui|Meu Primeiro Livro de Programação|programação infantil|abordagem lúdica multiplataforma|

perfil dos autores br: majoritariamente educadores-autores (caelum, alura), foco em português com contexto local, múltiplas mídias (livro + curso + youtube + blog), e vínculo institucional (universidades federais, ifs, empresas de educação tech).

## top 15 autores globais de programação

|#|nome|principais obras|temas|
|---|---|---|---|
|1|Robert C. Martin (Uncle Bob)|série Clean (Code, Coder, Architecture, Craftsmanship)|qualidade, arquitetura, solid|
|2|Martin Fowler|Refactoring, PoEAA, DSLs, NoSQL Distilled|refatoração, arquitetura, padrões|
|3|Donald Knuth|The Art of Computer Programming (1-4)|algoritmos, análise matemática|
|4|Erich Gamma|Design Patterns (gof)|padrões, oo (eclipse, vs code)|
|5|Eric Evans|Domain-Driven Design|modelagem de domínio|
|6|Thomas H. Cormen|CLRS|algoritmos, estruturas|
|7|Andrew Hunt|The Pragmatic Programmer|práticas profissionais|
|8|David Thomas|The Pragmatic Programmer|práticas profissionais|
|9|Steve McConnell|Code Complete, Rapid Development|construção de software|
|10|Gayle Laakmann McDowell|Cracking the Coding Interview|entrevistas técnicas|
|11|Frederick P. Brooks Jr.|The Mythical Man-Month|gestão de projetos (turing 1999)|
|12|Martin Kleppmann|Designing Data-Intensive Applications|sistemas distribuídos|
|13|Harold Abelson|SICP|fundamentos cs, fp, lisp|
|14|Kent Beck|TDD by Example, XP Explained|tdd, ágil, testes|
|15|Michael Feathers|Working Effectively with Legacy Code|código legado, refatoração|

perfil dos autores globais: longevidade (Martin desde 1970, knuth desde 1968), vínculo acadêmico (mit, stanford, cambridge, dartmouth), signatários do manifesto ágil de 2001 (fowler, martin, beck, hunt, thomas), contribuições além de livros (knuth/tex, beck/junit, gamma/eclipse), e laureados turing (knuth 1974, brooks 1999).

## análise qualitativa do mercado editorial

padrões temáticos dominantes (dos ~80 livros mapeados): qualidade e manutenibilidade de código (~25%, série clean lidera); algoritmos e estruturas (~20%, clrs como âncora, "entendendo algoritmos" democratizou); arquitetura e padrões (~18%, gof, ddd, fowler); ia e ml (categoria explosiva, domina bestsellers br 2024-2025, ainda ~5% global); linguagens específicas (~15%, python em ascensão).

global vs brasil: o brasil adota tendências emergentes mais rápido nos bestsellers (ia generativa chegou ao #1 imediatamente, enquanto global ainda é dominado por clássicos). barreira linguística e custo criam nicho para autores locais (traduções 30-50% mais caras). concentração editorial br em duas editoras (casa do código e novatec, ~70% do mercado técnico) vs dezenas globais (o'reilly, pragmatic, no starch, manning, addison-wesley). autores br tendem mais a ser educadores ativos; globais, engenheiros sênior de grandes empresas.

adulto vs infantil: adulto foca princípios atemporais (clean code e design patterns relevantes décadas depois). infantil precisa ancorar em plataformas concretas (scratch, minecraft), o que gera obsolescência mais rápida (scratch 2 vs 3). há transição difícil: lacuna entre infantil (até 12) e profissional, com pouco material para adolescentes de 13-17.

linhagens de pensamento: uncle bob (artesania de software, série clean, solid, criticado por dogmatismo); pragmática (hunt/thomas, pragmatic bookshelf); acadêmica formal (knuth, clrs, rigor matemático); padrões (gof, fowler enterprise); ddd (evans, modelagem rica de domínio); e ágil (beck, fowler, martin, manifesto 2001, tdd/xp).

lacunas e oportunidades no brasil: livros infantis de alta qualidade originais; material para adolescentes (13-17); temas emergentes em português (rust, go, webassembly, edge computing têm pouquíssimos livros, autores que dominarem encontram mercado receptivo); data science/ml com datasets brasileiros (saúde pública, agronegócio); acessibilidade e desenvolvimento inclusivo (quase inexistente em pt).

lacunas globais: arquitetura de ia/ml em produção (mlops, monitoramento); segurança moderna (supply chain, contêineres, zero trust); sustentabilidade em software (green computing); programação para não-programadores (no-code/low-code + ia); ética e impacto social (viés algorítmico).

linguagens sub-representadas: rust (cresce rápido em sistemas, wasm e infra, mas tem ~5% da cobertura de livros vs c++/java); fp moderno aplicado (kotlin, swift, js, vs acadêmico haskell/sicp); webassembly (literatura limitada); quantum computing (nicho crescente).

ponte com as partes 1 a 3: a lacuna global e brasileira em livros de rust e webassembly, apontada aqui, é exatamente o espaço mapeado nas partes 1 a 3. o único livro brasileiro de wasm + rust ("Desmistificando WebAssembly", Raphael Amorim, casa do código) reaparece nos dois lados, confirmando a lacuna.

---

# apêndice A, referência externa: SEO + webassembly

conteúdo de terceiro, não é material do projeto. fonte: artigo "The Future of SEO with WebAssembly" da gtechme (gtechme.com/insights/webassembly-seo-future-challenges), agência de desenvolvimento web nos eua. em inglês, originalmente com call-to-action de venda da gtechme, que foi removido. fica aqui só a espinha técnica, por ser adjacente aos clusters de wasm das partes 2 e 3 e útil para quem publica conteúdo em wasm.

tese central: wasm dá velocidade quase nativa no browser (edição de imagem, mapas 3d, analytics pesado, criptografia, sem round-trips de servidor), o que ajuda core web vitals e reduz bounce. mas ranqueamento depende de crawlers descobrirem, renderizarem e entenderem a página, e wasm não resolve crawlability por padrão.

o problema de opacidade: se conteúdo ou links importantes só aparecem depois que um script cliente roda, ou se o html entrega cascas vazias preenchidas só por um boot de wasm, o crawler pode ver conteúdo "fino". se a informação só existe na memória depois que uma função wasm executa, o crawler pode nunca vê-la.

correção (rendering híbrido): entregar html renderizado no servidor ou estático para páginas-chave, com texto, headings e links significativos no primeiro load; só então sobrepor wasm para o trabalho pesado. tratar js e wasm como complementares: js cuida da camada de ui e do glue (links e texto crawláveis), wasm cuida da parte intensiva. tratar hidratação e interatividade como progressive enhancement.

padrões que funcionam: pré-renderizar título, meta tags, canonical, structured data e conteúdo primário sem rodar scripts; code-split (carregar módulos wasm só nas rotas que precisam, reduzindo payload inicial e melhorando core web vitals); expor dados crawláveis (se wasm gera texto/detalhes importantes, também renderizar no servidor ou embutir no html); e instrumentar tracking na fronteira js, porque algumas libs de analytics não capturam interações dentro do módulo wasm.

frameworks: usar server components ou ssr para páginas de conteúdo e hidratar só onde necessário (react, vue, svelte com island architecture). o princípio é comum: entregar html indexável, depois acoplar interatividade.

medição: benchmark antes e depois, acompanhar lcp, inp e cls com dados de campo, e cruzar com análise de logs para confirmar frequência de crawl e latência de render.

contexto declarado no artigo (não verificado de forma independente): menção a wasm 3.0 com garbage collection e multi-threading padronizados, e a afirmação de que googlebot executa wasm mas com limites de recursos, podendo atrasar ou completar parcialmente o render, o que reforça manter uma versão estática do conteúdo para indexação confiável.

---

# bibliografia (por eixo)

## eixo 1, ecossistema rust open source

1. Awesome Rust, github.com/awesome-rust-com/awesome-rust (e github.com/rust-unofficial/awesome-rust)
2. LibHunt Rust, libhunt.com/l/rust
3. OSS Insight Rust, ossinsight.io/languages/Rust
4. GitHub Stars Leaderboard Rust, githublb.vercel.app/language/rust
5. Github Ranking Top 100 Stars in Rust, evanli.github.io/Github-Ranking/Top100/Rust.html
6. rust-in-production (ImplFerris), github.com/ImplFerris/rust-in-production
7. ripgrep benchmarks, ripgrep.dev/benchmarks e ripgrep.dev/vs/ag
8. "ripgrep is faster than grep, ag, git grep..." (Andrew Gallant), blog.burntsushi.net
9. denosaurs/bench, github.com/denosaurs/bench
10. "Bun vs Deno vs Node.js in 2026", dev.to/jsgurujobs
11. "Deno vs Bun vs Node.js: Performance & Benchmarks", dev.to/yogeshhrathod
12. discussão deno benchmarks #15121, github.com/denoland/deno/discussions/15121
13. "Vector, A High-Performance Logs & Metrics Router Written In Rust" (timber), dev.to/timber
14. material arquivado vector (timberio), archive.org/details/github.com-timberio-vector_-_2020-06-28_07-01-09
15. perf-book, benchmarking (nnethercote), nnethercote.github.io/perf-book/benchmarking.html
16. cloudflare/pingora, github.com/cloudflare/pingora (aberto pela cloudflare em 2024)
17. firecracker-microvm/firecracker, github.com/firecracker-microvm/firecracker

## eixo 2, livros e recursos de rust + webassembly

1. The Rust Programming Language, 3ª ed, ✓ jun/2026, doc.rust-lang.org/book e nostarch.com/rust-programming-language-3e (No Starch, mar 2026, ISBN 978-1718504448, +Chris Krycho, rust 2024)
2. Programming Rust, 3ª ed, ✓ jun/2026, oreilly.com (O'Reilly, jul 2026, ISBN 9781098176228, rust 2024)
3. Rust for Rustaceans (Jon Gjengset), rust-for-rustaceans.com
4. The Rust and WebAssembly Book, ✓ jun/2026 ARQUIVADO, rustwasm.github.io/book e github.com/rustwasm/book
5. "Sunsetting the rustwasm GitHub org" (Alex Crichton, 21 jul 2025), ✓ jun/2026, blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org
6. wasm-bindgen Guide (org nova), ✓ jun/2026, wasm-bindgen.github.io
7. wasm-pack Book (arquivado), rustwasm.github.io/wasm-pack/book
8. Programming WebAssembly with Rust (Kevin Hoffman, 2019), pragprog.com/titles/khrust
9. Practical WebAssembly (Sendil Kumar Nellaiyapen, 2022), ✓ jun/2026, packtpub.com (9781838828004)
10. Game Development with Rust and WebAssembly (Eric Smith, 2022), packtpub.com (9781801070973)
11. WebAssembly: The Definitive Guide (Brian Sletten, 2021), oreilly.com (9781492089834)
12. Server-Side WebAssembly (Manning, 2025), manning.com/books/server-side-webassembly
13. Learn WebAssembly (Mike Rourke, 2018), packtpub.com (9781788997379)
14. Leptos Book, book.leptos.dev ; Yew, yew.rs ; Dioxus, dioxuslabs.com
15. WASI Tutorial / Wasmtime, docs.wasmtime.dev
16. MDN, Compiling Rust to WebAssembly, developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm
17. Desmistificando WebAssembly (Raphael Amorim, Casa do Código), ✓ jun/2026, casadocodigo.com.br/products/livro-webassembly
18. Programação em Rust 2ª ed (Novatec), play.google.com (id jtrPEAAAQBAJ)
19. A linguagem de programação Rust, tradução comunitária, rust-br.github.io/rust-book-pt-br
20. FLAG não recomendados, ✓ jun/2026: títulos KDP gerados por IA ("Nexus AI" 97 livros, "ALI. E. PACE", "Cliff S. Armstrong"), confirmados via páginas goodreads/amazon com 0 avaliações e descrições genéricas

## eixo 3, pessoas e projetos de visualização em rust + wasm

1. "Sunsetting the rustwasm GitHub org", ✓ jun/2026, blog.rust-lang.org/inside-rust/2025/07/21/...
2. Rust governance / teams, rust-lang.org/governance/teams
3. GitHub topics visualization (rust), github.com/topics/visualization?l=rust
4. lib.rs/visualization
5. wasm-bindgen issues #2337 e #4533, github.com/rustwasm/wasm-bindgen/issues
6. Bytecode Alliance, bytecodealliance.org
7. Best of JS, rust + webassembly, bestofjs.org
8. made with webassembly, madewithwebassembly.com
9. repositórios dos projetos (links nas tabelas da parte 3): egui, bevy, leptos, dioxus, wgpu, vello, rerun, plotters, wasmer, wasmtime, petgraph, resvg, slint etc.

## eixo 4, mercado editorial de programação

1. "Saiu a lista dos mais vendidos da Casa do Código em 2025", desbugados.com.br/post/2025/12/30/...
2. "Casa do Código lança versão atualizada de Engenharia de Prompt para Devs" (nov 2025), ✓ jun/2026, desbugados.com.br/noticias/2025/11/24/...
3. Engenharia de Prompt para Devs (nova ed. nov 2025, 384pp, ISBN 9788555193712), ✓ jun/2026, casadocodigo.com.br/products/livro-engenharia-de-prompt
4. Livros mais vendidos, Novatec, novatec.com.br/mais-vendidos.php
5. Sobre a Casa do Código (grupo alura), casadocodigo.com.br/pages/sobre-a-casa-do-codigo
6. "The 25 best programming books of all time", goodreads.com/list/show/145844
7. "The most recommended programming books of all-time", reddit r/programming
8. influential-cs-books, github.com/cs-books/influential-cs-books
9. "8 Most Influential Books on Programming of All Time", bgosoftware.com
10. "Best programming books 2026", techtargets.net
11. páginas de autor/editora: martinfowler.com, cleancoder.com, www-cs-faculty.stanford.edu/~knuth, mitpress.mit.edu, pragprog.com, nostarch.com
12. autores br: python.nilo.pro.br (Nilo Menezes), loiane.com (Loiane Groner), aurelio.net (Aurelio Jargas)

## apêndice (referência externa)

1. "The Future of SEO with WebAssembly" (gtechme), gtechme.com/insights/webassembly-seo-future-challenges, conteúdo de terceiro com cta de venda (removido)
