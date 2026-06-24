
# Project Baseline

- Edition: 20024. State the edition in every `Cargo.toml`; do not rely on the default.
- MSRV: pin a minimum supported Rust version in `Cargo.toml` (`rust-version = "..."`)
  and do not use APIs newer than it without bumping the pin in the same change.
- Formatting is owned by `rustfmt` with the repo's `rustfmt.toml`. Never hand-format
  around it. CI runs `cargo fmt --check`.
- Lints: `cargo clippy --all-targets -- -D warnings` is a hard gate, not a suggestion.
  Warnings fail the build. Silence a lint only with a scoped `#[allow(...)]` plus a
  one-line reason; never a crate-wide allow.



## Error Handling

- No functions that panic on the happy path. Avoid `unwrap()` and `expect()` outside
  tests; propagate with `?` instead.
- Prefer `.get(index)` (returns `Option`) over direct indexing, which panics.
- Never silently discard a fallible result with `let _ = ...`. Choose one:
  - propagate with `?` when the caller should handle the failure;
  - log with `tracing` or `log` when it is safe to ignore but should stay visible;
  - `match` or `if let Err(...)` when custom recovery is needed.
- Library error types use `thiserror`. Library code never panics.
- Async operations that can fail must let the error reach a layer that can report it,
  so users get meaningful feedback rather than a swallowed failure.

## Unsafe

- Introduce `unsafe` only with a `// SAFETY:` comment naming the invariant the caller
  must uphold. No SAFETY comment, no merge.

## Async

- Scope clones with variable shadowing to keep borrowed references short-lived:

  ```rust
  executor.spawn({
      let task_ran = task_ran.clone();
      async move {
          *task_ran.borrow_mut() = true;
      }
  });
  ```

## Dependencies

- Adding a crate is a deliberate choice, not a reflex. Prefer the standard library
  or an existing dependency first.
- A new dependency needs a one-line justification in the PR body: what it buys, why
  std or current deps do not cover it.
- Avoid crates that pull large or duplicated transitive trees for a small win.

## Naming

- Use full words. Do not abbreviate: `queue` not `q`, `connection` not `conn`.
- Directories, doc files, and assets: `kebab-case` in English.
- Source files: snake_case (idiomatic Rust).
- When proposing a rename or move, list it as an `mv` command. After moving, fix and
  test every touched import, then run the full suite.

## Comments

- Comments explain "why" when the reason is non-obvious. Do not write comments that
  restate what the code already says.


## File Hygiene

- One module per file, single responsibility.
-
- When a module starts doing more than one thing, promote it to a folder
  (`mod.rs` plus submodules). Split by responsibility, not by line count.
- Unit tests stay in the same file under `#[cfg(test)] mod tests { ... }`.
  Integration tests go in the crate's `tests/` folder.
- Length is a review trigger, not a hard cap. The real rule is single responsibility.
  As a convenience heuristic, when a file you touched crosses ~539 lines, read it in
  full and check it still does one thing. A file that is long only because of one
  honest `match`, derive blocks, or its own test module is fine as-is. Refactor only
  on a genuine responsibility violation.

## Scope Discipline

- Implement what is specified. Prioritize correctness and clarity. Speed and efficiency are secondary unless the task says otherwise.



## Prose Style
- every doc starts with a yaml header for semantic resolution (helps devs, llm  / agents, vector search): project, audience, status, last-updated, domain. design notes that turn out wrong get a note on top. they are not deleted.
-
- Docs and comments diataxis, lowercase, yaml frontmatter, all lowercase except acronyms. no emoji. no em-dash
-
- No emoji, no em-dash. Use `,` `;` `.` `:` or a plain hyphen `-`.
- Short paragraphs, direct voice, no marketing copy.
- Commit messages in plain English, no Conventional Commits prefix. The body explains
  the why; the diff shows the what.
