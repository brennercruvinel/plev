---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-04-05
domain: lint
commit: 2d95911
---

# resolucao de 107 clippy warnings para zero

## contexto

o projeto acumulou 107 clippy warnings ao executar com `-D warnings`.
estes estavam distribuidos em 12 categorias distintas. a resolucao
foi requisitada como parte da politica de qualidade do projeto.

## categorias e solucoes aplicadas

### float excessive precision (36 ocorrencias)

literais float com mais digitos que f32 suporta (~7 significativos).
concentrados em matrizes OKLCH (`theme/color_space.rs`) e constante
kappa de bezier (`path/builder.rs`).

solucao: truncar para precisao representavel. usar separadores `_`
para legibilidade (ex: `0.412_221_5`).

### collapsible if (23 ocorrencias)

ifs aninhados que podem ser combinados com `&&`. distribuidos em
gesture recognizer, signal runtime, builder, text input, window events.

solucao: combinar com `&&` e `let` chains (edition 2024 suporta
`if let ... && condition`).

### default implementations (14 structs)

structs com `new()` sem argumentos que nao implementavam `Default`.
solucao: `impl Default for X { fn default() -> Self { Self::new() } }`.
para `App` e `AccessibilityState`, gated com `#[cfg(...)]` para manter
compilacao condicional.

### too many arguments (16 funcoes)

funcoes com >7 parametros. duas estrategias aplicadas:
1. structs de agrupamento (`ResolveResources`, `CardColors`, `CardLayout`)
   para funcoes frequentemente chamadas.
2. `#[allow(clippy::too_many_arguments)]` para funcoes showcase que
   recebem cores individuais do tema (agrupamento nao melhora legibilidade).

### complex types (7 ocorrencias)

tipos longos como `Option<(&wgpu::Buffer, &wgpu::Buffer, u32)>`.
solucionados junto com "too many arguments" via structs nomeados ou
reducao de visibilidade dos metodos afetados.

### outras (11 ocorrencias)

1. 2x clamp-like pattern: `.min(X).max(Y)` substituido por `.clamp(Y, X)`.
2. 1x clone on copy: `.clone()` removido de `NamedKey` (implementa copy).
3. 2x single-pattern match: substituido por `if let`.
4. 1x loop counter: substituido por `.enumerate()`.
5. 1x index in loop: substituido por `.iter().enumerate()`.
6. 1x derivable impl: manual `impl Default` substituido por `#[derive(Default)]`.
7. 1x module_inception: `component/component.rs` renomeado.

## armadilhas

### reducao de visibilidade pode quebrar exemplos

o warning "type more private than item" sugeriu `pub(crate)` para
`Compositor::resolve`. isso compilou na lib mas quebrou 14 exemplos.

**regra:** executar `cargo check --examples` apos qualquer mudanca de
visibilidade em API publica.

### agentes paralelos sobrescrevem mutuamente

quatro agentes de clippy operando no mesmo working tree simultaneamente
geraram conflitos. correcoes de um agente foram sobrescritas pelo outro.

**regra:** um unico agente para correcoes que tocam arquivos sobrepostos.
