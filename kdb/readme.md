---
type: reference
tags: [index, kdb, diataxis, adr]
date: 2026-06-10
commit: bb34a1c
---

# kdb: project knowledge base

knowledge captured from real defects and decisions in this repository,
organized by the diataxis framework. files are named by topic, never by
sequence number. every document carries YAML frontmatter (type, tags,
date, commit) for retrieval by agents and search tooling.

## structure

- `adr/` records why a technical decision was made: context, decision,
  consequences, and what to avoid. ADRs do not explain how; they cite the
  commit that implements them
- `how-to/` task-oriented operating procedures for developers and agents
- `explanation/` background and mental models behind the decisions
- `reference/` canonical tables and lookup material
- `ses/` session records: what was tried and failed, then the path that
  worked, cross-referenced to commits and ADRs

## reading order for a new agent or developer

1. how-to/code-against-the-plev-engine.md (the operating manual)
2. explanation/the-two-gamma-bugs.md and
   explanation/why-the-apps-bypassed-the-engine.md (the two costliest
   defect classes)
3. reference/hoff-visual-tokens.md (the visual contract)
4. how-to/validate-visuals-by-pixel.md (the verification protocol)
5. the ADRs, as needed per subsystem

## conventions for new documents

- english, lowercase prose except acronyms (GPU, sRGB, ADR, API)
- no long dashes, no emojis, scientific register
- one topic per file, kebab-case filename, no sequential numbering
- frontmatter mandatory; `commit` points at the implementing or
  motivating commit
