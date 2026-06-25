---
title: panorama de aplicacoes rust (primeira leva)
date: 2026-06-25
tags: [refs, rust, apps, panorama, ecossistema, caranguejovermelho]
fontes:
  - evanli.github.io/Github-Ranking/Top100/Rust.html (snapshot ~2026-06-25, via raw EvanLi/Github-Ranking/master/Top100/Rust.md)
  - api.github.com/repos/{owner}/{repo} (consultado 2026-06-25)
  - github.com/{owner}/{repo} contador de stars no HTML, atributo aria-label (consultado 2026-06-25)
  - github.com/rust-unofficial/awesome-rust (validado 200)
  - github.com/ImplFerris/rust-in-production (validado 200)
  - lib.rs (fonte conhecida; retorna 403 a curl por anti-bot, nao revalidada por curl nesta rodada)
status_validacao:
  itens_cobertos: 157
  links_validados_http_200: 157
  itens_nao_confirmados_nesta_rodada: 0 (todos os 157 resolvem; ver ressalva separada sobre legitimidade de stars de alguns top-rankers e sobre o caso bun)
  metodo: curl -s -L (segue redirect) capturando http_code e url_effective; stars cruzadas entre tres fontes citaveis
---

# panorama de aplicacoes rust (primeira leva)

esta e a dimensao "apps" do corpus. o objetivo nao e enciclopedia, e situar o
plev no mapa do que rust ja entrega em producao, com honestidade sobre onde a
linguagem ganha de fato e onde o ganho e marginal. rust domina tres frentes com
folga: cli rapida (substitutos de grep, find, cat, ls), runtimes e toolchains de
build (deno, swc, turborepo, biome), e infra de dados e rede (bancos, search,
proxies, pipelines de observabilidade). em gui e ai/ml a presenca cresce, mas
ainda atras de c++ e python, e boa parte do valor esta em bibliotecas, nao em
apps de usuario final.

a leva tem 157 aplicacoes e bibliotecas de infraestrutura, todas com link
validado (http 200 em 2026-06-25). a contagem de stars vem sempre de fonte com
link, nunca de estimativa. quando uma fonte nao confirmou um numero, o item fica
marcado, nao chutado.

## como ler as tabelas

colunas: nome, url, subdominio, maturidade, stars (fonte), por que relevante,
validacao. a categoria e o titulo de cada bloco.

legenda da fonte de stars (todas consultadas em 2026-06-25):

- `(E)` evanli Github-Ranking, Top 100 Rust, snapshot de ~2026-06-25.
- `(A)` api.github.com REST, campo `stargazers_count`.
- `(H)` contador de stars no HTML da pagina do repo, atributo `aria-label`.

maturidade e uma leitura qualitativa do ritmo do projeto, nao um numero: `maduro`
(estavel, usado em producao), `ativo` (desenvolvimento forte e corrente), `beta`
(usavel, api ainda muda), `manutencao` (vivo mas com pouco recurso novo),
`dormente` (ritmo muito baixo), `arquivado` (descontinuado).

validacao: `ok` significa que a url resolve com http 200; quando a fonte
redirecionou para um dono novo, a url ja esta na forma canonica atual e isso vai
anotado.

