---
type: reference
tags: [conventions, naming, testing, hygiene, docs]
date: 2026-06-11
status: living
---

# conventions

the operating contract lives in AGENTS.md (single source for agents and
contributors). this file is the quick card.

- tests: real, no mocks; happy path, error path, one edge case minimum;
  visual claims need pixel measurements
- backend before ui: logic implemented and tested before pixels
- naming: kebab-case dirs/docs/assets; idiomatic source files; apl-style
  3-char tokens where unambiguous
- hygiene: 369 hard line limit per source file, ~220 target for new files
- docs: diataxis, lowercase, yaml frontmatter, no emojis, no em-dash;
  wrong notes get a correction on top, never deleted
- architecture: doc/arc/{arc.md, arc.yaml, arc.mmd} update together with
  any structural change
- commits: plain prose, body explains why; agents never commit
- engine rules: kdb/how-to/code-against-the-plev-engine.md
