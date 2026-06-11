return {
  id  = "stu-slint",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "nota: slint NAO foi clonado aqui; copia ja existe em ref/render-texts/slint",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: o plano de mining inclui slint (linguagem .slint
compilada, runtime minimo), mas a copia ja existe em
ref/render-texts/slint (verificado em 2026-06-11). nao duplicar clone;
este no so aponta para la.
o que extrair (do clone em render-texts):
- o compilador .slint -> codigo (rust/c++) e o runtime minimo
- property system reativo resolvido em compile time
- como suportam mcu (software renderer) e desktop com o mesmo modelo
o que NAO copiar: a linguagem .slint em si; o interesse e o pipeline
compilado.
consome: tese (cam-2/3)
]=],
}