## bloco 1: devtools e cli

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| ripgrep | https://github.com/BurntSushi/ripgrep | busca/grep | maduro | 65437 (E) | busca regex respeitando gitignore; padrao de fato, base de comparativos com grep e ag | ok |
| fd | https://github.com/sharkdp/fd | busca de arquivos | maduro | 43463 (E) | alternativa ao find, paralela e amigavel | ok |
| bat | https://github.com/sharkdp/bat | visualizacao | maduro | 59406 (E) | cat com syntax highlight e integracao git | ok |
| ruff | https://github.com/astral-sh/ruff | lint/format python | ativo | 48203 (E) | linter e formatter de python em rust, ordens de magnitude mais rapido que flake8; projeto astral | ok |
| uv | https://github.com/astral-sh/uv | gerenciador de pacotes python | ativo | 86734 (E) | package e project manager de python, substitui pip e poetry; astral | ok |
| just | https://github.com/casey/just | task runner | maduro | 34411 (E) | command runner, alternativa a make sem o legado | ok |
| delta | https://github.com/dandavison/delta | git/diff | maduro | 31241 (E) | pager com syntax highlight para git diff, grep e blame | ok |
| zoxide | https://github.com/ajeetdsouza/zoxide | navegacao shell | maduro | 37609 (E) | cd inteligente com ranking por frequencia de uso | ok |
| starship | https://github.com/starship/starship | prompt shell | maduro | 58475 (E) | prompt minimalista e rapido para qualquer shell | ok |
| hyperfine | https://github.com/sharkdp/hyperfine | benchmark cli | maduro | 28354 (E) | ferramenta de benchmark de linha de comando, usada nos proprios comparativos do livro | ok |
| exa | https://github.com/ogham/exa | listagem de arquivos | arquivado | 24445 (E) | substituto de ls, descontinuado; o sucessor mantido e o eza | ok |
| eza | https://github.com/eza-community/eza | listagem de arquivos | ativo | 22396 (A) | fork comunitario e mantido do exa | ok |
| sd | https://github.com/chmln/sd | edicao de texto | maduro | 7217 (A) | find and replace intuitivo, alternativa ao sed | ok |
| dust | https://github.com/bootandy/dust | disco | maduro | 11878 (A) | du com visualizacao em arvore | ok |
| tokei | https://github.com/XAMPPRocky/tokei | metrica de codigo | maduro | 14610 (A) | conta linhas de codigo por linguagem, muito rapido | ok |
| bottom | https://github.com/ClementTsang/bottom | monitor de sistema | maduro | 13636 (A) | monitor de recursos no terminal (btm) | ok |
| procs | https://github.com/dalance/procs | processos | maduro | 6088 (A) | substituto moderno do ps | ok |
| bandwhich | https://github.com/imsnif/bandwhich | rede | maduro | 11831 (A) | uso de banda por processo no terminal | ok |
| gitui | https://github.com/gitui-org/gitui | git tui | maduro | 22177 (A) | cliente git no terminal; repo renomeado de extrawurst para gitui-org | ok (redirect) |
| mise | https://github.com/jdx/mise | gerenciador de toolchain | ativo | 30072 (E) | dev tools, env vars e task runner; sucessor de asdf e rtx | ok |
| atuin | https://github.com/atuinsh/atuin | historico de shell | ativo | 30345 (E) | historico de shell sincronizado e pesquisavel | ok |
| difftastic | https://github.com/Wilfred/difftastic | diff | maduro | 25539 (E) | diff estrutural que entende a sintaxe da linguagem | ok |
| broot | https://github.com/Canop/broot | navegacao de arquivos | maduro | 12774 (A) | navegacao em arvore de diretorios | ok |
| xh | https://github.com/ducaale/xh | http cli | maduro | 7885 (A) | cliente http amigavel, no espirito do httpie | ok |
| jless | https://github.com/PaulJuliusMartinez/jless | json viewer | maduro | 5438 (A) | visualizador e pager de json e yaml | ok |
| mdBook | https://github.com/rust-lang/mdBook | docs e site | maduro | 21866 (A) | gerador de livros e docs a partir de markdown; motor do proprio rust book | ok |
| watchexec | https://github.com/watchexec/watchexec | automacao | maduro | 7030 (A) | roda um comando quando arquivos mudam | ok |
| cargo-watch | https://github.com/watchexec/cargo-watch | automacao cargo | dormente | 2863 (A) | watch para projetos cargo; em fim de vida, recomenda watchexec e bacon | ok |
| sccache | https://github.com/mozilla/sccache | build cache | maduro | 7385 (A) | compiler cache compartilhado e distribuido; mozilla | ok |
| tealdeer | https://github.com/tealdeer-rs/tealdeer | docs/tldr | maduro | 6317 (A) | cliente tldr muito rapido | ok |
| onefetch | https://github.com/o2sh/onefetch | git info | maduro | 11934 (A) | resumo visual de um repo git no terminal | ok |
| ouch | https://github.com/ouch-org/ouch | compressao | ativo | 3636 (A) | comprime e descomprime com uma interface unica para varios formatos | ok |
| fselect | https://github.com/jhspetersson/fselect | busca de arquivos | ativo | 4448 (A) | busca arquivos com sintaxe parecida com sql | ok |
| typos | https://github.com/crate-ci/typos | lint | ativo | 4019 (A) | corretor de typos em codigo-fonte, usado em ci (e neste repo) | ok |
| jujutsu (jj) | https://github.com/jj-vcs/jj | controle de versao | ativo | 29780 (E) | vcs compativel com git, modelo de trabalho mais simples; nasceu no google | ok |
| gitoxide | https://github.com/GitoxideLabs/gitoxide | git lib/cli | ativo | 11638 (A) | implementacao de git em rust puro (gix); base de ferramentas de performance | ok |
| coreutils (uutils) | https://github.com/uutils/coreutils | utilitarios base | ativo | 23675 (E) | reescrita cross-platform do gnu coreutils; chegou a ser cogitada como default no ubuntu | ok |
| pueue | https://github.com/Nukesor/pueue | fila de tarefas | maduro | 6243 (A) | fila e gerenciador de comandos shell em background | ok |

