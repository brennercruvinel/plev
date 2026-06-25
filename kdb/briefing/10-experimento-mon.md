---
title: experimento mon (as 30 paginas abertas)
status: aguardando revisao, trabalho vivo
tags: [mon, lottie, monster, swf, flash, motion-ui, design-system]
---

# experimento mon

este e o trabalho que voce esta fazendo agora. o livro reserva ~30 paginas
abertas para ele, porque ainda esta vivo. nao fechar, nao concluir, manter como
laboratorio em publico.

## o que e (ancorado no repo real)

- crate `lot`: importer de lottie (bodymovin). serde model + keyframe eval +
  render hierarquico para TessellatedPath. e a porta de saida `cnv`: amostra uma
  vez, deduplica payloads, descobre deltas, codifica `.monster`. playback roda so
  no monster.
- crate `monster`: codec binario de animacao (.monster). ir/validate/write/read,
  delta ops completos, descoberta de delta a partir de frames amostrados,
  optimizer no encoder (collapse estatico, reducao rdp de keyframe, fusao
  colinear), player deterministico tick-driven que baixa ir para SceneNode.
- crate `parser`: transpiler poc (tsx/sass e gpui -> builder plev) com droplist
  honesto file:line. e a materializacao do "grafo de equivalencias".
- `notes.md` na raiz: o braindump cru do estudo. ja contem o argumento central.

## o argumento (de notes.md, a ser reconstruido com brennerwritter)

a ideia: abstrair ui por superset com mapeamento para widgets nativos. listar o
vocabulario de cada plataforma, achar a intersecao (o grafo de similaridades),
definir um declarativo unico sobre a intersecao, e ter dois modos de saida:
desenha na gpu (identico em todo lugar) ou mapeia para o componente nativo.

os buracos que o notes.md ja identifica, com honestidade (isso e o que faz o
capitulo bom):

- a intersecao e pequena, a diferenca e infinita. botao/texto/lista convergem; o
  trabalho real (date picker, gesto de voltar, teclado, permissao, scroll bounce)
  diverge e cresce a cada versao de cada os.
- comportamento nao e mapeavel, so aparencia. flutter, mesmo desenhando tudo,
  teve que criar cupertino e material separados. a unificacao empurra de volta
  para a ramificacao.
- texto nativo nao e desenhar letrinha: selecao, cursor, teclado virtual,
  autocorrecao, copiar/colar, rtl, ime. no modo gpu voce reimplementa esse
  universo. e onde toolkits own-rendering mais sofrem.
- o asterisco do "identico em todo lugar": mobile (ios) e web sao os elos fracos.
  wgpu garante pixel identico de graca? nao. rasterizacao de borda, subpixel do
  texto, blending srgb podem divergir entre metal/vulkan/webgpu. a pergunta de
  estudo: voce valida com snapshot pixel-a-pixel cross-backend, ou confia no
  contrato? e ai que os toolkits serios gastam anos.
- design tokens: o componente nunca usa valor cru, referencia um token (cor,
  espacamento, raio, sombra, tipografia, duracao). e a indirecao que deixa o
  sistema coerente e re-tematizavel.
- a fonte canonica de comportamento neutro de plataforma: o w3c aria authoring
  practices guide (apg). e o grafo de componentes que define o que um combobox E
  e como se comporta, sem dizer como o material pinta. casa com o accesskit.

## a engenharia binaria (swf/flash) que entra nas 30 paginas

estudar a engenharia binaria de swf/flash como referencia historica de motion ui
em formato binario compacto, e comparar com a escolha do `.monster` (deltas
descobertos, payload deduplicado, player deterministico). e um capitulo de
formato binario: por que um codec proprio, o que o swf acertou, o que o lottie
custa em json, e onde o monster se posiciona.

## pesquisa que falta (alimenta este capitulo)

dotlottie-rs, rive (runtime e formato), velato, keyframe, mina. design tokens
cross-platform (style-dictionary, tokens-studio, open-props). aria apg como
grafo. tudo ja parcialmente listado em `refs/animation-motion.md` e no notes.md.

## como tratar (fase 8)

trabalho vivo, baixa pressao de fechamento. um agente reconstroi o notes.md com
brennerwritter (perfil cientifico/timeline) preservando o argumento e os buracos
honestos. nao transformar em manifesto vitorioso: o valor e a honestidade sobre o
que e dificil. as 30 paginas ficam editaveis enquanto o experimento anda.
