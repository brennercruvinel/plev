---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# task-34 extraction: fase e (WASM runtime) + fase f (competitors)

data: 2026-03-11

---

## pattern 1: dynamic WASM module compilation via shared table+memory

**source:** waforth, `/Users/aac/Dev/bc/bunker/repos/deployment/wasm-browser/waforth/src/waforth.wat` (lines 254-2576) + `src/web/waforth.ts` (lines 207-232)

**description:** waforth compiles forth words to WASM *at runtime* from within WASM itself. the core waforth module is hand-written wat that:

1. maintains a module header template in linear memory (at `MODULE_HEADER_BASE = 0x1000`), a complete WASM module skeleton with placeholder bytes for sizes, function type, table index, and local count.
2. when a `:` (colon definition) is parsed, `$startColon` resets the compilation pointer (`$cp`) to `MODULE_BODY_BASE`. the compiler then emits raw WASM bytecodes into linear memory via helper functions (`$emitConst`, `$emitGetLocal`, `$emitSetLocal`, `$compileExecute`, `$compilePushConst`).
3. when `;` (semicolon) is reached, `$endColon` patches the header placeholders (code size, body size, local count, table index) using leb128 encoding, then calls `$shell_load(offset, length)`.
4. the host JS `load()` function reads the bytes from linear memory, instantiates a new `WebAssembly.Module` + `WebAssembly.Instance`, passing the *same* table and memory as imports. the new function is automatically registered in the shared function table at the next index.
5. future calls dispatch through `call_indirect` on the shared table, so compiled words are as fast as built-in words.

**key insight:** the shared `table` + `memory` pattern allows dynamically compiled modules to call each other and access the same data stack. this is how extensibility works without serialization overhead.

**applicability to plev:** if plev ever supports user-defined WASM plugins (task-33), the shared-table pattern is the zero-copy way to let plugins call host functions and each other. however, for plev's use case (UI component plugins), the extism-style message-passing model is more practical and safer.

**decision:** watch, fascinating technique but too low-level for plev's plugin needs. the principle of "shared table for interop" is worth remembering if we ever need hot-patching shaders or animation curves from WASM.

---

## pattern 2: extism plugin lifecycle with fuel-based sandboxing

**source:** extism, `/Users/aac/Dev/bc/bunker/repos/deployment/wasm-browser/extism/runtime/src/plugin.rs` (lines 130-185, 869-1009) + `plugin_builder.rs` + `pdk.rs`

**description:** extism implements a complete plugin lifecycle:

1. **pluginbuilder** (builder pattern): `PluginBuilder::new(wasm_bytes).with_wasi(true).with_function("my_fn", args, returns, user_data, callback).build()`. host functions are registered with explicit signatures (`Vec<ValType>` for args/returns) and a closure receiving `&mut CurrentPlugin`.

2. **plugin struct** holds: wasmtime `Linker`, `Store<CurrentPlugin>`, `InstancePre` (pre-linked), an instantiation counter (to detect memory accumulation), a `CancelHandle` (cross-thread cancellation via epoch interruption), and an `Output` struct tracking offset/length in the plugin's linear memory.

3. **raw_call()** implements the call cycle:
   - reset fuel if fuel-limited (`config.consume_fuel(true)`)
   - reset store if previous call ran `_start` (to handle re-initialization)
   - instantiate from `instance_pre` (lazy, only when needed)
   - copy input bytes into plugin memory, set host context
   - start a timer thread that will trigger epoch interruption after timeout
   - call the function via `func.call(&[], &mut results)`
   - read output from plugin memory
   - timer stop, error extraction

4. **memory model:** the plugin allocates memory through the extism kernel (a small WASM module linked alongside the user module). the host reads/writes plugin memory via offset+length handles (`MemoryHandle`). this avoids sharing host memory with the plugin.

5. **host functions (pdk):** exposed as `extism:host/env` imports. each is a rust function taking `Caller<CurrentPlugin>` + `&[Val]` + `&mut [Val]`. the `CurrentPlugin` provides `memory_handle()`, `memory_str()`, `memory_new()`, `memory_free()` for safe memory access. variables (key-value store) are limited to `max_var_bytes` from the manifest.

**key insight:** the separation between `CompiledPlugin` (reusable, `Clone`) and `Plugin` (stateful, per-call) allows pre-compilation to be done once and shared. the fuel+epoch dual mechanism provides both deterministic (fuel) and wall-clock (epoch) limits.

**applicability to plev:** direct model for task-33 (WASM plugins). key decisions to adopt:
- builder pattern for plugin construction
- manifest-based configuration (allowed hosts, memory limits, timeouts)
- input/output via linear memory offsets (not shared memory)
- host functions as typed closures with `CurrentPlugin` context
- fuel budgets for untrusted plugins

