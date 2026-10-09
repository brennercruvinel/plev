# docs

the knowledge of plev, in four places by what a reader comes for. the
architecture in one file is [ATLAS.toml](ATLAS.toml) (tables for machines,
prose for humans, the frame-flow diagram at the end); the names of the
tree, old and new, are in [TERMS.toml](TERMS.toml).

| where | what it holds | start with |
|---|---|---|
| [adr/](adr/) | the decisions, numbered across one sequence and grouped by theme (generated, importing, interface, recording, rendering, semantics, structure, workflows) | [adr/README.md](adr/README.md), the index; [adr/TEMPLATE.md](adr/TEMPLATE.md) for a new one |
| [ref/](ref/) | references to work by: the operating manual, design notes of each subsystem, the component catalog, the research and the comparisons | [ref/code-against-the-plev-engine.md](ref/code-against-the-plev-engine.md), read before touching ui or rendering code |
| [log/](log/) | what happened and what is next: the program and its tasks (steps/), what each task changed (changes/), the learnings, the journal, the changelog | [log/README.md](log/README.md), the state of the mission; [log/generative-runtime-program.md](log/generative-runtime-program.md), the program |
| [pub/](pub/) | what is written for outside readers: the thesis, the paper draft and outline, the benchmark results | [pub/generative-runtime-thesis.md](pub/generative-runtime-thesis.md) |
| [img/](img/) | images the other docs show | |

a doc names code by its path from the repository root (`rust/engine/src/...`)
and other docs by a relative link, so a move breaks a link the gate's
`lychee --offline` pass reports.
