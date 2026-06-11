return {
  id  = "stu-shame",
  typ = "study",
  sts = "pendente",
  dom = "lang",
  dat = "2026-06-11",
  ttl = "dsl leve para escrever pipelines gpu inteiros em rust (shaders no type system)",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: shaders metaprogramados em rust; estudar para o
futuro do nosso pipeline de efeitos. embute todo o shader/pipeline no
sistema de tipos rust, sem linguagem externa.
o que extrair:
- como o type system rust codifica estagios, bindings e layouts
- geracao do pipeline (o que vira wgsl/spirv, quando)
- ergonomia: o que uma dsl embutida ganha sobre wgsl em string
- limites do approach (beta): onde a metaprogramacao dói
o que NAO copiar: nada por enquanto; e aposta de futuro, nao dependencia.
consome: tese
]=],
}