## bloco 2: runtimes e toolchains de build

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| deno | https://github.com/denoland/deno | runtime js/ts | maduro | 107284 (E) | runtime js/ts seguro, core em rust sobre o v8 | ok |
| bun | https://github.com/oven-sh/bun | runtime js | ativo | 93442 (E) | runtime, bundler, test runner e package manager; ver ressalva, o core e zig, nao rust | ok (linguagem divergente) |
| tauri | https://github.com/tauri-apps/tauri | runtime desktop/mobile | maduro | 108301 (E) | apps desktop e mobile com frontend web e backend rust; alternativa enxuta ao electron | ok |
| wasmtime | https://github.com/bytecodealliance/wasmtime | runtime wasm | maduro | 18250 (A) | runtime webassembly e wasi da bytecode alliance | ok |
| wasmer | https://github.com/wasmerio/wasmer | runtime wasm | maduro | 20843 (A) | runtime wasm universal com varios backends de compilacao | ok |
| swc | https://github.com/swc-project/swc | compilador web | maduro | 34133 (E) | compilador e bundler js/ts em rust; usado pelo next.js | ok |
| biome | https://github.com/biomejs/biome | toolchain web | ativo | 25186 (E) | formatter e linter para projetos web; sucessor do rome | ok |
| rome (tools) | https://github.com/rome/tools | toolchain web | arquivado | 23416 (E) | toolchain unificada js/ts descontinuada; continuou como biome | ok |
| turborepo | https://github.com/vercel/turborepo | build system | maduro | 30594 (E) | build system para monorepos js/ts, reescrito em rust; vercel | ok |
| wasm-bindgen | https://github.com/wasm-bindgen/wasm-bindgen | interop wasm/js | maduro | 9057 (A) | ponte rust e js para wasm; repo renomeado de rustwasm | ok (redirect) |
| trunk | https://github.com/trunk-rs/trunk | bundler wasm | ativo | 4316 (A) | empacotador para apps web em rust e wasm | ok |
| wasm-pack | https://github.com/wasm-bindgen/wasm-pack | build wasm | manutencao | 7218 (A) | empacota crates rust para npm e wasm; repo renomeado de rustwasm | ok (redirect) |
| fnm | https://github.com/Schniz/fnm | version manager node | maduro | 26062 (E) | gerenciador de versoes do node em rust | ok |
| volta | https://github.com/volta-cli/volta | toolchain js | manutencao | 13017 (A) | gerenciador de toolchain js | ok |

