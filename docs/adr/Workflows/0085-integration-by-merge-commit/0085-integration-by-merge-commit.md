---
id: adr-0085
title: "integration into main is a pull request merged by merge commit"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: git
recorded: 2026-10-31
topics: [git, process, agents]
relations: [operations-autonomy, operations-orchestrate-coding-agents-on-this-repo, learning-generative-runtime, task-66, learning-cycle-0085]
evidence: [f41ba225, 5f755a6f, d51ec1a2, 3c0d6f3b, 9c4bca2d]
source: reconstruction
source_revision: c4f6c343
revised: 2026-11-02
review: reviewed
---

# integration into main is a pull request merged by merge commit

Records the repository's integration rule: the working branch enters main through a pull request merged with a merge commit, without squash and without rebase merge, and with the local green gate stated in the PR while GitHub's CI has no billing. The rule first appears in `f41ba225` (2026-09-23) and was restated in `5f755a6f` (2026-09-30), which replaced rebase and fast-forward with merge commit in the manuals; the merges into main have followed it since 2026-09-26. There was no ADR for it; this page reconstructs it from the manuals, the commits and the merge history.

## Context

- The autonomous mode that the archive has described since 2026-09-23 does not supervise each integration. Publishing an integration is branch, PR, CI and merge commit, "never squash or rebase merge, never force push" (`docs/Research/Workflows/autonomy.md:19` and `:114`).
- The message of `5f755a6f` says that AGENTS.md and the orchestration how-to replaced rebase and fast-forward with merge commit. The text of `f41ba225` already prohibited squash and rebase merge; the archive does not explain the difference between the two texts.
- GitHub's CI had no billing. `docs/adr/Synthesis/0070-generative-runtime-results/generative-runtime.md:91-92` says that the local gate counts and that the PR says so.
- The merges into main record the practice: PR #128 (`d51ec1a2`, 2026-09-30) and PR #142 (`3c0d6f3b`, 2026-10-27), among others since PR #125 (2026-09-26).
- Two artifacts that `5f755a6f` cited were dropped later. `.github/pull_request_template.md` and `.zed/tasks.json` were removed in `9c4bca2d` (2026-10-07). `./script/gate` does not exist at HEAD. The current manual cites `./tool/tasks/fullcheck.sh` (`docs/Research/Workflows/orchestrate-coding-agents-on-this-repo.md:55`).

## Decision

1. The working branch enters main through a pull request merged with a merge commit. Squash and rebase merge are not used (`autonomy.md:114`; `orchestrate-coding-agents-on-this-repo.md:57`).
2. The branch may be rebased before integration. The task page records the base used and the number of commits in the branch, as in `docs/Tasks/task-66-headless-first-class/task-66-headless-first-class.md:24` ("integrated into `main` at `c6738be` (nine commits)").
3. With no active CI, the local gate must be green and the PR says so (`orchestrate-coding-agents-on-this-repo.md:58`; `aprendizado-generative-runtime`, lines 91-92).
4. There is no force push in integration (`autonomy.md:114`).

## Consequences

- Easier: task pages cite the integration sha in main (`task-66-headless-first-class.md:24`), and `git log --merges` lists the PRs merged into main.
- Harder: each PR adds a merge commit to the history. With CI without billing, nothing in the repository prevents a merge without the gate; the check is manual, through the PR review and what the manual asks for.
- Divergence: the gate named in `5f755a6f` (`./script/gate`) does not exist at HEAD, and the PR template that cited it was removed in `9c4bca2d`. Today the reference to the gate is in the orchestration manual, not in a versioned PR artifact.
- Not covered: the archive does not say when CI will run again. `.github/workflows/gatecheck.yml` exists at HEAD and `docs/ATLAS.toml:41` calls it CI; the archive does not record whether it runs today.
- No test checks the merge method. What keeps the decision true is the merge history on main and the rule written in the autonomy manual.

## Alternatives

- Rebase and fast-forward: the message of `5f755a6f` says they were replaced by merge commit. The archive's texts give no other argument against them.
- Squash merge and rebase merge: ruled out in `autonomy.md:114` and in `aprendizado-generative-runtime` (line 92), with no reason written in those texts.

## Evidence

- `f41ba225`: first occurrence of "never squash or rebase" in the archive's texts (2026-09-23).
- `5f755a6f`: replaces rebase and fast-forward with merge commit, and cites the gate and the PR template (2026-09-30).
- `d51ec1a2`: merge of PR #128 into main (2026-09-30).
- `3c0d6f3b`: merge of PR #142 into main (2026-10-27).
- `9c4bca2d`: removal of `.github/pull_request_template.md` and `.zed/tasks.json` (2026-10-07), cited here as a divergence.

## learnings

- [what the autonomous cycle taught: integration into main is a pull request merged by merge commit](learned-integration-by-merge-commit.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [what the autonomous cycle taught: integration into main is a pull request merged by merge commit](learned-integration-by-merge-commit.md)
- [Generative runtime session: what worked and what bit us](../../Synthesis/0070-generative-runtime-results/generative-runtime.md)
- [task-66: headless as a first-class citizen](../../../Tasks/task-66-headless-first-class/task-66-headless-first-class.md)
- [operating mode: autonomous orchestrator](../../../Research/Workflows/autonomy.md)
- [how to orchestrate coding agents on this repo](../../../Research/Workflows/orchestrate-coding-agents-on-this-repo.md)
<!-- /related -->
