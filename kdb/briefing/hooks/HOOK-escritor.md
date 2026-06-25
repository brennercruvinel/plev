---
title: hook do agente escritor
status: aguardando revisao
aplica-se: livro, blog, tutoriais, reconstrucao de inputs
---

# hook do escritor

cole este checklist no fim do prompt de todo agente escritor. o agente entrega o
texto E o preenchimento deste hook. entrega sem o hook preenchido volta.

## antes de escrever

- [ ] li o contrato brennerwritter inteiro (`kdb/briefing/03-brennerwritter.md`)
- [ ] li os refs reais: `~/.claude/skills/brennerwritter/refs/voice.md`,
      `refs/taboo.md`, `refs/guide.md`
- [ ] identifiquei o destino e o perfil correto (blog, cientifico, doc, etc)
- [ ] levantei a ancora real (diff/commit/adr/bench) e tenho file:line
- [ ] para capitulo ou post grande: entreguei o sumario do passo 0 ao orquestrador
      e recebi o ok antes de escrever o corpo

## ao escrever

- [ ] apliquei os mantras (minuscula, sem em dash, voz, ritmo variado)
- [ ] todo trecho de codigo compila e foi conferido contra a versao do cargo.toml
- [ ] toda afirmacao tecnica tem ancora; nenhuma por suposicao
- [ ] preservei o sinal do material bruto, sem inventar conexao

## antes de entregar (passo 5, obrigatorio)

- [ ] rodei os 24 taboos no texto
- [ ] zero tolerancia limpos: emoji, em dash, chatbot artifacts, sycophantic,
      curly quotes, knowledge cutoff disclaimer
- [ ] qualitativos revisados: inflation, name dropping, cliche, copula avoidance,
      negative parallelism, regra do tres, hedging, conclusao generica
- [ ] sem emoji e sem em dash em lugar nenhum

## relatorio de entrega (preencher)

- destino e perfil aplicado:
- ancoras usadas (diff/commit/adr/bench, com file:line):
- taboos encontrados e corrigidos:
- o que NAO consegui confirmar (marcado no texto como nao confirmado):
- nao commitei: [sim/nao]