## bloco 3: infra, observabilidade e bancos de dados

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| vector | https://github.com/vectordotdev/vector | observabilidade | maduro | 22091 (A) | pipeline de logs, metricas e traces de alto desempenho; datadog | ok |
| meilisearch | https://github.com/meilisearch/meilisearch | search engine | maduro | 58282 (E) | engine de busca rapido com busca hibrida por ai | ok |
| qdrant | https://github.com/qdrant/qdrant | vector db | maduro | 32624 (E) | banco vetorial de larga escala para ai | ok |
| influxdb | https://github.com/influxdata/influxdb | time series db | maduro | 31592 (E) | datastore de series temporais; o core v3 foi reescrito em rust com datafusion | ok |
| tikv | https://github.com/tikv/tikv | kv distribuido | maduro | 16747 (A) | key-value transacional distribuido; projeto graduado da cncf | ok |
| surrealdb | https://github.com/surrealdb/surrealdb | multi-model db | ativo | 32469 (E) | banco multi-modelo document-graph para a web em tempo real | ok |
| polars | https://github.com/pola-rs/polars | dataframe | ativo | 38868 (E) | engine de dataframes colunar sobre arrow; rival direto do pandas | ok |
| tantivy | https://github.com/quickwit-oss/tantivy | full-text search lib | maduro | 15460 (A) | biblioteca de busca full-text no estilo lucene | ok |
| quickwit | https://github.com/quickwit-oss/quickwit | log search | ativo | 11361 (A) | busca de logs sobre object storage; adquirida pela datadog | ok |
| datafusion | https://github.com/apache/datafusion | query engine | maduro | 8914 (A) | engine sql e dataframe sobre arrow; apache | ok |
| sled | https://github.com/spacejam/sled | embedded db | beta/dormente | 9037 (A) | banco embarcado estilo b-tree; ainda beta e com ritmo reduzido | ok |
| redb | https://github.com/cberner/redb | embedded kv | ativo | 4597 (A) | key-value embarcado em rust puro, no espirito do lmdb | ok |
| databend | https://github.com/databendlabs/databend | data warehouse | ativo | 9356 (A) | data warehouse cloud-native, alternativa ao snowflake | ok |
| greptimedb | https://github.com/GreptimeTeam/greptimedb | time series db | ativo | 6375 (A) | banco de series temporais e observabilidade | ok |
| lancedb | https://github.com/lancedb/lancedb | vector db | ativo | 10716 (A) | banco vetorial multimodal sobre o formato lance; core em rust (o repo detecta html como linguagem dominante) | ok |
| paradedb | https://github.com/paradedb/paradedb | postgres search | ativo | 8962 (A) | extensao de postgres para busca e analytics, alternativa ao elasticsearch | ok |
| neon | https://github.com/neondatabase/neon | postgres serverless | ativo | 22366 (A) | postgres serverless com storage desacoplado, escrito em rust | ok |
| materialize | https://github.com/MaterializeInc/materialize | streaming db | ativo | 6318 (A) | banco de views materializadas incrementais sobre timely e differential dataflow | ok |
| risingwave | https://github.com/risingwavelabs/risingwave | streaming db | ativo | 9108 (A) | banco de stream processing compativel com o protocolo postgres | ok |
| arroyo | https://github.com/ArroyoSystems/arroyo | stream processing | ativo | 4947 (A) | engine de stream processing com sql sobre streams | ok |
| chroma | https://github.com/chroma-core/chroma | vector db | ativo | 28571 (E) | infra de busca para ai; o core foi reescrito em rust | ok |
| rustfs | https://github.com/rustfs/rustfs | object storage | ativo | 29167 (E) | storage de objetos s3-compativel, posicionado como alternativa ao minio | ok |

## bloco 4: terminais, editores e camada grafica

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| alacritty | https://github.com/alacritty/alacritty | terminal | maduro | 64668 (E) | emulador de terminal acelerado por gpu (opengl) | ok |
| wezterm | https://github.com/wezterm/wezterm | terminal | maduro | 26888 (E) | terminal e multiplexer acelerado por gpu | ok |
| zellij | https://github.com/zellij-org/zellij | multiplexer | ativo | 33868 (E) | workspace de terminal, alternativa ao tmux, com plugins em wasm | ok |
| nushell | https://github.com/nushell/nushell | shell | ativo | 39812 (E) | shell estruturado, trata dados tabulares como pipeline | ok |
| helix | https://github.com/helix-editor/helix | editor | ativo | 44976 (E) | editor modal pos-moderno com tree-sitter e lsp nativos | ok |
| zed | https://github.com/zed-industries/zed | editor | ativo | 85907 (E) | editor multiplayer de alta performance com rendering por gpu (gpui); criadores do atom | ok |
| lapce | https://github.com/lapce/lapce | editor | ativo | 38573 (E) | editor rapido com rendering por gpu (floem) | ok |
| yazi | https://github.com/sxyazi/yazi | file manager | ativo | 39748 (E) | gerenciador de arquivos no terminal com i/o assincrono | ok |
| warp | https://github.com/warpdotdev/warp | terminal | ativo | 62335 (E) | ambiente de dev agentico baseado em terminal; o app em si nao e open-source | ok |
| rio | https://github.com/raphamorim/rio | terminal | ativo | 6957 (A) | terminal acelerado por gpu (wgpu); autor brasileiro raphael amorim, relevante para a secao brasil | ok |
| wgpu | https://github.com/gfx-rs/wgpu | gpu/rendering lib | maduro | 17444 (A) | implementacao rust de webgpu; base da engine plev e de varios editores com gpu | ok |
| fish-shell | https://github.com/fish-shell/fish-shell | shell | maduro | 33737 (E) | shell amigavel; a base foi portada de c++ para rust | ok |

