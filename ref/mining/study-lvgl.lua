return {
  id  = "stu-lvgl",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "biblioteca grafica embarcada em c puro; roda ui completa com ~100KB de ram",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: dirty regions e partial buffer rendering em c puro
para mcu sem gpu; o minimo absoluto de recursos. a prova de quao baixo o
chao pode ser.
o que extrair:
- dirty regions: invalidacao por area e merge de retangulos sujos
- partial buffer: renderizar a tela em fatias menores que o framebuffer
- o draw pipeline por software (sem gpu, sem malloc surpresa)
- gerenciamento da arvore de objetos e estilos com orcamento de bytes
o que NAO copiar: widgets/estilo; macros e oop-em-c.
consome: tese
]=],
}
