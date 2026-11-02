---
id: operations-generative-runtime-guidelines
title: "orchestrator and agent guidelines"
project: plev
type: manual
status: active
date: 2026-09-29
date_basis: header
recorded: 2026-09-23
topics: [agents, process, git, workspace]
relations: [adr-index, operations-autonomy, operations-build-and-serve-the-web-target, operations-code-against-the-plev-engine, operations-orchestrate-coding-agents-on-this-repo, operations-validate-visuals-by-pixel, generative-runtime-program, task-57]
source: docs/ref/generative-runtime-guidelines.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# orchestrator and agent guidelines

obrigatórias para o programa em [`generative-runtime-program.md`](../Synthesis/generative-runtime-program.md). servem
para não desviar e não gerar AI slop. a parte 1 e a parte 2 são as
diretrizes do CTO, completas. a parte 3 adapta essas regras a este
repositório (regras do engine, gate, worktrees, formato dos arquivos de task). a
parte 4 é o ciclo de trabalho, com os templates de briefing e de handoff.

regra de origem, acima de tudo: nenhum agente é disparado sem plano e sem
briefing escrito. o orquestrador valida cada entrega pessoalmente (diff
inteiro, gate, testes) e não dispara agentes de verificação extra. o
brenner e o orquestrador são os avaliadores; os agentes implementam.
começa pequeno, uma entrega por vez, e cada validação reorganiza a
próxima task.

---

# parte 1. diretrizes para o agente orquestrador

## responsabilidade central

o orquestrador é responsável pela coerência global do sistema. ele não
apenas distribui trabalho. ele controla:

- dependências entre goals;
- contratos entre módulos;
- interfaces públicas compartilhadas;
- ordem de integração;
- critérios de aceitação;
- conflitos entre implementações;
- estado real da `main`.

nenhum agente individual toma decisões que alterem a arquitetura
compartilhada sem passar pelo orquestrador.

## antes de distribuir trabalho

- mapear os goals e suas dependências explícitas.
- identificar quais arquivos, módulos, tipos e interfaces são
  compartilhados entre goals.
- definir para cada goal:
  - objetivo;
  - escopo;
  - dependências;
  - arquivos ou módulos sob responsabilidade;
  - interfaces que pode consumir;
  - interfaces que pode alterar;
  - critérios de aceitação;
  - testes esperados.
- distribuir preferencialmente um goal independente por agente.
- não paralelizar goals que disputam a mesma superfície arquitetural sem
  antes definir o contrato compartilhado.
- estabilizar interfaces fundamentais antes de liberar agentes
  dependentes.

## distribuição e dependências

- respeitar o grafo de dependências.
- nenhum goal pode assumir comportamento de uma dependência que ainda não
  existe.
- um goal dependente só começa quando o contrato necessário está definido.
- quando a implementação concreta da dependência é necessária, ela precisa
  estar integrada à `main`.
- não permitir que dois agentes implementem independentemente a mesma
  abstração.
- se dois goals precisam da mesma funcionalidade, extrair um contrato ou
  goal comum antes de continuar.

## ownership

- cada goal possui uma superfície de código claramente definida.
- evitar dois agentes modificando simultaneamente o mesmo módulo.
- arquivos compartilhados possuem ownership explícito.
- mudanças em interfaces públicas compartilhadas exigem aprovação do
  orquestrador.
- mudanças arquiteturais transversais são tratadas como trabalho próprio,
  não escondidas dentro de outro goal.

## contratos entre goals

sempre que um goal expõe funcionalidade para outro, definir
explicitamente:

- tipos;
- invariantes;
- ownership;
- lifecycle;
- error model;
- comportamento esperado;
- compatibilidade;
- testes do contrato.

o consumidor depende do contrato, não da implementação interna. não
aceitar integração baseada em conhecimento implícito entre agentes.

## validação

antes de aceitar um goal:

- revisar o diff;
- confirmar que o escopo foi respeitado;
- verificar se não houve duplicação de código;
- conferir se abstrações existentes foram reutilizadas;
- validar novos tipos e interfaces;
- conferir tratamento de erros;
- conferir invariantes relevantes;
- rodar os testes específicos do goal;
- rodar lint e formatação;
- rodar a suíte de integração necessária;
- validar regressões nos módulos dependentes.

um goal não está concluído só porque compila.

## commits

