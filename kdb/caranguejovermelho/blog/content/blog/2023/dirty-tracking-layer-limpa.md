+++
authors = ["Brenner Cruvinel"]
title = "dirty tracking por hash, layer limpa nao paga nada"
description = "como o compositor do plev usa um hash por layer pra pular render pass, rebuild de geometria e reshape quando nada mudou, e por que perguntar custa 3.3us por 1000 nos."
# data provisoria. evento real ancorado em kdb/adr/layer-system.md (2026-03-08)
# e no benchmark de dirty tracking (2026-03-11). o ano da pasta (2023) e
# provisorio e sera ajustado pela timeline real depois.
date = 2023-03-08
draft = true
path = "blog/dirty-tracking-layer-limpa"
[taxonomies]
tags = ["building plev", "rust", "gpu", "wgpu", "performance", "compositor"]
+++

a primeira versao do compositor do plev redesenhava o mundo inteiro a cada frame. mexeu o mouse, a engine reconstruia toda a geometria, remandava o texto pro shaper e abria um render pass por layer. funcionava. e era burro de um jeito que me incomodava muito: um painel parado num canto, que nao mudou um pixel, pagava o mesmo preco de um painel sendo arrastado pela tela.

essa simetria errada e o que me tirava do serio. trabalho deveria ser proporcional a mudanca, nao ao tamanho da cena. um cursor piscando num campo de texto nao tem por que custar a re-tesselagem de um grafico que nem esta sob o ponteiro.

## a cena como conteudo enderecavel

a saida foi parar de tratar a cena como uma sequencia de comandos de desenho e comecar a trata-la como conteudo que tem identidade. cada layer no plev carrega a sua propria textura offscreen RGBA, os seus proprios buffers de quad e de texto, e um hash da sua cena. no fim de cada frame eu calculo esse hash com o `fxhasher`. se o hash bate com o do frame anterior, a layer nao mudou. e ai nao tem o que redesenhar.

note que a granularidade aqui e a layer, nao o no. eu nao hasheio nem comparo cada widget individualmente. a layer e a unidade de dirty tracking porque ela ja e a unidade de composicao: tem textura propria, ordem em `z_order`, e a default (id=0, pra onde o `push()` vai) sempre existe. usar a layer como fronteira me deu um lugar natural pra guardar o cache, que e a propria textura offscreen do frame passado.

o que uma layer limpa compra e bem concreto. zero render passes, zero rebuild de geometria, zero shaping. a textura do frame anterior continua valida, e ela vai direto pro composite. em estado estacionario, uma tela cheia de painel parado nao faz a GPU reconstruir absolutamente nada. so resta o trabalho de juntar as texturas no fim.

## o preco de perguntar

aqui entra a parte honesta, que e a que eu mais gosto de contar. hashear nao e de graca. pra saber que posso ser preguicoso, eu pago pra perguntar. o benchmark de dirty tracking no criterion, rodado num mac m4, deu 3.3us por 1000 nos. esse e o custo de varrer a cena de uma layer e descobrir se ela mudou.

e isso muda como eu penso no ganho. nao e "de graca", e barato. 3.3 microssegundos pra mil nos contra abrir um render pass, reconstruir os vertices e remandar o texto pro shaper, que e ordens de grandeza mais caro. a aposta e estatistica: na maioria dos frames, a maioria das layers esta parada. enquanto essa premissa for verdade, pagar o hash compensa com folga. se cada layer mudasse todo frame, eu estaria pagando o hash por nada, em cima do trabalho que teria de fazer de qualquer jeito. nesse caso patologico o dirty tracking e overhead puro. mas ui de verdade nao se comporta assim, ui fica parada quase o tempo todo.

## o composite ainda acontece

uma duvida que aparece sempre: se a layer esta limpa e eu nao desenho nada, como ela continua na tela? e que o "nao fazer nada" se refere ao conteudo da layer, nao a composicao final. toda layer visivel, suja ou limpa, passa pelo composite pass. e ele e ridiculo de barato: um full-screen triangle gerado por `vertex_index`, sem vertex buffer, tres vertices, um `draw(0..3)` por layer. o shader em `composite.wgsl` recebe a textura da layer no bind group 0 e a opacidade num uniform `f32` no bind group 1.

a composicao funcionar depende de uma decisao que parece detalhe e nao e: alpha premultiplicado. o pipeline inteiro passou de `SrcAlpha/OneMinusSrcAlpha` pra `One/OneMinusSrcAlpha`, e os shaders emitem `rgb * a, a`. sem isso o operador `over` nao compoe layers translucidas corretamente. pra cores opacas o resultado e visualmente identico, entao e o tipo de mudanca que nao aparece na captura de tela mas conserta a matematica por baixo.

o custo de memoria desse esquema e real e vale dizer em voz alta: cada layer mantem a sua textura, mais ou menos 8mb de VRAM por layer em 1920x1080. dirty tracking troca memoria por tempo. eu guardo o frame renderizado pra nao precisar renderiza-lo de novo. pra um punhado de layers compoe bem, mas nao e o tipo de coisa que eu sairia multiplicando sem pensar.

## onde isso ja existia antes

nada disso e novo, e eu nao quero fingir que e. a ideia de nao recomputar o que nao mudou e a mesma intuicao por tras da reconciliation do react, do retained mode contra o immediate mode, de qualquer cache que voce ja escreveu na vida. o que dava pra escolher, e onde eu acho que esta a parte interessante, e onde colocar a fronteira e qual chave usar.

a fronteira virou a layer, porque ela ja carregava o estado certo: textura, buffers, ordem. a chave virou o hash da cena via `fxhasher`, que e rapido o bastante pra rodar todo frame sem aparecer no perfil. e o cache virou a textura offscreen, que eu ja tinha de manter pro composite de qualquer forma. as tres pecas ja existiam por outros motivos, o dirty tracking so as costurou.

tem um detalhe de implementacao que fecha o ciclo: o buffer por tras de tudo isso, o `gpuvec`, e grow-only e faz escrita parcial via `queue.write_buffer`. ele cresce e nunca encolhe, e quando uma layer muda so um pedaco, so esse pedaco e reescrito. e o mesmo `gpuvec` compartilhado entre o compositor, por layer, e o sistema de texto. dirty tracking decide se a layer trabalha, o `gpuvec` decide o quao pouco ela escreve quando trabalha.

no fim o que me deixou satisfeito nao foi numero de fps. foi o silencio. layer parada, a engine nao faz nada, e isso e exatamente o que deveria acontecer. levou um tempo pra eu confiar que "nao fazer nada" era o comportamento correto e nao um bug de tela congelada. era so a engine sendo preguicosa do jeito certo.

## rastros

- decisao e arquitetura: `kdb/adr/layer-system.md` (premultiplied alpha, dirty tracking per-layer, textura offscreen, composite pass, `gpuvec` compartilhado)
- numero do dirty tracking: 3.3us/1000 nos, criterion no mac m4, registrado em `kdb/adr/benchmark-results.md`
- task de origem: task-07, layer system + composite pass, em `kdb/mission/readme.md`
