---
type: adr
status: accepted
tags: [fonts, cosmic-text, typography, fontdb, rubik]
date: 2026-06-10
commit: 21fdc8c
---

# embed every font weight the UI uses, and pin the default families

## context

the UI requested inter at weights 500, 600 and 700 while only the 400 face
was embedded. cosmic-text resolves a missing family+weight pair by falling
back to system fonts (apple SD gothic neo, menlo on the test machine),
whose advances ran up to 35% wider. the symptom was severe: titles rendered
visibly letter-spaced ("c a r d s") and centered text overflowed. the
failure was initially misread as a letter-spacing bug and as a layout bug;
both readings were wrong.

a second iteration replaced inter with rubik after measuring the live
reference site, which loads rubik through next/font.

## decision

- every weight the UI can request (400, 500, 600, 700) is embedded via
  `include_bytes!` and registered at startup (src/text/fonts.rs)
- the default sans-serif family is pinned with
  `db.set_sans_serif_family("Rubik")` and monospace with
  `set_monospace_family`. resolution never depends on host fonts
- static faces are instanced from the upstream variable font with
  fonttools when the foundry does not ship statics
- a kill test protocol validates causality: reverting the embed makes the
  named regression tests fail with the exact production symptom

## consequences

- rendering is deterministic across machines, CI and wasm (fontdb starts
  empty on web and mobile; embedded faces are the only faces)
- binary size grows by the embedded faces. accepted: correctness of
  typography is a core product property in this project
- tests pin the resolved face per weight
  (`default_family_resolves_rubik_faces_for_all_ui_weights`)

## avoid

- never assume a family registered at one weight covers other weights.
  cosmic-text only honors the requested family on an exact weight match
- never diagnose "spaced out text" as a tracking bug before checking which
  face actually resolved. log or test the resolved face first