- rejeitar goals entregues como um único commit gigante.
- exigir commits progressivos e semanticamente separados.
- cada commit representa uma mudança lógica compreensível.
- commits intermediários mantêm o repositório em estado utilizável sempre
  que possível.
- não aceitar commits que misturem refactor não relacionado com
  implementação funcional.
- preservar histórico suficiente para entender como a solução foi
  construída.

## integração

- integrar branches uma por vez.
- rebase ou atualização da branch contra a `main` atual antes da
  integração.
- rodar as validações de novo depois de cada merge.
- resolver conflitos de contrato antes de liberar goals dependentes.
- nunca permitir que agentes resolvam silenciosamente conflitos
  arquiteturais escolhendo uma versão arbitrária.
- depois de mudanças em interfaces compartilhadas, revalidar todos os
  consumidores conhecidos.

## stubs e implementações temporárias

- stubs existem apenas para permitir desenvolvimento isolado.
- são claramente identificados.
- não introduzem comportamento fictício que possa ser confundido com
  implementação real.
- não são integrados como solução definitiva.
- antes do merge final de um goal, confirmar se todos os stubs relevantes
  foram removidos ou explicitamente aceitos.

## controle de escopo

o orquestrador rejeita:

- features não solicitadas;
- refactors oportunistas sem relação com o goal;
- novas abstrações sem necessidade demonstrada;
- duplicação de infraestrutura;
- mudanças de arquitetura escondidas dentro de implementação local;
- dependências novas sem justificativa;
- expansão de escopo não discutida.

melhorias encontradas durante o trabalho são registradas separadamente.

## handoff obrigatório

ao terminar um goal, o agente entrega ao orquestrador:

- resumo do que foi implementado;
- arquivos e módulos alterados;
- interfaces adicionadas ou modificadas;
- invariantes introduzidas;
- decisões técnicas relevantes;
- testes adicionados;
- comandos usados para validar;
- limitações conhecidas;
- dívida técnica criada;
- dependências ainda pendentes;
- riscos para outros goals.

o orquestrador precisa conseguir validar a entrega sem reconstruir
mentalmente todo o trabalho do agente.

---

# parte 2. diretrizes para os agentes

## escopo

- trabalhar em um único goal por vez.
- ler completamente a definição do goal antes de modificar código.
- identificar dependências e contratos existentes antes de implementar.
- não implementar funcionalidades pertencentes a outros goals.
- não ampliar o escopo porque encontrou algo interessante.
- melhorias fora do escopo são registradas e deixadas para decisão do
  orquestrador.

## antes de escrever código

- ler o código existente relacionado ao goal.
- procurar tipos, funções, traits, módulos e abstrações já existentes.
- entender os invariantes atuais antes de alterá-los.
- verificar os testes existentes.
- verificar os consumidores das interfaces que pretende modificar.
- confirmar se a funcionalidade necessária já existe parcial ou
  integralmente.
- preferir estender uma abstração existente a criar uma paralela
  equivalente.

## reutilização

nunca duplicar tipos, enums, traits, funções, módulos, pipelines, caches,
abstrações, protocolos ou estruturas de estado quando já existe uma
implementação com responsabilidade equivalente. se a abstração existente
não é suficiente, explicar por que antes de criar outra.

## arquivos e ownership

- editar apenas os arquivos necessários ao goal.
- não alterar módulos compartilhados fora do escopo sem autorização.
- não fazer mudanças cosméticas em arquivos não relacionados.
- não reorganizar código apenas por preferência pessoal.
- não fazer renames amplos sem necessidade funcional.

diff pequeno e preciso é preferível a diff amplo e elegante.

## interfaces compartilhadas

- não alterar interfaces públicas compartilhadas sem aprovação do
  orquestrador.
- não mudar silenciosamente assinaturas de funções consumidas por outros
  goals.
- não alterar semântica existente mantendo a mesma API sem registrar a
  mudança.
- se precisar de uma alteração de contrato, interromper o trabalho
  dependente e reportar ao orquestrador.

## dependências entre goals

se precisar de algo que pertence a outro goal:

- registrar explicitamente a dependência;
- consumir o contrato acordado;
- usar um stub mínimo somente quando necessário para desenvolvimento
  isolado;
- não implementar a responsabilidade do outro agente;
- não criar uma segunda versão temporária da mesma arquitetura.

## branches

- trabalhar em branch própria por goal.
- a branch parte de uma `main` válida.
- atualizar a branch quando dependências relevantes forem integradas.
- fazer rebase na `main` atual antes de solicitar integração.
- resolver apenas conflitos locais e mecânicos.
- conflitos arquiteturais vão para o orquestrador.

