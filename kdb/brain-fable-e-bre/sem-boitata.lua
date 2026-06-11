return {
  id  = "sem-boitata",
  typ = "thesis",
  sts = "living",
  dom = "web-semantics-a11y-seo",
  dat = "2026-06-11",
  ttl = "o pecado semantico do flash e os olhos do boitata",
  lnk = { "ths-compilador", "anm-formato", "org-docs" },
  txt = [[
ensaio do brenner (preservado quase verbatim; fundamenta o backend web):

o pecado original do flash nao foi tecnico, foi semantico. o flash morreu
porque era uma caixa opaca dentro de um meio que e, na essencia, um
documento. crawler nao via nada, leitor de tela nao via nada, ctrl+f nao
achava nada, link profundo nao existia, o botao voltar quebrava. o flutter
web canvaskit repete isso: dom praticamente vazio, seo pessimo, e o
proprio time do google admite que flutter web e pra web apps, nao pra
sites de conteudo (removeram o html renderer, dobrando a aposta no
canvas). a critica "novo flash" pegou porque procede.

o diagnostico no vocabulario do pipeline: a semantica foi descartada cedo
demais. quando voce chega nos pixels, a informacao de "isso e um h1, isso
e um link, isso e um botao" ja morreu. nao ha como reconstrui-la depois,
so simula-la com gambiarras (a arvore de acessibilidade paralela que o
flutter projeta em elementos dom invisiveis e exatamente essa gambiarra).

a solucao vem de graca no design do ths-compilador: a arvore de
acessibilidade e derivada do mesmo statechart, outra lowering do mesmo IR.
pois seo e acessibilidade sao primos consumidores da mesma coisa:
semantica. o googlebot e so mais um leitor de tela, um sem olhos, sem gpu
e sem paciencia pra javascript pesado. se o pipeline preserva o papel
semantico de cada no ate o ultimo estagio, atender o crawler e emitir
mais um alvo de compilacao, nao um hack.

concretamente, o backend web faz lowering consciente do meio: o que e
documento (texto, navegacao, formulario, heading) vira html semantico
real: h1, nav, a href, button. o navegador ja e o motor de renderizacao
de texto mais otimizado do planeta, com vinte anos de engenharia: reusar,
nao reimplementar (o argumento do tauri aplicado dentro da propria web).
canvas e webgpu ficam reservados pras ilhas genuinamente graficas: o
grafico, o mapa, o editor, o jogo. arquitetura de ilhas com a base certa:
documento como chao, grafico como ilha, nunca o contrario.

as camadas de seo tecnico colapsam no compile time: como a dsl e total e
analisavel, o compilador avalia a ui em build e emite html estatico com
hydration minima ou nula. a url vira a serializacao do statechart de
navegacao: deep linking, historico e botao voltar derivam do grafo de
estados em vez de serem remendados (no flash o voltar quebrava: sintoma
da mesma doenca). meta tags, open graph, json-ld/schema.org, sitemap,
canonical: tudo projecao do mesmo grafo de conhecimento da aplicacao,
gerado como artefato de build, nao mantido a mao num arquivo paralelo que
dessincroniza. mais: llm.txt para modelos, mermaids do conteudo,
multi-idioma, tudo opcao de cli, com dry-run pra inspecao.

segundo eixo que ninguem liga ao render: performance e fator de ranking.
core web vitals (lcp, inp, cls) punem exatamente o que um framework de
canvas carrega: payload wasm gigante, first paint tardio, tela branca ate
o motor subir. aqui entra o fragmento do qwik: resumability, serializar o
estado no html e nao re-executar no cliente o que o build ja computou. a
mesma lei von neumann aplicada a rede: nao transfira o que pode ser
pre-computado, nao re-execute o que nao mudou.

o meta-padrao ganha mais uma linha: o mcu joga a semantica fora no ultimo
estagio (ninguem indexa termostato), a web materializa ela inteira. mesmo
IR, politicas diferentes por alvo. flash, silverlight e flutter web
falharam porque tinham um so backend e ele era cego. o boitata nao pode
ter olhos so pra gpu, tem que ter olhos pro crawler tambem.

acessibilidade nativa (obrigatoria, nao retrofit): accesskit (at-spi,
nsaccessibility, ui automation) alimentado pela mesma arvore; wcag atual
como teste numerico (contraste medido como medimos gamma); reduce motion
plugado no Tween (de graca pra todo consumidor); legibilidade para
dislexos; animacoes se descrevem: track textual por keyframe gerada por
nlp leve no build (ver anm-formato), narrando o movimento pra quem nao ve.
]],
}
