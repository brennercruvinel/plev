---
title: hook do estagio de ingest (cocriacao)
status: revisado 2026-06-25
aplica-se: pipeline de dataset do claude2026
---

# hook do estagio de ingest

checklist operacional. o agente entrega o artefato do estagio e este hook
preenchido. entrega sem hook preenchido, ou com integridade quebrada, volta.

## isolamento e fonte

- [ ] fonte `/Volumes/500G-SSD/claude2026` lida em read-only; nenhuma escrita,
      mover ou delete na fonte
- [ ] saida apenas no meu sub-escopo de `kdb/cocriacaoclaudinho/`
- [ ] varri tambem arquivos ocultos (dotfiles)
- [ ] descarte via `trash`, nunca `rm`
- [ ] nao commitei

## proveniencia e schema

- [ ] toda unidade de saida tem lineage: campo `source` com o caminho original
- [ ] schema conforme `08-cocriacaoclaudinho.md`: `id`, `source`, `captured`,
      `model_tier`, `project`, `kind`, `turns`, `scrubbed`, `status`, `tags`
- [ ] `model_tier` preenchido quando inferivel; marcado `desconhecido` quando nao,
      nunca chutado
- [ ] dedupe por `id` (hash estavel); colisao resolvida, nao duplicada

## integridade de trajetoria

- [ ] ordem input/raciocinio/tool-call/resposta preservada na trajetoria
- [ ] tool calls e traces de raciocinio vinculados a trajetoria de origem
- [ ] input humano caotico reconstruido com brennerwritter, sinal preservado,
      versao bruta nao publicada

## scrub de dado pessoal

- [ ] dado pessoal sensivel removido antes de qualquer saida
- [ ] log do scrub registra so a categoria removida, nunca o conteudo
- [ ] caso ambiguo marcado e escalado ao orquestrador, nao decidido localmente

## relatorio de entrega (preencher)

- estagio e particao:
- arquivos lidos / unidades de saida / duplicatas descartadas:
- cobertura por tier e por projeto (numeros):
- unidades com `scrubbed=true` (contagem, sem conteudo):
- lacunas para a auditoria de completude:
- fonte intacta, sem commit: [sim/nao]
