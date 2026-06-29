---
title: medir, nao achar
parte: 5
status: stub
rastros: []
---

# parte 5, medir, nao achar

espinha de secoes. so a estrutura, sem corpo. benchmarks criterion, notebooks
jupyter e o caminho ate o paper arxiv. cada numero mostra como foi obtido.

## 5.1 por que medir, criterion e harness false

### criterion com harness false, o setup
### benchmark-results, m4 mac como baseline
### medir, nao achar

## 5.2 rect throughput, o numero de capa

### push_rects, 159-222m rects/s
### o que o numero significa e o que nao significa

## 5.3 scene build, o custo de montar a scene

### engine/scene_build.rs
### nb-scene-build, do bench ao notebook

## 5.4 dirty tracking, custo por layer

### layer-system e o custo de 3.3us / 1000 layers
### nb-dirty-tracking

## 5.5 rope edit, build mais insert/delete roundtrip

### rope/edit.rs
### nb-rope-edit

## 5.6 tessellation, microssegundos por shape

### lyon, 1.5-3.7us / shape
### nb-tessellation

## 5.7 signals, nanossegundos por cycle

### signal-system-design e 67ns / cycle
### nb-signals

## 5.8 codec, convert e transpile, os benches de borda

### monster/codec.rs, lot/convert.rs, parser/transpile.rs
### nb-monster-codec, nb-lot-convert, nb-parser-transpile

## 5.9 notebooks jupyter, a regra da reprodutibilidade

### kernel limpo do topo ao fim
### hardware, so, rust e versao da crate declarados
### divergencia entre rodadas registrada, nao mascarada

## 5.10 do benchmark ao paper arxiv

### arxiv-paper-outline, as 11 secoes
### arxiv-paper-draft, do rascunho ao texto
### honestidade de marketing vs ganho real