## commits

- commits pequenos e progressivos.
- um commit representa uma única mudança lógica.
- não entregar o goal inteiro em um único commit.
- não misturar refactor, feature, testes, mudanças de API e limpeza não
  relacionada sem necessidade clara.
- mensagem de commit com o identificador do goal no início.
- cada commit compila sempre que tecnicamente possível.
- commits que alteram comportamento incluem testes no mesmo commit ou
  imediatamente depois.

## qualidade

antes de considerar uma etapa concluída:

- compilar;
- rodar os testes relevantes;
- rodar lint;
- rodar formatação;
- verificar warnings;
- conferir comportamento de erro;
- conferir invariantes;
- conferir regressões locais.

não usar `unwrap`, `expect`, `unsafe`, suppressions de lint ou bypasses
equivalentes só para fazer o código passar, salvo quando o projeto permite
explicitamente e a decisão está justificada.

## testes

- testar comportamento, não apenas implementação.
- cobrir casos normais, bordas e erros relevantes.
- ao corrigir um bug, adicionar teste que falharia antes da correção.
- ao introduzir um contrato, testar o contrato.
- evitar testes acoplados demais a detalhes internos quando o
  comportamento público pode ser validado.

## ambiguidade

quando houver ambiguidade que afete arquitetura, contrato, ownership,
comportamento público, dependências, persistência, compatibilidade ou
segurança, não escolher arbitrariamente. reportar ao orquestrador:

```text
problema
opções possíveis
impacto de cada opção
recomendação do agente
decisão necessária
```

ambiguidades puramente locais e reversíveis podem ser resolvidas pelo
próprio agente.

## decisões técnicas

registrar decisões relevantes de forma curta:

```text
contexto
decisão
razão
consequências
```

não criar documentação longa para decisões triviais.

## não esconder dívida técnica

não mascarar limitações. se a implementação tem fallback temporário,
performance conhecida como ruim, edge case não resolvido, stub,
comportamento parcial, incompatibilidade ou risco, isso é declarado
explicitamente no handoff. "funciona" não significa "completo".

## handoff

ao concluir o goal, reportar:

```text
Goal:
Status:

Implementado:
-

Arquivos principais:
-

Interfaces adicionadas/alteradas:
-

Testes:
-

Validação executada:
-

Decisões:
-

Limitações:
-

Dependências pendentes:
-

Riscos de integração:
-

Commits:
-
```

o goal só está concluído quando o orquestrador consegue reproduzir a
validação.

## regra fundamental

cada agente é responsável pela qualidade local do seu goal. o orquestrador
é responsável pela coerência global do sistema. nenhum dos dois assume
silenciosamente a responsabilidade do outro.

---

# parte 3. adaptação a este repositório

## o que vale neste repositório

as regras do repositório valem para os agentes deste programa; o texto
completo das regras do engine está em
[`code-against-the-plev-engine.md`](code-against-the-plev-engine.md). em especial:

- as engine rules: um `TextStyle` por run, compartilhado por medição e
  desenho; srgb linearizado uma vez ao entrar na GPU e render target só
  via `gpu.surface_render_view`; geometria de container derivada do espaço
  disponível; todo handler que muda estado visível invalida; accessibility,
  reduced motion, contraste wcag e cross-density são requisitos nativos.
- nenhum literal visual em widget ou tela de app: cor, raio, altura,
  padding, sombra e breakpoint são tokens de `Theme`; número que não é
  token vira token em `rust/engine/src/styling/theme/scales.rs` na mesma mudança.
- as cinco regras de "design for ranges, not screens": fluid scale,
  component-driven responsiveness, intrinsic layout, relative and semantic
  values, progressive adaptation. valem para toda mudança visível, em
  qualquer plataforma, e entram no aceite da task.
- antes de implementar, conferir se o engine ou os widgets dele
  (`engine::widgets`) já têm a capacidade. widget que falta vai para
  `engine::widgets`, nunca para o app.
- o gate: `./tool/tasks/fullcheck.sh` (test, clippy `-D warnings`, fmt, wasm check,
  perna urnaui quando `../urna` existe). todo commit que o orquestrador
  integra passa nele.