## bloco 5: web (frameworks, servidores e frontend)

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| actix-web | https://github.com/actix/actix-web | web framework | maduro | 24695 (E) | framework web de alta performance, historicamente topo dos benchmarks techempower | ok |
| axum | https://github.com/tokio-rs/axum | web framework | maduro | 26346 (E) | framework web ergonomico do time tokio, construido sobre tower | ok |
| Rocket | https://github.com/rwf2/Rocket | web framework | maduro | 25749 (E) | framework web focado em ergonomia e type-safety | ok |
| pingora | https://github.com/cloudflare/pingora | proxy/rede | maduro | 26901 (E) | framework de servicos de rede da cloudflare; substituiu nginx em parte da borda deles | ok |
| tokio | https://github.com/tokio-rs/tokio | async runtime | maduro | 32375 (E) | runtime assincrono de fato do ecossistema rust | ok |
| hyper | https://github.com/hyperium/hyper | http lib | maduro | 16156 (A) | biblioteca http de baixo nivel; base de reqwest e axum | ok |
| tower | https://github.com/tower-rs/tower | middleware lib | maduro | 4217 (A) | abstracao de service e middleware; base de axum e tonic | ok |
| reqwest | https://github.com/seanmonstar/reqwest | http client | maduro | 11690 (A) | cliente http ergonomico, o mais usado do ecossistema | ok |
| warp (web) | https://github.com/seanmonstar/warp | web framework | manutencao | 10344 (A) | framework web por composicao de filtros; homonimo do terminal warp, ver ressalva | ok |
| tide | https://github.com/http-rs/tide | web framework | dormente | 5099 (A) | framework web minimalista sobre async-std; pouca atividade | ok |
| poem | https://github.com/poem-web/poem | web framework | ativo | 4410 (A) | framework web completo com suporte a openapi | ok |
| salvo | https://github.com/salvo-rs/salvo | web framework | ativo | 4378 (A) | framework web simples e flexivel | ok |
| leptos | https://github.com/leptos-rs/leptos | fullstack web/wasm | ativo | 21004 (H) | framework fullstack reativo com reatividade fina, ssr mais wasm | ok |
| yew | https://github.com/yewstack/yew | frontend wasm | maduro | 32699 (E) | framework frontend wasm no estilo react e elm | ok |
| dioxus | https://github.com/DioxusLabs/dioxus | fullstack ui | ativo | 36510 (E) | framework de ui cross-platform para web, desktop e mobile | ok |
| sycamore | https://github.com/sycamore-rs/sycamore | frontend wasm | ativo | 3328 (H) | framework reativo wasm sem virtual dom | ok |
| loco | https://github.com/loco-rs/loco | web framework | ativo | 8970 (H) | framework full-stack no estilo rails, sobre axum | ok |
| shuttle | https://github.com/shuttle-hq/shuttle | plataforma de deploy | ativo | 6922 (H) | plataforma de deploy com infraestrutura declarada no proprio codigo rust | ok |

## bloco 6: seguranca

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| vaultwarden | https://github.com/dani-garcia/vaultwarden | password manager server | maduro | 62850 (E) | servidor compativel com bitwarden, leve, em rust | ok |
| rustls | https://github.com/rustls/rustls | tls lib | maduro | 7473 (H) | biblioteca tls moderna em rust, alternativa de memoria segura ao openssl | ok |
| boringtun | https://github.com/cloudflare/boringtun | vpn/wireguard | maduro | 7097 (H) | implementacao wireguard da cloudflare, usada no warp deles | ok |
| rage | https://github.com/str4d/rage | criptografia de arquivos | maduro | 3539 (H) | implementacao rust do formato age de criptografia | ok |
| ring | https://github.com/briansmith/ring | primitivas crypto | maduro | 4093 (H) | primitivas criptograficas; base do rustls | ok |
| sudo-rs | https://github.com/trifectatechfoundation/sudo-rs | sistema/privilegio | ativo | 4412 (H) | reescrita de sudo e su em rust; em adocao planejada por ubuntu e debian | ok |
| quiche | https://github.com/cloudflare/quiche | quic/http3 | maduro | 11594 (H) | implementacao de quic e http/3 da cloudflare | ok |
| RustScan | https://github.com/bee-san/RustScan | scanner de portas | ativo | 19987 (H) | scanner de portas rapido; repo renomeado para bee-san | ok (redirect) |
| feroxbuster | https://github.com/epi052/feroxbuster | recon web | ativo | 7875 (H) | brute-force de diretorios e arquivos em alvos web | ok |

