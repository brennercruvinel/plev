---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-78
wave: trilha
depends: []
depends-optional: [task-73, task-66]
branch: task/78-ai-native
---

# task-78: aplicações AI-native

## objetivo

mostrar, medindo, que um app com modelo local, embeddings, áudio, vídeo,
canvas, milhares de tokens em streaming, visualizações e GPU compute roda
melhor como um processo Rust com runtime da aplicação, modelo e
renderização juntos do que como Chromium, JS heap, DOM, IPC, backend
nativo e GPU. o foco é a próxima geração de desktop AI.

## ligação com a tese

seção 17 (aplicações AI-native; um processo Rust; onde memória, latência,
GPU, streaming e interação rica fazem o custo das abstrações importar),
seção 18 (GPU-first é infraestrutura). a nota "talvez para além das
fronteiras da arquitetura de von neumann" fica registrada como horizonte de
pesquisa, sem critério de aceite neste programa.

## estado atual no código

- o urnaui já é a semente: explorer de banco vetorial `.urna` com busca
  exata, ann, graph e híbrida, embedders offline chamados em subprocesso
  python, frames de mídia por ffmpeg lidos no lugar, worker que acorda o
  loop (`crates/urnaui`, ADRs `urnaui-media-frames-in-place-via-ffmpeg-subfile`
  e `worker-results-wake-the-event-loop`).
- charts e grafo com layout de força em comps e engine.
- sem inferência de LLM em processo; sem captura de áudio; sem compute
  shader de app.

## escopo

1. app de referência (o urnaui estendido ou um app novo, decisão no
   briefing): pergunta sobre um corpus `.urna`, resposta de um LLM local em
   streaming com as fontes citadas, visualização dos trechos no grafo.
2. inferência em processo: comparação de bibliotecas (candle, bindings de
   llama.cpp, burn ou outra) por tamanho, velocidade e wasm; decisão por
   ADR com o brenner; modelos fora do repositório.
3. streaming de tokens para a view: shaping e layout incrementais (depende
   da 73); latência de token até pixel medida.
4. GPU compute no mesmo device do renderer para algo que a UI mostra (por
   exemplo projeção de embeddings para o grafo), sem cópia de volta para a
   CPU quando não precisa.
5. vídeo: frames decodificados direto para textura; áudio: captura e
   reprodução (o microfone vem da task-76).
6. comparação honesta com um equivalente Electron com o mesmo recorte de
   funcionalidade: memória, startup, latência de token até pixel, tempo de
   frame sob streaming. metodologia escrita antes de medir.

## fora de escopo

treinar modelo; hardware fora de von neumann.

## dependências

independente para começar; 73 para o streaming barato; 66 para as capturas.

## superfície sob responsabilidade

o app de referência, o módulo de inferência (crate próprio se a decisão
pedir), ADR `docs/ADR/ai-native-single-process.md`. `crates/urnaui` só se
ele for o app escolhido.

## interfaces que consome

engine, comps, urna runtime (via urnaui), `sem` para que o app também seja
operável por agente.

## interfaces que pode alterar

as próprias.

## critérios de aceitação

- app de referência rodando no macOS com modelo local em streaming e fontes
  citadas.
- tabela de medição contra o equivalente Electron, com metodologia.
- `./script/gate` verde (o app, se depender de coisa fora do workspace,
  segue o modelo do urnaui: workspace próprio e perna opcional no gate).

## testes esperados

testes do módulo de inferência com um modelo minúsculo de fixture ou com a
interface falsa; testes de streaming para a view (sem frame velho).

## riscos e perguntas abertas

- tamanho de modelo e binário; wasm provavelmente fora para inferência.
- a comparação com Electron precisa de um equivalente justo; se não houver,
  a limitação é escrita.

## commits esperados

grupos por item, `task-78:` no início.

## pare e reporte se

- a decisão de biblioteca de inferência (sempre do brenner);
- o app escolhido exigir dependência fora do workspace.
