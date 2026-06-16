---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-04-05
domain: modularization
commit: 2d95911
---

# modularizacao SRP com limite de 300 linhas

## contexto

o codebase acumulou 44 arquivos rust acima de 300 linhas (maior: 1219
linhas em narrate_runtime.rs). multiplas responsabilidades coexistiam em
modulos unicos: definicoes de tipos, logica de execucao, API publica,
testes e pipeline GPU. a regra de 300 linhas por arquivo foi estabelecida
como invariante de qualidade para o projeto.

## decisoes registradas

### adotamos submodulos rust com re-export via mod.rs

cada arquivo monolitico foi convertido em diretorio com mod.rs e
submodulos por responsabilidade. a API publica permanece identica via
`pub use` no mod.rs. rust resolve `pub mod signal;` tanto para
`signal.rs` quanto para `signal/mod.rs` automaticamente, portanto
`lib.rs` nao requer alteracao.

consequencias:
1. navegacao por responsabilidade em vez de por scroll.
2. `impl` blocks podem ser distribuidos entre arquivos.
3. visibilidade requer `pub(crate)` ou `pub(super)` para itens
   compartilhados entre submodulos do mesmo modulo.

### adotamos worktrees isoladas para refatoracao paralela

agentes de refatoracao foram executados em worktrees git isoladas para
evitar conflitos de escrita simultanea. cada agente operou em um modulo
independente.

consequencias:
1. paralelismo seguro: 3-4 modulos refatorados simultaneamente.
2. restricao: worktrees so contem arquivos rastreados pelo git.
   arquivos untracked (examples-wip/) nao aparecem na worktree.

### adotamos resolveresources struct para compositor::resolve

o metodo `Compositor::resolve` recebia 8 argumentos posicionais (device,
queue, format, width, height, composite_bgl, opacity_bgl, sampler).
foram agrupados em `ResolveResources<'a>` com campos nomeados.

consequencias:
1. legibilidade: campos nomeados em vez de posicao.
2. todos os 14 exemplos e o window/render.rs foram atualizados.
3. o struct deve permanecer `pub` (nao `pub(crate)`) porque exemplos
   o utilizam diretamente.

### adotamos cardcolors/cardlayout para showcase scene

funcoes de card recebiam 6-9 argumentos de cor. dois structs foram
criados: `CardLayout` (posicao + dimensoes) e `CardColors` (paleta).

## armadilhas encontradas

### worktrees nao contem arquivos untracked

ao tentar refatorar `examples-wip/` com worktree isolation, os agentes
reportaram sucesso mas as alteracoes nao existiam (diretorio ausente da
worktree). arquivos fora do git requerem execucao direta no working tree.

**regra:** verificar `git ls-files <path>` antes de usar worktree para
arquivos que podem estar em `.gitignore`.

### pub(crate) em compositor::resolve quebrou 14 exemplos

o clippy warning "type is more private than item" sugeriu reduzir
visibilidade de `resolve()` para `pub(crate)`. isso compilou na lib mas
quebrou todos os exemplos que chamam `compositor.resolve(...)` de fora
do crate.

**regra:** antes de reduzir visibilidade de metodos publicos, verificar
se exemplos (`cargo check --examples`) e crates dependentes os utilizam.
a solucao correta foi tornar o struct `ResolveResources` `pub` em vez
de esconder o metodo.

### agentes paralelos sem isolamento corrompem estado

quando 4 agentes de clippy operaram no mesmo working tree sem worktree,
alteracoes de um agente foram sobrescritas por outro. resultado: build
quebrado com metades de refatoracoes aplicadas.

**regra:** para alteracoes nos mesmos arquivos, usar agentes sequenciais
ou um unico agente. paralelizar apenas quando os conjuntos de arquivos
sao disjuntos.

### module_inception: component/component.rs

clippy reporta warning quando um modulo tem o mesmo nome que seu
diretorio pai. renomeado para `component/lifecycle_impl.rs`.

## metricas

| metrica | antes | depois |
|---------|-------|--------|
| arquivos .rs > 300 linhas | 44 | 0 |
| maior arquivo .rs | 1219 | 300 |
| clippy warnings (-d warnings) | 107 | 0 |
| testes passando | 470 | 470 |
| exemplos compilaveis | 0 | 15 |
| total arquivos .rs | ~60 | 271 |

## padrao de divisao aplicado

1. `types.rs` ou `card_types.rs`: structs, enums, constantes.
2. `engine.rs`, `processor.rs`, `execution.rs`: logica central.
3. `api.rs`: interface publica do modulo.
4. `tests.rs` (ou `tests/`): testes unitarios e de integracao.
5. `mod.rs`: declaracoes de submodulos e `pub use` re-exports.

funcoes auxiliares internas: `helpers.rs`, `utils.rs`.
pipeline GPU: `pipelines.rs`, `render.rs`, `gpu.rs`.

## checklist para futuras modularizacoes

1. verificar se arquivo excede 300 linhas.
2. identificar responsabilidades distintas (SRP).
3. criar diretorio com mesmo nome do modulo.
4. mover codigo para submodulos por responsabilidade.
5. criar mod.rs com `pub use` para manter API identica.
6. executar `cargo check` e `cargo test`.
7. verificar `cargo check --examples` se existirem.
8. confirmar zero warnings com `cargo clippy -- -D warnings`.
