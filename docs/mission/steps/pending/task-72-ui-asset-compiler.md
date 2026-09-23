---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-72
wave: trilha
depends: []
waits-for: [task-63]
branch: task/72-asset-compiler
---

# task-72: UI asset compiler

## objetivo

levar a filosofia do monster (formato externo complexo, compilado uma vez
em build time para uma representação própria otimizada, runtime pequeno)
para os outros assets de UI: SVG, fontes, ícones, animações, imagens,
layouts estáticos, shaders e design assets. e gravar interação real do app
como asset `.monster`.

## ligação com a tese

seção 11 (a filosofia do monster), seção 24 (UI asset compiler; gravar
interação real como asset: o app gera o motion, não o designer). as
anotações: "o monster é pequeno hoje, mas precisa de mais atenção".

## estado atual no código

- `lot` (lottie para monster) e `svg` (svg estático para monster, via usvg
  e lyon) já seguem a filosofia; o player não carrega runtime estrangeiro.
- ícones lucide são tesselados em runtime e cacheados
  (`crates/comps/src/icons/mod.rs:127`, `tessellate_icon`).
- fontes: ttf inteiros embutidos (`crates/engine/src/text/fonts.rs:38-52`),
  raster de glifo em runtime no atlas, determinístico.
- shaders WGSL carregados e validados em runtime (hot reload lê do disco em
  debug).
- imagens decodificadas em runtime (`image` com png e jpeg).
- o container do monster é superfície da task-63 enquanto ela roda.

## escopo

1. inventário medido: o que é parseado, tesselado, decodificado ou validado
   em runtime que poderia ir para build time, com o custo atual de cada um
   (startup, memória, tamanho de binário). nada é movido sem esse número.
2. candidatos, em ordem de ganho medido:
   - ícones: tesselação em build time para os tamanhos usados pelos tokens
     (`theme.control` icon sizes), guardados como `asset_path` do monster;
   - shaders: validação naga em build time (erro de shader vira erro de
     build, não de runtime), sem mudar o hot reload de debug;
   - fontes: subset só para fontes de conteúdo fixo (codicons); fontes de
     texto do usuário continuam inteiras (cobertura de glifo, task-74);
   - imagens: pré-decodificação ou formato de textura quando o ganho
     compensar o tamanho;
   - layouts: só para telas estáticas (splash); o contrato de layout
     derivado do espaço disponível continua valendo para o resto;
   - design assets: tokens gerados a partir de fonte de design só se a
     fonte puder ser nomeada em arquivo versionado (o kit privado não é
     citado em arquivo rastreado, regra do AGENTS.md).
3. ferramenta: um passo de build (build.rs por crate ou um CLI de assets,
   decisão no ADR) com saída determinística e cache por hash.
4. gravar interação real: com a trilha visual da task-63, gravar uma
   interação (spring, fade, gesto), passar pelo discover e pelo optimize do
   monster e exportar como asset tocável pelo player.

## fora de escopo

mudar o formato do monster antes da task-63 fechar o ADR de sessão.

## dependências

nenhuma para o inventário e para ícones e shaders. o item 4 e qualquer
mudança de container esperam a task-63.

## superfície sob responsabilidade

`crates/lot`, `crates/svg`, o passo de build novo, `crates/comps/src/icons`
(só a origem dos dados tesselados), proposta de ADR
`docs/ADR/ui-asset-compiler.md`. `crates/monster/src/container.rs` só
depois da 63.

## interfaces que consome

`monster` (codec, discover, optimize, `asset_path`), a trilha visual (63).

## interfaces que pode alterar

a forma como ícones e shaders chegam ao runtime, sem mudar a API pública de
comps.

## critérios de aceitação

- inventário com números antes e depois de cada item movido.
- ícones: pixels iguais antes e depois (headless, 66) em todos os tamanhos
  de token.
- shaders: um erro de sintaxe em WGSL quebra o build com mensagem clara.
- `./script/gate` verde; build release e wasm continuam.

## testes esperados

determinismo do passo de build (duas execuções, mesmo byte); igualdade de
pixel dos ícones; teste do erro de shader.

## riscos e perguntas abertas

- build time maior; medir e registrar.
- wasm: o passo de build roda no host, a saída entra no binário wasm.

## commits esperados

1. `task-72: inventário medido de trabalho em runtime`
2. `task-72: ícones tesselados em build time`
3. `task-72: validação de WGSL em build time`
4. `task-72: <próximo item pelo ganho medido>`
5. `task-72: interação gravada exportada como asset` (depois da 63)

## pare e reporte se

- o ganho medido de um item não justificar a complexidade;
- mover um asset para build time exigir mudar contrato de layout ou de
  texto.
