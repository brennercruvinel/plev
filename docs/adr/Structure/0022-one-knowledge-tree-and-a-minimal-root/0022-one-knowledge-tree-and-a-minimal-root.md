---
id: adr-0022
title: "one knowledge tree, and a minimal root"
project: plev
type: adr
status: accepted
date: 2026-07-13
date_basis: header
recorded: 2026-07-13
topics: [editorial, workspace]
relations: [adr-0013, changelog, research-mon-experiment-notes, operations-build-and-serve-the-web-target]
source: docs/adr/structure/0022-one-knowledge-tree-and-a-minimal-root.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# one knowledge tree, and a minimal root

Decides on a single knowledge tree in `docs/` and a root that keeps only what the tools resolve there. Before, the knowledge lived in three places at once (`kdb/`, `doc/arc/` and the root), with a duplicated changelog. The result is that the reader starts at two points: the README for the surface and `docs/` for depth. The paths in the body are those of the time of the decision, and the name correction of 2026-09-21 is in the text itself.

correction (2026-09-21): the decisions directory is docs/adr/ (upper
case, the acronym), its index is docs/adr/adr.md so github renders it
on the directory page, and the changelog is [docs/log/changelog.md](../../../CHANGELOG.md) at the
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
  [docs/Research/Importing/mon-experiment-notes.md](../../../Research/Importing/mon-experiment-notes.md) (moved there from steps/ongoing/ on
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
  the old root dist/ path follows [docs/Research/Workflows/build-and-serve-the-web-target.md](../../../Research/Workflows/build-and-serve-the-web-target.md).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0013: engine at root, libraries and apps in crates, demos in examples](../0013-workspace-engine-at-root-libs-in-crates-demos-in-examples/0013-workspace-engine-at-root-libs-in-crates-demos-in-examples.md)
- [mon experiment](../../../Research/Importing/mon-experiment-notes.md)
- [how to build and serve the web target](../../../Research/Workflows/build-and-serve-the-web-target.md)
<!-- /related -->
