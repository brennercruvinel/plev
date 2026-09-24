---
project: plev
audience: [orchestrator, ai-agents]
type: operating-mode
status: in-force
since: 2026-09-23
tags: [orchestration, autonomy, loop, status]
---

# modo de operação: orquestrador autônomo

desde 2026-09-23 o brenner não supervisiona cada integração. o orquestrador
segue sozinho dentro da missão (`steps/ongoing/generative-runtime-thesis.md`),
das regras (`steps/ongoing/generative-runtime-guidelines.md`) e do programa
(`steps/ongoing/generative-runtime-program.md`). o objetivo não é executar a
lista de tasks: é manter um ciclo contínuo de implementação, validação,
aprendizado e reorganização do programa.

## o ciclo

`observe → choose → brief → implement → validate → learn → update → replan → repeat`

1. **observe.** antes de escolher trabalho novo, reconstruir o estado real da
   `main`: contratos, tasks integradas e pendentes, bugs, limites, métricas,
   dependências, ownership, decisões abertas. a documentação nunca tem mais
   autoridade que o código; divergência se investiga e a doc se corrige antes
   de planejar com ela.
2. **choose.** maior ganho para a missão, respeitando dependências, nesta
   ordem: invariantes quebradas; bugs achados por validação real; contratos
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
   briefing). registrado em `state/learnings.md`, seção "processo".

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
- ação destrutiva ou irreversível fora do repositório local.

não é motivo de parada: naming local, organização interna reversível, bug
pequeno, conflito mecânico, escolha de implementação encapsulada, criar
teste, corrigir documentação, refactor necessário para cumprir contrato já
aprovado. os checkpoints do programa (seção 9) param só quando o resultado
cai numa das condições acima; senão o orquestrador decide, registra no
programa e segue.

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
| truth | `state/truth.md` | o que o código faz agora, com arquivo:linha; reescrito quando uma integração muda um contrato |
| program | `steps/ongoing/generative-runtime-program.md` | o que falta construir e por quê: tasks, grafo, ondas, contratos, ownership, decisões, estado |
| learnings | `state/learnings.md` | o que experimentos e implementações ensinaram, e o que isso mudou no programa |

mais um: `state/journal.md`, o diário do ciclo autônomo (integrações,
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
build em `docs/how-to/orchestrate-coding-agents-on-this-repo.md`.