**decision:** adapt, plev should use extism or a similar wasmtime-based approach for plugins. the builder+manifest pattern is solid. the exact pdk can be simplified for plev's needs (UI components don't need http, vars, etc.).

---

## pattern 3: leptos three-state push-pull reactive graph (clean/check/dirty)

**source:** leptos reactive_graph, files:
- `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/graph/node.rs`
- `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/graph/subscriber.rs`
- `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/graph/sets.rs`
- `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/computed/inner.rs`
- `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/signal/arc_rw.rs`

**description:** leptos implements the "reactively" algorithm with three states:

```
Clean  -- known up-to-date
Check  -- *may* have changed (source was dirty, but memo might filter)
Dirty  -- definitely changed, must recompute
```

the propagation works as follows:
1. when a signal is written, its direct subscribers are marked **dirty**
2. downstream subscribers of those (memos that depend on memos) are marked **check**
3. when a memo is read (or an effect runs), `update_if_necessary()` is called:
   - if **clean**, return false (no work)
   - if **dirty**, recompute
   - if **check**, walk sources and call `update_if_necessary()` on each. if any source actually changed (returned true) or state became dirty during the walk, recompute. otherwise, mark clean.

**key architectural choices:**
- **arc<rwlock<subscriberset>>** for the subscriber set, thread-safe, can be shared across threads
- **fxindexset** (rustc-hash + indexmap) for both sourceset and subscriberset, fxhash for speed, indexset for insertion-order iteration (critical for correctness: outer effects must run before inner effects)
- **weak<dyn source/subscriber + send + sync>** for type-erased nodes, allows garbage collection when the owner is dropped
- **thread-local observer** for auto-tracking: `OBSERVER` is a `RefCell<Option<ObserverState>>` in a `thread_local!`. when reading a signal, if there's a current observer, the signal adds it as a subscriber.
- **arcrwsignal vs rwsignal**: two tiers, `Arc`-based (reference-counted, `Clone` but not `Copy`) and arena-allocated (`Copy`, disposed with owner). arena signals are just thin wrappers around arc signals stored in an owner.

**comparison with plev:**

| aspect | plev | leptos |
|--------|------|--------|
| storage | slotmap (generational keys) | arc<rwlock<t>> + owner arena |
| thread safety | single-thread only (refcell) | send+sync (rwlock) |
| node identity | nodeid (slotmap key, copy) | usize + weak<dyn trait> |
| sets | vec (linear scan) | fxindexset (hashed, ordered) |
| three-state | yes (clean/check/dirty) | yes (clean/check/dirty) |
| type erasure | box<dyn any> | generics with storage trait |
| borrow safety | rc<dyn fn> clone-out pattern | rwlock + separate value/reactivity locks |

**key insight:** leptos's use of fxindexset for subscriber ordering prevents a subtle correctness bug: if an outer effect (e.g., a conditional that checks `.is_some()`) must run before an inner effect (that unwraps), maintaining insertion order in the subscriber set guarantees this.

**applicability to plev:** plev's signal system already implements the same three-state algorithm. the main gap is:
1. **vec for subscribers**, o(n) contains-check on every track(). should migrate to fxindexset for correctness (ordering) and performance (dedup).
2. **no send+sync**, fine for now (single-threaded rendering), but if plev ever moves signal updates to a background thread, the leptos model shows how.
3. **no owner/disposal hierarchy**, leptos ties signal lifetime to an owner that cleans up on component unmount. plev's `dispose_node()` is manual.

**decision:** adapt, migrate `subscribers: Vec<NodeId>` and `sources: Vec<NodeId>` to fxindexset<nodeid> for correctness and performance. consider adding an owner system when plev has proper component mounting/unmounting.

---

## pattern 4: dioxus generational-box copy signals with drop-guard notification

**source:** dioxus signals, `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/dioxus/packages/signals/src/signal.rs` (lines 1-551)

**description:** dioxus signals use `generational-box` for arena storage, giving them the ability to be `Copy` (not just `Clone`). the key innovations:

1. **copyvalue<signaldata<t>, s>**: the signal is a thin `Copy` handle (generational index). `SignalData<T>` holds both `subscribers: Arc<Mutex<HashSet<ReactiveContext>>>` and `value: T`.

2. **dual storage backends**: `UnsyncStorage` (thread-local, no arc overhead, default) and `SyncStorage` (arc-based, for cross-thread). this is implemented via a `Storage` trait.

3. **write notification via drop guard**: `SignalSubscriberDrop<T, S>` is returned as metadata when writing. when the write guard is dropped, `update_subscribers()` is called. this means:
   ```rust
   // Writing triggers notification on drop, not on write:
   *signal.write() += 1; // notification fires here, when temp is dropped
   ```
   this is elegant because it batches mutations naturally, if you do multiple mutations in a block, notification only fires once when the guard drops.

