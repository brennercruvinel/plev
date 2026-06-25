---
title: cocriacao, ingest e dataset do claude2026
status: revisado 2026-06-25
tags: [dataset, harness, agentic, scaffolding, ingestao, ml]
destino: kdb/cocriacaoclaudinho
fonte: /Volumes/500G-SSD/claude2026 (read-only)
---

# cocriacao, ingest e dataset

## a ambicao real

esta fase nao e so preservacao. e a construcao de um corpus de trajetorias
agenticas de qualidade alta o suficiente para ser referencia de harness e de
agentic scaffolding. anos de interacao real (input humano, raciocinio, tool
calls, memoria, codigo que deu certo e que deu errado) sao exatamente o material
que falta no mercado de dados para tooling agentico. o objetivo e que o dataset
sustente escrutinio de quem constroi modelo, e que sirva tres saidas ao mesmo
tempo: dataset versionado, exemplos do livro, e notebooks de analise.

cobertura por tier de modelo importa. o corpus atravessa varias geracoes e
tamanhos de modelo (do Opus as tiers menores, ate o Fable), o que torna o dado
util para estudar como scaffolding e comportamento de harness variam com a
capacidade do modelo. os rotulos exatos de tier sao confirmados com voce e
gravados no schema, nao chutados.

## a fonte

`/Volumes/500G-SSD/claude2026`: 17gb, 68.029 arquivos. read-only, so copia para
`kdb/cocriacaoclaudinho`. distribuicao por tipo:

- 5.966 `.jsonl`: traces de conversa, uma mensagem por linha (formato natural de
  trajetoria agentica)
- 6.703 `.json`: exports, memoria de mcp, payloads de tool call
- 2.664 `.md`: memoria e preferencia (`__MEMORY.md`, `feedback_*`, `project_*`,
  `user_*`), incluindo o periodo phi
- 19.873 `.js`, 4.108 `.ts`, e o restante: codigo dos experimentos, inclusive os
  que falharam

ha monolitos json grandes e arquivos ocultos. nomes antigos do projeto (phi)
aparecem na memoria. nada na fonte e editado.

## o schema da trajetoria (alvo)

cada unidade do dataset e uma trajetoria normalizada, com proveniencia completa:

```yaml
---
id: <hash estavel>
source: <caminho original>           # lineage para auditoria
captured: <timestamp inferido>
model_tier: <opus|sonnet|haiku|fable|desconhecido>   # confirmado no schema
project: <phi|plev|outro>
kind: <trajetoria|trace-tool|trace-reasoning|memoria|codigo|experimento-falho>
turns: <n>                            # input humano + resposta + tool calls
scrubbed: <true|false>                # passou pelo scrub de dado pessoal
status: <bruto|normalizado|reconstruido|qa-ok>
tags: [<assunto>, ...]
---
```

input humano caotico (transcricao truncada, braindump, nota mesclada) e
reconstruido com brennerwritter para ficar legivel, preservando o sinal, sem
inventar conexao. a versao bruta nunca aparece no dado publicado; so a
reconstruida e o log de que houve reconstrucao.

## o pipeline (8 estagios, paralelizaveis por particao)

| estagio | funcao | saida |
|---------|--------|-------|
| ingest | varredura completa (inclusive ocultos), classificacao, lineage | manifesto com tipo, tamanho, ano, projeto e tier inferidos |
| parse de trajetoria | jsonl e json em sequencias input/output/tool-call ordenadas | trajetorias normalizadas, ordem preservada |
| extracao de trace | isolar tool calls e raciocinio como traces de primeira classe | traces tipados, vinculados a trajetoria de origem |
| mineracao de memoria | os md de memoria e preferencia em timeline de decisao | timeline que alimenta o historiador (fase 2) |
| harvest de codigo | snippets ok e experimentos falhos, com proveniencia e diagnostico | corpus de codigo rotulado ok/falho, pronto para patch |
| reconstrucao de input | brennerwritter sobre o input humano caotico | input legivel + log de reconstrucao (categoria, nao conteudo) |
| montagem do dataset | schema final, dedupe por hash, splits, versionamento | dataset versionado com schema documentado |
| qa e auditoria de completude | o que nao foi varrido, claim nao verificado, lacuna de cobertura | relatorio que vira a proxima rodada |

dedupe e cross-stage: a montagem deduplica por hash estavel para nao contar a
mesma trajetoria vinda de jsonl e de json. o estagio de qa re-varre e mede
cobertura por tier e por projeto, e nada de "cobrimos tudo" sem o numero que
prova.

## qualidade e integridade (o que separa dataset serio de dump)

- proveniencia obrigatoria: toda unidade aponta o caminho de origem. nada
  anonimo sem lineage.
- ordem de trajetoria preservada: input, raciocinio, tool call e resposta na
  sequencia real. trajetoria embaralhada e dado morto para harness.
- scrub de dado pessoal antes de qualquer saida externa: remove dado pessoal
  sensivel, loga so a categoria removida, nunca o conteudo. na duvida, marca e
  escala, nao decide sozinho.
- nada de numero inventado, nada de trajetoria sintetica passando por real.
- publicar e irreversivel: o corpus passa por uma revisao de saida antes de
  qualquer destino externo (marketplace, parceria, dataset publico).

## execucao (fase 1)

8 agentes, um por estagio (particionados por tipo de arquivo onde o estagio
permite), em background para nao prender a sua maquina. cada um carrega o hook
`hooks/HOOK-cocriacao-extrator.md`. eu valido a primeira leva antes de escalar a
varredura completa dos 68k arquivos.
