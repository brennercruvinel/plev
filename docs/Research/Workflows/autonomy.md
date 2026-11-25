---
id: operations-autonomy
title: "operating mode: autonomous orchestrator"
project: plev
type: manual
status: active
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [agents, process, git]
relations: [learning-general, research-autonomous-cycle-journal, operations-generative-runtime-guidelines, operations-orchestrate-coding-agents-on-this-repo, generative-runtime-program, task-75, task-96, task-97, generative-runtime-thesis]
source: docs/ref/autonomy.md
source_revision: ec0a224c
revised: 2026-10-31
review: reviewed
---

# operating mode: autonomous orchestrator

desde 2026-09-23 o brenner não supervisiona cada integração. o orquestrador
segue sozinho dentro da missão ([`generative-runtime-thesis.md`](../Synthesis/generative-runtime-thesis.md)),
das regras ([`generative-runtime-guidelines.md`](generative-runtime-guidelines.md)) e do programa
([`generative-runtime-program.md`](../Synthesis/generative-runtime-program.md)). o objetivo não é executar a
lista de tasks: é manter um ciclo contínuo de implementação, validação,
aprendizado e reorganização do programa.

## o ciclo

`observar → sondar → implementar → validar → aprender → replanejar`

em detalhe: `observe → choose → brief → implement → validate → learn →
update → replan → repeat`. a sonda (reproduzir por teste ou comando o
cenário que motiva o trabalho) vem antes de implementar: o orquestrador
faz antes do briefing ou o briefing marca o cenário como hipótese e manda
o agente sondar primeiro, parando se o resultado for outro.

1. **observe.** antes de escolher trabalho novo, reconstruir o estado real da
   `main`: contratos, tasks integradas e pendentes, bugs, limites, métricas,
   dependências, ownership, decisões abertas. a documentação nunca tem mais
   autoridade que o código; divergência se investiga e a doc se corrige antes
   de planejar com ela.
2. **choose.** fechar antes de abrir (regra de 2026-09-25, abaixo em
   "contra gasto de tokens"): o próximo trabalho sai da lista de tasks
   abertas (`status: in-progress` em `docs/Tasks/`), a mais perto do próprio critério de
   aceite, nunca a fase seguinte da task que acabou de integrar só porque
   ela está à mão. dentro disso, maior ganho para a missão, respeitando
   dependências, nesta ordem: invariantes quebradas; bugs achados por validação real; contratos
   que várias partes precisam; fechar experimento ou hipótese aberta;
   observabilidade e capacidade de validar; funcionalidade nova;
   otimização; refinamento cosmético. nunca pelo número da task.
3. **brief.** nenhum agente sem briefing: código relevante relido, estado
   confirmado, objetivo, escopo, arquivos permitidos e proibidos, contratos
   que não mudam, aceite objetivo, testes, fora do escopo, quando parar e
   voltar. um agente por responsabilidade. sem agente verificador.
4. **implement.** o agente: branch e worktree próprios, lê antes de editar,
   não duplica abstração, não muda contrato central sem autorização, commits
   progressivos e isolados, testes, limites reportados, handoff
   reproduzível.
5. **validate.** o orquestrador não aceita pelo resumo: diff inteiro, commit
   por commit, escopo, duplicação, invariantes, testes específicos, gate,
   reprodução manual do comportamento crítico quando preciso, comparação com
   o comportamento anterior quando há risco de regressão. falha volta ao
   mesmo agente com correção específica. teste não se relaxa para caber a
   implementação. falha inesperada é informação: investigar antes de
   mascarar.
6. **learn.** depois de cada integração: o que aprendemos, que hipótese
   ganhou ou perdeu força, que pressuposto estava errado, que bug estrutural
   apareceu, que abstração ficou redundante, que task ficou desnecessária,
   que dependência mudou, que task nova é realmente necessária, que métrica
   mudou a prioridade. só aprendizado concreto; nada de task especulativa.
7. **update.** programa, estado das tasks, ADRs, contratos, arquitetura,
   dependências, riscos, métricas, decisões. task velha que não faz mais
   sentido se modifica, divide, substitui ou cancela.
