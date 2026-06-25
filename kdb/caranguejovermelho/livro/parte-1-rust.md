---
title: rust de verdade
parte: 1
status: stub
rastros: []
---

# parte 1, rust de verdade

espinha de secoes. so a estrutura, sem corpo. cada conceito de rust sai de um
arquivo do plev que existe e compila, ancorado no adr que registrou a decisao.

## 1.1 ownership e borrow pelo GpuVec e pelos buffers persistentes

### o conceito em linguagem humana
### o codigo real, GpuVec e write parcial
### o porque arquitetural, render-on-demand-requires-explicit-invalidation

## 1.2 traits como contrato, View, Lifecycle, Interpolate

### trait como interface minima
### View e ViewContext sem ref ao compositor (view-trait-design)
### Lifecycle separada de View (component-design)

## 1.3 edition 2024 na pratica

### o que mudou da 2021 para a 2024
### #[unsafe(no_mangle)] e os entrypoints por plataforma
### workspace-engine-at-root-libs-in-crates-demos-in-examples

## 1.4 erro como valor

### Result, Option, o tipo que carrega o erro
### a limpeza de lint, error-handling-lint-cleanup
### zero warnings, clippy-zero-warnings

## 1.5 modularizar por responsabilidade

### o limite das 300 linhas por arquivo
### adr-003-srp-modularization, 44 monolitos em submodulos
### armadilhas do processo, srp-modularization

## 1.6 workspace virtual, a crate como fronteira

### Cargo.toml virtual, members e workspace.package
### engine como crate, apps e libs irmas
### crate boundary, o que cruza e o que nao cruza

## 1.7 async sem runtime pesado

### futures sem tokio, o init da gpu
### eventloopproxy para a inicializacao assincrona
### async-gpu-init-and-single-wasm-entry
