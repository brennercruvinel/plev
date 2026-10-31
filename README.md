# plev

Uma engine de compositing GPU-first em Rust. Um único código desenha o
mesmo frame no macOS (Metal), no Linux (Vulkan), no navegador (WebGPU), no
iOS (Metal) e no Android (Vulkan). A gallery foi capturada no navegador e
no simulador do iOS e comparada com o desktop byte a byte. No navegador, 79 das 82 páginas
são idênticas; as outras três mostram conteúdo diferente de propósito,
porque o navegador não tem Claude nem git.

A scene é reconstruída a cada frame, o compositor redesenha só as layers que
mudaram, e tudo termina numa surface sRGB. Há glass, backdrop blur, analytic
shadows e layout que acompanha o conteúdo. O texto passa por shaping de
verdade, é rasterizado na escala da tela e cobre os scripts latino, árabe,
hebraico, devanágari, chinês, japonês e coreano, além de color emoji, com
carets bidi, input methods e um editor que abre um milhão de linhas em cerca
de 20 ms numa build de release. Quando nada muda, nada é desenhado: um campo
com foco e sem toque custa dois frames por segundo para o caret, e um app
idle não custa nenhum. Os apps consomem a engine e nada passa por fora dela.

Cada controle que um app desenha também é dado: um id, um role, seu estado e
suas actions. Um screen reader, um agente fora do processo e uma sessão
gravada leem esse mesmo contrato. Um agente age pelo mesmo dispatcher que um
clique, e só sobre o estado que lhe foi mostrado; agir sobre algo que mudou
desde então é recusado, com o nome de quem mudou. Uma tela pode ser gerada a
partir de um intent, como uma IR verificada, e uma sessão gravada abre no
inspector ou roda de novo como teste.

## Por que isso existe

Comecei isso há uns seis anos, bem antes dos LLMs, quando decidi aprender
Rust do jeito honesto: construindo algo real. O algo real era um app
educativo para a minha filha, cem por cento em Rust, e ele precisava de uma
engine que não existia. O plev é a engine por baixo desse sonho.

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

