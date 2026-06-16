---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# pattern extraction: tui apps (yazi, television, bottom)

data: 2026-03-11
status: completo
fase: task-34 fase d

---

## pattern 1: event batching with render throttle (yazi)

**source:** `yazi-fm/src/app/app.rs` lines 27-65

**description:**
yazi pre-allocates a vec with capacity 50 and uses `tokio::mpsc::recv_many(&mut events, 50)` to drain up to 50 events per iteration. after processing each event, it checks an atomic `NEED_RENDER` flag. if render is needed but fewer than 10ms have elapsed since last render, it defers the render via `tokio::select!`, either the 10ms timeout fires (then render), or more events arrive (process those first). this prevents the common problem of rendering after every single keypress in a burst.

```
loop {
    if let Some(t) = timeout.take() {
        select! {
            _ = sleep(t) => { render(); }
            n = rx.recv_many(&mut events, 50) => { drain_events!(); }
        }
    } else if rx.recv_many(&mut events, 50).await != 0 {
        drain_events!();
    }
}
```

the `drain_events!()` macro dispatches each event sequentially and checks `NEED_RENDER` after each one. if elapsed time exceeds 10ms it renders immediately; otherwise it sets a timeout for the remaining time.

**key insight:** the 10ms throttle + batch-of-50 design means rapid keypresses (e.g., holding down arrow) process all pending events before triggering a single render. this is o(1) renders per user-perceivable frame, not o(n) renders per keypress.

**applicability to plev:** plev's event queue (`src/input/mod.rs`) processes events individually in `process_event()`. during rapid touch/keyboard input, each event triggers scene rebuild + GPU submit. a batch-drain approach would let plev accumulate events per frame, process all, then do one compositor.resolve() + render_pass(). this is especially valuable on mobile where touch move events fire at 120hz+.

**decision:** **adapt.** plev is not async/tokio-based, but the concept maps directly: drain all pending winit events before calling begin_frame()/build_scene()/resolve()/render(). winit 0.30 already batches via `about_to_wait()`, the pattern would be to accumulate input events during `window_event()` and process them all in `about_to_wait()` before rendering.

---

## pattern 2: partial vs full render via atomic flag (yazi)

**source:** `yazi-shared/src/event/mod.rs` line 3, `yazi-fm/src/dispatcher.rs` lines 58-65, `yazi-fm/src/app/render.rs` lines 18-51

**description:**
yazi uses a global `AtomicU8` called `NEED_RENDER` with three states:
- 0 = no render needed
- 1 = full render needed
- 2 = partial render needed (only progress bar + notifications)

the dispatcher sets this flag. partial renders use `compare_exchange(0, 2)`, they only request a partial render if no render was already requested. full renders use `store(1)`, they always override. the render method checks: if `need_render == 2`, call `render_partially()` which only redraws the progress and notification overlays. otherwise, full redraw of the entire widget tree.

**key insight:** not all state changes require a full repaint. background task progress, cursor blinks, and notifications can be drawn as overlays on top of the existing frame. this saves the cost of re-laying-out and re-rendering the entire UI for high-frequency updates.

**applicability to plev:** plev's compositor already has per-layer dirty tracking via fxhasher. the atomic-flag pattern could complement this: when only a cursor blink or animation tick occurs (no actual scene change), plev could skip `build_scene()` entirely and only update the specific layer that changed. the 3-state flag (none/partial/full) maps to: 0=skip frame, 1=full rebuild, 2=update only animation/cursor layers.

