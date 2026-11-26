#  plev - real time UI rendering

Devs, agents, and designers playing on the same canvas.

Like or not, acredito que interfaces estáticas estão com os dias contados por causa dos llms, estamos atravesando (again) uma mudança brusca na maneira como construimos interfaces para humanos, tenho chamado esse novo ciclo de real time UI rendering.

O PLEV é uma engine de compositing GPU-first voltado a UI generativa cross devices priorizando perfomance e controle de memória. Um único código renderiza o
mesmo frame no macOS (Metal), no Linux (Vulkan), no navegador (WebGPU), no
iOS (Metal) e no Android (Vulkan).


# Short story
Comecei esse projeto há uns seis anos, com intuito de ensinar rust para meus filhos, desde que os LLMs começaram a fazer o meu trabalho, tenho tratado essa engine como meu laboratório de aprendizado e experimentos de gui e gpu. 

** Atualmente estou lapidando o que estou nomeando temporariamente como `.monster`., ** Um codec binário próprio de animação inspirado no falecido flash e action script (btw, nunca superei a morte do SWF, fuck you, apple), estou aplicando conceitualmente a mesma ideia do H.264 aplicada a vetores: keyframes são snapshots completos
da cena (seek em O(1)), interframes são segmentos por propriedade (valor,
easing em 1 byte, duração) que o player interpola, independentes de fps e
scrubáveis. A cena é um mapa plano `depth → node` com bit de presença por
campo, então o que não mudou custa zero bytes. Valores quantizados no
arquivo (twips, rgba8, u16 fixo), f32 na memória; seções com sha256;
decoder estrito pra evitar não bugar com panic. O encoder tem dois modos: lower
de uma timeline autoral, ou descoberta de deltas a partir de frames
amostrados, sem ninguém declarar nada. É esse segundo modo que permite ao
`record` gravar uma sessão de uso como dado, nunca como vídeo: a trilha
visual e a trilha semântica (quem visualizou, quem interagiu, quem codoi, sobre qual estado, com
que resultado) no mesmo arquivo. Um bug reproduzido vira um `.monster`, e o
`.monster` vira um teste no gate. Contrui uma engine de converte o lottie para o formato (very experimental).

