return {
  id  = "stu-thorvg",
  typ = "study",
  sts = "pendente",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "motor vetorial leve em c++ (~150KB) com player lottie embutido",
  lnk = { "ref-study", "anm-formato" },
  txt = [=[
por que esta aqui: player vetorial leve com lottie embutido; concorrente
direto do nosso player. estudar a api de cena e a disciplina de binario
pequeno multiplataforma.
o que extrair:
- api de cena: Canvas/Scene/Shape/Picture/Animation e seu ciclo de vida
- como o loader lottie mapeia json para a cena interna
- rasterizacao sw vs gl: a camada de abstracao de backend
- como mantem ~150KB de core (o que cortaram, o que e opt-in)
- primitivas suportadas (paths, gradientes, mascaras, trim, clip)
o que NAO copiar: c++ e a arquitetura de retained scene 1:1; estudar para
superar, e concorrente.
consome: ws-anim
]=],
}