## bloco 7: ai e ml

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| candle | https://github.com/huggingface/candle | ml framework | ativo | 20553 (H) | framework de ml minimalista da huggingface, foco em inferencia e treino enxuto | ok |
| burn | https://github.com/tracel-ai/burn | ml framework | ativo | 15482 (H) | framework de deep learning em rust com varios backends (wgpu, cuda, ndarray) | ok |
| tokenizers | https://github.com/huggingface/tokenizers | nlp | maduro | 10843 (H) | tokenizers rapidos da huggingface; core rust com bindings python | ok |
| mistral.rs | https://github.com/EricLBuehler/mistral.rs | inferencia de llm | ativo | 7364 (H) | engine de inferencia de llms rapida em rust | ok |
| llm (rustformers) | https://github.com/rustformers/llm | inferencia de llm | arquivado | 6153 (H) | inferencia de llms sobre ggml em rust; descontinuado | ok |
| ort | https://github.com/pykeio/ort | onnx runtime | ativo | 2359 (H) | wrapper rust para o onnx runtime | ok |
| tch-rs | https://github.com/LaurentMazare/tch-rs | libtorch bindings | maduro | 5433 (H) | bindings rust para libtorch, o backend do pytorch | ok |
| linfa | https://github.com/rust-ml/linfa | ml classico | ativo | 4686 (H) | toolkit de ml classico no estilo scikit-learn | ok |
| dfdx | https://github.com/chelsea0x3b/dfdx | deep learning | dormente | 1911 (H) | deep learning com shapes no sistema de tipos; repo transferido e com manutencao reduzida | ok (redirect) |
| kornia-rs | https://github.com/kornia/kornia-rs | computer vision | ativo | 665 (H) | visao computacional 3d em rust | ok |
| luminal | https://github.com/luminal-ai/luminal | ml compiler | ativo | 2868 (H) | framework de deep learning compilado; repo renomeado para luminal-ai | ok (redirect) |
| text-generation-inference | https://github.com/huggingface/text-generation-inference | serving de llm | ativo | 10863 (H) | servidor de inferencia de llms da huggingface; o router e em rust | ok |
| safetensors | https://github.com/safetensors/safetensors | formato de ml | maduro | 3788 (H) | formato seguro de serializacao de tensores; repo renomeado de huggingface | ok (redirect) |
| tabby | https://github.com/TabbyML/tabby | assistente de codigo | ativo | 33651 (E) | assistente de codigo self-hosted | ok |

## bloco 8: embedded e sistemas

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| embassy | https://github.com/embassy-rs/embassy | embedded async | ativo | 9442 (H) | framework async para microcontroladores | ok |
| redox | https://github.com/redox-os/redox | sistema operacional | ativo | 16394 (H) | so unix-like escrito em rust, com microkernel | ok |
| firecracker | https://github.com/firecracker-microvm/firecracker | microvm | maduro | 35114 (E) | microvms para serverless; roda por baixo de aws lambda e fargate | ok |
| rtic | https://github.com/rtic-rs/rtic | rtos embarcado | ativo | 2356 (H) | framework de concorrencia em tempo real para embarcados | ok |
| probe-rs | https://github.com/probe-rs/probe-rs | debug embarcado | ativo | 2800 (H) | toolkit de debug e flash para microcontroladores | ok |
| smoltcp | https://github.com/smoltcp-rs/smoltcp | stack tcp/ip | maduro | 4500 (H) | stack tcp/ip standalone para embarcados e no_std | ok |
| defmt | https://github.com/knurling-rs/defmt | logging embarcado | ativo | 1187 (H) | logging eficiente para embarcados; projeto knurling | ok |
| embedded-hal | https://github.com/rust-embedded/embedded-hal | abstracao embarcada | maduro | 2600 (H) | traits de abstracao de hardware; base do ecossistema embarcado rust | ok |
| youki | https://github.com/youki-dev/youki | container runtime | ativo | 7463 (H) | runtime de containers oci em rust; repo renomeado para youki-dev | ok (redirect) |
| cloud-hypervisor | https://github.com/cloud-hypervisor/cloud-hypervisor | vmm | maduro | 5835 (H) | monitor de maquina virtual para cloud sobre linux e kvm | ok |
| hubris | https://github.com/oxidecomputer/hubris | so embarcado | ativo | 3548 (H) | so para sistemas embarcados criticos; oxide computer | ok |
| tock | https://github.com/tock/tock | so embarcado | maduro | 6361 (H) | so embarcado com isolamento para microcontroladores | ok |

## bloco 9: apps de usuario final, gui e criativo

