return {
  id  = "stu-gitlogue",
  typ = "study",
  sts = "pendente",
  dom = "ide",
  dat = "2026-06-11",
  ttl = "tui rust que reproduz a historia de um repo git como animacao de digitacao",
  lnk = { "ref-study", "org-docs" },
  txt = [=[
por que esta aqui: arquitetura de replay/animacao sobre dados git em
rust; ideia direta para o basic-ide: historia de commit como narrativa
visual.
o que extrair:
- replay de commits: como transforma diffs em eventos de digitacao
- syntax highlight e transicoes de arvore de arquivos no terminal
- o loop de animacao numa tui (timing, frames, interrupcao)
- leitura eficiente da historia git (libgit2? shelling out?)
o que NAO copiar: a estetica e o foco em "assistir"; queremos interagir.
consome: ws-ide (fila)
]=],
}
