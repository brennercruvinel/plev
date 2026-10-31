---
id: task-77/fase-fechamento
title: "fechamento (orquestrador, 2026-09-25)"
project: plev
type: secao
parent: task-77
status: referencia
date: 2026-09-25
date_basis: texto
topics: [determinismo, plataforma, mobile]
relations: [adr-0064, task-18, task-19, task-20]
source: docs/log/steps/task-77-visual-determinism.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fechamento (orquestrador, 2026-09-25)

Registra o fechamento da task-77 pelo critério de aceite, item por item: o relatório de web e iOS sobre as 82 páginas do desktop, a causa de cada diferença calada e a ligação com o ADR `what-identical-means`. O resultado é web com 79 páginas iguais em todo byte e 3 caladas, iOS com 78 de diferença pedida na barra de status e 4 caladas, e um defeito de web corrigido (o id da tabela em `usize` estourava no wasm32). As pendências são a janela contra o headless, o `colorspace` num display P3 e o job de CI.

fechada pelo critério de aceite, item por item. as ferramentas da web e
do iOS foram instaladas com o ok do brenner (tasks 20 e 19).

- relatório de todas as páginas: `tool/tasks/screencap.sh web --all` e
  `tool/tasks/screencap.sh ios --all` desenham as 82 páginas do desktop
  (`regress capture`), abrem cada página no alvo (web: a query da URL,
  `?page=…&theme=…`, que o build web agora lê por
  `engine::window::launch_args`; iOS: os argumentos de lançamento) e
  classificam cada frame contra o do desktop, uma linha por página em
  `target/determinism/<run>/report.txt`. tema core.
  - web, 1280x800 a 1x, WebGPU sobre Metal: 79 iguais em todo byte, 3
    caladas.
  - iOS, iPhone 17 Pro, 402x874 a 3x: 78 com diferença só na barra de
    status (pedida), 4 caladas.
- cada calada com causa:
  - Surface e Silences (web e iOS): o seletor de quem propõe mostra
    "claude" no desktop, que tem o programa, e "rules" onde não há Claude.
    decisão do app, não do renderer.
  - Git (web e iOS): a página diz que o workspace git pede o CLI e uma
    thread, só no desktop. decisão do app.
  - Editor (iOS): 7x60 px físicos, o cursor: ele pisca e a captura cai em
    outra fase do piscar. tempo, não renderer.
  - Table e Data Table (web, na primeira rodada): defeito, corrigido. o id
    em hex da tabela era calculado em `usize`, que tem 32 bits no wasm32 e
    estourava; agora em u64 (`view/components/table.rs`), e as duas
    páginas ficaram iguais em todo byte.
- ADR: `what-identical-means`, com a adição da fase 2; a tabela
  `requested.json` confirma as linhas medidas (web `size`; iOS
  `status-bar`, `home-indicator`, `size`; android `navigation-bar`,
  `size`).
- o android (task-18) ficou a no máximo 2/255 do desktop, na GPU por
  software do emulador.

pendências:
- a janela contra o headless nas páginas e o `colorspace` num display P3
  (fase 2 original); o job de CI no mac quando o CI voltar.
- android `status-bar` segue `to_confirm`: a captura não mostrou
  diferença ali.
- as capturas foram em um tema e uma escala por alvo; os outros temas e
  escalas não foram varridos página a página.
