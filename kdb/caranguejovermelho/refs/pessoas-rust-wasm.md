---
title: pessoas relevantes em rust e wasm (mapa de referencia)
date: 2026-06-25
tags: [refs, pessoas, rust, wasm, ecossistema, caranguejo-vermelho]
fontes:
  - https://www.rust-lang.org/governance/teams
  - https://github.com/rustwasm/team
  - https://github.com/rust-unofficial/awesome-rust
  - https://rustlang.com.br/
  - https://github.com/rust-br
  - perfis individuais do github, um por linha, na coluna github de cada pessoa
status: >
  validado parcial. handle do github conferido abrindo o perfil em 2026-06-25.
  dado nao exibido no perfil fica marcado, nunca preenchido por suposicao.
  itens "nao confirmado" listados na ultima secao.
---

# pessoas relevantes em rust e wasm

este arquivo mapeia quem sustenta o ecossistema rust e wasm que o livro precisa
creditar. cada linha foi conferida abrindo o perfil no github em 25 de junho de
2026. quando o perfil nao mostrou um dado, nome real, local ou site, o campo
fica marcado, nunca preenchido por suposicao.

o mapa esta em clusters que se sobrepoem. o nucleo da linguagem e das
proc-macros: graydon hoare criou o rust, niko matsakis desenhou boa parte do
sistema de tipos e do borrow checker, david tolnay mantem o serde e o syn que
quase todo crate usa por baixo. a toolchain wasm: alex crichton e nick
fitzgerald no wasm-bindgen e no wasmtime, luke wagner que ajudou a desenhar o
proprio webassembly e o asm.js antes dele, lin clark na wasi, daxpedda mantendo
o wasm-bindgen hoje. os frameworks de ui web que compilam para wasm: leptos do
greg johnston, dioxus do jonathan kelley, yew do denis kolodin. as engines
gpu-first, que e o territorio do plev: egui, iced, slint, bevy, makepad, zed
com o gpui, e o linebender com vello e xilem. e o cluster de visualizacao em
wasm que o capitulo de viz precisa: plotters, charming, plotly.rs, egui_graphs,
fdg.

duas coisas que nao viram afirmacao limpa.

o corpus presumiu que jorge aparicio, o japaric, e brasileiro. o perfil dele
informa local na alemanha e emprego na ferrous systems, e o nome completo nas
fontes e jorge aparicio rivera. a pesquisa nao confirmou a nacionalidade, e nao
achei evidencia de que ele seja do brasil. ele aparece na secao brasil porque o
corpus pediu, com a ressalva no lugar: a premissa de brasileiro nao se sustenta
no que foi achado.

os numeros de estrelas que cada perfil mostra sao foto do momento da consulta.
por isso ficam fora da tabela. quando uma metrica importar para o texto, ela
entra marcada com a data e a ordem de reconferir antes da publicacao. o tauri,
no perfil do lucas nogueira, apareceu com leitura de seis digitos que precisa
ser reconferida antes de virar frase.

