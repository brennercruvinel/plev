return {
  id  = "stu-rhai",
  typ = "study",
  sts = "pendente",
  dom = "lang",
  dat = "2026-06-11",
  ttl = "linguagem de script embarcada e engine de avaliacao para rust",
  lnk = { "ref-study", "anm-formato" },
  txt = [=[
por que esta aqui: scripting embarcado rust; o espirito actionscript para
o nosso runtime de animacao. api de engine/scope/eventos e a fronteira
segura entre script do autor e engine.
o que extrair:
- api Engine/Scope/AST: compilar uma vez, avaliar por evento
- registro de tipos e funcoes rust expostos ao script
- sandboxing: limites de operacoes, profundidade, timeouts
- modelo de eventos/callbacks script <-> host
- custo de tree-walking vs pre-compilacao (quando vira gargalo)
o que NAO copiar: a sintaxe inteira; nosso script de autoria pode ser
mais estreito e tipado.
consome: ws-anim scripting
]=],
}
