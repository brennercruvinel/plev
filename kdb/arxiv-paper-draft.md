---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-13
domain: paper
---

# φ

## abstract

**φ: um motor de compositing GPU unificado para targets nativos e web a partir de um unico codebase rust**

a fronteira entre renderizacao nativa e renderizacao web nao e consequencia de constrangimento fisico. e um acidente historico. essa distincao importa porque muda fundamentalmente o que se torna possivel: se a separacao e acidental, ela e eliminavel, e sua eliminacao e agora realizavel dentro das garantias de memoria segura e abstracoes de custo zero de uma linguagem de sistemas operando no momento exato de maturidade da padronizacao webgpu.

apresentamos φ, um motor de compositing GPU-first implementado em rust que dissolve essa dicotomia no nivel de compilacao, nao no nivel de abstracao. o mesmo codebase nao modificado compila para metal em macos e ios, vulkan em linux e android, direct3d 12 em windows, e webgpu no browser via wasm32-unknown-unknown, com execucao de shaders identica em todos os targets. nenhum framework existente propoe isso sem degradar ao menos um dos lados: abstracoes que sacrificam fidelidade de renderizacao na web, ou codebases bifurcados que garantem divergencia comportamental ao longo do tempo.

o modelo de compositing e dirigido por scene graph regenerado integralmente por frame, evitando deliberadamente tanto reconciliacao de virtual DOM quanto diffing de retained mode tradicional, em favor de um compositor que raciocina diretamente sobre estado de GPU. dirty region tracking garante que apenas regioes de cena mutadas geram submissoes a GPU por frame, propriedade atualmente ausente em todos os competidores baseados em wgpu, incluindo iced e egui. renderizacao de texto, historicamente o ponto de falha decisivo de todo framework rust direcionado a webassembly, e resolvida atraves de uma arquitetura de atlas de glifos residente em GPU construida sobre cosmic-text e harfbuzz, entregando shaping unicode de qualidade de producao, layout bidirecional, suporte a ligatures e fidelidade subpixel equivalente a de editores nativos como zed e sublime text, sem rasterizacao CPU por frame para regioes de texto estavel.

o timing nao e incidental. webgpu atingiu suporte estavel em todos os browsers principais em 2024 e maturidade cross-platform completa incluindo safari e ios em 2026, representando o primeiro intervalo em que essa arquitetura e fisicamente realizavel sem degradacao. quem tentar isso em 2027 encontrara um ecossistema estabelecido.

ate onde e de nosso conhecimento, φ constitui o primeiro motor de compositing a entregar semantica uniforme, shaders identicos, atlas de glifos em GPU, dirty tracking e compositing de camadas independentes nos seis targets principais a partir de uma unica unidade de compilacao. a implicacao nao e apenas tecnica: e que decadas de duplicacao de esforco, abstracoes degradantes e codebases bifurcados foram escolhas, nao inevitabilidades, e que desfaze-las e agora o trabalho de um codebase, nao de uma geracao.

---

## 1. o que e o φ

tecnicamente o φ e um **compositing engine**. nao e um framework de widgets. nao e um runtime reativo. e a camada que transforma descricoes de cenas em draw calls na GPU, e que faz isso de forma identica independente de estar rodando sobre vulkan, metal, dx12 via wgpu-native, ou sobre webgpu via wgpu no browser.

o modelo central e um scene graph gerado por rust puro a cada frame, nao reconciliado como virtual DOM, nao diffado como retained mode tradicional. cada frame descreve o que existe, o compositor decide o que mudar na GPU. isso elimina a categoria inteira de bugs de estado inconsistente entre UI e dado.

o que o φ unifica que ninguem unificou ainda: o atlas, o compositor, o scene graph, o event system - tudo isso e o mesmo codigo rust compilado para `wasm32-unknown-unknown` com backend webgpu, ou para `x86_64`/`aarch64` com backend nativo. nao e abstracao em cima de dois renderers diferentes. e um renderer com dois targets de compilacao.

---

## 2. o problema - o gap que o φ resolve

todo framework existente trata web e nativo como alvos separados com codebases separadas ou abstracoes que degradam os dois. o que nao existe e um modelo de compositing unificado que compila para wgpu-native e wgpu-web sem concessoes, onde a UI rodando no mac e rodando no browser de outro usuario e literalmente o mesmo binario rust compilado para targets diferentes.

a questao tecnica central e **text rendering** - e onde todo framework rust-para-WASM falha. a resposta correta e cosmic-text para shaping e layout de texto, renderizado via wgpu com atlas de glifos na GPU. isso e o que separa um experimento de algo real.