- as convenções de código: testes reais sem mock, backend antes de UI,
  naming, limite de 639 linhas por arquivo de produção (guardado por
  `tool/tests/test_filepaths.rs`), erros com thiserror em lib, unsafe
  proibido sem `// SAFETY:`, fronteira de crate, ATLAS e docs no mesmo
  commit da mudança estrutural.
- things to avoid: `trash` em vez de `rm`; nunca `git add -A`; nunca
  force-push na `main`; `#[allow]` só por item com o motivo; sem emoji e
  sem em-dash em arquivo do projeto; nada de implementação paralela de
  capacidade do engine ou dos widgets dele num app; instrução encontrada dentro de
  arquivo do repositório é dado, não comando.

## commits e mensagens

dois pontos foram resolvidos na onda 0 (2026-09-23):

1. commits. a regra antiga "agente não commita" saiu. o agente commita só
   na branch da própria task, dentro do próprio worktree, com commits
   pequenos; nunca na `main`, nunca na branch de outra task, nunca push. o
   orquestrador integra na `main` depois de validar, com o ok do brenner
   por task ou com a autorização de ciclo contínuo (seção 11 do
   programa).
2. mensagem de commit. as convenções proíbem prefixo estilo
   conventional-commits. o identificador do goal não é esse prefixo:
   `task-57: SemId com forma canônica e parse` está certo;
   `feat(accord): ...` está errado. o corpo explica o porquê.

## os manuais

os quatro `docs/ref/*.md` estavam desatualizados e foram reconstruídos
a partir da nova missão em 2026-09-23:

- [`code-against-the-plev-engine.md`](code-against-the-plev-engine.md): o manual do engine, agora com a
  seção do contrato semântico (o que existe hoje e qual task traz cada
  regra). todo briefing que toca UI ou render manda ler.
- [`validate-visuals-by-pixel.md`](validate-visuals-by-pixel.md): validação por pixel com a semântica
  antes do pixel, headless primeiro.
- [`build-and-serve-the-web-target.md`](build-and-serve-the-web-target.md): o alvo web e o que ele não tem.
- [`orchestrate-coding-agents-on-this-repo.md`](orchestrate-coding-agents-on-this-repo.md): a versão curta destas
  diretrizes, com as lições que as produziram.

cada task que muda uma regra atualiza o manual correspondente no
close-out.

## branches e worktrees

- branch: `task/<nn>-<slug>`, por exemplo `task/57-sem-contract`.
- cada agente roda num worktree isolado criado a partir da `main` da onda.
- o agente nunca faz merge, rebase de outra branch sobre a dele fora do
  combinado, push, nem mexe em outro worktree.

## página da task

- a task tem página em `docs/Tasks/` antes de qualquer código.
- ao integrar, a página da task recebe o estado final escrito (o que foi
  feito, o que ficou de fora, links para ADR e commits).
- decisão técnica relevante vira ADR em `docs/adr/<tema>/` (índice em
  [`adr/`](../../adr/adr.md)) no mesmo change set da integração.
- estrutura, crate ou contrato novo atualiza `docs/ATLAS.toml` e o
  `README.md`; o `tool/tests/test_repoaudit.rs` falha o build se um
  crate do workspace não estiver nos dois.

## docs escritos por agentes

frontmatter yaml (type, tags, date, status ou commit), tudo em minúscula
exceto siglas e identificadores de código, português com acentuação
completa ou inglês, sem emoji, sem em-dash, sem markdown decorativo, aspas
retas.

---

# parte 4. o ciclo de trabalho

## sequência por task

1. o orquestrador confere o critério de aceite da task item por item
   ([`autonomy.md`](autonomy.md), "contra gasto de tokens"): se já bate, a
   task fecha e não há briefing; se não, escolhe o item que esta rodada
   fecha. confere também que as dependências estão integradas na `main` e
   que a superfície está livre (seção 8 do programa).
2. o orquestrador escreve o briefing a partir do arquivo da task e do
   código atual (diagnóstico com arquivo:linha), no template abaixo.
3. um agente, num worktree isolado, recebe o briefing.
4. o agente para e reporta no formato de ambiguidade sempre que bater numa
   decisão de arquitetura, contrato, ownership, comportamento público,
   dependência, persistência, compatibilidade ou segurança.
5. o agente entrega o handoff.
6. o orquestrador valida sozinho: lê o diff inteiro commit por commit,
   confere escopo e ownership, confere duplicação e reutilização, roda os
   testes da task, o gate e os testes dos consumidores. sem agente
   verificador.
