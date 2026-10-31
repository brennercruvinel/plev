---
id: adr-0085
title: "A integração na main é um pull request fundido por merge commit"
project: plev
type: decisao
status: aceito
date: 2026-09-23
date_basis: git
topics: [git, processo, agentes]
relations: [manual-autonomy, manual-orchestrate-coding-agents-on-this-repo, aprendizado-generative-runtime, task-66]
evidence: [f41ba225, 5f755a6f, d51ec1a2, 3c0d6f3b, 9c4bca2d]
source: reconstrucao
source_revision: c4f6c343
revised: 2026-10-30
review: revisado
---

# A integração na main é um pull request fundido por merge commit

Registra a regra de integração do repositório: a branch de trabalho entra na main por pull request fundido com merge commit, sem squash e sem rebase merge, e com o gate local verde dito no PR enquanto o CI do GitHub está sem billing. A regra aparece pela primeira vez em `f41ba225` (2026-09-23) e foi reafirmada em `5f755a6f` (2026-09-30), que trocou rebase e fast-forward por merge commit nos manuais; os merges da main a praticam desde 2026-09-26. Não havia ADR para ela; esta página a reconstrói a partir dos manuais, dos commits e do histórico de merges.

## Contexto

- O modo autônomo que o acervo descreve desde 2026-09-23 não supervisiona cada integração. Publicar uma integração é branch, PR, CI e merge commit, "nunca squash nem rebase merge, nunca force push" (`docs/referencia/operacao/autonomy.md:19` e `:114`).
- A mensagem de `5f755a6f` diz que o AGENTS.md e o how-to de orquestração trocaram rebase e fast-forward por merge commit. O texto de `f41ba225` já proibia squash e rebase merge; o acervo não explica a diferença entre os dois textos.
- O CI do GitHub estava sem billing. `docs/aprendizados/generative-runtime.md:91-92` diz que o gate local vale e que o PR diz isso.
- Os merges da main registram a prática: PR #128 (`d51ec1a2`, 2026-09-30) e PR #142 (`3c0d6f3b`, 2026-10-27), entre outros desde PR #125 (2026-09-26).
- Dois artefatos que `5f755a6f` citava saíram depois. `.github/pull_request_template.md` e `.zed/tasks.json` foram removidos em `9c4bca2d` (2026-10-07). `./script/gate` não existe no HEAD. O manual atual cita `./tool/tasks/fullcheck.sh` (`docs/referencia/operacao/orchestrate-coding-agents-on-this-repo.md:55`).

## Decisão

1. A branch de trabalho entra na main por pull request fundido com merge commit. Não se usa squash nem rebase merge (`autonomy.md:114`; `orchestrate-coding-agents-on-this-repo.md:57`).
2. A branch pode ser rebaseada antes da integração. A página da task registra a base usada e o número de commits da branch, como em `docs/tarefas/task-66-headless-first-class.md:24` ("integrada na `main` em `c6738be` (nove commits)").
3. Sem CI ativo, o gate local precisa estar verde e o PR diz isso (`orchestrate-coding-agents-on-this-repo.md:58`; `aprendizado-generative-runtime`, linhas 91-92).
4. Não há force push na integração (`autonomy.md:114`).

## Consequências

- Fica fácil: as páginas de task citam o sha da integração na main (`task-66-headless-first-class.md:24`), e `git log --merges` lista os PRs fundidos na main.
- Fica difícil: cada PR acrescenta um commit de merge ao histórico. Com o CI sem billing, nada no repositório impede um merge sem o gate; a conferência é manual, pela revisão do PR e pelo que o manual pede.
- Divergência: o gate nomeado em `5f755a6f` (`./script/gate`) não existe no HEAD, e o PR template que o citava foi removido em `9c4bca2d`. Hoje a referência ao gate está no manual de orquestração, não num artefato versionado de PR.
- Não coberto: o acervo não diz quando o CI volta a rodar. `.github/workflows/gatecheck.yml` existe no HEAD e o `docs/ATLAS.toml:41` o chama de CI; o acervo não registra se ele roda hoje.
- Nenhum teste verifica o método de merge. O que mantém a decisão verdadeira é o histórico de merges na main e a regra escrita no manual de autonomia.

## Alternativas

- Rebase e fast-forward: a mensagem de `5f755a6f` diz que foram trocados por merge commit. Os textos do acervo não dão outro argumento contra eles.
- Squash merge e rebase merge: vetados em `autonomy.md:114` e em `aprendizado-generative-runtime` (linha 92), sem motivo escrito nesses textos.

## Evidências

- `f41ba225`: primeira ocorrência de "nunca squash nem rebase" nos textos do acervo (2026-09-23).
- `5f755a6f`: troca de rebase e fast-forward por merge commit, e cita o gate e o PR template (2026-09-30).
- `d51ec1a2`: merge do PR #128 na main (2026-09-30).
- `3c0d6f3b`: merge do PR #142 na main (2026-10-27).
- `9c4bca2d`: remoção de `.github/pull_request_template.md` e `.zed/tasks.json` (2026-10-07), citada aqui como divergência.
