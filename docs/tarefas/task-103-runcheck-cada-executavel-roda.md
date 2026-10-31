---
id: task-103
title: "task-103: runcheck, cada executável do repositório roda"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [processo, plataforma, workspace]
relations: [task-98, task-102]
evidence: [205fc5f8]
source: reconstrucao
source_revision: c4f6c343
revised: 2026-10-30
review: revisado
---

# task-103: runcheck, cada executável do repositório roda

Registra `tool/tasks/runcheck.sh`, a verificação que executa cada programa do repositório (aplicações, demos, linhas de comando de crates e ferramentas) para confirmar que ele sobe e faz o seu trabalho nesta máquina. O gate prova compilação e testes; o runcheck prova o que o gate não prova. O script entrou na main pelo commit `205fc5f8` (2026-10-27), sob a mensagem "Ogrnaize folders", e nenhuma página do acervo o descrevia antes desta. O acervo não guarda uma execução dele nem o seu relatório.

## Problema e execução

O cabeçalho do script diz a diferença: o gate (`tool/tasks/fullcheck.sh`) prova que o código compila e que os testes passam; o runcheck prova que cada programa inicia e faz o seu trabalho (`tool/tasks/runcheck.sh:2-5`).

- Janelas (`gui`, `runcheck.sh:99-139`): o programa passa se continua vivo depois de `WAIT` segundos, com padrão de 6 (`:28`). No macOS, cada janela é capturada com `screencapture` e contada por cores distintas numa grade abaixo da barra de título; uma janela de uma cor só não passa (`:116-137`). Cada captura tem até três tentativas, com três segundos entre elas (`:118-122`).
- Linhas de comando (`cli`, `runcheck.sh:141-156`): passa com código de saída 0, ou com uma linha esperada quando o argumento `expect` é dado. O limite é de 300 s (`:147`).
- Insumos: o script usa os fixtures do repositório (`:41-43`). Dois programas não têm insumo e o script diz isso em vez de falhar: `llm-bench` sem `PLEV_LLM_MODEL` (`:188-198`) e `urnadbapp` sem o checkout irmão `../urna` (`:215-218`). O `intent` roda só as regras, com `PLEV_CLAUDE` apontando para um programa inexistente, para não gastar uma chamada ao modelo (`:21-22`, `:177`).
- Cobertura: o workspace é compilado antes (`:158-167`); ferramentas e crates de linha de comando (`:170-180`); a build wasm de `uigallery-web` (`:181-187`); oito demos e as aplicações de janela (`:206-214`); a ponte do agente com `PLEV_MIRROR=1` (`:226-238`).
- Relatório: `target/runcheck/report.md`, com uma linha por programa, tipo, resultado e evidência (`:240-251`). O código de saída é 1 quando algo falha.

## Limites

- A captura de janela só existe no macOS. Fora dele, uma janela passa só por continuar viva (`:129`).
- Uma janela que passa mostrou pelo menos duas cores numa grade; isso não prova o conteúdo (`:134-135`).
- Linhas de comando sem `expect` passam pelo código de saída e não conferem a saída.
- Nenhum resultado de execução está registrado no acervo; esta página descreve o script, não uma rodada dele.

## Aprendizado em shell

O script usa `perl -e 'alarm shift; exec @ARGV'` como limite de tempo, com o comentário de que é um timeout que todo Unix tem (`runcheck.sh:50-51`). O script não diz por que não usa `timeout`; a leitura provável é que o comando não vem por padrão no macOS, o que é inferência e não está escrito no código.

## Evidências

- `205fc5f8`: adiciona `tool/tasks/runcheck.sh` (251 linhas), junto com a remoção de `.devops/check/*.toml` e a inclusão de `rust/llmrun/Cargo.lock` (2026-10-27). É o único commit que toca o arquivo (`git log -- tool/tasks/runcheck.sh`).
