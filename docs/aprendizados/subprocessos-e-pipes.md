---
id: rust-subprocessos-pipes
title: "Por que esperar um processo pode bloquear a leitura da sua saída"
project: plev
type: aprendizado
status: referencia
date: 2026-10-16
date_basis: git
topics: [aprendizado]
relations: [task-99, adr-0083]
evidence: [701d2993]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Por que esperar um processo pode bloquear a leitura da sua saída

Considere um filho que escreve um vetor grande em stdout. O buffer do
pipe enche, e o filho espera o pai consumir os bytes. Se o pai estiver
em `wait`, esperando o filho terminar antes de começar a leitura,
nenhum dos dois avança.

O executor do explorador toma stdout e stderr do `Child` e entrega cada
pipe a uma thread. O dono do processo espera com timeout enquanto essas
threads drenam os dados. Após o término, reúne as saídas e classifica o
resultado. O teste `captures_stdout_larger_than_a_pipe_buffer` exercita
justamente uma saída maior que o buffer.

O ownership torna a distribuição dos handles explícita: `take()` retira
o pipe do `Child`, a thread passa a possuí-lo, e seu `JoinHandle` permite
recolher o vetor de bytes. Isso resolve o impasse de leitura tardia; não
substitui uma política de limite de bytes ou de processos descendentes.

Fonte: [apps/urnadbapp/src/backend/workers/subprocess.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/subprocess.rs).

## Evidências

- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
