---
title: seo, wasm e desafios
parte: 6
status: stub
rastros: []
---

# parte 6, seo, wasm e desafios

espinha de secoes. so a estrutura, sem corpo. descoberta por ai, json-ld @graph,
crawlability de wasm, ssr e pre-render. parte das ancoras vive no blog (zola),
nao em adr, e isso fica marcado no texto.

## 6.1 descoberta por ai, nao so por humano

### o crawler mudou, o leitor mudou
### o que torna conteudo legivel para modelo

## 6.2 json-ld @graph, o grafo da serie building plev

### @graph como mapa de entidades
### linkar post, capitulo, adr e diff

## 6.3 crawlability de wasm, o conteudo que o crawler nao ve

### wasm-webgpu-validation, o bundle opaco
### o custo de servir uma tela que so a gpu desenha

## 6.4 ssr e pre-render, servir html antes da gpu acordar

### async-gpu-init-and-single-wasm-entry, o gap de init
### html primeiro, canvas depois

## 6.5 a entrada wasm unica e o custo de bundle

### web-entry vs android-entry, uma porta por target
### o build de 2.4mb e o que cabe cortar
