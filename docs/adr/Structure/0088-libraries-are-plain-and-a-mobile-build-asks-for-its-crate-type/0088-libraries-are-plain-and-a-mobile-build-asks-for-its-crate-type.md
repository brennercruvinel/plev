---
id: adr-0088
title: "libraries are plain, and a mobile build asks for its crate type"
project: plev
type: adr
status: accepted
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, mobile, performance]
relations: [task-106, adr-0080, platform-ios-build, platform-android-emulator-deploy, learning-general, research-build-directories-in-large-rust-workspaces]
evidence: [9a57115d, 91f812ed, b2834503, e1d415c4]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# libraries are plain, and a mobile build asks for its crate type

Decides that no library of the workspace declares a `crate-type`: the engine and the apps' libraries are plain rlibs, and the build that makes an Android `.so` or an iOS `.a` asks for that type on its command line. A desktop build stops linking 523 MB of static archives nobody loads ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

The engine declared `cdylib`, `staticlib` and `rlib`. Every desktop build linked a 196 MB static archive and a dylib of it, and nothing loaded either: a mobile app links the engine into its own library. `uigallery` and `urnadbapp` declared the mobile types too, 327 MB more for the gallery on every desktop build.

The extra types also took the hash out of the file name of the engine's rlib. Two workspaces writing one build directory then wrote the same `libengine.rlib`, one built against another `accord`, and the build failed with "multiple different versions of crate `accord`". The gate's urnadbapp leg keeps a subdirectory of its own because of it.

## decision

- A library's manifest has no `crate-type`.
- `tool/tasks/mobileapp.sh` asks for the type when it builds: `cargo rustc -p uigallery --lib --crate-type staticlib` for iOS, and the same with `cdylib` through cargo-ndk for Android. The artifacts keep their names, `libuigallery.a` and `libuigallery.so`.
- The iOS build reads the library from the directory cargo reports (`cargo metadata`), so `CARGO_TARGET_DIR` is honored; the Android build removes an old `libengine.so` from a reused `jniLibs` before it packages.

## consequences

A rebuild of the engine for one demo goes from 14 s and 485 MB to 11 s and 296 MB. The engine's rlib carries its hash, the condition for two workspaces to share a build directory; that sharing is not measured yet, and the gate's leg keeps its subdirectory until it is.

`cargo build -p uigallery` no longer leaves a mobile library behind: only the mobile script makes one.

Not verified: a real iOS or Android build. Both crate types were built on the host triple and cargo-ndk's forwarding of a `rustc` subcommand was read in its source; the machine of the task has no mobile target.

## alternatives

- A crate of its own for each mobile target, the shape of Zed's `gpui_ios` beside `gpui`: one more crate and manifest per platform for what one flag on the command line says.
- Keeping the types and paying the archives: the cost is per build directory and per profile.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0080: the workspace separates libraries, apps, demos and tools](../0080-workspace-by-responsibility/0080-workspace-by-responsibility.md)
- [what the autonomous cycle taught about the way of working](../../../Research/Workflows/autonomous-cycle-learnings.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
- [how large rust workspaces keep a build directory small](../../../Research/Ecosystem/build-directories-in-large-rust-workspaces.md)
- [android emulator deploy](../../../Research/Workflows/android-emulator-deploy.md)
- [ios build - decisions and knowledge](../../../Research/Workflows/ios-build.md)
<!-- /related -->
