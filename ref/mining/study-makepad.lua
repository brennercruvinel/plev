return {
  id  = "stu-makepad",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "ambiente de dev rust com ui runtime gpu e dsl de design live-editavel",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: o paralelo mais proximo do plev em ambicao: runtime
gpu proprio, dsl live, ide proprio. estudar as estruturas de dados que
eles acertaram.
o que extrair:
- turtle layout: o modelo de layout imperativo-com-cursor deles
- portallist + fenwick tree: listas infinitas com altura variavel
- live dsl: hot reload de design sem recompilar rust
- dockitem/docking: o modelo de paineis do ide
- como o mesmo codigo alveja desktop, web e mobile
o que NAO copiar: o ide inteiro nem a dsl como sintaxe; conceitos, nao
codigo.
consome: tese + ide
]=],
}
