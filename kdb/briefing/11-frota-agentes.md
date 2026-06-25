---
title: frota de agentes (desenho dos workflows, pos-aprovacao)
status: aguardando revisao
tags: [workflows, agentes, orquestracao, cloud]
---

# frota de agentes

so disparo isto depois do seu ok. aqui esta o desenho para voce avaliar antes.

## principios de orquestracao (do que ja deu certo neste repo)

de `kdb/how-to/orchestrate-coding-agents-on-this-repo.md`, regras que
sobreviveram ao contato com a realidade:

- diagnostico antes de frota. toda onda boa comecou com agentes de leitura
  read-only devolvendo causa raiz com file:line, e so entao a frota de execucao,
  com esses achados embutidos verbatim no prompt. a onda que pulou diagnostico
  produziu mudanca superficial que foi rejeitada.
- prompt com mecanismo produz fix de raiz; prompt com sintoma produz band-aid.
- particionar escopo: agentes compartilham a arvore so se os escopos sao
  declarados disjuntos no prompt ("toque so em X/"). quando precisam sobrepor,
  worktree isolada.
- agentes nao commitam. eu commito tematicamente depois de cross-validar.
- validacao obrigatoria no fim de cada prompt. claim visual exige numero de pixel.
- template de prompt que funcionou: missao em 1 frase (causa raiz), achados de
  diagnostico com file:line, escopo exato (permitido/proibido), a regra de design
  a impor, comandos de validacao obrigatorios, "nao commite; reporte mudancas,
  testes, numeros".

## o modelo de execucao (workflow tool + background/cloud)

- uso a ferramenta de workflow para as fases que sao fan-out determinístico
  (ler N arquivos, extrair N conversas, gerar N notebooks, escrever N posts).
- pipeline por padrao (cada item passa por todas as etapas sem barreira), barreira
  so quando uma etapa precisa do conjunto inteiro da anterior (dedupe global,
  early-exit por contagem zero).
- background/cloud para nao travar sua maquina. workflows longos rodam em
  background; sou notificado ao terminarem. sweep de orfaos depois de cada batch.
- entre fases, eu valido. nada entrega no final.

## as fases como workflows (escala proposta, ajustavel)

| fase | workflow | padrao | agentes aprox | verificacao |
|------|----------|--------|---------------|-------------|
| 1 cocriacao | inventario -> particao por tipo -> extracao -> hygienizacao -> dataset | pipeline + barreira no dedupe | 8 (A1..A8) + verificadores | critico de completude (A8) re-varre |
| 2 historiador | ler memoria/feedback + diffs + adrs -> correlacionar -> timeline | pipeline | 4-6 | eu confiro a timeline contra adrs datados |
| 3 corpus pesquisa | dedupe P1..P11 -> validar links -> limpar prosa -> yaml | pipeline | 6-10 | verificador de links + taboos |
| 4 benchmarks | 1 agente por notebook -> executa -> verifica reprodutibilidade | pipeline | ~9 + verificadores | re-execucao de subconjunto |
| 5 livro | por capitulo: leitor -> escritor (sumario p/ mim) -> corpo -> verificador adversarial | pipeline | 2-4 por capitulo | eu valido capitulo a capitulo |
| 6 blog | por post: leitor -> escritor -> verificador -> zola build | pipeline | 2-3 por post | build do zola verde |
| 7 tutoriais | por tutorial: escritor -> executa comandos -> verificador | pipeline | 2 por tutorial | smoke test roda |
| 8 mon | reconstrucao do notes.md + capitulo binario swf/monster | sequencial leve | 2-3 | eu valido, trabalho vivo |

numeros sao teto inicial, ajusto a frota ao tamanho real de cada fase (loop ate
secar na descoberta, nao corte arbitrario; se cortar, eu logo o que ficou de fora).

## o que todo prompt de agente carrega

1. missao em 1 frase, causa raiz.
2. achados de diagnostico inline (file:line) quando aplicavel.
3. escopo exato: caminhos permitidos e proibidos (ex: "escreva so em
   kdb/cocriacaoclaudinho/, leia so /Volumes/500G-SSD/claude2026 read-only").
4. o hook do tipo (`hooks/HOOK-*.md`) com validacao obrigatoria.
5. para escritores: o contrato brennerwritter inteiro (`03-brennerwritter.md`).
6. "nao commite; reporte mudancas, contagens, numeros, e rode seu hook".

## meu papel (orquestrador e validador)

eu nao despejo no final. para cada fase: disparo o workflow, recebo as entregas,
rodo a validacao (o hook do tipo + meu olho), devolvo o que veio raso com o hook
apontando o que faltou, e so libero a proxima fase quando a atual passou. eu sou
quem commita, tematicamente, mantendo cada commit buildavel.