## nucleo da linguagem e proc-macros

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Graydon Hoare | Canada (Vancouver) | criador original do rust | rust (design inicial), monotone | [graydon](https://github.com/graydon) | - | - |
| Niko Matsakis | EUA (Boston) | design de tipos e borrow checker, lang team | rust, moro, rustacean-principles | [nikomatsakis](https://github.com/nikomatsakis) | smallcultfollowing.com/babysteps | bsky @nikomatsakis.com |
| David Tolnay | EUA (Redwood City, CA) | proc-macros que o ecossistema inteiro usa | serde, syn, thiserror, anyhow, cxx | [dtolnay](https://github.com/dtolnay) | - | x @davidtolnay |
| withoutboats | Alemanha (Berlim, residencia) | design de async/await em rust | fehler, ringbahn, iou | [withoutboats](https://github.com/withoutboats) | without.boats | - |
| Yehuda Katz | EUA (Portland, OR) | envolvido no rust inicial e no design do cargo (documentado; perfil atual mostra so projetos js) | ember.js, handlebars, cargo (historico) | [wycats](https://github.com/wycats) | yehudakatz.com | - |

## toolchain wasm (bindgen, runtimes, wasi)

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Alex Crichton | nao informado no perfil | core historico do rust, wasm e cargo | wasm-bindgen, wasm-pack, wasmtime, cargo | [alexcrichton](https://github.com/alexcrichton) | - | - |
| Nick Fitzgerald | nao informado (fuso UTC-7) | wasm tooling e runtimes | wasmtime, wasm-tools, bumpalo, cargo-fuzz | [fitzgen](https://github.com/fitzgen) | fitzgen.com | - |
| Luke Wagner | EUA | co-desenho do webassembly e do asm.js, component model | webassembly, asm.js, wasi/component model | [lukewagner](https://github.com/lukewagner) | lukewagner.name | - |
| Lin Clark | EUA (Pittsburgh, PA) | divulgacao e wasi, code cartoons | wasi, code-cartoons | [linclark](https://github.com/linclark) | code-cartoons.com | - |
| Till Schneidereit | Alemanha | bytecode alliance, wasi, wasmtime | wasmtime, wasi, wasi-sdk | [tschneidereit](https://github.com/tschneidereit) | - | - |
| Syrus Akbary | Espanha (mora em San Francisco, EUA) | ceo da wasmer, runtime wasm | wasmer, graphene | [syrusakbary](https://github.com/syrusakbary) | syrusakbary.com | x @syrusakbary |
| daxpedda | nao informado (nome real nao exibido) | mantenedor atual do wasm-bindgen e web-sys | wasm-bindgen, web-sys | [daxpedda](https://github.com/daxpedda) | - | - |
| Ingvar Stepanyan | Ucrania -> Reino Unido | consultor wasm, serde-wasm-bindgen | serde-wasm-bindgen, wasm-bindgen-rayon, wasmbin | [RReverser](https://github.com/RReverser) | rreverser.com | - |
| Guy Bedford | Canada (Vancouver) | es modules e wasm na cloudflare | jspm, es-module-shims, es-module-lexer | [guybedford](https://github.com/guybedford) | guybedford.com | - |

## libs de sistema, cli e rede

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Andrew Gallant | EUA (Marlborough, MA) | regex e busca de alto desempenho | ripgrep, regex, jiff | [BurntSushi](https://github.com/BurntSushi) | burntsushi.net | - |
| Sean McArthur | nao informado no perfil | stack http em rust | hyper, reqwest, warp, tower | [seanmonstar](https://github.com/seanmonstar) | seanmonstar.com | masto @seanmonstar@masto.ai, bsky @seanmonstar.com |
| Carl Lerche | EUA (Portland, OR) | runtime async tokio | tokio, mio, bytes | [carllerche](https://github.com/carllerche) | - | x @carllerche |
| Stjepan Glavina | nao informado (conta sem repos publicos hoje) | autor de smol, async-std, crossbeam (historico) | smol, async-std, crossbeam | [stjepang](https://github.com/stjepang) | - | - |
| Kevin K. | EUA (DC) | parser de cli padrao do ecossistema | clap, cargo-outdated | [kbknapp](https://github.com/kbknapp) | k8p.me | - |
| Pascal Hertleif | Suecia (Gotemburgo, origem alema) | cli e ferramentas de cargo | cargo-edit, quicli, rustfix | [killercup](https://github.com/killercup) | pascalhertleif.de | - |

## educadores e divulgacao

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Steve Klabnik | EUA (Austin, TX) | co-autor do the rust programming language | rust book, jujutsu-tutorial | [steveklabnik](https://github.com/steveklabnik) | steveklabnik.com | - |
| Carol Nichols | EUA (Pittsburgh, PA) | co-autora do the rust programming language | rust book, rustlings | [carols10cents](https://github.com/carols10cents) | carol-nichols.com | - |
| Ashley Williams | nao informado no perfil | ex-core do rust, wasm-pack, rustbridge | wasm-pack, wasm-bindgen, cargo-generate | [ashleygwilliams](https://github.com/ashleygwilliams) | - | - |
| Jon Gjengset | Noruega (Oslo) | educador rust, streamer, livro rust for rustaceans | rust-for-rustaceans, inferno, left-right | [jonhoo](https://github.com/jonhoo) | thesquareplanet.com | - |
| Luca Palmieri | Italia (Roma) | autor de zero to production, pavex | zero-to-production, pavex, cargo-chef | [LukeMathWalker](https://github.com/LukeMathWalker) | lpalmieri.com | - |
| Amos Wenger | Franca (Lyon) | educador, fasterthanli.me, facet | facet, rc-zip | [fasterthanlime](https://github.com/fasterthanlime) | fasterthanli.me | - |

## frameworks de ui web (rust -> wasm)

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Greg Johnston | nao informado no perfil | criador do leptos, signals que inspiraram o signal.rs do plev | leptos, custom-elements | [gbj](https://github.com/gbj) | - | - |
| Jonathan Kelley | EUA (San Francisco) | criador do dioxus | dioxus, blitz, taffy | [jkelleyrtp](https://github.com/jkelleyrtp) | jonathan-kelley.com | - |
| Evan Almloff | EUA (Kansas) | mantenedor do dioxus, ml e gui | dioxus, kalosm, sledgehammer-bindgen | [ealmloff](https://github.com/ealmloff) | evanalmloff.com | x @demonthos |
| Denis Kolodin | Montenegro | inventor do yew (conta ativa em therustmonk; DenisKolodin redireciona) | yew | [therustmonk](https://github.com/therustmonk) | knowledge.dev | - |
| Anthony Dodd | EUA (San Antonio, TX) | criador do trunk, empacotador wasm | trunk, async-raft | [thedodd](https://github.com/thedodd) | - | - |

## engines e rendering gpu-first (territorio do plev)

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Emil Ernerfeldt | Suecia (Estocolmo) | criador do egui, cto da rerun | egui, rerun, egui_plot | [emilk](https://github.com/emilk) | ilikebigbits.com | - |
| Hector Ramon | Espanha (Barcelona) | criador do iced | iced, coffee, wgpu_glyph | [hecrj](https://github.com/hecrj) | - | - |
| Olivier Goffart | Alemanha (Berlim) | cofundador do slint | slint, qmetaobject-rs, rust-cpp | [ogoffart](https://github.com/ogoffart) | slint.dev | - |
| Carter Anderson | nao informado (ex-microsoft) | criador da bevy engine | bevy | [cart](https://github.com/cart) | - | bsky @cart.work |
| Alice Cecile | Canada (Vancouver Island, BC) | mantenedora-lider da bevy, bevy foundation | bevy | [alice-i-cecile](https://github.com/alice-i-cecile) | bevy.org | - |
| Rik Arends | nao informado (projeto na org makepad) | criador do makepad, ui gpu-first, dsl live | makepad | [rikarends](https://github.com/rikarends) | - | x @rikarends |
| Nathan Sobo | EUA (Boulder, CO) | cofundador da zed, criador do gpui (documentado; pinned mostra atom/xray) | zed, gpui (atom, xray no historico) | [nathansobo](https://github.com/nathansobo) | - | - |
| Raph Levien | nao informado no perfil | fundador do linebender, compute shaders | vello, xilem, kurbo, peniko, font-rs | [raphlinus](https://github.com/raphlinus) | levien.com | - |
| Daniel McNab | Australia (Sydney) | mantenedor do xilem, linebender | xilem, vello, bevy | [DJMcNab](https://github.com/DJMcNab) | - | - |
| Patrick Walton | EUA (San Francisco) | servo, pathfinder, rendering da bevy (documentado) | servo, pathfinder, font-kit, offset-allocator | [pcwalton](https://github.com/pcwalton) | pcwalton.github.io | - |
| Nico Burns | Reino Unido (Londres) | taffy e blitz, layout css em rust | taffy, blitz, parley, stylo | [nicoburns](https://github.com/nicoburns) | nicoburns.com | - |
| Jeremy Soller | EUA (Colorado) | cosmic-text, pop_os, bdfl do redox | cosmic-text, redox, orbtk | [jackpot51](https://github.com/jackpot51) | soller.dev | - |
| Matt Campbell | EUA (Wichita, KS) | criador do accesskit, infra de acessibilidade (lead pelo AUTHORS do repo) | accesskit | [mwcampbell](https://github.com/mwcampbell) | - | - |

## js e web tooling adjacente

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Tobias Koppers | Alemanha | autor do webpack, hoje no turbopack/next na vercel | webpack, turbopack, next.js | [sokra](https://github.com/sokra) | - | - |
| Surma | Alemanha / Reino Unido (vive em Bristol) | web platform, comlink, squoosh (nome real nao exibido) | comlink, squoosh, proxx | [surma](https://github.com/surma) | surma.dev | - |

## visualizacao e grafos em wasm

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Hao Hou | EUA (Salt Lake City, univ. utah) | criador do plotters, plotagem wasm e nativa | plotters, d4-format | [38](https://github.com/38) | - | x @haohou302 |
| Yuankun Zhang | Singapura | criador do charming, viz em rust | charming, go-echarts | [yuankunzhang](https://github.com/yuankunzhang) | yuankun.me | - |
| Ioannis Giagkiozis | Reino Unido | plotly.rs (atribuicao documentada; nao no pinned na consulta) | plotly.rs | [igiagkiozis](https://github.com/igiagkiozis) | - | - |
| blitzarx1 | Chipre | criador do egui_graphs (nome real nao exibido) | egui_graphs | [blitzarx1](https://github.com/blitzarx1) | - | linkedin in/blitzarx1 |
| Grant Handy | EUA (Salt Lake City, UT) | criador do fdg, force-directed graph | fdg, claui, egui-themer | [grantshandy](https://github.com/grantshandy) | grantshandy.github.io | - |

## brasil

a secao brasil real, alem do japaric com a ressalva acima, tem gente que de
fato programa em rust e e do brasil. lucas fernandes nogueira cofundou o tauri e
o perfil dele informa brasil. raphael amorim escreve o rio, um terminal
acelerado por gpu sobre wgpu, e produziu material em portugues sobre wasm. mario
idival, de campina grande na paraiba, e ativo na rust-br. bruno cesar rocha
nasceu no brasil, hoje trabalha na red hat de portugal, e mantem o marmite, um
gerador de site estatico em rust. a comunidade se organiza na rust-br no github
e no telegram da rust brasil, que passa de dois mil membros segundo a propria
comunidade em rustlang.com.br.

| nome | pais | papel | projetos | github | site | redes |
|------|------|-------|----------|--------|------|-------|
| Lucas Fernandes Nogueira | Brasil | cofundador do tauri | tauri, wry, tauri-action | [lucasfernog](https://github.com/lucasfernog) | - | - |
| Raphael Amorim | Brasil (perfil nao exibe local; documentado) | criador do rio terminal e do sugarloaf (wgpu), material wasm em pt | rio, sugarloaf | [raphamorim](https://github.com/raphamorim) | rapha.land | - |
| Mario Idival | Brasil (Campina Grande, PB) | comunidade rust-br, projetos em rust | limit, rust-by-example (contrib), rust-br | [marioidival](https://github.com/marioidival) | - | x @marioidival |
| Bruno Cesar Rocha | Brasil (vive em Portugal, red hat) | autor do marmite em rust, py2rs | marmite, py2rs, dynaconf | [rochacbruno](https://github.com/rochacbruno) | bruno.rocha.social | - |
| Jorge Aparicio (japaric) | nao confirmado (perfil: Alemanha, ferrous systems) | rust embedido, rtic/rtfm, cortex-m. ressalva: nao confirmado brasileiro | cortex-m, embedded-hal, rtic | [japaric](https://github.com/japaric) | blog.japaric.io | x @japaricious |

### comunidade e recursos brasil

- rust-br: organizacao da comunidade no github, traducao do rust book para
  portugues, e o repo eu-uso-rust com empresas e projetos brasileiros em rust.
  membros visiveis incluem dlight, rochacbruno, marioidival, gugahoa.
  https://github.com/rust-br
- rust brasil: site e telegram da comunidade, citado com mais de dois mil
  membros pela propria comunidade. https://rustlang.com.br/

## ressalvas e itens nao confirmados

- jorge aparicio (japaric): nacionalidade nao confirmada. perfil informa
  alemanha e ferrous systems; nome completo nas fontes e jorge aparicio rivera.
  nao ha evidencia, no que foi achado, de que seja brasileiro. a premissa do
  corpus ("alem do japaric", tratando-o como brasileiro) nao se sustenta.
- daxpedda, blitzarx1, withoutboats: handle do github validado, nome real nao
  exibido no perfil. nao preenchido.
- surma: usa so "surma" profissionalmente; nome completo nao confirmado.
- stjepan glavina (stjepang): a conta resolve mas nao tem repos publicos na
  consulta. atribuicao de smol, async-std e crossbeam e historica e documentada,
  nao verificavel pelo perfil atual.
- rik arends: handle pessoal rikarends existe mas e minimo; o makepad vive na
  org github.com/makepad. handle social x @rikarends.
- denis kolodin: a conta DenisKolodin redireciona para a ativa therustmonk.
- atribuicoes documentadas, nao exibidas no pinned na consulta: plotly.rs
  (igiagkiozis), accesskit como lead individual (mwcampbell, via AUTHORS do
  repo), vello/xilem/kurbo (raphlinus, via org linebender, confirmado tambem
  pelo pinned de DJMcNab), zed/gpui (nathansobo), servo/pathfinder/font-kit
  (pcwalton).
- numeros de estrelas e downloads: nao entram na tabela. quando lidos nos
  perfis, sao foto de 2026-06-25 e devem ser reconferidos na publicacao. o
  tauri apareceu com leitura de seis digitos no perfil do lucas nogueira que
  precisa de reconferencia.
- locais marcados "nao informado no perfil" nao foram preenchidos por
  suposicao, mesmo quando a residencia e amplamente conhecida.
</content>
</invoke>