7. o orquestrador leva o veredito ao brenner: aceito, ou volta com
   correções específicas (arquivo, linha, o que está errado, o que é
   esperado).
8. com o ok do brenner, o orquestrador integra (rebase na `main`, gate de
   novo depois do merge), atualiza task, ADR, arc, README e changelog.
9. a próxima task é revisada à luz do que esta ensinou antes do próximo
   briefing. o que sobrou da entrega vai para "pendências" no arquivo da
   task; não vira fase nem task nova sem o ok do brenner.

## template de briefing

```text
task: task-NN, <nome>
branch: task/NN-<slug>   base: main @ <sha>

mantra:
  o plev deve permitir que uma capacidade mantenha identidade e semântica
  enquanto sua representação visual pode nascer, desaparecer ou se
  transformar em tempo real.
  criar um sistema de dependências incremental independente da
  representação visual, capaz de propagar invalidação mínima entre estado,
  semântica, layout, scene construction, hit testing e rendering.

missão (uma frase, pela causa, não pelo sintoma):

critério de aceite que esta rodada fecha (copiado do arquivo da task, item
por item; sem item, sem briefing):

leia antes de tocar em código (só as seções citadas, não o arquivo inteiro):
  docs/ref/generative-runtime-guidelines.md
  docs/log/steps/task-NN-<slug>.md, seções <quais>
  docs/ref/code-against-the-plev-engine.md (quando toca UI ou render)
  <ADRs e arquivos específicos>

diagnóstico (arquivo:linha, o mecanismo, não o sintoma):

contrato a implementar ou consumir (assinaturas exatas):

arquivos permitidos:
arquivos proibidos:

design for ranges (quando a task toca algo visível): as cinco regras do
  manual do engine entram no aceite

aceite:
testes obrigatórios:
validação obrigatória (comandos):
  cargo test -p <crate>
  ./tool/tasks/fullcheck.sh

commits: pequenos, na branch da task, mensagem começando com task-NN
pare e reporte se: <as decisões que não são do agente>
entrega: o handoff da parte 2, com contagem de testes antes e depois

economia (vale para toda rodada):
  - a sonda só com o que decide a implementação, até umas 60 linhas no
    arquivo da task;
  - ler só os arquivos e as seções que o briefing cita; arquivo de task
    ou ADR grande se lê pela seção, nunca inteiro;
  - saída de cargo, gate e regress vai para um arquivo; ler só o fim ou o
    erro;
  - nada de diff inteiro, JSON de baseline, log ou tabela gerada colados
    nos docs: a lista, um resumo e o sha do commit;
  - o handoff curto (umas 40 linhas): commits, o que mudou, o item do
    critério de aceite que fechou, testes, contagem;
  - o que sobra vai numa lista "pendências", curta, sem propor fase
    nem task.
```

## o que o orquestrador olha no diff, sempre

- a tese: a capacidade mantém identidade e semântica quando a representação
  muda? a invalidação é semântica (hover não invalida, seleção sim)?
- um caminho só: humano, agente, accessibility e teste chegam na mesma
  ação pelo mesmo `dispatch`, ou alguém abriu um atalho paralelo?
- nenhuma segunda versão de algo que já existe (theme, hit test, action
  registry, clipboard, text measure).
- design for ranges, not screens: escala fluida vinda de token, componente
  que responde ao espaço que recebe, layout intrínseco, valor relativo e
  semântico, adaptação a input, accessibility e capacidade da plataforma.
  número solto em componente, breakpoint para ajustar pixel e regra por
  device voltam para o agente.
- erro tipado em lib, nenhum panic em caminho de input.
- testes de comportamento, com caso de erro e de borda, e contagem que não
  caiu.
- commits progressivos, cada um compilando.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-57: semantic contract (crate `sem`)](../../Tasks/task-57-semantic-contract/task-57-semantic-contract.md)
- [plev: thesis of the generative interaction runtime](../Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../Synthesis/generative-runtime-program.md)
- [operating mode: autonomous orchestrator](autonomy.md)
- [how to build and serve the web target](build-and-serve-the-web-target.md)
- [how to code against the plev engine](code-against-the-plev-engine.md)
- [how to orchestrate coding agents on this repo](orchestrate-coding-agents-on-this-repo.md)
- [how to validate visuals by pixel](validate-visuals-by-pixel.md)
- [autonomous cycle journal](autonomous-cycle-journal.md)
<!-- /related -->
