---
type: adr
status: accepted
tags: [input, touch, mobile, gestures, events]
date: 2026-06-10
commit: 5854941
---

# touch input synthesizes pointer events into the existing mouse path

## context

the engine carried a full touch stack (multi-finger tracker, recognizers
for tap, double tap, long press, drag, pinch, swipe) whose output was
drained into a `log::debug!` call and discarded. `WidgetEvent` only knew
mouse variants. on iOS or android the apps would have launched and ignored
every touch. the alternative designs were (a) a parallel touch event
vocabulary for widgets, or (b) translation of the primary touch into the
pointer events widgets already understand.

## decision

design (b). a pure state machine (`TouchPointerSynth`) translates the
primary finger into synthetic pointer events injected into the same
`InputState` functions that real mouse input uses:

- touch started: cursor moved to position, then primary button down
- touch moved: cursor moved
- touch ended or cancelled: button up, then cursor left (a lifted finger
  must not leave a hover state behind; a cancel must not leave a widget
  stuck in pressed state)
- secondary fingers are ignored by the synthesizer; multi-finger gestures
  remain the recognizer's domain

## consequences

- every existing widget gained tap, drag and focus behavior with zero
  widget changes, because hit testing, hover and click dispatch are the
  same code that mouse input exercises
- the synthesizer is pure and unit tested, including finger promotion and
  cancel semantics
- higher-level gestures (pinch zoom, long-press menus) remain available
  through the recognizer when a feature needs them explicitly

## avoid

- do not invent a parallel widget event vocabulary for a new input device
  when a faithful translation into the existing one exists. every widget
  would need to opt in, and most would never be updated
- do not leave recognized input in a debug log. an input pipeline that
  terminates in a log statement is indistinguishable from missing support
  at the product level