Ela já serve além do sonho original: o
[urna](https://github.com/hoffresearch/urna), um vector database embarcado e
soberano, entrega seu explorer (`apps/urnadbapp`) sobre o plev. E o app
infantil que começou tudo isso continua sendo a próxima coisa a construir.

Se algo disso fizer sentido para você, venha construir comigo.

## O que há dentro

**A engine.** Compositor por layers com hash de sujeira, texto com
cosmic-text num atlas de glifos na escala física da tela, layout com taffy,
paths com lyon, tudo numa surface sRGB. Um design system inteiro
(`engine::widgets`) mora dentro dela: botões, formulários, navegação,
tabelas, overlays, charts, grafos e um editor de texto em rope que abre um
milhão de linhas em cerca de 20 ms. A gallery de 82 páginas é a prova de
regressão: 1312 frames (82 páginas, 4 temas, 4 escalas) iguais bit a bit
entre execuções, e o desktop é a referência contra a qual web, iOS e Android
são comparados pixel a pixel.

**O contrato semântico.** Cada controle desenhado também é um registro
(`accord`): id, role, estado, ações e as precondições de cada ação. Esse
registro é o que o AccessKit projeta para o leitor de tela, o que um agente
observa e opera pelo socket local, e o que uma sessão gravada guarda. Uma
tela pode nascer de uma intenção como IR (`plevir`), verificada antes de
existir contra as capabilities que o app declarou, e ser transformada em
runtime (esconder, agrupar, adensar, dar ênfase) sem perder identidade; cada
transformação carrega sua inversa exata.

**O `.monster`.** Um codec binário próprio de animação e de gravação, com a
mesma ideia do H.264 aplicada a vetores: keyframes são snapshots completos
da cena (seek em O(1)), interframes são segmentos por propriedade (valor,
easing em 1 byte, duração) que o player interpola, independentes de fps e
scrubáveis. A cena é um mapa plano `depth → node` com bit de presença por
campo, então o que não mudou custa zero bytes. Valores quantizados no
arquivo (twips, rgba8, u16 fixo), f32 na memória; seções com sha256;
decoder estrito que nunca entra em pânico. O encoder tem dois modos: lower
de uma timeline autoral, ou descoberta de deltas a partir de frames
amostrados, sem ninguém declarar nada. É esse segundo modo que permite ao
`record` gravar uma sessão de uso como dado, nunca como vídeo: a trilha
visual e a trilha semântica (quem observou, quem agiu, sobre qual estado, com
que resultado) no mesmo arquivo. Um bug reproduzido vira um `.monster`, e o
`.monster` vira um teste no gate. Uma animação importada do Lottie sai pela
mesma porta: amostrada uma vez, codificada, e depois disso o plev só toca
monster.

**Os importers.** Telas de outros mundos entram por um transpiler em três
estágios: parse (tree-sitter para React TSX + Sass, HTML + CSS e gpui; o
JSON da REST para Figma), resolve para uma árvore comum com cores mapeadas
nos tokens do tema dentro de uma tolerância medida, e dois emissores sobre
essa árvore: código Rust contra o builder, ou plev IR com um relatório que
põe cada construção da origem em `supported`, `transformed`, `approximated`
ou `impossible`, com `arquivo:linha`. Nada é descartado em silêncio, e o
importer não declara capability nenhuma: um botão sai como `binding
required`, e é o verificador que decide se a tela pode existir. SVG entra
como path tesselado. O escopo é pequeno de propósito: um componente de cada
origem, provado de ponta a ponta, antes de prometer o formato inteiro.

**Em volta.** Demos que provaram uma capacidade cada (grafos, câmera,
gestos, mobile, Lottie, um arcade), um cliente git (`idestudio`) que serve de
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

As demos que abrem uma janela sozinhas:

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

A demo de grafos tem quatro páginas; sem argumento abre a primeira:

```
cargo run -p graphs -- lesmis
cargo run -p graphs -- compound
cargo run -p graphs -- elk
cargo run -p graphs -- elk-groups
```

Três demos pedem um arquivo, e `commit` não abre janela: imprime no
terminal os três receipts do cenário em que um agente tenta commitar o que
viu e a pessoa mudou o staging no meio.

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

### No navegador

Cada comando imprime o endereço a abrir. Uma porta já ocupada é pulada para
a próxima livre, então todos podem rodar ao mesmo tempo. Redimensione a
janela e o layout acompanha.

```
tool/tasks/webtarget.sh uigallery
tool/tasks/webtarget.sh graphs
tool/tasks/webtarget.sh primes
```

Acrescente `?page=Typography&theme=hoff` ao endereço da gallery para abrir
uma página num tema, e `?page=compound` (ou `elk`, `elk-groups`) ao dos
graphs para ver outro grafo.

### No celular

O simulador do iOS precisa do Xcode e do xcodegen; o Android precisa do seu
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

As ferramentas além dessas (uma página para PNG, o custo de um frame, o agent
port, o mirror, as sessões gravadas, os importers, o modelo de linguagem)
estão em [docs/referencia/operacao/run-the-tools.md](docs/referencia/operacao/run-the-tools.md). Uma build de
debug faz hot reload: salve um `.wgsl`, uma string ou uma cor e a janela
aberta mostra a mudança.

## Workspace

A raiz do repositório é um workspace virtual; cada crate mora em uma de
quatro casas, conforme o que ela é.

| Casa | Onde | O quê |
|---|---|---|
| Libraries | `rust/` | `engine` (compose, control, drawing, runtime, styling e o design system em `engine::widgets`; o inventário está em [docs/referencia/arquitetura/component-catalog.md](docs/referencia/arquitetura/component-catalog.md)), `accord` (o contrato semântico), `plevir` (a IR do plev e seu verifier), `screen` (surfaces materializadas a partir da IR), `offers` (os proposers), `claude` (o Claude como proposer), `socket` (o agent port e a CLI do plev), `record` (o flight recorder), `replay` (a leitura de uma sessão gravada, plev-inspect), `motion` (o codec .monster), `import` (Lottie, SVG, React, HTML, Figma e gpui para dentro), `editor` (o core de edição em rope), `gitdrv` (git), `markup` + `macros` (a markup DSL e as proc-macros), `unions` (sem código: um só feature set para toda build, mantido pelo cargo hakari), `llmrun` (workspace próprio) |
| Apps | `apps/` | `uigallery` (a gallery do design system: desktop, web, Android, iOS), `idestudio` (o cliente git), `agenteval` (o lab do experimento), `inspector` (a janela do session inspector), `mirroring` (uma página espelhada pelo scene socket), `urnadbapp` (o explorer de `.urna`, workspace próprio) |
| Demos | `demo/` | uma crate por demo: arcade, camera, commit, graphs, inputs, intent, layers, lottie, mobile, mockup, player, primes, shapes, ticker |
| Tooling | `tool/` | `bench/measure` e `bench/regress` (as tabelas de custo e a regressão estrutural), `tasks` (os scripts), `tests` (os testes de arquitetura) |

`data/media/lottie` guarda as referências Lottie que o importer e os gates
leem, e `data/graphs/<formato>` as entradas de grafo que a demo graphs desenha
(d3, cytoscape). O [docs/TERMS.toml](docs/TERMS.toml) mapeia cada nome antigo
(sem, showcase, comps, ...) para seu lugar nesta árvore.

## Contratos

Violar um deles é defeito, mesmo quando os pixels parecem certos.

- Um TextStyle por text run, compartilhado entre medição e desenho. A única
  fonte de largura é o shaper, nunca uma estimativa aritmética.
- sRGB: decode uma vez ao entrar no trabalho da GPU, encode uma vez na
  escrita da surface.
- A geometria dos containers deriva do espaço disponível, nunca de
  constantes.
- A UI é desenhada para conteúdo, constraints, capabilities e ranges, não
  para telas e dispositivos: escala fluida, responsividade guiada pelo
  componente, layout intrínseco, valores relativos e semânticos, adaptação
  progressiva.
- Uma mudança de estado visível implica invalidação, sem redraw silencioso e
  sem frame stale na tela.
- Um agente age só pelo dispatcher do app, o mesmo caminho de um clique, e
  só sobre o estado que lhe foi mostrado.

Um guard test varre o repositório e falha se algum código desenhar texto
construindo uma raw key em vez de passar pelo caminho único do TextStyle.

## Docs

- [docs/ATLAS.toml](docs/ATLAS.toml): a arquitetura num arquivo só. Tabelas para as máquinas, prosa para as pessoas, o diagrama do frame flow no fim.
- [docs/](docs/README.md): o acervo, com a [linha do tempo](docs/historia/linha-do-tempo.md), as tasks, os ADRs em [docs/decisoes/](docs/decisoes/README.md), as pesquisas e os aprendizados.
- [docs/referencia/operacao/code-against-the-plev-engine.md](docs/referencia/operacao/code-against-the-plev-engine.md): o manual de operação, para ler antes de tocar em código de UI ou de rendering. Seus vizinhos em [docs/referencia/operacao/](docs/referencia/operacao/): validação de pixels, o target web, orquestração de agentes.
- [docs/historia/programa-runtime-generativo.md](docs/historia/programa-runtime-generativo.md): para onde o plev vai, de engine de UI na GPU a generative interaction runtime, com o grafo de tarefas e o estado de cada uma. A tese está em [docs/pesquisas/tese-runtime-generativo.md](docs/pesquisas/tese-runtime-generativo.md) e as regras que todo agente segue em [docs/referencia/operacao/generative-runtime-guidelines.md](docs/referencia/operacao/generative-runtime-guidelines.md).
- [docs/historia/changelog.md](docs/historia/changelog.md): o que mudou, do mais novo para o mais antigo.
- [docs/TERMS.toml](docs/TERMS.toml): os nomes da árvore, antigos e novos, crate por crate e módulo por módulo.
- [.devops/check/](.devops/check/): os contratos das ferramentas (lychee, nextest, typos).

## Licença

MIT, veja [LICENSE](LICENSE).

Feito simples, mas significativo.

Mantido por Brenner Cruvinel (brenner@hoffresearch.com).
Toda contribuição é bem-vinda.
(∂μfμν = jν)

Brenner Cruvinel
