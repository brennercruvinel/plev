---
id: adr-0077
title: "the person's composition and clipboard stay the person's"
project: plev
type: adr
status: accepted
date: 2026-09-26
date_basis: header
recorded: 2026-09-26
topics: [input, text-editing, agents]
relations: [task-74, learning-generative-runtime]
source: docs/adr/interface/0077-the-persons-text-and-clipboard-stay-the-persons.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# the person's composition and clipboard stay the person's

Establishes that the text composition (IME) and the clipboard belong to the person: the field keeps the pre-edit apart from the buffer and only records what was confirmed, the recording logs the text commit as `InputKind::Text` without the content, and the clipboard is reached only through the person's keys and the field's menu, where an agent's choice of an item fails. It matters because, before task-74, the field discarded the composition and, in the gallery, cmd+v typed the letter v.

## context

a text field in plev is operated by three actors through one contract:
the person at the keyboard, a screen reader, and an agent through the
port. an input method (Japanese, Chinese) composes text before it commits
it, and the system clipboard holds whatever the person copied anywhere on
the machine. both belong to the person. neither is app state until the
person puts it in a field.

before task-74 phase 4, the field threw the composition away
(`TextInput::handle_ime` ignored the preedit) and no app except the editor
example turned the IME on. no field had copy or paste, and in the gallery
cmd+v typed a literal "v", because the shell sent `Key::Character` to the
view with no modifier check.

## decision

the composition:

- `TextInput` keeps the preedit and its caret apart from the buffer. the
  field draws it inline, in place of the selection or at the caret,
  underlined, with the caret held still where the input method puts it.
- a commit goes in the way typing does (`TextField::insert`, over the
  selection). losing the focus or `Ime::Disabled` drops the composition,
  and the text stays.
- the preedit is only drawn. the field's projected `text`, the snapshot,
  the port and the recording hold what was committed; the recording keeps
  a commit as `InputKind::Text`, never its text. a test reads the bytes of
  the saved session and does not find the preedit in them.
- the gallery turns the IME on only while a field on the Input page or the
  Editor has the focus, and puts the candidate window at the caret after
  each frame (`app/events/ime.rs`). with no field focused, the letters are
  the gallery's shortcuts ("t" cycles the theme), and an active Japanese
  input source would compose them instead. winit 0.30 on macOS only runs a
  key through the input method while the IME is allowed
  (`platform_impl/macos/view.rs:466`).

the clipboard:

- `TextField` copies, cuts, pastes and selects all through a
  `ClipboardProvider` the owner hands in (`ClipKey`), so the field stays
  `Clone`. a paste into a single-line field trims the line breaks at the
  ends, turns the inner ones and tabs into spaces, and drops other control
  characters (`text_input::single_line`).
- the primary modifier is cmd on macOS and ctrl elsewhere. a character
  pressed with cmd or ctrl never types into a field; ctrl with alt is
  AltGr and still types. with no field focused, a chord goes where a bare
  key goes, as before.
- only the person's keys and the field's menu reach the clipboard. the
  right-click menu of a field (Cut, Copy, Paste, Select all) reuses the
  context menu's overlay and keyboard; an agent's pick of one of its items
  fails in the overlay's executor (`dispatch/overlay.rs`), while the
  person and a screen reader go through.
- Paste stays enabled. disabled on an empty clipboard, it told the
  projection, and so an agent and the recording, whether the person's
  clipboard held anything. a paste of nothing does nothing.
- pasted text becomes the field's text, which is app state and shows in the
  projection like typed text. the clipboard itself shows nowhere.
- tests use `LocalClipboard` only and never touch the system clipboard.

## consequences

- the ide gets the clipboard keys in its intent line, the fields of a
  generated surface and the commit form (whose keymap binds `edit::`
  actions for both cmd and ctrl; only the platform's primary acts).
- open: the IME on the gallery's other pages, in the ide and in urnaui;
  the field menu outside the Input page; copy and cut in a generated
  surface's field (they need API in the `surface` crate); and a per-item
  actor policy in the comps `ContextMenu`, so the projection itself says
  the items are the person's only (today an agent sees them as actionable
  and is refused when it acts).
- the IME script on macOS passed in the editor (Brenner, 2026-09-25,
  Japanese and Chinese). the same script in a field of the Input page
  closes the item in task-74.

## alternatives

- colar com a área de transferência vazia desabilitado: rejeitado. A projeção diria se a área de transferência da pessoa tem algo, e isso chegaria a um agente e à gravação; colar sem nada não faz nada (ver `decision`, the clipboard).
- IME ligado o tempo todo na galeria: não adotado. Com nenhum campo focado, as letras são atalhos, e uma fonte de entrada japonesa as comporia (ver `decision`, the composition).

## learnings

- [Generative runtime session: what worked and what bit us](../../Generative/0070-generative-runtime-results/generative-runtime.md)
