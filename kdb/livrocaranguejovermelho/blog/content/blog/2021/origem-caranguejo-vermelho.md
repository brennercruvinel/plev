+++
authors = ["Brenner Cruvinel"]
title = "A origem do caranguejo vermelho"
description = "O primeiro fio da série building plev: por que comecei a escrever um livro de Rust quando a Aurora nasceu, de onde veio o nome (de phi até plev) e por que prefiro creditar quem fez a fundação a fingir que reinventei a roda."
# data provisoria. o ano da pasta sera ajustado pela timeline real (phi -> plev) depois.
date = 2021-09-01
path = "blog/origem-caranguejo-vermelho"
[taxonomies]
tags = ["building plev", "Rust", "Caranguejo Vermelho", "Origem"]
+++

quando a aurora nasceu, eu decidi escrever um livro. nao um livro qualquer: um livro tecnico, sobre construir software de verdade em rust. a ideia era teimosa e simples, deixar pra ela alguma coisa que durasse mais que um post, mais que um repo, mais que a moda da semana. um legado que ela possa abrir daqui a quinze anos e entender de onde o pai veio.

e um pai escrevendo pra filha. quem quiser aprender junto, senta aqui do lado.

## por que rust

a aposta no rust nao e estetica. e a camada de sistemas, e essa camada nao some quando o modelo da vez muda. memoria sem garbage collector e controle fino do que roda na maquina, com um sistema de tipos que te segura antes do bug chegar em producao. as ai escrevem cada vez mais codigo, e justamente por isso a linguagem em que da pra confiar embaixo importa mais, nao menos. quando o llm da vez for trocado por outro daqui a seis meses, o ownership continua valendo. o borrow checker nao tem turno.

quero que a aurora cresca sabendo que existe uma camada do computador onde voce ainda manda, byte a byte.

## a jornada, a engine ja existe

o livro conta a construcao do plev, um compositing engine gpu-first em rust. um codebase, varios mundos: macos com metal e o browser com webgpu ja rodam o mesmo render, android e ios estao em progresso. nao e um framework de widgets. e a camada que pega um scene graph e vira draw call, identica em todo lugar.

e nao e promessa de slide. enquanto escrevo isso, a engine tem mais de quatrocentos testes verdes e o core ja passou de quinze mil linhas. os numeros de performance sao medidos em criterion num mac m4, nao chutados. cada afirmacao do livro vai linkar um diff, um commit, um adr, um numero. essa e a diferenca entre um livro de opiniao e um livro que aconteceu.

## os nomes, de phi ao caranguejo vermelho

o projeto teve muitos nomes em quase quatro anos antes de virar plev. um deles foi phi, a letra grega. parte dessa historia se perdeu de um jeito bobo e que ainda doi: o forgejo self-hosted que eu rodava numa vps da hostinger foi apagado. recuperei a aplicacao, mas nao o historico git. entao muita coisa nao mora em commit, mora em nota solta, conversa e doc espalhado, e vai ter que ser remontada na mao pra virar a linha do tempo do livro.

caranguejo vermelho e o nome do conjunto todo, o livro mais o blog mais os tutoriais. e um aceno ao ferris, o caranguejo mascote do rust, e a propria ferrugem (rust, em ingles, e ferrugem). o vermelho fecha a imagem.

## humildade, e nao reinventar a roda

eu nao to reinventando a roda, e nao quero fingir que to. o areweguiyet.com existe por um motivo, tem gui em rust pra todo lado, e o mundo nao precisa de mais uma com a minha cara estampada. o que eu quero aqui e construir com eficiencia, aprender em publico, e dar credito a quem fez a fundacao.

entao credito, com nome. makepad, do rik arends e time, gpu-first e dsl ao vivo, o vizinho mais proximo do que eu faco. zed e o gpui, da zed industries, que provaram na pratica que da pra fazer ui nativa na gpu. bevy, do carter anderson, ecs e wgpu, com uma comunidade que eu invejo. flutter, com impeller e skia, a referencia conceitual de rendering proprio cross-platform, e o plev se posiciona como "skia pra rust", digo isso com respeito. o ecossistema linebender, vello, xilem, parley, kurbo, com a abordagem de compute shader que e quase o oposto da minha e me ensina justamente por ser oposta. e o leptos, do greg johnston, cujo sistema de signals inspirou direto o signal.rs do plev.

humildade tambem aparece no codigo, nao so no agradecimento. teve um momento em que eu avaliei trocar a stack de texto, cosmic-text por parley, e a resposta honesta da pesquisa foi: espera, nao migra agora. ta registrado num adr, com o porque inteiro. dizer "ainda nao" e bem mais dificil que dizer "fiz".

sinceramente, eu sou eterno aprendiz nisso, e tem dia que o sentimento vem contraditorio, orgulho e duvida no mesmo cafe. mas e isso que eu quero que a aurora veja um dia. nao um pai que sabia tudo, um pai que construiu em publico, errou com registro e creditou quem veio antes. less, but better. simple, but significant.

esse e o primeiro fio. o livro puxa a partir daqui.

## rastros

- a visao e a postura de humildade: `kdb/briefing/00-visao.md`
- o mapa da arquitetura e os numeros da engine: `doc/arc/arc.yaml`, `kdb/mission/readme.md`
- as decisoes tecnicas que o livro linka uma a uma: `kdb/adr/index.md`
- o "ainda nao" do texto (cosmic-text vs parley): `kdb/adr/parley-vs-cosmic-text.md`
- a estrutura do livro, parte 0 (origem): `kdb/briefing/05-livro.md`
