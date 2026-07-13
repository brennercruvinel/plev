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
root carried loose files, notes.md (the scratch thesis note) and a changelog
that existed both as doc/changelog.md and kdb/adr/changelog.md, byte-equal
except for the frontmatter date. the root also carried the trunk entry
index.html. every extra top-level entry has the same cost: one more place a
reader checks before trusting they saw the whole picture, and one more path
for the sync guards and the instruction files to track.

## decision

one tree for knowledge, and a root that keeps only what tools resolve there.

- kdb/ is the single knowledge tree: adr/ (decisions), arc/ (the
  architecture trio, moved from doc/arc/), how-to/ (operating manuals),
  mission/ (goals, rules, task history), refs/ (study notes), notes.md
  (moved from the root). doc/ is retired; the changelog copy it held was
  the stale one, kdb/adr/changelog.md is the living one.
- the trunk entry lives at web/index.html; Trunk.toml points at it and
  keeps emitting dist/ from the root, so `trunk serve` still runs from the
  root and nothing changes for the browser.
- the root keeps the files whose tools require them at the top: Cargo.toml
  and Cargo.lock (workspace), README.md and LICENSE (github renders both
  from the root only), Trunk.toml, rustfmt.toml, rust-toolchain.toml,
  .editorconfig, .gitattributes, .gitignore, and the dot directories
  (.cargo, .contracts, .github, .zed). everything else is a named subtree:
  script/ holds the gate runner (the Makefile is retired; the commands it
  wrapped are one cargo call each, documented in the README), and the
  instruction source for agents and contributors is
  .contracts/.agents/AGENTS.md alone, with no root AGENTS.md or CLAUDE.md.
- docs reference external study material by repository name plus the path
  inside that repository, never by machine path. a clone location is
  developer-specific; a name is reproducible on any machine.

paths updated with the move: crates/engine/tests/arc_sync_guard.rs reads
kdb/arc/, the canonical AGENTS.md and the README point at kdb/arc/, the pr
template names the new paths, and typos.toml excludes the bilingual kdb
subtrees one by one so kdb/arc/ (english-facing) stays spell-checked.

## consequences

- a reader learns the repo from two entry points: README for the surface,
  kdb/ for the depth. there is no third place.
- the arc trio sits next to the decisions it summarizes; a structural
  change touches kdb/arc/ and kdb/adr/ in one tree, one commit.
- the workspace-layout adr
  (workspace-engine-at-root-libs-in-crates-demos-in-examples) keeps its
  three tiers; this adr amends where the docs about them live, not the
  tiers themselves.
- relative links inside kdb/ survive unchanged; only the handful of
  external pointers (guard test, agents chain, readme, pr template,
  typos config) moved, and the arc sync guard fails the build if the trio
  drifts from Cargo.toml again.
