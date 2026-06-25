---
title: hook do agente de benchmark
status: aguardando revisao
aplica-se: jupyter notebooks de benchmark
---

# hook do benchmark

## regra dura: o notebook tem que rodar

- [ ] o notebook executa do topo ao fim com kernel limpo, sem erro
- [ ] re-executei depois de limpar o estado, e o resultado se mantem

## integridade do dado

- [ ] todo numero vem de medicao real (criterion, csv, execucao), nao digitado
- [ ] todo grafico gera a partir do dado, nao e imagem colada
- [ ] declarei hardware, so, versao do rust, versao da crate
- [ ] divergencia entre rodadas registrada, nao mascarada
- [ ] nao comparei numero do plev contra numero de terceiro medido em outra
      maquina como se fosse equivalente

## saida para o livro e o paper

- [ ] o notebook produz a figura e o paragrafo-resumo que o capitulo 5 vai usar
- [ ] honestidade: onde o ganho e grande e comprovado, e onde e marginal

## relatorio de entrega (preencher)

- benchmark e crate/bench:
- ambiente (cpu, so, rust, crate version):
- numeros principais:
- o notebook roda do zero: [sim/nao]
- nao commitei: [sim/nao]
