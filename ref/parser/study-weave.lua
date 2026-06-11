return {
  id  = "stu-weave",
  typ = "study",
  sts = "pendente",
  dom = "parser",
  dat = "2026-06-11",
  ttl = "merge/diff estrutural por arvore (ataraxy labs); ve o que o git nao ve",
  lnk = { "ref-study", "prs-transpiler" },
  txt = [=[
por que esta aqui: diff estrutural (arvores, nao texto); a tecnica de
VERIFICACAO do output do transpiler: provar que entrada e saida sao
estruturalmente equivalentes ou que a mudanca e a esperada.
o que extrair:
- alinhamento de arvores entre duas versoes (matching de nos)
- como distingue mover codigo de apagar+criar
- resolucao de conflito por estrutura (funcoes diferentes nao conflitam)
- o contrato de equivalencia: quando duas arvores "sao o mesmo programa"
o que NAO copiar: acoplamento ao stack ataraxy (sem/inspect/opensessions).
consome: ws-parser (verificacao do transpiler)
]=],
}
