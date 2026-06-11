return {
  id  = "stu-shadcn-ui",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "componente como dado copiavel, nao dependencia; o modelo de distribuicao",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: o modelo de distribuicao: componente e codigo-fonte
que o usuario copia e possui, nao pacote que ele instala e reza. "use
this to build your own component library".
o que extrair:
- o registry: schema json que descreve componentes, deps e arquivos
- a cli: como resolve, copia e adapta codigo pro projeto do usuario
- anatomia de componente pensado para ser editado, nao encapsulado
- como versionam algo que vive copiado em milhares de repos
o que NAO copiar: tailwind/radix; o que importa e o modelo, nao o stack.
consome: tese + biblioteca
]=],
}
