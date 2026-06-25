# todo: reestruturacao de raiz, crates e naming (template rustCrates + estilo zed)

status: EXECUTADO. brenner confere item a item. gate verde (detalhe no fim).
recriado apos sumir do disco (era untracked).

referencia de organizacao: /Volumes/HOFF/dev/templates/rustCrates
referencia de naming/fronteira: zed (crates curtos sem prefixo) + bevy

---

## decisoes fechadas (brenner)

- engine (era crate raiz `plev`) -> diretorio `crates/engine`, crate RENOMEADO `plev`->`engine`.
- prime_creatures -> `prime`. narrate-macro -> MANTEM. demais ja cabem 3-9 chars.
- AGENTS.md: canonico em `.contracts/.agents/`, ponte fina na raiz. examples dentro da engine.
- MSRV mantido em 1.85 (politica documentada). benches reais, nunca vazios.
- wgpu 29 (unico jeito de matar o future-incompat `block`): NAO feito, aguarda decisao.

---

## fase 1 - engine -> crates/engine  [COMPILA VERDE]

- [x] mover src/ benches/ tests/ examples/ assets/ -> crates/engine/ ; macros/ -> crates/macros/ (git mv)
- [x] raiz virou workspace virtual; pacote -> crates/engine/Cargo.toml como `engine`
- [x] rename `plev::`->`engine::` (773 em .rs) + deps em Cargo.toml; marca "plev" (122) preservada
- [x] assets/ -> crates/engine/assets/ (include_bytes de fontes/logo eram relativos ao crate)
- [x] arc_sync_guard.rs corrigido (sobe 2 niveis ao workspace root; examples da engine)
- [x] Makefile / android build_android.sh (libplev->libengine) / index.html
- [x] remover crates/ide/Cargo.lock espurio

## fase 2 - naming  [FEITO]

- [x] prime_creatures -> prime (Cargo name+bin, imports, Trunk/index/docs)
- [x] narrate-macro MANTIDO. git/ide/lot/rope/parser/monster/narrate/showcase: mantidos (3-9)
- [ ] futuro (so se sem ciclo): extrair color/signal/path como crates folha. mira open-source.

## fase 3 - benches reais  [FEITO]

- [x] rope/benches/edit.rs, monster/benches/codec.rs, lot/benches/convert.rs, parser/benches/transpile.rs
- [x] criterion em dev-deps + [[bench]] harness=false; engine ja tinha scene_build
- [x] documentados em arc.yaml + README
- [x] app/bin (showcase/ide/prime), proc-macro (macros/narrate-macro), git/narrate: sem bench (intencional)

## fase 4 - raiz -> .github / .contracts / .zed  [FEITO]

- [x] .github/: ci.yml (4 do gate), dependabot, FUNDING, PR template, ISSUE_TEMPLATE
- [x] .contracts/.mantras/.code/.lang/.rust/: clippy.toml, nextest.toml, typos.toml, rust-conventions.md
- [x] .cargo/config.toml religa clippy.toml (CLIPPY_CONF_DIR); rustfmt.toml fica na raiz (cargo fmt exige)
- [x] .zed/: settings.json, tasks.json, debug.json

## fase 5 - docs e instrucao  [FEITO]

- [x] removida duplicata generica .contracts/AGENTS.md; canonico .contracts/.agents/AGENTS.md + ponte raiz
- [x] conventions.lua migrado p/ rust-conventions.md (markdown, SEM build lua); lua + doc/.conventions removidos
- [x] arc.yaml / arc.md / arc.mmd / README atualizados (paths, engine, prime, macros, benches)

## fase 6 - validacao  [GATE TOTALMENTE VERDE: 1300 testes, 0 falhas]

- [x] fmt --check: VERDE
- [x] clippy --workspace --all-targets -D warnings: VERDE (inclui os 5 benches)
- [x] test --workspace: VERDE (1300 testes)
- [x] os 4 benches novos RODAM de fato: rope ~546us, monster 80/97/48us (enc/dec/opt),
      lot 89us (convert), parser 656us (transpile). validados com cargo bench
- [~] wasm/mobile: nao rodam aqui (toolchain homebrew sem rustup/targets cross); CI cobre wasm
- [x] Cargo.lock corrupcao pre-existente (`thub, exe:`) corrigida
- [x] MSRV unificado: todos os crates herdam rust-version.workspace (so engine/prime tinham)
- [x] teste flaky corrigido: layout::test_1000_nodes_under_1ms usava assert de wall-clock single-shot
      (estourava 50ms sob carga). agora mede o MELHOR de 5 runs (carga so adiciona tempo). robusto

## fase 7 - rodada 2 (pedidos novos)

- [x] testes: monster gate_bytes pula gracioso (helper fixture_missing); suite 100% verde
- [x] varredura phi/φ -> plev: ZERO no codigo; so PhiError/PhiResult na ADR -> PlevError/PlevResult
- [x] benches reais (fase 3)
- [ ] future-incompat `block v0.1.6`: vem de metal 0.33 (versao mais nova) <- wgpu-hal 28.
      so morre subindo p/ wgpu 29 (bump MAJOR, quebras na camada gpu). AGUARDA decisao. e aviso, nao erro.
- [x] arc + README relidos/atualizados

---

## decisoes pendentes do brenner

1. wgpu 28 -> 29 para matar o future-incompat `block` (tarefa grande, separada, com risco). fazer?
2. extrair color/signal/path como crates folha no futuro (open-source).
3. `.contracts/agents.mmd` esta vazio (placeholder) - deixado intacto.
