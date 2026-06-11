return {
  id  = "stu-min-sized-rust",
  typ = "study",
  sts = "pendente",
  dom = "lang",
  dat = "2026-06-11",
  ttl = "doc-only: guia de dieta de binario rust (README salvo em min-sized-rust.md)",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: doc de dieta de binario; nao e clone, e o README de
johnthagen/min-sized-rust salvo ao lado (min-sized-rust.md). o player e
o output do compilador precisam ser minusculos.
o que extrair:
- flags de release: opt-level=z, lto, codegen-units, strip
- panic=abort e build-std para cortar a std
- ferramentas de medicao (cargo-bloat, twiggy para wasm)
- trade-offs de cada flag (tamanho vs velocidade vs debug)
o que NAO copiar: receitas as cegas; medir cada flag no nosso wasm.
consome: tese
]=],
}