| nome | url | subdominio | maturidade | stars (fonte) | por que relevante | validacao |
|------|-----|------------|------------|---------------|-------------------|-----------|
| rustdesk | https://github.com/rustdesk/rustdesk | desktop remoto | maduro | 116873 (E) | alternativa open-source ao teamviewer; um dos maiores apps de usuario em rust | ok |
| typst | https://github.com/typst/typst | typesetting | ativo | 54543 (E) | sistema de composicao tipografica, alternativa moderna ao latex | ok |
| bevy | https://github.com/bevyengine/bevy | game engine | ativo | 46835 (E) | engine de jogos data-driven baseada em ecs | ok |
| servo | https://github.com/servo/servo | browser engine | ativo | 37184 (E) | engine web embeddable de alta performance | ok |
| spacedrive | https://github.com/spacedriveapp/spacedrive | file explorer | ativo | 38405 (E) | explorador de arquivos cross-platform sobre um vdfs; usa tauri e rspc | ok |
| Graphite | https://github.com/GraphiteEditor/Graphite | editor grafico | ativo | 26389 (E) | editor 2d procedural node-based para design e arte | ok |
| anki | https://github.com/ankitects/anki | educacao/flashcards | maduro | 28759 (E) | flashcards de repeticao espacada; o core foi migrado para rust | ok |
| sniffnet | https://github.com/GyulyVGC/sniffnet | rede/monitor | ativo | 39582 (E) | monitor de trafego de internet com gui em iced | ok |
| iced | https://github.com/iced-rs/iced | gui toolkit | ativo | 30825 (E) | toolkit gui cross-platform inspirado em elm | ok |
| egui | https://github.com/emilk/egui | gui toolkit | ativo | 29495 (E) | gui immediate-mode que roda em web e nativo | ok |
| slint | https://github.com/slint-ui/slint | gui toolkit | ativo | 23000 (E) | toolkit declarativo de ui nativa para rust, c++, js e python | ok |
| nautilus_trader | https://github.com/nautechsystems/nautilus_trader | trading | ativo | 24192 (E) | engine de trading algoritmico event-driven, core rust | ok |
| tree-sitter | https://github.com/tree-sitter/tree-sitter | parsing | maduro | 25991 (E) | sistema de parsing incremental usado por dezenas de editores | ok |
| niri | https://github.com/niri-wm/niri | compositor wayland | ativo | 25517 (E) | compositor wayland scrollable-tiling | ok |
| hyperswitch | https://github.com/juspay/hyperswitch | pagamentos | ativo | 43098 (E) | plataforma de pagamentos composavel e pci-compliant; juspay | ok |
| Pake | https://github.com/tw93/Pake | empacotador web | ativo | 57384 (E) | transforma uma webpage em app desktop com um comando, sobre tauri | ok |
| Handy | https://github.com/cjpais/Handy | speech-to-text | ativo | 24844 (E) | app de transcricao offline, extensivel | ok |
| SpacetimeDB | https://github.com/clockworklabs/SpacetimeDB | db/backend | ativo | 24741 (E) | banco que roda a logica da aplicacao dentro dele; mira jogos multiplayer | ok |

## ressalva sobre o ranking e a contagem de stars

a evanli Github-Ranking foi a fonte de partida para o top de stars, e os numeros
foram cruzados contra a api do github (batem, com deriva pequena de horas). duas
coisas precisam ficar explicitas, porque o livro nao pode tratar star como
verdade limpa.

primeiro, a divergencia de linguagem. o ranking lista o bun (oven-sh/bun) como
projeto rust. o core do bun e zig, nao rust; ele aparece no ranking de rust por
heuristica do github sobre os arquivos do repo. mantive o item por relevancia de
runtime, mas marcado. o mesmo cuidado vale para o lancedb, cujo repo o github
classifica como html embora o core seja rust.

segundo, e mais serio, o topo do ranking de 2026 esta poluido por repos com
sinais de inflacao de stars. o caso mais gritante e ultraworkers/claw-code, no
primeiro lugar com 194297 stars e 109900 forks pela api do github em 2026-06-25,
criado em 2026-03-31. um repo com menos de tres meses, com numero de forks
equivalente a mais da metade do numero de stars (o rust-lang/rust, para
comparar, tem 15010 forks para 114170 stars), descrito como "exhibit de museu
gerenciado por agente, sem intervencao humana". o numero e real no sentido de que
o github o reporta, mas a legitimidade do crescimento nao se confirma. nao
descrevi esse e outros casos parecidos como apps do panorama. ficam listados
abaixo como nao confirmados.

repos de alto ranking que ficaram de fora por legitimidade ou topicalidade nao
confirmada (numeros conforme evanli e api, 2026-06-25):

- ultraworkers/claw-code, 194297 stars, 109900 forks: ratio fork/star anomalo,
  idade menor que tres meses. nao confirmado.
