# agents.md (bridge)

the single instruction source lives at `.contracts/.agents/AGENTS.md`. this
root file is a thin bridge: the tools that auto-read a root AGENTS.md (codex
and most agentic tooling) and claude code (which imports it through the
one-line CLAUDE.md stub, `@AGENTS.md`) all route to the same contract. do not
add content here; edit `.contracts/.agents/AGENTS.md`. do not create
GEMINI.md, CODEX.md, or any other parallel instruction doc.

@.contracts/.agents/AGENTS.md
