---
id: dc7b458c0fde
source: /Volumes/500G-SSD/claude2026/projects/-Volumes-HOFF-dev-777-brainstorming-demo-web/18929a30-0ddf-4451-88ae-c0bf9276dfe2/workflows/scripts/refactor-love-stable-map-wf_7c6615cf-b7c.js
captured: 2026-06-02T22:06:35Z
model_tier: desconhecido
project: outro
kind: trajetoria
turns: 7
scrubbed: false
status: reconstruido
tags: [777, love2d, lua, graph-viewer, deterministic-layout, round-trip-serialization, workflow-orchestration, no-emoji-rule]
---

# refactor-love-stable-map, viewer de grafo estavel em LÖVE

workflow script do projeto 777 que refatora o viewer do grafo de brainstorming
pro padrao que o usuario escreveu a mao. cinco fases encadeadas (Spec, CoreLua,
Love, Verify, Review), uma chamada de agente por fase, cada uma alimentando a
proxima. a ordem input -> raciocinio -> tool-call -> resposta esta na sequencia
das fases.

## intencao (input humano reconstruido)

refator do viewer pro padrao que eu escrevi. mapa estavel em LÖVE, lua puro,
zero js. layout deterministico cacheado como dado, cor e significado, rotulo
sempre, ego, proveniencia, e edicao com escrita de volta (round-trip
serializavel). reusa engine.lua, view.lua e lib.lua que ja rodam e ja foram
verificados, nao reescreve esses.

regras duras: lua nativo via love2d, nada de javascript, wasm ou DOM. o js
antigo em web/, web-force/, web-cosmos/ nao se toca nem se deleta, e evidencia
minha. dado e o 3ch.lua curado (array de nos: id, typ, sts, dom, ttl, lnk,
rep/doc/dat), nao sobrescreve esse arquivo. comentario em pt-br minuscula. a
regra que eu meco: nada de em dash e nada de emoji no codigo nem no comentario,
usa virgula, dois pontos, parenteses. e nada de teatro: sem animacao de
abertura, o layout calcula uma vez, deterministico, cacheado, e o mapa aparece
estavel na hora.

o padrao, a verdade do alvo, em tres eixos:

ver: nao pode virar hairball. cor e significado (faceta). sts sentido por
opacidade e tracejado, nao por consulta. zoom semantico: longe mostra hubs mais
rotulo de cluster, perto mostra texto, endereco e links. o trigrama entre
colchetes sempre legivel, ele e a identidade, o ponto, nao decoracao.
proveniencia no no (rep/doc/dat reais, abre com uma tecla).

editar: edita o no onde ele esta (txt, lnk) sem sair do contexto. arrasta no
pra no cria aresta e pergunta o tipo (ctx, vs, par). tudo escreve de volta no
lua, round-trip. a viz e uma lente que se escreve atraves, nao read-only.

sentir: carga cognitiva baixa, escuro, alinhado, sem chartjunk, sem emoji.
determinismo, o mapa estavel, cada grupo sempre no mesmo lugar, e o mecanismo
da memoria muscular, nao estetica. confianca, a ligacao mostrada e real e
tipada. e instantaneidade.

## trajetoria (ordem preservada)

### fase 1 / Spec (tool-call: agent)

trava a spec de build em markdown denso pt-br. define os arquivos alvo em
demo/love/ (conf.lua, main.lua, mklayout.lua que gera o cache, layout.lua que e
o cache gerado, ser.lua serializador de volta, out.lua alvo do write-back que
nao e o 3ch.lua), o fluxo de layout (mklayout roda em luajit, dofile do 3ch,
engine.build, engine.step ate assentar, escreve layout.lua; main.lua so carrega
o cache, instantaneo, sem fisica na tela), o render love.graphics, a interacao
e o contrato de honestidade (a janela GL nao roda headless, so a logica pura e
verificavel). schema SPEC: spec_md.

### fase 2 / CoreLua (tool-call: agent)