- ruvnet/RuView, 75402 stars: projeto sem rastro proporcional ao ranking, acima
  do ripgrep. nao confirmado.
- rtk-ai/rtk, 65823 stars: cli proxy desconhecido com star de top 15. nao
  confirmado.
- farion1231/cc-switch, 108442 stars (api): switcher de config de agentes,
  acima do deno; ratio de fork normal, crescimento pode ser real no boom de
  ferramentas de coding agent, mas nao verificavel de forma independente. nao
  confirmado.
- openinterpreter/openinterpreter, 64127 stars, marcado rust: existe um projeto
  famoso e homonimo em python (KillianLucas/open-interpreter); a relacao entre os
  dois e a natureza deste repo nao se confirmam. nao confirmado.
- Hmbown/CodeWhale, tinyhumansai/openhuman, zeroclaw-labs/zeroclaw,
  AlexsJones/llmfit, lbjlaq/Antigravity-Manager, googleworkspace/cli (marcado
  rust), xai-org/x-algorithm, vercel-labs/agent-browser, BloopAI/vibe-kanban:
  cluster de ferramentas de agente de 2026, alto ranking, fora do escopo de app
  estabelecido e sem verificacao independente de organicidade. nao confirmados
  para esta leva.

uma observacao honesta para o capitulo de panorama: o proprio ranking de stars
de rust em 2026 virou um artefato a ser lido com ceticismo, porque a onda de
repos de coding agent e os incentivos de star-farming deformaram o topo. isso e
material, nao ruido. cabe uma nota no livro sobre por que star nao mede
maturidade nem adocao.

## proxima rodada (nada cortado em silencio)

esta primeira leva cobre 157 itens. ha mais candidatos ja identificados e
deliberadamente adiados, agrupados por motivo:

- cluster blockchain e web3 do top 100, legitimo mas de um subdominio especifico:
  FuelLabs/sway, FuelLabs/fuel-core, FuelLabs/fuels-rs, unionlabs/union,
  linera-io/linera-protocol, zama-ai/fhevm. seis itens.
- recursos educacionais e listas que aparecem no top 100 mas nao sao apps:
  rust-lang/rustlings, google/comprehensive-rust, sunface/rust-course,
  TheAlgorithms/Rust, rust-unofficial/awesome-rust. cinco itens.
- cluster de coding agents nao confirmados acima: por volta de doze itens (ver
  ressalva), a revisitar quando houver como verificar adocao real.
- mais devtools e cli para a proxima leva: bacon, cargo-nextest, cargo-make, lsd,
  dog, gping, grex, hexyl, oha, miniserve, navi, choose, hck, frawk, huniq,
  ripsecrets. cerca de quinze itens.
- mais libs de web e rede: tonic (grpc), async-std, smol, salvo coberto, actix
  core, sea-orm, sqlx, diesel. cerca de oito itens.
- mais infra e db: clickhouse (core c++, descartavel aqui), fjall, indradb, kuzu,
  cozodb, nativelink, sccache coberto. cerca de cinco itens validos.
- mais ai/ml e tui: rten, ratchet, ratatui (lib de tui), crossterm, cursive.
  cerca de cinco itens.

soma de adiados declarados: aproximadamente 56 candidatos, sem corte silencioso.
a proxima rodada valida link e star de cada um com o mesmo metodo desta.

## metodologia e validacao

- inventario inicial montado a partir das fontes do briefing (evanli Top 100,
  awesome-rust, ImplFerris/rust-in-production) e dos blocos pedidos.
- deduplicacao: a pesquisa colada repetia itens entre blocos (ui libs, devtools).
  cada app aparece uma vez, no bloco onde e mais representativo.
- validacao de link: curl seguindo redirect, capturando http status e url final.
  157 de 157 resolveram com http 200. oito apresentaram redirect por renomeacao
  ou transferencia de repo, e a url canonica atual ja esta na tabela com a marca
  redirect: gitui, wasm-bindgen, wasm-pack, RustScan, dfdx, luminal, safetensors,
  youki.
- contagem de stars: nunca estimada. top 100 vem do snapshot evanli; o restante
  vem da api do github (campo stargazers_count) ou, quando a api bateu no rate
  limit de 60 por hora sem autenticacao, do contador no html da pagina do repo
  (aria-label). as tres fontes foram conferidas entre si nos casos de overlap e
  bateram.
- lib.rs ficou registrada como fonte conhecida mas retornou 403 a curl por
  anti-bot, entao nao serviu de fonte de numero nesta rodada.
