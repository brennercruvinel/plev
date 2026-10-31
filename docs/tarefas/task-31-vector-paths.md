---
id: task-31
title: "task-31: vector path rendering (lyon integration)"
project: plev
type: tarefa
status: concluido
date: 2023-06-12
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-31-vector-paths.md, docs/log/changes/task-31.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-31: vector path rendering (lyon integration)

## resultado
implementado modulo completo de vector paths via lyon tessellation, integrado com compositor e builder API.

## implementacao

### arquivos criados/modificados
- `src/path.rs` (novo, ~250 LOC), pathbuilder, tessellatedpath, fillvertexconstructor bridge
- `src/compositor.rs`, scenenode::path variant, hash_u64 + rebuild_quad_geometry
- `src/builder.rs`, elementkind::path, path() constructor, flatten + estimate
- `src/lib.rs`, pub mod path
- `examples/paths_demo.rs`, circle, star, rounded rect, ellipse, bezier curves, triangle
- `Cargo.toml`, lyon_tessellation + lyon_path

### features
- **pathbuilder**: move_to, line_to, quadratic_bezier_to, cubic_bezier_to, close
- **convenience shapes**: circle(), rounded_rect(), ellipse()
- **fill + stroke**: fill(), fill_with_tolerance(), stroke(), stroke_with_tolerance()
- **hash estavel**: mesmos comandos = mesmo hash (fxhasher sobre pathcommands)
- **dirty tracking**: scenenode::path usa data.hash no hash_u64()
- **zero shader novo**: fillvertexconstructor<quadvertex> produz vertices identicos ao quad pipeline

### testes (13 novos)
- path.rs: circle/rect/ellipse/line/bezier produzem vertices
- path.rs: hash estavel e diferente para paths diferentes
- path.rs: stroke produz vertices
- path.rs: cores corretas nos vertices
- compositor.rs: scenenode::path participa do dirty tracking
- builder.rs: path element produz path node, path em div funciona

## checklist
- [x] adicionar lyon_tessellation + lyon_path ao cargo.toml
- [x] criar src/path.rs, pathbuilder wrapper + fillvertexconstructor bridge
- [x] scenenode::path variant no compositor
- [x] reusar quad pipeline (mesmo vertex layout, zero shader novo)
- [x] integrar no compositor: path nodes geram geometry via lyon
- [x] dirty tracking para paths (hash dos path commands)
- [x] builder API: path(data) constructor
- [x] testes: 13 unit tests
- [x] exemplo: paths_demo.rs com 7 shapes
- [x] strokevertexconstructor para stroke support

## Registro de entrega

### novos arquivos
- `src/path.rs`, pathbuilder, tessellatedpath, lyon bridge

### modificados
- `src/compositor.rs`, scenenode::path, hash + geometry rebuild
- `src/builder.rs`, elementkind::path, path() constructor
- `src/lib.rs`, pub mod path
- `Cargo.toml`, lyon_tessellation, lyon_path

### novos examples
- `examples/paths_demo.rs`

### testes adicionados
- 11 em path.rs, 2 em builder.rs, 1 em compositor.rs