implementa a camada lua pura, verificavel em luajit sem love. mklayout.lua
gerando layout deterministico em %.3f (rodar 2x da byte-identico). ser.lua com
serialize(nodes) que produz lua valido preservando lnk v1 (lista) e v2 (mapa),
e round-trip (dofile da saida reconstroi nos equivalentes, mesmas arestas via
view.edges_of). o agente roda a verificacao real e cola a saida: sintaxe via
luajit loadfile, determinismo via md5 do layout em duas rodadas, round-trip
contando nos e arestas. schema BUILT: files, verification, passed, notes.

### fase 3 / Love (tool-call: agent)

implementa o backend LÖVE consumindo a camada pura. conf.lua (janela escura,
vsync) e main.lua: love.load monta package.path via getSource, carrega
layout.lua sem rodar fisica, monta paleta por faceta. love.draw desenha arestas
(semantica colorida, estrutural cinza fraca), rotulos de faceta, nos (raio por
sqrt do pagerank, cor por faceta, opacidade por sts), trigrama sempre, declutter
por zoom, tooltip no hover. input: wheel zoom no cursor, drag pan, clique no no
chama engine.ego, tecla abre proveniencia via love.system.openURL, modo criar
aresta que muda lnk em memoria e chama ser.write pra out.lua. verificacao: luajit
loadfile parseia (load real precisa de love, que nao roda headless, e isso e
declarado como esperado e honesto). schema BUILT.

### fase 4 / Verify (tool-call: agent)

verificacao headless real, cola saida verdadeira. sintaxe de engine.lua,
view.lua, lib.lua e dos arquivos love/. determinismo (mklayout duas vezes,
layout.lua byte-identico por md5). round-trip do serializador (nos e total de
arestas batem). confirma que main.lua nao importa js e carrega o cache (sem step
de fisica no draw/update do caminho normal). confirma que 3ch.lua nao foi
modificado. schema VER: results, all_passed, failures.

### fase 5 / Review (tool-call: agent)

review adversarial cetico, le main.lua, ser.lua, mklayout.lua. checa contra o
padrao: sem animacao de abertura (o mapa carrega estavel do cache, sem tremor de
fisica na tela, defeito grave se houver), trigrama sempre legivel, cor por
faceta de verdade, sts por opacidade e tracejado, determinismo real (cache, nao
re-layout a cada abertura), write-back valido e re-carregavel em out.lua e nao
no 3ch.lua, proveniencia acessivel, ego chamando o engine de verdade, zero js,
zero em dash ou emoji vazado (o usuario mede), e honestidade (o codigo nao finge
que a janela GL foi testada). schema REV: issues[] (severity, file, problem,
fix) e verdict.

### resposta (return)

spec_files (os seis arquivos love/), core e love (passed, files, verification,
notes de cada), e os objetos verify e review completos.

## raciocinio que o script codifica

o eixo do design e determinismo como funcao, nao enfeite: posicao fixa por
grupo e o que constroi memoria muscular, entao o layout vira dado cacheado e a
abertura nunca re-roda fisica. o segundo padrao e o contrato de honestidade
explicito: a janela GL nao roda headless, entao so a camada pura (layout
deterministico, round-trip do serializador, sintaxe) e verificavel, e o script
exige que o agente diga isso em vez de fingir que testou a tela. o terceiro e a
disciplina anti tell que o proprio usuario impoe no codigo: proibido em dash e
emoji, medido no review, o mesmo mantra que rege este pipeline de dataset.

## reconstrucao (log)

- fonte do sinal: as strings de prompt do workflow (meta.description, a
  constante CTX com o bloco "padrao escrito pelo usuario", e os prompts das
  cinco fases)
- reconstruido: a intencao e o padrao de design na voz do brenner, minuscula,
  sem em dash, sinal preservado, sem inventar conexao. o bloco do padrao do
  usuario foi limpo de marcadores em caixa alta e reescrito em prosa, mantendo
  os tres eixos (ver, editar, sentir) e cada criterio concreto
- a versao bruta (codigo do script) nao e republicada, so a trajetoria
  normalizada e este log

## scrub (log)

- categorias avaliadas: secrets/tokens, emails, usernames, nomes proprios, IPs,
  telefones
- removido: nada. nenhuma PII sensivel. os paths sao diretorios de projeto em
  /Volumes/HOFF/dev/777, drive externo, sem usuario exposto