**decision:** **adapt.** the concept of tiered render urgency is directly applicable. instead of a global atomic, plev could track this per-layer: each layer has a dirty flag, and the frame loop skips scene rebuild for unchanged layers. this is partially implemented via fxhasher dirty tracking, but the explicit "partial render" concept (only animate, don't rebuild scene) is new.

---

## pattern 3: layer-based action routing (yazi)

**source:** `yazi-shared/src/layer.rs` lines 7-20, `yazi-fm/src/executor.rs` lines 18-31, `yazi-fm/src/router.rs` lines 16-53

**description:**
yazi defines a `Layer` enum with 11 variants (app, mgr, tasks, spot, pick, input, confirm, help, cmp, which, notify). every action carries a `layer` field. the executor dispatches to the correct handler based on `action.layer`. the router resolves keyboard input by: (1) checking if the current layer's keymap matches the key, (2) for multi-key chords, activating the "which" popup to show available completions.

each layer acts as a modal scope, the same key can have different meanings in different layers. the cmp (completion) layer falls through to input if no match is found, implementing inheritance. the event type is a tagged union:

```rust
pub enum Event {
    Call(ActionCow),   // named action with layer + args
    Seq(Vec<ActionCow>),  // action sequence (macros)
    Render(bool),      // render request (partial/full)
    Key(KeyEvent),     // raw keyboard input
    Mouse(MouseEvent), // mouse input
    Resize, Focus, Paste(String),
}
```

**key insight:** the two-phase dispatch (router -> executor) cleanly separates "which action does this key trigger?" from "what does this action do?". the layer enum prevents accidental cross-modal interaction. fallthrough (cmp -> input) allows composable modal behaviors without duplicating keybindings.

**applicability to plev:** plev's input system uses a flat event queue with linear hit-testing. there's no concept of modal layers for keyboard routing. for a UI toolkit, modal input contexts (e.g., text editing mode, dialog mode, navigation mode) are essential. a layer enum + keymap-per-layer would let plev users define different keyboard behaviors for different UI states without custom dispatch logic.

**decision:** **adapt.** plev doesn't need 11 layers, but the concept of inputcontext (an enum of active modes) with per-context keymaps is valuable. this would live alongside the existing hit-test system: hit-testing handles spatial (mouse/touch) routing, while inputcontext handles keyboard modal routing.

---

## pattern 4: priority task scheduler with cancellation (yazi)

**source:** `yazi-scheduler/src/lib.rs` lines 7-9, `yazi-scheduler/src/runner.rs` lines 24-61, `yazi-scheduler/src/scheduler.rs`

**description:**
yazi's scheduler uses `async_priority_channel` (a priority-aware async mpsc) with three priority levels: low (file operations), normal (plugins, size calculation), high (hooks/cleanup). each task category (file, plugin, fetch, preload, size, process, hook) has its own priority channel and configurable worker count. workers loop on `rx.recv()` which returns the highest-priority item first.

cancellation is per-task via `CompletionToken`. each worker wraps its work in:
```rust
select! {
    r = do_work(input) => r,
    false = token.future() => Ok(())  // cancelled
}
```

the `Ongoing` registry tracks all active tasks. `cancel(id)` marks a task as cancelled and submits its cleanup hook at high priority, ensuring cleanup always runs before new work starts.

**key insight:** priority channels prevent expensive background operations (file copy, plugin execution) from blocking lightweight operations (metadata fetch, UI updates). the cancellation pattern (token checked via select!) means work stops immediately when no longer needed, not at the next polling interval.

**applicability to plev:** plev doesn't have a task scheduler yet, but will need one for: async image loading, font shaping of large text, preloading offscreen content, and plugin execution. the priority-channel pattern would prevent a large image decode from blocking a font cache miss that's needed for the current frame.

**decision:** **adopt** (when building async task system). the 3-level priority (high=frame-critical, normal=visible-soon, low=background) maps perfectly to GPU rendering priorities. the cancellation-via-token pattern is better than kill/abort because it allows cleanup (GPU resource release).

---

## pattern 5: lua plugin isolation (yazi)

**source:** `yazi-plugin/src/lua.rs`, `yazi-plugin/src/isolate/isolate.rs`, `yazi-plugin/src/isolate/entry.rs`

**description:**
yazi runs plugins in two modes: (1) a shared "main" lua VM for UI components (stage 1+2 in `lua.rs`), and (2) isolated "slim" vms per-task for background plugins. the `slim_lua()` function creates a fresh lua instance with a limited API surface, no pubsub, no theme globals. each slim VM gets its own `Runtime::new_isolate(name)`.

plugin entry points are loaded via `LOADER.ensure()` (lazy load + cache), then called with `call_async_method("entry", job)`. the blocking lua execution is wrapped in `tokio::task::spawn_blocking()` to avoid blocking the async runtime.

the host exposes 6 global tables to plugins: `ui` (render elements), `ya` (utility functions), `fs` (filesystem), `ps` (pubsub), `rt` (runtime info), `th` (theme). each is a curated lua table, not raw FFI.

**key insight:** two-tier VM isolation (shared for UI, isolated for tasks) balances performance (shared VM avoids lua init overhead for UI) with safety (isolated vms prevent task plugins from corrupting UI state). the limited global surface (`ui`, `ya`, `fs`, `ps`, `rt`, `th`) acts as a capability system, plugins can only do what the host explicitly exposes.

**applicability to plev:** task-33 (WASM plugins) could adopt the two-tier model: a shared WASM instance for UI extensions (custom components, themes) and isolated instances for background tasks (data processing, network). the capability-table pattern (explicit host function groups) maps directly to WASM imports/exports.

**decision:** **adapt** (for task-33). the two-tier isolation concept is directly transferable. WASM replaces lua, but the architecture (shared UI instance + isolated task instances + curated host API) is the same. the 6 global tables pattern suggests a plugin API design: `plev.ui`, `plev.layout`, `plev.input`, `plev.animation`, `plev.data`.

---

## pattern 6: channel abstraction for pluggable data sources (television)

**source:** `television/channels/channel.rs`, `television/channels/entry_processor.rs`, `television/matcher/mod.rs`

**description:**
television abstracts every data source as a `Channel<P: EntryProcessor>`. a channel wraps:
- a `source_command` (shell command that produces lines on stdout)
- a `Matcher<P::Data>` (fuzzy matching engine backed by `nucleo`)
- an `Injector<P::Data>` (thread-safe push-end for feeding items)

loading is fully async: `load()` spawns a tokio task that reads the command's stdout line-by-line, batches entries in groups of 10,000 (`BATCH_SIZE`), and flushes each batch via `spawn_blocking()` to avoid blocking the async runtime. a `MAX_CONCURRENT_FLUSHES = 4` semaphore caps memory usage at ~20mb.

three processor variants handle different data types:
- `PlainProcessor`, `Matcher<()>`, no extra data per entry (most memory-efficient)
- `AnsiProcessor`, `Matcher<String>`, strips ansi codes for matching but preserves them for display
- `DisplayProcessor`, `Matcher<String>`, applies custom display template

the `ChannelKind` enum wraps all three and uses a `delegate_to_channel!` macro to forward method calls, avoiding boilerplate match arms.

**key insight:** the injector/matcher split decouples data production (async, IO-bound) from data consumption (CPU-bound fuzzy matching). the 10k batch + 4-concurrent-flush pipeline keeps both the IO thread and the matcher thread saturated without unbounded memory growth. the three-tier processor design (plain/ansi/display) shows how to optimize for the common case (plain text) while supporting rich formatting.

**applicability to plev:** plev's compositor processes data synchronously. for features like search/filter in list views, autocomplete, or asset loading, the injector/matcher pattern provides a ready-made architecture: a background thread feeds data into a lock-free channel, and the UI thread queries results each frame via `results(num_entries, offset)`. the batch-flush pattern is also applicable to loading large datasets (e.g., log viewers, data tables).

**decision:** **adapt** (when building list/search components). the injector pattern is more general than fuzzy search, it's a way to bridge async data production with synchronous UI rendering. plev could use this for any "streaming data source" component.

---

## pattern 7: render gating by action type (television)

**source:** `television/action.rs` lines 292-319, `television/television.rs` lines 569-616

**description:**
television classifies actions into two categories: those that `affects_results()` (input changes, navigation) and those that don't (preview scrolling, panel toggles, layout changes). the `should_render()` function combines multiple heuristics:

1. always render the first n ticks (initial paint)
2. render at regular intervals (rendering_interval) during idle
3. render more frequently (rendering_interval_fast) when the channel is actively running (loading data)
4. always render immediately for UI-affecting actions (input changes, selection, toggles)
5. suppress rendering while a channel is reloading (200ms delay via `RELOAD_RENDERING_DELAY`)

the reload suppression is particularly clever: when switching channels, a 200ms `AtomicBool` flag suppresses rendering to prevent flickering from partially-loaded data.

**key insight:** not all user actions need the same rendering response. rapid data changes (channel loading) should throttle renders to avoid wasted work, while direct user interactions (typing, clicking) need immediate visual feedback. the `affects_results()` classification lets the system skip expensive data pipeline processing for actions that only affect visual chrome.

**applicability to plev:** plev could classify scene changes by impact: (1) structural changes (new nodes, removed nodes) need full resolve, (2) property changes (opacity, color) need only buffer updates, (3) animation ticks where nothing visually changes can skip the GPU entirely. the reload-suppression pattern is directly applicable to plev's layer system: when swapping layer content, suppress rendering for 1-2 frames to avoid partial-update flicker.

**decision:** **adapt.** the tiered rendering concept extends plev's existing dirty tracking. currently, plev hashes the entire scene per-layer. adding action classification would let it skip hashing entirely for actions known to not affect the scene (e.g., input focus change with no visual effect).

---

## pattern 8: mode/scope transitions with shared input (television)

**source:** `television/television.rs` lines 45-60, `television/channels/remote_control.rs`

**description:**
television defines three modes: `Channel` (main fuzzy finder), `RemoteControl` (channel switcher), `ActionPicker` (action palette). each mode has its own picker state and data source, but they share the same input bar and keybinding system.

switching modes preserves context: the current channel stays loaded when entering remotecontrol. the remotecontrol itself is a fuzzy-searchable list of available channels, it reuses the same matcher infrastructure. selecting a channel from remotecontrol triggers `zap()` which loads the new channel prototype and rebuilds the data pipeline.

the `MissingRequirementsPopup` shows when a channel requires binaries that aren't installed, demonstrating graceful degradation in scope transitions.

**key insight:** modes share infrastructure (input bar, fuzzy matching, rendering) but have independent state (picker position, data source). this is cheaper than full screen transitions because only the data source and result list change, the chrome stays the same. the "meta-channel" pattern (remotecontrol is itself a searchable list) shows how a system can be self-describing.

**applicability to plev:** for any app built on plev that needs modal navigation (e.g., command palette, settings panel, search overlay), the shared-chrome-with-swappable-data pattern avoids full-screen rebuilds. in plev's compositor terms: the chrome layers stay cached, only the content layer's scene nodes change. this maximizes dirty-tracking efficiency.

**decision:** **adapt.** the mode enum + shared input pattern is a UX pattern for plev applications, not for the engine itself. but plev could provide a `ModalStack` component that manages mode transitions with layer-aware caching: push a mode = create new content layer, pop = destroy it, chrome layers persist unchanged.

---

## pattern 9: toml-defined declarative layout (bottom)

**source:** `bottom/src/options/config/layout.rs`, `bottom/src/app/layout_manager.rs`

**description:**
bottom defines its entire widget layout in TOML with a hierarchical structure:

```toml
[[row]]
  ratio = 1
  [[row.child]]
    type = "cpu"
  [[row.child]]
    ratio = 2
    type = "mem"
[[row]]
  [[row.child]]
    type = "proc"
    default = true
```

the parser converts this to a `BottomLayout` tree: `BottomRow` -> `BottomCol` -> `BottomColRow` -> `BottomWidget`. each level has a `ratio` for proportional sizing. the layout manager then computes navigation mappings: for each widget, it calculates `up_neighbour`, `down_neighbour`, `left_neighbour`, `right_neighbour` by intersecting geometric line segments.

the navigation mapping algorithm (lines 40-350 of `layout_manager.rs`) works in two passes:
1. build a `BTreeMap<LineSegment, ...>` of all widget positions as percentages
2. for each widget, find the nearest intersecting widget in each direction

**key insight:** declarative layout in config files lets users customize their dashboard without code changes. the auto-computed navigation graph means keyboard navigation "just works" for any layout, users don't need to manually wire up widget transitions. the two-pass algorithm (position -> neighbor mapping) is o(n^2) in widgets but runs once at startup.

**applicability to plev:** plev uses taffy for flexbox layout, which is more powerful than ratio-based rows/columns. but the auto-navigation concept is valuable: given a layout tree, plev could automatically compute directional focus navigation (tab/shift-tab, arrow keys) without requiring developers to manually specify focus order. this is also required for accessibility (task-30), screen readers need a logical focus traversal order.

**decision:** **adapt** (for focus navigation). the TOML layout format itself isn't applicable (plev has taffy + DSL), but the auto-computed directional navigation from layout geometry is directly useful. it would be a function: `compute_focus_graph(layout: &ComputedLayout) -> FocusGraph` where focusgraph maps each focusable widget to its directional neighbors.

---

## pattern 10: widget maximize/restore toggle (bottom)

**source:** `bottom/src/app.rs` lines 92, 1299-1321, `bottom/src/canvas.rs` line 206

**description:**
bottom has a boolean `is_expanded` flag on the app struct. pressing `e` toggles it. when expanded, the canvas renderer skips the normal multi-widget layout and instead gives the current widget 100% of the terminal area:

```rust
if app_state.is_expanded {
    let rect = Layout::default()
        .margin(0)
        .constraints([Constraint::Percentage(100)])
        .split(terminal_size);
    match &app_state.current_widget.widget_type {
        Cpu => self.draw_cpu(f, app_state, rect[0], ...),
        Mem => self.draw_mem(f, app_state, rect[0], ...),
        ...
    }
}
```

esc exits expanded mode. widget navigation (ctrl+arrows) is disabled while expanded (`!self.is_expanded` guard in `move_widget_selection_logic`). the `is_force_redraw` flag ensures immediate visual update on toggle.

**key insight:** the implementation is trivially simple (one boolean + one branch in the render path) but provides high UX value: users can drill into any widget for detail, then return to the overview. the key design choice is that expansion doesn't destroy state, the widget keeps its scroll position, selection, and data when toggling back.

**applicability to plev:** for dashboard-style applications, a maximize/restore pattern is essential. in plev's layer system, this could be implemented as: (1) store the current layout, (2) set the target widget's layer to z_order=max and bounds=fullscreen, (3) set all other layers to invisible. on restore, revert. since plev has per-layer dirty tracking, the invisible layers incur zero GPU cost while maximized.

**decision:** **adopt** (as a utility pattern). this is simple enough to implement as a helper function or component behavior. `fn maximize_widget(compositor, widget_layer_id)` and `fn restore_layout(compositor)`. the layer visibility system already supports this.

---

## pattern 11: configurable update rate with minimum floor (bottom)

**source:** `bottom/src/options.rs` lines 706-717, `bottom/src/lib.rs` lines 222-289

**description:**
bottom's data collection rate is configurable via cli (`--rate 500`) or config file, with a default of 1000ms and a minimum floor of 250ms. the value is parsed by `parse_ms_option!` which accepts either milliseconds as integer or duration strings like "1s".

the collection thread runs in a loop:
```rust
loop {
    data_collector.update_data();
    sender.send(BottomEvent::Update(data));
    cancellation_token.sleep_with_cancellation(Duration::from_millis(update_sleep));
}
```

this is separate from the render loop: input events trigger immediate re-renders, while data updates arrive at the configured rate. the `sleep_with_cancellation()` call allows the thread to exit cleanly without waiting for the full sleep duration.

**key insight:** separating data collection rate from render rate is critical for resource-intensive monitoring. the minimum floor (250ms) prevents users from accidentally pegging their CPU. the cancellable sleep pattern ensures clean shutdown without threads hanging.

**applicability to plev:** plev runs at display refresh rate (typically 60fps = 16.6ms). for applications that poll external data (network, sensors, databases), a configurable poll rate separate from the render rate avoids unnecessary work. this is a pattern for plev applications rather than the engine itself, but the engine could provide a `Timer` utility that fires events at configurable intervals, integrated with the event loop.

**decision:** **adapt** (as engine utility). a `PollTimer { interval: Duration, callback: fn() }` component that integrates with plev's event queue would let applications separate data polling from rendering. the minimum-floor concept prevents footgun configurations.

---

## summary table

| # | pattern | source | decision | priority |
|---|---------|--------|----------|----------|
| 1 | event batching + render throttle | yazi | adapt | high, reduces GPU work on rapid input |
| 2 | partial vs full render flag | yazi | adapt | medium, extends existing dirty tracking |
| 3 | layer-based action routing | yazi | adapt | medium, needed for keyboard modal input |
| 4 | priority task scheduler | yazi | adopt (future) | low, needed when async tasks exist |
| 5 | two-tier plugin isolation | yazi | adapt (task-33) | low, informs WASM plugin design |
| 6 | channel/injector data abstraction | television | adapt (future) | low, for list/search components |
| 7 | render gating by action type | television | adapt | medium, complements dirty tracking |
| 8 | mode transitions + shared chrome | television | adapt | low, UX pattern for apps on plev |
| 9 | auto-navigation from layout | bottom | adapt | high, needed for a11y (task-30) |
| 10 | widget maximize/restore | bottom | adopt | low, trivial + high UX value |
| 11 | configurable update rate | bottom | adapt | low, engine utility for data-polling apps |

## top 4 patterns for plev (immediate relevance)

1. **event batching (pattern 1):** highest impact. plev processes every winit event individually; batching before render would cut GPU work by 5-10x during rapid input. implementation: accumulate events in `window_event()`, process batch in `about_to_wait()`.

2. **auto-navigation from layout (pattern 9):** required for accessibility (task-30). given taffy's computed layout, auto-generate a directional focus graph. bottom proves the two-pass algorithm (position map -> neighbor lookup) works for arbitrary layouts.

3. **partial render flag (pattern 2):** extends plev's existing per-layer fxhash dirty tracking with explicit "skip frame entirely" and "update only animation layers" states. reduces wasted frames during idle.

4. **layer-based action routing (pattern 3):** plev needs modal keyboard input for text editing vs navigation vs dialog contexts. the layer enum + fallthrough pattern provides a clean model.
