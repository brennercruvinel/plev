---
title: origem
parte: 0
status: amostra
idioma: pt-br
rastros:
  - 00-visao.md (aurora, nomes, humildade e creditos)
  - kdb/refs/competitors.md (makepad, gpu-first, skia para rust)
  - Cargo.toml:21-26 (edition 2024, rust-version 1.85)
  - crates/engine/src/signal/mod.rs:1 (credito leptos no codigo)
  - crates/engine/src/signal/api.rs:167 (create_signal)
  - kdb/adr/benchmark-results.md:11-44 (m4, push_rects, dirty, signals)
  - doc/arc/arc.yaml:8,17-22 (stack, targets)
---

# parte 0, origem

## 0.1 a decisao, quando a aurora nasceu

a aurora nasceu e eu decidi escrever um livro. nao um livro sobre ser pai, um
livro sobre rust. parece um non sequitur e talvez seja, mas a ligacao, para mim,
e direta: um filho te faz pensar no que sobra depois de voce, e a coisa que eu
sei fazer, a coisa que eu queria deixar registrada com algum cuidado, e
construir software e explicar como ele foi construido.

entao o livro nasceu junto com ela, mais ou menos. nao como um diario, como um
legado tecnico. eu queria deixar algo que ela pudesse abrir aos treze anos e
acompanhar, e que um engenheiro de ml de uma anthropic ou de uma microsoft
pudesse ler sem achar raso. essa e a regra de tom do livro inteiro, a que eu
chamo de diadica: o conceito primeiro em linguagem humana, depois o codigo que
roda de verdade, depois o porque arquitetural com o adr e o numero do benchmark.
abre acessivel, aprofunda ate doer.

eu poderia ter escrito sobre qualquer coisa. escolhi escrever sobre a engine que
eu estava construindo de qualquer jeito, porque a unica forma honesta que eu
conheco de ensinar uma linguagem e mostrar um sistema real sendo feito nela, com
os erros e as decisoes que precisei reverter, com os asteriscos no lugar onde os
asteriscos ficam. um livro de api eu nao
escreveria. esse aqui linka cada afirmacao a um diff, um commit, um adr, um
benchmark. e a diferenca entre um livro de opiniao e um livro que aconteceu.

nao vou fingir que isso e altruismo puro. tem ego ai, claro que tem. mas o
nucleo e simples: um pai escrevendo para a filha e, de quebra, para quem quiser
aprender junto. less, but better. simple, but significant.

## 0.2 os nomes, de phi a plev, e caranguejo vermelho

a engine se chama plev. nao se chamou sempre. teve varios nomes ao longo de
quase quatro anos antes de chegar nesse, e um deles foi phi, a letra grega, as
vezes escrita em ascii romano quando o terminal nao colaborava. parte dessa
historia eu nao consigo te mostrar em diff, e preciso ser honesto sobre isso
agora, no comeco: o forgejo self-hosted que eu rodava numa hostinger foi
deletado. a aplicacao voltou, o historico git nao. entao muita coisa do periodo
phi vive espalhada em docs, conversas e notas, e precisa ser correlacionada e
reconstruida para virar linha do tempo. onde eu nao tiver o commit, eu vou
marcar que nao tenho o commit. e mais ou menos o oposto de um livro que inventa
uma narrativa limpa por cima de um processo que foi sujo.

> nota de rastreabilidade: a referencia ao periodo phi vem de um volume de
> contexto antigo (claude2026, `project_phi_context.md`). eu nao reabri esse
> arquivo para esta amostra, entao trato a datacao exata como nao confirmada. o
> que esta confirmado e que o nome mudou de phi para plev e que o historico git
> anterior se perdeu com o forgejo.

caranguejo vermelho e o nome do meta-projeto, o conjunto livro mais blog mais
tutoriais. e um aceno ao ferris, o caranguejo mascote do rust, e a propria
palavra rust, ferrugem. vermelho fecha a imagem. nenhum dos dois nomes tenta ser
esperto. plev e curto e cabe num `cargo run -p`, caranguejo vermelho e uma
piada interna que eu resolvi levar a serio.

## 0.3 por que rust nao some quando o modelo de turno muda

essa parte e opiniao, e eu vou marcar como opiniao. eu escolhi rust para a
engine, e escolhi rust para ensinar no livro, porque acho que rust e uma
linguagem que sobrevive a essa onda de ai e LLM melhor do que quase todo o
resto. nao porque ela seja imune ao hype, e sim por onde ela mora.