8. **replan.** o próximo passo sai do estado novo, não do plano velho.
9. **melhorar o processo.** quando um briefing foi insuficiente, um aceite
   deixou passar falso positivo, um bug escapou do gate, dois agentes
   disputaram ownership, uma validação manual poderia ser teste, faltou algo
   no handoff, ou uma classe de erro se repetiu: corrigir o caso e o sistema
   que o permitiu (teste ou gate para a classe, contrato ou briefing menos
   ambíguo, regra de ownership, automação, revisão obrigatória antes do
   briefing). registrado em [`general.md`](autonomous-cycle-learnings.md), seção "processo".

## severidade antes de priorizar

toda descoberta nova (de validação, sonda, experimento, handoff ou
leitura de código) recebe uma severidade antes de virar task ou mudar
prioridade. só evidência concreta de P0 ou P1 gera trabalho bloqueante.

| nível | o que é | o que fazer |
|---|---|---|
| P0 | quebra uma invariante ou um contrato central | bloqueia o avanço: corrigir e provar com teste |
| P1 | invalida um experimento, benchmark, medição ou conclusão em uso | corrigir antes de repetir ou usar esse resultado |
| P2 | melhora explicabilidade, ergonomia ou completude sem invalidar comportamento | registrar; executar quando couber na onda, sem bloquear |
| P3 | otimização, cosmética, melhoria oportunista | backlog; não interrompe a missão |

a severidade fica escrita onde a descoberta é registrada (arquivo da task,
journal, programa). na dúvida entre dois níveis, a evidência decide: sem
cenário reproduzido que quebra a invariante ou invalida o resultado, não é
P0 nem P1.

ownership: se a próxima task lógica disputa uma superfície já em uso por
outro agente, escolher trabalho independente que avance a onda, em vez de
esperar ou dividir a superfície.

ondas: a seguinte não abre enquanto os critérios essenciais da atual não
estiverem satisfeitos (listados no programa, seção 6), e fechar uma onda
não exige perfeição local: o que sobra como P2 e P3 segue no backlog.

## autonomia

sem pedir aprovação a cada passo: criar, alterar, dividir, cancelar e
reordenar tasks; escrever briefings; disparar agentes; devolver correções;
integrar trabalho aprovado; atualizar ADRs e documentação; adicionar testes;
corrigir bugs achados na validação; reorganizar dependências; publicar cada
integração no GitHub (branch, PR, CI, merge commit; nunca squash nem rebase
merge, nunca force push); seguir para o próximo ciclo.

## parar e consultar o brenner somente quando

- mudar a tese central do plev;
- escolher entre arquiteturas fundamentais com consequências difíceis de
  reverter;
- alterar de forma incompatível um contrato central já adotado por várias
  partes;
- remover funcionalidade de forma significativa;
- nova dependência estrutural ou plataforma externa que mude o projeto;
- decisão de segurança sem resposta conservadora evidente;
- precisar de credenciais, serviços pagos ou recursos que o orquestrador não
  tem;
- resultado experimental que indique que uma hipótese central da missão
  falhou;
- duas alternativas tecnicamente plausíveis com trade-offs estratégicos
  relevantes;
- ação destrutiva ou irreversível fora do repositório local;
- criar task nova, ou abrir uma fase que o critério de aceite da task não
  pede.

não é motivo de parada: naming local, organização interna reversível, bug
pequeno, conflito mecânico, escolha de implementação encapsulada, criar
teste, corrigir documentação, refactor necessário para cumprir contrato já
aprovado. os checkpoints do programa (seção 9) param só quando o resultado
cai numa das condições acima; senão o orquestrador decide, registra no
programa e segue.

## contra gasto de tokens (2026-09-25)

o brenner parou o ciclo: em vez de fechar as tasks abertas, o orquestrador
abria fase nova da task que tinha acabado de integrar (75 fase 1, 2a, 2b),
transformava o "o que fica para depois" de cada handoff na fase seguinte e
criava task nova a partir de resultado de demo (96, 97). cada agente custou
de 500 a 660 mil tokens. as regras:

1. **uma task fecha pelo critério de aceite do arquivo dela.** antes de
   cada briefing o orquestrador confere a seção "critérios de aceitação"
   item por item. bateu, a task passa a `status: done` e não abre fase.
2. **todo briefing diz qual critério de aceite ele fecha.** fase que não
   fecha nenhum item do critério não tem briefing.
3. **o que sobra de uma entrega vira pendência, não fase.** vai para a
   seção "pendências" do arquivo da task, em lista curta. só vira trabalho
   se um item do critério de aceite pedir, ou com o ok do brenner.
