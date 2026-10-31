---
id: acervo
title: plev — acervo técnico
project: plev
type: indice
status: ativo
date: null
date_basis: gerado
topics: [navegacao]
relations: [convencoes, linha-do-tempo, cronologia-fontes]
source: docs/README.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# plev — acervo técnico

O plev começou em 2021 como um jeito de aprender Rust desenhando na GPU e virou
uma engine de composição com um contrato semântico que pessoas, leitores de tela
e agentes operam pelo mesmo caminho. Este acervo guarda essa trajetória: as
tasks, as decisões, as pesquisas, as medições e o que cada erro ensinou, com o
código que sustenta cada afirmação.

## Por onde entrar

| Para | Comece em |
|---|---|
| seguir a história no tempo | [linha do tempo](historia/linha-do-tempo.md), e [cronologia e fontes](historia/cronologia-e-fontes.md) para ler as datas |
| ver o que foi feito, task a task | [tarefas](tarefas/README.md) |
| entender por que o código é assim | [decisões](decisoes/README.md) |
| o que foi estudado antes de construir | [pesquisas](pesquisas/README.md), com a [tese do runtime generativo](pesquisas/tese-runtime-generativo.md) |
| números, método e limites | [experimentos](experimentos/README.md) |
| o que deu errado e o que ficou | [aprendizados](aprendizados/README.md) |
| trabalhar no código hoje | [referência](referencia/README.md), a começar pelo [manual da engine](referencia/operacao/code-against-the-plev-engine.md) |
| a arquitetura numa página | [ATLAS.toml](ATLAS.toml); os nomes antigos e os de hoje em [TERMS.toml](TERMS.toml) |

## Como as peças se ligam

Uma pesquisa levanta uma hipótese; uma task a implementa; um ADR fixa a decisão
que ficou; um experimento mede; um aprendizado diz o que mudou no entendimento.
Cada página aponta para as outras no campo `relations` do cabeçalho, e o
[CATALOG.json](CATALOG.json) reúne todas, com os títulos internos, para quem
quer consultar o acervo como dado. A forma de cada página está nas
[convenções](referencia/editorial/convencoes.md).

## Lendo um registro antigo

Uma página histórica fala com os nomes e os caminhos da sua época (`sem`,
`comps`, `showcase`, `crates/`…); o TERMS.toml leva cada um ao nome de hoje.
Um comando ou um número de uma task vale para a revisão em que foi escrito.
O git só guarda o histórico a partir de junho de 2026; antes disso, a data de
uma página vem do seu cabeçalho, e a [cronologia](historia/cronologia-e-fontes.md)
diz onde ela foi revista depois do fato.
