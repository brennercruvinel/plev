---
id: adr-0022
title: "one knowledge tree, and a minimal root"
project: plev
type: decisao
status: aceito
date: 2026-07-13
date_basis: cabecalho
topics: [editorial, workspace]
relations: [adr-0013, changelog, experimento-mon, manual-build-and-serve-the-web-target]
source: docs/adr/structure/0022-one-knowledge-tree-and-a-minimal-root.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# one knowledge tree, and a minimal root

Decide uma única árvore de conhecimento em `docs/` e uma raiz que guarda só o que as ferramentas resolvem ali. Antes, o conhecimento vivia em três lugares ao mesmo tempo (`kdb/`, `doc/arc/` e a raiz), com um changelog duplicado. O resultado é que o leitor começa em dois pontos: o README para a superfície e `docs/` para a profundidade. Os caminhos do corpo são os da época da decisão, e a correção de nomes de 2026-09-21 está no próprio texto.

correction (2026-09-21): the decisions directory is docs/adr/ (upper
case, the acronym), its index is docs/adr/README.md so github renders it
on the directory page, and the changelog is [docs/historia/changelog.md](../../historia/changelog.md) at the
knowledge tree's root. the paths below are as written when the decision
was taken.

## context

knowledge lived in three places at once: kdb/ held the decisions, manuals,
mission and study references; doc/arc/ held the architecture trio; and the
root carried loose files, notes.md (the mon experiment working note) and a
changelog that existed both as doc/changelog.md and kdb/adr/changelog.md,
byte-equal except for the frontmatter date. the root also carried the trunk
entry index.html, a Makefile whose targets were one cargo call each, and an
.env.example with nothing to configure. every extra top-level entry has the
same cost: one more place a reader checks before trusting they saw the
whole picture, and one more path for the sync guards and the instruction
files to track.

## decision

one tree for knowledge, one directory per concern, and a root that keeps
only what tools resolve there.

- docs/ is the single knowledge tree, absorbing kdb/ under the name github
  and the wider ecosystem render by default: adr/ (decisions), arc/ (the
  architecture trio, moved from doc/arc/), how-to/ (operating manuals),
  mission/ (goals, rules, task history), refs/ (study notes). doc/ is
  retired; the changelog copy it held was the stale one, docs/adr/
  changelog.md is the living one. the mon working note left the root and
  lives as study notes in
  [docs/experimentos/mon-notas.md](../../experimentos/mon-notas.md) (moved there from steps/ongoing/ on
  2026-09-25, with the program, the thesis and the guidelines moving up to
  docs/log/); a loose notes.md is
  not a place.
- web/ owns the whole web target: Trunk.toml, index.html, and the dist/
  output. script/ owns the repeatable commands: tool/tasks/fullcheck.sh (the four-part
  gate, stops on the first red) and tool/tasks/webtarget.sh (trunk --config web serve).
  a directory earns its place by owning a concern end to end, not by
  holding one stray file.
- the root keeps the files whose tools require them at the top: Cargo.toml
  and Cargo.lock (workspace), README.md and LICENSE (github renders both
  from the root only), rustfmt.toml, rust-toolchain.toml, .editorconfig,
  .gitattributes, .gitignore, and the dot directories (.cargo, .contracts,
  .github, .zed). the Makefile and .env.example are retired. the
  instruction source for agents and contributors is
  .contracts/.agents/AGENTS.md alone, with no root AGENTS.md or CLAUDE.md.
- docs reference external study material by repository name plus the path
  inside that repository, never by machine path. a clone location is
  developer-specific; a name is reproducible on any machine.

paths updated with the move: tool/tests/test_repoaudit.rs reads
docs/, the canonical AGENTS.md and the README point at docs/, the pr
template names the new paths, typos.toml excludes the bilingual docs
subtrees one by one so docs/ (english-facing) stays spell-checked, and
every crate docstring that cited kdb/adr/ now cites docs/adr/.

## consequences

- a reader learns the repo from two entry points: README for the surface,
  docs/ for the depth. there is no third place.
- the arc trio sits next to the decisions it summarizes; a structural
  change touches docs/ and docs/adr/ in one tree, one commit.
- the workspace-layout adr
  (workspace-engine-at-root-libs-in-crates-demos-in-examples) keeps its
  three tiers; this adr amends where the docs about them live, not the
  tiers themselves.
- relative links inside the tree survive the rename unchanged; only the
  external pointers (guard test, instruction source, readme, pr template,
  typos config, crate docstrings) moved, and the arc sync guard fails the
  build if the trio drifts from Cargo.toml again.
- the web target relocates its build output to web/dist/; anything serving
  the old root dist/ path follows [docs/referencia/operacao/build-and-serve-the-web-target.md](../../referencia/operacao/build-and-serve-the-web-target.md).
