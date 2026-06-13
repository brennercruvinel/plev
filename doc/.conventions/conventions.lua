return {
  id  = "conventions",
  typ = "contract",
  sts = "living",
  dom = "process",
  dat = "2026-06-13",
  ttl = "the operating contract: tests, naming, hygiene, docs, sync",
  txt = [[
the single instruction source is AGENTS.md; this file holds the
conventions it points to. read it at task start, follow it, and update it
whenever a new convention is established. lua so it parses with luajit and
reads as a graph node, the project's own format.
]],

  -- every change ships real tests, no mocks: happy path, error path, one
  -- edge case minimum, against real artifacts (rendered scenes, measured
  -- pixels, golden fixtures). nothing merges without executable proof.
  -- visual claims require pixel measurement, not "looks the same"
  -- (kdb/how-to/validate-visuals-by-pixel.md).
  tests = "real, no mocks; happy + error + one edge min; pixel-measure visual claims",

  -- no ui before the logic behind it is implemented and tested. chart
  -- geometry, state machines, codecs, parsers: pure modules with unit
  -- tests first, pixels second.
  backend_before_ui = "logic implemented and tested before pixels",

  -- directories, docs and assets: kebab-case english. source files:
  -- idiomatic to the language (rust module dirs stay snake_case). prefer
  -- short, intuitive, global names; never sacrifice clarity for brevity.
  -- propose renames as mv commands, fix every touched import, run the suite.
  naming = "kebab dirs/docs/assets; idiomatic source; short global names; renames as mv + fix imports + test",

  -- hard limit 369 lines per source file; target ~220 for new files. a
  -- file created or modified in a session that exceeds the limit is read
  -- in full and split along single-responsibility lines. generated files
  -- and lockfiles are exempt.
  file_hygiene = "369 hard limit, ~220 target; split oversize by single responsibility",

  -- diataxis style. all lowercase except acronyms. no emoji. no em-dash
  -- (comma, semicolon, period or hyphen). no decorative markdown. every
  -- doc opens with yaml frontmatter (type, tags, date, commit or status).
  -- a design note that turns out wrong gets a correction on top, never a
  -- delete.
  docs = "diataxis, lowercase, yaml frontmatter, no emoji, no em-dash; wrong notes corrected on top, not deleted",

  -- after any change to structure, boundaries, data flow, module layout,
  -- public contracts or runtime behavior, update doc/arc/{arc.md, arc.yaml,
  -- arc.mmd} in the same change. keep them concise. one architecture doc,
  -- no parallel second.
  arc_sync = "update doc/arc/{arc.md,arc.yaml,arc.mmd} together with any structural change",

  -- keep README.md current after user-facing changes (new crate, new run
  -- command, new format behavior). keep this conventions file current when
  -- a new convention is established.
  doc_sync = "README current after user-facing changes; this file current on a new convention",

  -- on finishing a task: full audit of every session change, no summary,
  -- from devops, code quality and secops angles. write a temporary markdown
  -- manifest under tmp to track executed tasks. find dead code, stale
  -- generated files, items in wrong folders; fix or report. run tests after.
  audit_on_finish = "full session audit (devops/quality/secops) + tmp manifest; fix or report; run tests",

  -- commit messages: plain english or portuguese, no conventional-commits
  -- prefix; body explains the why, the diff shows the what. agents do not
  -- commit; the orchestrator commits thematically. code comments state
  -- constraints the code cannot show, nothing else.
  style = "commit body explains why; agents never commit; comments state constraints only",

  -- engine rules live in the manual; the short list is in AGENTS.md.
  engine_manual = "kdb/how-to/code-against-the-plev-engine.md",
}