4. **peek() vs read()**: `read()` subscribes the current reactive context. `peek()` reads without subscribing. this replaces the old `write_silent()` pattern (which was global opt-out) with a local opt-out on the *read* side.

5. **reactivecontext auto-subscription**: when `try_read_unchecked()` is called and `ReactiveContext::current()` returns `Some`, the context subscribes to this signal's subscriber set. the reactivecontext is scope-local, similar to leptos's observer but tied to dioxus's component scope system.

**key insight:** the peek/read split is better API design than untracked/tracked write. it puts the control at the point of consumption (where you know *why* you don't need to track) rather than at the point of mutation (where the consequences are non-local).

**applicability to plev:**
- plev's readsignal.get() always tracks. there is no peek() equivalent. adding `get_untracked()` / `peek()` would be useful for logging, debugging, or performance-critical paths where you know re-render is not needed.
- the drop-guard write notification pattern is cleaner than plev's current `WriteSignal::update()` which notifies inside the with_runtime closure. if plev ever supports guards (`signal.write()` returning a `WriteGuard`), the drop pattern should be used.

**decision:** adapt, add `ReadSignal::peek()` (untracked read) to plev's signal API. the generational-box `Copy` approach is elegant but would require a significant rewrite of plev's slotmap-based system for marginal gain (plev signals are already copy via slotmap keys).

---

## pattern 5: slint property<t> with lazy binding evaluation and intrusive dependency lists

**source:** slint, `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/slint/internal/core/properties.rs` (lines 830-1027, 100-400)

**description:** slint's `Property<T>` is a fundamentally different reactive model from leptos/dioxus/plev signals:

1. **lazy evaluation**: bindings are not pushed eagerly. when a source changes, dependents are marked dirty but not recomputed. recomputation happens only when `Property::get()` is called (pull on demand). this is ideal for UI where most properties are only read during rendering.

2. **propertyhandle as tagged pointer**: the `handle` field stores either a pointer to a `BindingHolder` (if bit 1 is set) or a pointer to the `DependencyListHead` (if not). bit 0 is a borrow lock flag. this packs the entire state into a single `usize`.

3. **intrusive doubly-linked list** for dependencies: `DependencyNode<T>` forms an intrusive linked list. when a binding is dropped, it automatically removes itself from all dependency lists. no garbage collection needed, the destructor handles cleanup. this is `O(1)` removal (unlike plev's `Vec::retain` which is `O(n)`).

4. **bindingcallable trait + vtable pattern**: bindings are type-erased via a hand-written vtable (`BindingVTable`) with function pointers for `drop`, `evaluate`, `mark_dirty`, `intercept_set`, `intercept_set_binding`. this avoids `dyn Trait` overhead and allows `#[repr(C)]` for FFI with the slint c++ runtime.

5. **current_binding via scoped tls**: during binding evaluation, the current binding is set as thread-local. any `Property::get()` called during evaluation registers the current binding as a dependency via the intrusive list. when evaluation ends, the binding's `dep_nodes` list contains all dependencies.

6. **constant property optimization**: if a property has no binding and was never written to (or was explicitly marked constant), its `DependencyListHead` points to `CONSTANT_PROPERTY_SENTINEL`. future reads skip all dependency tracking. this is a zero-cost optimization for the common case of static UI properties.

7. **twowaybinding**: bindings can intercept `set()` via `intercept_set()`, enabling bidirectional data flow (e.g., a text input that both reads from and writes to a model property).

**key insight:** the lazy evaluation model is superior for rendering engines. in a frame, you might have 10,000 properties, but only 200 change per frame, and only the ones actually read during `build_scene()` need to be recomputed. plev's current model eagerly propagates to effects, which is fine for small graphs but could be wasteful at scale.

**applicability to plev:**
- the lazy-pull model aligns well with plev's frame lifecycle: signals change -> mark dirty -> during `build_scene()`, reading a dirty memo triggers recomputation. plev already does this for memos (lazy) but effects run eagerly. for pure rendering (no side effects), lazy is better.
- the intrusive list for `O(1)` dependency cleanup is superior to plev's `Vec::retain()`, but requires pin + unsafe code. the complexity tradeoff is not worth it yet at plev's scale.
- the constant-property sentinel is a free optimization plev could adopt: if a signal is created and never written to, skip tracking entirely.

**decision:** watch for lazy evaluation; adopt constant-signal sentinel. the intrusive list is too much unsafe for the current phase. the constant-signal optimization (if a signal has never been written, skip dependency tracking) is zero-cost to implement.

---

## pattern 6: leptos observer stack with RAII drop guard

**source:** leptos reactive_graph, `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/leptos/reactive_graph/src/graph/subscriber.rs` (lines 7-99)

**description:** leptos manages the current observer (the effect/memo currently executing and tracking dependencies) with an elegant RAII pattern:

```rust
thread_local! {
    static OBSERVER: RefCell<Option<ObserverState>> = const { RefCell::new(None) };
}

struct SetObserverOnDrop(Option<AnySubscriber>);

impl Drop for SetObserverOnDrop {
    fn drop(&mut self) {
        Observer::set(self.0.take());
    }
}

impl Observer {
    fn replace(observer: Option<AnySubscriber>) -> SetObserverOnDrop {
        SetObserverOnDrop(
            OBSERVER.with_borrow_mut(|o| {
                mem::replace(o, observer.map(|subscriber| ObserverState {
                    subscriber,
                    untracked: false,
                }))
            }).map(|o| o.subscriber),
        )
    }
}
```

the `_prev = Observer::replace(Some(self.clone()))` call returns a guard that, when dropped, restores the previous observer. this guarantees correct nesting even if the closure panics.

additionally, `ObserverState` has an `untracked: bool` field. this enables `untrack()` without removing the observer from the stack, it just sets a flag. reads check the flag and skip tracking if set.

**comparison with plev:** plev uses `observer_stack: Vec<NodeId>` with explicit push/pop in `with_runtime`. if a closure panics between push and pop, the stack is corrupted. leptos's RAII guard prevents this.

**applicability to plev:** the RAII observer guard pattern should be adopted. the `untracked` flag on the observer state is also a clean way to implement `untrack()` without stack manipulation.

**decision:** adopt, replace plev's explicit push/pop observer stack with a drop guard. this prevents stack corruption on panic and simplifies the borrow-safety dance.

---

## pattern 7: dioxus native renderer via vello (anyrender abstraction)

**source:** dioxus native, `/Users/aac/Dev/bc/bunker/repos/rust-ecosystem/dioxus/packages/native/src/dioxus_renderer.rs`

**description:** dioxus's native renderer wraps `anyrender_vello::VelloWindowRenderer` behind a `WindowRenderer` trait. key points:

- `DioxusNativeWindowRenderer` wraps `Rc<RefCell<InnerRenderer>>`, single-threaded, rc-shared across components
- `WindowRenderer` trait with `resume()`, `suspend()`, `is_active()`, `set_size()` maps directly to winit lifecycle
- ios sim special-case: `#[cfg(all(target_os = "ios", target_abi = "sim"))]` switches to `VelloCpuWindowRenderer` (no GPU on ios simulator)
- `CustomPaintSource` trait for user-defined wgpu rendering within the framework

**key insight:** dioxus does not have its own GPU renderer. it delegates to vello (via the `anyrender` abstraction). blitz (mentioned in the task) is now a separate project. this validates plev's approach of building its own GPU renderer, there's a gap in the ecosystem for a lightweight, GPU-first renderer that isn't vello's full compute pipeline.

**applicability to plev:** validates plev's strategic positioning. dioxus depends on vello, which is a heavy compute-shader pipeline. plev's fragment-only approach (quad + text pipelines, no compute) is deliberately lighter and more portable.

**decision:** ignore (for code patterns) but note for strategic validation.

---

## summary: decisions for plev

| # | pattern | source | decision | priority |
|---|---------|--------|----------|----------|
| 1 | shared WASM table+memory interop | waforth | watch | low |
| 2 | plugin lifecycle with fuel+epoch | extism | adapt for task-33 | medium |
| 3 | fxindexset for subscriber ordering | leptos | adopt | high |
| 4 | peek() untracked read + drop-guard write | dioxus | adapt (add peek) | medium |
| 5 | lazy binding eval + constant sentinel | slint | adopt sentinel | medium |
| 6 | RAII observer drop guard | leptos | adopt | high |
| 7 | vello delegation validates own renderer | dioxus | strategic note |, |

### concrete action items for plev signals:

1. **replace `Vec<NodeId>` with `FxIndexSet<NodeId>`** for `sources` and `subscribers` in `ReactiveNode`. this fixes both the o(n) `contains()` check on every `track()` and ensures subscriber execution order matches insertion order (pattern 3).

2. **add `ReadSignal::peek()`**, read without tracking. useful for logging/debugging within effects without creating unwanted dependencies (pattern 4).

3. **add RAII observer guard**, replace explicit `observer_stack.push()`/`pop()` with a drop guard that restores the previous observer. prevents stack corruption on panic (pattern 6).

4. **add constant-signal optimization**, if a signal has never been written to (`state` is still `Clean` after creation and no `set()` was ever called), skip dependency tracking on `get()`. track a `has_been_set: bool` flag on the node (pattern 5).
