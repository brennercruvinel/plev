---
title: visao e missao
status: aguardando revisao
tags: [visao, historia, legado, humildade]
---

# visao

## o que estamos construindo

dois artefatos que se alimentam um do outro:

1. plev, a engine. um compositing engine gpu-first em rust, um codebase, varios
   targets (macos/metal, web/webgpu shipping; android/ios em progresso;
   linux/windows pendente). nao e um framework de widgets, e a camada que vira
   scene graph em draw call identica em todo lugar. ja existe e ja funciona.

2. caranguejo vermelho, o livro. um livro de programacao rust que conta a
   jornada de construir o plev, ensina rust de verdade, e linka cada afirmacao a
   um diff, um commit, um adr, um benchmark. junto vem o blog (zola) e os
   tutoriais. o livro tem entre 569 e 963 paginas, com ~30 paginas deixadas
   abertas para o experimento atual (mon: lottie, swf/flash, motion ui).

o livro nao e um manual de api do plev. e a historia de uma pessoa construindo
uma engine tao robusta quanto um flutter, cross-device, dentro de limitacoes
reais, e usando isso para ensinar rust de forma diadica: serve para uma crianca
de 13 anos comecar, e serve para um engenheiro de ml da anthropic ou da
microsoft ler sem achar raso.

## o porque (a historia que abre o livro e o blog)

o primeiro post do blog e o primeiro fio do livro contam a mesma origem: a
decisao de escrever um livro tecnico quando a aurora, a filha do brenner,
nasceu. deixar um legado. ensinar algo a ela sobre rust, e explicar por que rust
e uma linguagem que vai sobreviver inclusive as ai e aos llms (a camada de
sistemas, memoria sem gc, controle fino, seguranca por tipo, nao some quando o
modelo de turno muda). e contar a jornada honesta de construir uma engine
universal cross-device.

esse e o coracao emocional. nao e marketing. e um pai escrevendo para a filha e,
de quebra, para quem quiser aprender junto.

## a postura: humildade, nao arrogancia

regra firme de tom para o livro e o blog inteiros: nao dizer que estamos
reinventando a roda. nao dizer que vamos criar mais uma gui (o brenner
referencia o areweguiyet.com de proposito, com ironia). o objetivo nao e
substituir nada, e construir com eficiencia, aprender em publico, e creditar
quem fez fundacao.

creditar explicitamente, com respeito e nome:

- makepad (rik arends e time): gpu-first, dsl live, o concorrente mais proximo.
- zed e o gpui (zed industries): ui gpu-native, prova que da pra fazer.
- bevy (carter anderson): ecs + wgpu, gpu-first, comunidade.
- flutter (impeller/skia): a referencia conceitual de "rendering proprio,
  cross-platform completo". o plev se posiciona como "skia para rust".
- ecossistema linebender: vello, xilem, parley, kurbo, peniko. abordagem de
  compute shaders, oposta e inspiradora.
- leptos (greg johnston): o sistema de signals que inspirou o signal.rs do plev.
- dioxus, slint, iced, egui, floem, ribir, vizia: cada um com sua escolha.

o livro reconhece a complexidade e a divida com esses projetos. o brenner e
eterno aprendiz, e o texto soa assim. nada de "ninguem chegou nisso", nada de
arco de revelacao heroico. less, but better. simple, but significant.

## os nomes (phi -> ... -> plev, e caranguejo vermelho)

o projeto teve varios nomes ao longo de quase 4 anos antes de chegar em plev.
um deles foi phi (a letra grega, as vezes escrita em ascii romano). o volume
claude2026 tem `project_phi_context.md` e referencias a esse periodo. parte
dessa historia se perdeu: o forgejo self-hosted na hostinger foi deletado, e a
aplicacao foi recuperada mas nao o historico git. entao muita coisa nao esta em
diff, esta espalhada em docs, conversas e notas, e precisa ser correlacionada e
reconstruida para virar a linha do tempo do livro.

caranguejo vermelho e o nome do meta-projeto (livro + blog + tutoriais). e um
aceno ao ferris, o caranguejo mascote do rust, e a ferrugem (rust). vermelho
fecha a imagem.

## o pacto entre nos

o brenner foi explicito: ele quer que eu atue como orquestrador e validador, nao
como executor que despeja tudo no fim. os agentes em paralelo tendem a entregar
qualquer coisa rapido so para fechar a tarefa. por isso cada agente recebe poc,
hook obrigatorio e guia claro, e eu reviso e corrijo entrega por entrega, sem
deixar acumular para o final. o objetivo nobre: provar que da pra fazer um
software grandioso, sem prepotencia, e contar essa historia ensinando.
