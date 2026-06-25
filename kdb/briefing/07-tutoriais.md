---
title: tutoriais
status: aguardando revisao
tags: [tutoriais, executavel]
destino: kdb/caranguejovermelho/tutoriais
---

# tutoriais

tutoriais sao o irmao pratico do livro: passo a passo executavel, perfil "doc
tecnica" do brennerwritter (densidade alta, exemplo que roda, sem digressao).
cada tutorial tem um teste de fumaca: roda do zero, compila, da o resultado
prometido. nada de tutorial que so funciona na cabeca de quem escreveu.

## trilhas propostas

| trilha | tutoriais | ancora |
|--------|-----------|--------|
| primeiros passos | rodar o showcase, ler o arc, entender scene node | AGENTS.md, arc.yaml |
| construir contra a engine | um app minimo: view + builder + layout + animation | kdb/how-to/code-against-the-plev-engine.md |
| web/wasm | trunk serve, o asterisco do wasm, validar visual | kdb/how-to/build-and-serve-the-web-target.md, task-20 |
| mobile | android (cargo-ndk + gradle), ios (simulador) | adr android-emulator-deploy, ios-build |
| lottie -> monster | importar bodymovin, descobrir deltas, tocar .monster | crate lot, crate monster, exemplos lottie_player/monster_player |
| transpiler | tsx/gpui -> builder plev, ler o droplist honesto | crate parser |
| validar por pixel | snapshot pixel-a-pixel, o caso #121212 vs #303030 | kdb/how-to/validate-visuals-by-pixel.md |

## regra de qualidade

cada tutorial declara: pre-requisitos, comandos exatos, resultado esperado, e o
que fazer quando quebra (os erros conhecidos do `kdb/mission/rules.md`, ex:
swiftshader trava no android, xcrun arm64 mismatch, binario plev-app vs plev).
o agente que escreve o tutorial roda os comandos antes de entregar, ou declara
explicitamente que nao rodou e por que.