4. **task nova só com o ok do brenner.** resultado de demo ou de medição
   se registra na task que mediu; não vira task.
5. **tudo esperando o brenner, o ciclo para.** se o que resta aberto depende
   de decisão, instalação ou teste manual dele, o orquestrador pergunta uma
   vez, com as decisões juntas, e espera. não inventa trabalho para seguir.
6. **agentes em paralelo quando as superfícies forem disjuntas.** desde
   2026-10-31 (brenner, task-104), no lugar de "um agente por vez", a
   primeira metade da regra de 2026-09-25. as condições estão em
   [orchestrate-coding-agents-on-this-repo](orchestrate-coding-agents-on-this-repo.md), "parallel work".
7. **o custo no journal.** a entrada de cada integração no
   [journal](autonomous-cycle-journal.md) diz quantos tokens
   o agente gastou; com agentes em paralelo, uma linha por agente. é a
   segunda metade da regra de 2026-09-25, que continua valendo.

## regra contra deriva

antes de qualquer task nova, resposta concreta para: como isso aproxima o
plev de um runtime em que intenção, estado e capacidades viram interação
nativa, segura, verificável e inspecionável? sem resposta concreta, não
executa. o projeto não vira coleção de features, competição de widgets,
laboratório infinito de otimização, reimplementação por orgulho nem
arquitetura pela arquitetura.

## três registros separados

| registro | arquivo | o que guarda |
|---|---|---|
| truth | `truth.md` | o que o código faz agora, com arquivo:linha; reescrito quando uma integração muda um contrato |
| program | [`generative-runtime-program.md`](../Synthesis/generative-runtime-program.md) | o que falta construir e por quê: tasks, grafo, ondas, contratos, ownership, decisões, estado |
| learnings | [`general.md`](autonomous-cycle-learnings.md) | o que experimentos e implementações ensinaram, e o que isso mudou no programa |

mais um: [`journal.md`](autonomous-cycle-journal.md), o diário do ciclo autônomo (integrações,
decisões autônomas, bugs, métricas, em ordem), que alimenta o `status`.
nenhum substitui outro.

## `status`

quando o brenner pedir `status`, a resposta sai destes arquivos e do `git
log`, nesta ordem: estado da missão; o que mudou desde a última intervenção
dele (a última entrada "intervenção do brenner" no journal); tasks
integradas; bugs relevantes; aprendizados; hipóteses confirmadas e
refutadas; decisões autônomas importantes; métricas; o que está rodando
agora; o próximo trabalho escolhido e por quê; as decisões que realmente
precisam dele.

## máquina

o ciclo roda nesta sessão local do Claude Code, com `/loop` dinâmico (volta
a cada 10 min enquanto há agente rodando, para pegar agente parado; o fim de
um agente acorda o orquestrador na hora). `caffeinate -i -w <pid>` mantém o
Mac acordado enquanto a sessão existir; um hook `Notification` avisa o
brenner no macOS quando o orquestrador espera por ele. regras de disco e de
build em [`orchestrate-coding-agents-on-this-repo.md`](orchestrate-coding-agents-on-this-repo.md).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0085: integration into main is a pull request merged by merge commit](../../adr/Workflows/0085-integration-by-merge-commit/0085-integration-by-merge-commit.md)
- [what the autonomous cycle taught about the way of working](autonomous-cycle-learnings.md)
- [Generative runtime session: what worked and what bit us](../../adr/Synthesis/0070-generative-runtime-results/generative-runtime.md)
- [task-75: production-ready platform accessibility](../../Tasks/task-75-platform-accessibility/task-75-platform-accessibility.md)
- [task-96: a generated surface in the ide](../../Tasks/task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-97: code and diff in the IR vocabulary](../../Tasks/task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [task-62 phase c, second round (2026-09-23)](../../Tasks/task-62-ide-human-agent-proof/phase-c-round-2.md)
- [task-62 phase c, third round: the task-89 rule (2026-09-24)](../../Tasks/task-62-ide-human-agent-proof/phase-c-round-3.md)
- [plev: thesis of the generative interaction runtime](../Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../Synthesis/generative-runtime-program.md)
- [orchestrator and agent guidelines](generative-runtime-guidelines.md)
- [how to orchestrate coding agents on this repo](orchestrate-coding-agents-on-this-repo.md)
- [autonomous cycle journal](autonomous-cycle-journal.md)
<!-- /related -->