rust mora na camada de sistemas. memoria sem garbage collector, controle fino do
que acontece com cada byte, seguranca garantida pelo tipo em vez de garantida por
torcida. essa camada nao evapora quando o modelo da vez muda. o modelo que voce
usa hoje para gerar codigo vai ser substituido, e o proximo tambem. o que o
codigo gerado precisa tocar, a GPU, a surface, o buffer que persiste entre
frames, continua exigindo alguem que saiba exatamente onde a borrow termina.
quando o assistente erra, e ele erra, o compilador do rust e quem te segura. o
erro que em outra linguagem viraria um crash silencioso em producao, aqui vira um
texto vermelho antes do binario existir.

tem um detalhe concreto que eu gosto de mostrar cedo. o workspace inteiro do plev
declara isto, e nao e enfeite:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
```

edition 2024 nao e um numero de marketing. ela mudou regras reais, por exemplo a
forma como voce marca um `no_mangle` num entrypoint de plataforma virou
`#[unsafe(no_mangle)]`, porque a linguagem decidiu que voce tem que dizer em voz
alta quando esta saindo da area segura. isso e o espirito da coisa. o unsafe
existe, ninguem finge que nao, mas ele e nomeado, cercado, e voce assina embaixo.

e tem outra coisa que rust faz bem e que combina com a forma como eu trabalho
hoje, em cocriacao com LLM o tempo todo. o sistema de signals da engine, o
reativo que atualiza um no sem reconstruir a arvore toda, abre o arquivo com este
comentario:

```rust
//! Reactive signal system -- push-pull hybrid (Leptos/Reactively style).
```

a API que sai disso e pequena e dificil de usar errado:

```rust
let (count, set_count) = create_signal(0);
let n = count.get();      // le e registra a dependencia
set_count.set(n + 1);     // escreve e acorda quem depende
```

`create_signal` devolve um par, um lado so le, o outro so escreve. o tipo te
impede de confundir os dois. quando o modelo me sugere um trecho que mexe nisso,
eu nao preciso confiar na sugestao, eu confio no tipo. e disso que eu falo quando
digo que rust sobrevive ao turno: o conhecimento nao esta no modelo, esta na
forma do programa.

## 0.4 a jornada, uma engine cross-device, sem heroismo

a ambicao do plev e ser uma engine tao robusta quanto um flutter, um codebase, o
mesmo desenho saindo identico em todo lugar. e eu preciso ser exato sobre onde
isso esta, porque um livro que mente sobre o proprio estado nao serve para
ensinar nada.

o que ja roda em producao: macos no metal, e o browser no webgpu via WASM.
android e ios rodam o showcase e estao em progresso, nao em paz. linux e windows
estao pendentes, ponto. a engine roda sobre wgpu 28, winit 0.30, cosmic-text
0.18 e taffy 0.9, e o nucleo passou de quatrocentos testes em torno de quinze mil
linhas. nada disso e promessa, e o estado de hoje, com as lacunas no lugar onde
as lacunas estao.

a parte que ja existe, existe medida. num macbook m4, a construcao da cena no
lado da CPU fica entre 159 e 222 milhoes de rects por segundo, dependendo do
tamanho da cena. o dirty tracking de mil rects parados custa 3.3 microssegundos,
o que na pratica quer dizer que uma cena estatica e de graca depois do primeiro
frame. um ciclo de signal, criar mais ler mais escrever, sai a 67 nanossegundos.
eu nao trago esses numeros para me gabar, trago para ancorar a frase anterior: a
engine existe, e da para conferir.

agora a parte que mais me importa no tom. eu nao estou reinventando a roda, e nao
estou fazendo mais uma GUI. tem um site, o areweguiyet.com, que cataloga as
tentativas de GUI em rust com uma ironia carinhosa, e eu cito ele de proposito,
porque a piada e comigo tambem. o objetivo nao e substituir nada. e construir com
eficiencia, aprender em publico, e creditar quem fez a fundacao.

entao eu credito, com nome:

makepad, do rik arends e time, e o concorrente mais proximo do que eu faco,
GPU-first de verdade, com DSL live. eles lancaram a 1.0 em maio de 2025 depois de
mais de seis anos de trabalho, e eu aprendi olhando o que eles resolveram e o que
eles deixaram de fora, como acessibilidade. zed e o gpui provaram que da para
fazer UI GPU-native num produto real que as pessoas usam para trabalhar. bevy, do
carter anderson, mostrou ECS mais wgpu com uma comunidade que sustenta o projeto.
flutter, com o impeller e o skia, e a referencia conceitual de rendering proprio
cross-platform completo, e o plev se posiciona, sem timidez e sem arrogancia, como
um skia para rust, a camada de baixo que outros frameworks poderiam consumir. o
ecossistema linebender, vello, xilem, parley, kurbo, peniko, segue um caminho de
compute shaders que e quase o oposto do meu, e por ser oposto me ensina mais. e o
leptos, do greg johnston, e a referencia direta do sistema de signals que eu
mostrei ali em cima, ao ponto de o credito estar escrito no comentario do proprio
arquivo, nao numa nota de rodape.

nenhum desses projetos me deve nada e eu devo um pouco a todos. o livro inteiro
e construido sobre essa divida reconhecida. eu sou eterno aprendiz, as vezes com
sentimento contraditorio sobre o que estou fazendo, e o texto vai soar assim de
proposito. sem arco de revelacao, sem heroi, sem ninguem chegando onde ninguem
chegou. so um pai, uma engine que roda em alguns lugares e ainda nao em outros, e
a tentativa honesta de mostrar como.

## rastros

afirmacoes deste capitulo e a fonte de cada uma. file:line onde foi possivel.

- a decisao de escrever quando a aurora nasceu, o legado, o publico diadico
  (crianca de 13 e engenheiro de ml): `kdb/briefing/00-visao.md:30-42`.
- ancoragem por diff/commit/adr/benchmark como regra do livro:
  `kdb/briefing/05-livro.md:18-23`.
- nome phi -> plev, quase 4 anos, forgejo na hostinger deletado e historico git
  perdido, caranguejo vermelho como aceno ao ferris/rust:
  `kdb/briefing/00-visao.md:67-80`.
- claude2026 `project_phi_context.md`: citado em `kdb/briefing/00-visao.md:72-73`.
  NAO CONFIRMADO por leitura direta nesta amostra (datacao do periodo phi tratada
  como incerta).
- por que rust sobrevive a ai/LLM (camada de sistemas, memoria sem GC, controle
  fino, seguranca por tipo, nao some quando o modelo de turno muda):
  `kdb/briefing/00-visao.md:30-38` (opiniao, marcada como tal).
- edition 2024, rust-version 1.85, version 0.1.0: `Cargo.toml:21-26` (snippet toml
  conferido contra o arquivo).
- `#[unsafe(no_mangle)]` exigido na edition 2024: `kdb/adr/index.md:27`.
- credito ao leptos no codigo do signal: `crates/engine/src/signal/mod.rs:1`
  (comentario `push-pull hybrid (Leptos/Reactively style)`, reproduzido literal).
- API `create_signal` -> `(ReadSignal, WriteSignal)`, `.get()`, `.set()`:
  `crates/engine/src/signal/api.rs:167,32,123` (snippet rust conferido contra o
  arquivo, compila com a versao do `Cargo.toml`).
- targets: macos/metal e web/webgpu shipping, android/ios em progresso,
  linux/windows pendentes: `doc/arc/arc.yaml:17-22`.
- stack wgpu 28, winit 0.30, cosmic-text 0.18, taffy 0.9: `doc/arc/arc.yaml:8` e
  `Cargo.toml:50-56`.
- 404 testes, ~15.000 LOC core: `kdb/mission/readme.md:15` (no texto, arredondado
  para "passou de quatrocentos testes" e "quinze mil linhas").
- benchmarks m4: push_rects 159-222m rects/s, dirty tracking 3.31us/1000, signal
  67ns/cycle: `kdb/adr/benchmark-results.md:11,18-20,30,44`.
- makepad (rik arends, 1.0 maio 2025, 6+ anos, acessibilidade zero):
  `kdb/refs/competitors.md:16,34,36`; creditos nominais a makepad, zed/gpui, bevy,
  flutter/impeller, linebender, leptos: `kdb/briefing/00-visao.md:51-61`.
- posicionamento "skia para rust": `kdb/briefing/00-visao.md:56` e
  `kdb/refs/competitors.md:258`.
- areweguiyet.com citado com ironia, "nao reinventar a roda, nao mais uma GUI":
  `kdb/briefing/00-visao.md:44-49`.
