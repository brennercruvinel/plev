corpus da auditoria de texto por escrita (task-74, fase 1).

um arquivo por escrita. linhas que começam com `#` descrevem a frase:
`escrita`, `cobre` (o que a frase exercita), `significado` e `fonte da
frase`. cada linha restante é uma frase do corpus, curta e legível, escrita
à mão para este corpus (nada gerado por tabela unicode). escapes `\u{XXXX}`
são decodificados pelo teste, para que marca combinante, ZWJ e VS16 fiquem
visíveis no arquivo.

o leitor é rust/engine/tests/text_audit.rs; o golden do shaping com as
fontes embutidas é rust/engine/tests/fixtures/text_audit.txt.
