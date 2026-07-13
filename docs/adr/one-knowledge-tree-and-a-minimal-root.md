---
type: adr
status: accepted
tags: [workspace, docs, organization, structure]
date: 2026-07-13
---

# one knowledge tree, and a minimal root

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
  lives next to its experiment as
  docs/mission/steps/ongoing/mon-experiment-notes.md; a loose notes.md is
  not a place.
- web/ owns the whole web target: Trunk.toml, index.html, and the dist/
  output. script/ owns the repeatable commands: script/gate (the four-part
  gate, stops on the first red) and script/web (trunk --config web serve).
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

paths updated with the move: crates/engine/tests/arc_sync_guard.rs reads
docs/arc/, the canonical AGENTS.md and the README point at docs/, the pr
template names the new paths, typos.toml excludes the bilingual docs
subtrees one by one so docs/arc/ (english-facing) stays spell-checked, and
every crate docstring that cited kdb/adr/ now cites docs/adr/.

## consequences

- a reader learns the repo from two entry points: README for the surface,
  docs/ for the depth. there is no third place.
- the arc trio sits next to the decisions it summarizes; a structural
  change touches docs/arc/ and docs/adr/ in one tree, one commit.
- the workspace-layout adr
  (workspace-engine-at-root-libs-in-crates-demos-in-examples) keeps its
  three tiers; this adr amends where the docs about them live, not the
  tiers themselves.
- relative links inside the tree survive the rename unchanged; only the
  external pointers (guard test, instruction source, readme, pr template,
  typos config, crate docstrings) moved, and the arc sync guard fails the
  build if the trio drifts from Cargo.toml again.
- the web target relocates its build output to web/dist/; anything serving
  the old root dist/ path follows docs/how-to/build-and-serve-the-web-target.md.
