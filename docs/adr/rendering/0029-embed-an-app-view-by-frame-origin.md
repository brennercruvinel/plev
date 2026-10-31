---
id: adr-0029
title: "embed an app view by giving it a frame origin"
project: plev
type: adr
status: accepted
date: 2026-09-21
date_basis: header
topics: [layout, platform]
relations: [adr-0026]
source: docs/adr/rendering/0029-embed-an-app-view-by-frame-origin.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# embed an app view by giving it a frame origin

Decides that an embedded view receives a frame with an origin (`set_frame`), not its own `begin_frame`: the view that owns the window keeps `vw` and `vh`, and the host passes the rectangle. The reason: the showcase needed to host the editor and the git client, and the ide workspace drew from (0, 0). The result is that the editor and git become app sections, and the limitations are made explicit: the ide header, the modal dialog and the toasts still center on the window.

## context

the showcase is the place where a comps change is seen everywhere at
once, so it had to host the editor and the git client. the editor view
already took a `Rect`; the ide workspace did not: it drew from (0, 0)
with `vw` / `vh` as the window, called `begin_frame` itself, and its
rail (`comps::nav::Sidebar`) only existed at the window's top-left.

## decision

- a view that owns a window keeps `vw` / `vh`; a host embedding it gives
  it a frame: `WorkspaceView::set_frame(x, y, w, h)` sets `origin`,
  flags `embedded` (no header of its own; the rail stays) and resizes.
  input arrives in window coordinates and every hit test adds the
  origin; the modal viewport becomes `origin + size` so its scrim also
  dims the host.
- `begin_frame` belongs to the host, never to a view.
- `comps::nav::Sidebar.origin`: the rail's rect starts there.
- `ide` is a lib + bin; the git plumbing (`UiRequest` <-> `GitCommand` /
  `GitEvent`, the diff target, the log and branch caches) is
  `ide::session::GitSession`, used by the ide binary and the showcase.
- a worker wakes the showcase loop through a `Wake` callback
  (`UserEvent::Wake`), the pattern of adr
  worker-results-wake-the-event-loop.
- Editor and Git are app sections: they fill `app_rect` (sidebar edge to
  window edge, header to bottom), no page gutter.

## consequences

- the ide header, `Modal::dialog_rect` and `ToastManager` still center
  on a window size, not on a rect; embedding them fully is the next step.
- `comps::editor::EditorView::handle_key` takes a winit key, so the
  "winit-free" view now imports winit for that one route.
