---
id: c7e8cfbb4cec
source: /Volumes/500G-SSD/claude2026/Library-Application-Support-Claude/Claude Extensions/ant.dir.ant.anthropic.filesystem/node_modules/ajv/lib/compile/errors.ts
captured: 2026-06-02T19:49:36Z
model_tier: desconhecido
project: outro
kind: codigo
turns: 0
scrubbed: false
status: normalizado
tags: [codigo, typescript, dependencia, terceiro, ajv, json-schema, mcp-extension, filtrar-no-qa]
---

# ajv errors.ts (dependencia vendorada, sem cocriacao)

modulo de geracao de erro do ajv, o validador de json-schema. nao e codigo do
brenner nem trajetoria agentica: e dependencia de terceiro empacotada dentro do
node_modules da extensao filesystem do claude desktop (ant.dir.ant.anthropic.
filesystem). entra no corpus so porque a varredura pegou a arvore inteira da
extensao, incluindo as deps transitivas.

## diagnostico

o arquivo monta os objetos de erro que o ajv emite quando uma keyword de schema
falha. trabalha sobre a camada de codegen do proprio ajv (CodeGen, Name, Code) e
expoe um punhado de funcoes: reportError e reportExtraError (anexam um erro ou
retornam a lista), resetErrorsCount, extendErrors (propaga instancePath e
schemaPath pelos erros ja acumulados), e a familia errorObject/errorObjectCode/
extraErrorProps que materializa keyword, params, message, schema e propertyName
conforme as opts (messages, verbose). e codigo de biblioteca estavel, MIT, sem
estado proprio do projeto.

## por que fica fora da camada de cocriacao

[ ] nenhum input humano, nenhuma sequencia input -> raciocinio -> tool-call
[ ] nenhuma decisao do brenner, nenhum experimento, nem ok nem falho
[ ] proveniencia e node_modules de uma extensao, nao um workspace de projeto

registrado aqui so para manter lineage (todo arquivo varrido aponta a origem). a
recomendacao para a montagem do dataset e filtrar a classe inteira de
dependencia vendorada (node_modules) antes de qualquer split. mantido como
unidade de codigo rotulada terceiro, nao como material de harness.

## scrub (log)

- categorias avaliadas: secrets/tokens, emails, nomes proprios, paths de usuario
- removido: nada. e codigo open-source publico (ajv), sem PII