O plev não compete com o [Flutter](https://flutter.dev), o
[egui](https://github.com/emilk/egui), o [Dioxus](https://dioxuslabs.com) ou
o [iced](https://iced.rs). Ele existe porque eu precisava de uma engine com
controle real de memória, performance em primeiro lugar, tudo na GPU e um só
código para qualquer dispositivo, e nenhuma delas era isso. Alguns projetos
eu estudei mais do que usei: o [Makepad](https://makepad.dev) e o
[gpui](https://www.gpui.rs) do Zed estão entre os trabalhos de engenharia de
mais alto nível que vi nos últimos anos, e o [Bevy](https://bevyengine.org)
me ensinou como um projeto em Rust pode ser ambicioso e acolhedor ao mesmo
tempo. O plev deve a eles boa parte do que há de bom aqui.



**O sonho de um "universal importers** A ideia é que interfaces de outros frameworks entram por um transpiler em três
estágios: parse (tree-sitter para React TSX + Sass, HTML + CSS e gpui; o
JSON da REST para Figma), resolve para uma árvore comum com cores mapeadas
nos tokens do tema dentro de uma tolerância medida, e dois emissores sobre
essa árvore: código Rust contra o builder, ou plev IR com um relatório que
põe cada construção da origem em `supported`, `transformed`, `approximated`
ou `impossible`, com `arquivo:linha`. por enquanto, quase nada é descartado, e o
importer não declara capability nenhuma: um botão sai como `binding
required`, e é o verificador que decide se a tela pode existir. SVG entra
como path tesselado. O escopo é pequeno de propósito: um componente de cada
origem, provado end to end, antes de prometer o formato inteiro.

**Playground.** Demos que provaram uma capacidade cada (grafos, câmera,
gestures, mobile, Lottie, um arcade), um cliente git (`idestudio`) que serve de
app real para o contrato semântico, um inspector de sessões, e inferência
local (`llmrun`, candle e burn no device do renderer) para o que o plev
precisa fazer sem sair do processo.

## Como rodar

Você precisa do [rustup](https://rustup.rs) (ele lê o `rust-toolchain.toml`
e instala o Rust certo). Para o navegador, precisa também do trunk
(`cargo install trunk`) e do Chrome ou Edge 113+. Rode todos os comandos a
partir da raiz do repositório; a primeira build leva alguns minutos.

### No desktop

A gallery do design system, cada componente na sua própria página:

```
cargo run -p uigallery
```

Playground demos:

```
cargo run -p arcade
cargo run -p camera
cargo run -p inputs
cargo run -p layers
cargo run -p lottie
cargo run -p mobile
cargo run -p primes
cargo run -p shapes
cargo run -p ticker
```

Algumas demos de grafos:

```
cargo run -p graphs -- lesmis
cargo run -p graphs -- compound
cargo run -p graphs -- elk
cargo run -p graphs -- elk-groups
```

```
cargo run -p commit
cargo run -p lottie -- data/media/lottie/buttontap/animation.json
cargo run -p player -- <arquivo.monster>
cargo run -p intent -- --app todo "show only the active items"
```

O cliente git abre no repositório atual, ou no caminho que você passar:

```
cargo run -p idestudio
cargo run -p idestudio -- ../urna
```

### O dev mode

A gallery tem um dev mode do próprio engine, sem dependência externa: o que
cada frame fez, medido pelo engine e mostrado num painel dentro da janela.
Ele só existe quando você pede:

```
PLEV_DEV=1 cargo run -p uigallery        # a engrenagem no canto inferior direito
PLEV_DEV=gpu cargo run -p uigallery      # o painel já aberto numa aba
```

Clique na engrenagem, ou aperte F12, para abrir e fechar. No navegador o
pedido vai no endereço: `?dev=1` ou `?dev=gpu`. Sem o pedido nada é
desenhado nem medido.

| Aba | O que mostra |
|---|---|
| Frame | para onde foi o tempo do frame, fase por fase, e os **insights**: o que passou do orçamento de 16,6 ms, o que foi redesenhado à toa, o que o heap e o log dizem |
| GPU | cada render pass com o layer que desenhou e o tempo medido pela própria GPU |
| Zones | as zonas de CPU numa linha do tempo, com tabela e histograma da zona escolhida |
| Plots | o que cada frame contou, em gráficos, e os valores que o seu código der |
| Memory | o heap bloco a bloco e o que o engine guarda na GPU |
| Log | o que o código disse e o que o processo logou, por nível |
| Inspect | os controles da página em árvore, e qualquer coisa que o frame desenha: aponte com o seletor e clique |

A linha logo abaixo das abas fica em qualquer uma delas: tempo do frame
contra o orçamento, CPU, GPU, draw calls, layers, heap e memória. À direita
você escolhe o frame mostrado (`Live`, `Slowest`, ou um clique numa barra
do histórico), move o painel para o lado e fecha. A borda do painel arrasta
para redimensionar.

Três macros alimentam o painel e fazem uma leitura atômica
quando ele está desligado:

```rust
engine::zone!("scene build");            // uma zona nomeada, até o fim do bloco
engine::plot!("controls bound", count);  // um valor por frame, sob um nome
engine::message!("page {name} staged");  // uma linha no log
```

Para uma imagem sem abrir janela:

```
cargo run -p uigallery -- --render Todo --dev frame --out frame.png
```

### No navegador

```
tool/tasks/webtarget.sh uigallery
tool/tasks/webtarget.sh graphs
tool/tasks/webtarget.sh primes
```

Acrescente `?page=Typography&theme=hoff` ao endereço da gallery para abrir
uma página num tema, e `?page=compound` (ou `elk`, `elk-groups`) ao dos
graphs para ver outro grafo.

### No celular

O simulador do iOS precisa do Xcode e do xcodegen; o Android precisa do 
SDK e NDK, do JDK 17 e do cargo-ndk.

```
tool/tasks/mobileapp.sh ios
tool/tasks/mobileapp.sh ios-run
tool/tasks/mobileapp.sh android
```

### O explorer de .urna

Ele lê arquivos `.urna` com o [urna](https://github.com/hoffresearch/urna),
clonado ao lado deste repositório (`../urna`, v0.4.0 ou posterior).

```
cargo run --manifest-path apps/urnadbapp/Cargo.toml
tool/tasks/webtarget.sh urnadbapp
```

### Checks

O gate prova que tudo compila e os testes passam; o `regress` compara os
frames da gallery com a referência; o `runcheck` abre cada programa do
repositório nesta máquina e diz se ele subiu.

```
tool/tasks/fullcheck.sh
cargo run -p regress -- check
tool/tasks/runcheck.sh
tool/tasks/runcheck.sh arcade lottie
```

O `builddir.sh` cuida do peso do build: `size` diz quanto pesa cada diretório,
`trim` remove o que o workspace gravou quando o `target/` passa do limite (as
dependências ficam) e `seed` dá a outro checkout um diretório já aquecido.

```
tool/tasks/builddir.sh size
tool/tasks/builddir.sh trim --yes
tool/tasks/builddir.sh seed target/agents/<task>
```

Os testes de integração de um crate são um executável só, `tests/suite.rs`,
com um módulo por arquivo. Os testes de um arquivo:
`cargo test -p engine --test suite capture::`.

As ferramentas além dessas (uma página para PNG, o agent
port, o mirror, as sessões gravadas, os importers, o modelo de linguagem)
estão em [docs/Research/Workflows/run-the-tools.md](docs/Research/Workflows/run-the-tools.md). Uma build de
debug faz hot reload: salve um `.wgsl`, uma string ou uma cor e a janela
aberta mostra a mudança.

## Workspace

Cada crate mora em uma de
quatro modulos, conforme o que ela é.

| Modulo | Onde | O quê |
|---|---|---|
| Libraries | `rust/` | `engine` (compose, control, drawing, runtime, styling e o design system em `engine::widgets`; o inventário está em [docs/Research/Structure/component-catalog.md](docs/Research/Structure/component-catalog.md)), `accord` (o contrato semântico), `plevir` (a IR do plev e seu verifier), `screen` (surfaces materializadas a partir da IR), `offers` (os proposers), `claude` (o Claude como proposer), `socket` (o agent port e a CLI do plev), `record` (o flight recorder), `replay` (a leitura de uma sessão gravada, plev-inspect), `motion` (o codec .monster), `import` (Lottie, SVG, React, HTML, Figma e gpui para dentro), `editor` (o core de edição em rope), `gitdrv` (git), `markup` + `macros` (a markup DSL e as proc-macros), `unions` (sem código: um só feature set para toda build, mantido pelo cargo hakari), `llmrun` (workspace próprio) |
| Apps | `apps/` | `uigallery` (a gallery do design system: desktop, web, Android, iOS), `idestudio` (o cliente git), `agenteval` (o lab do experimento), `inspector` (a janela do session inspector), `mirroring` (uma página espelhada pelo scene socket), `urnadbapp` (o explorer de `.urna`, workspace próprio) |
| Demos | `demo/` | uma crate por demo: arcade, camera, commit, graphs, inputs, intent, layers, lottie, mobile, mockup, player, primes, shapes, ticker |
| Tooling | `tool/` | `bench/measure` e `bench/regress` (as tabelas de custo e a regressão estrutural), `tasks` (os scripts), `tests` (os testes de arquitetura) |

`data/media/lottie` guarda as referências Lottie que o importer e os gates
leem, e `data/graphs/<formato>` as entradas de grafo que a demo graphs desenha
(d3, cytoscape). O [docs/TERMS.toml](docs/TERMS.toml) mapeia cada nome antigo
(sem, showcase, comps, ...) para seu lugar nesta árvore.

## Contratos


- Um TextStyle por text run, compartilhado entre medição e desenho. A única
  fonte de largura é o shaper, nunca uma estimativa aritmética.
- sRGB: decode uma vez ao entrar no trabalho da GPU, encode uma vez na
  escrita da surface.
- A geometria dos containers deriva do espaço disponível, não de
  constantes.
- A UI é desenhada para conteúdo, constraints, capabilities e ranges, escala fluida, responsividade guiada pelo
  componente, layout intrínseco, valores relativos e semânticos, adaptação
  progressiva.
- Uma mudança de estado visível gera invalidação, principalmente pra evitar redraw silencioso e frame stale na tela.

Um guard test varre o repositório e falha se algum código desenhar texto
construindo uma raw key em vez de passar pelo caminho único do TextStyle.

## Docs

- [docs/ATLAS.toml](docs/ATLAS.toml): centralizando temporariamente as diretrizes de arquitetura num arquivo só pra conseguir manter o controle. A ideia é table são para os llms, prosa para humanos, o diagrama do frame flow no fim.
- [docs/](docs/README.md): o acervo, com o [changelog](docs/CHANGELOG.md) por data, as [tasks](docs/Tasks/Tasks.md) (uma pasta por task), as decisões em [docs/adr/](docs/adr/adr.md) (uma pasta por ADR, com os aprendizados ao lado) e o que sabemos em [docs/Research/](docs/Research/Research.md), pelos mesmos temas dos ADRs, com os experimentos em `Research/Exploring`. Toda página termina com uma lista Related de links comuns, então Obsidian, Quartz, Zola e o GitHub desenham o mesmo grafo.
- Ler o acervo como site: [apps/zolaviews](apps/zolaviews/README.md) (`node plev/book.mjs --serve`, Zola) ou [apps/quartzweb](apps/quartzweb/README.md) (`npx quartz build --serve`, em `localhost:8080`); ou abrir `docs/` como vault no Obsidian.
- [docs/Research/Workflows/code-against-the-plev-engine.md](docs/Research/Workflows/code-against-the-plev-engine.md): o manual de operação, para ler antes de tocar em código de UI ou de rendering. Seus vizinhos em [docs/Research/Workflows/](docs/Research/Workflows/): validação de pixels, o target web, orquestração de agentes.
- [docs/Research/Synthesis/generative-runtime-program.md](docs/Research/Synthesis/generative-runtime-program.md): para onde o plev vai, de engine de UI na GPU a generative interaction runtime, com o grafo de tarefas e o estado de cada uma. A tese está em [docs/Research/Synthesis/generative-runtime-thesis.md](docs/Research/Synthesis/generative-runtime-thesis.md) e as regras que todo agente segue em [docs/Research/Workflows/generative-runtime-guidelines.md](docs/Research/Workflows/generative-runtime-guidelines.md).
- [docs/CHANGELOG.md](docs/CHANGELOG.md): o que mudou, do mais novo para o mais antigo, uma seção por data.
- [docs/TERMS.toml](docs/TERMS.toml): os nomes da árvore, antigos e novos, crate por crate e módulo por módulo, e onde mora hoje cada caminho antigo do acervo.
- [.config/](.config/): os contratos das ferramentas (hakari, lychee, nextest, typos).

## Licença

MIT, veja [LICENSE](LICENSE).

<details>
    
----

Copyright [Hoff Research](https://hoffresearch.com)

Made it simple, but significant (∂μFμν = Jν)

Author: Brenner Cruvinel

</details>
