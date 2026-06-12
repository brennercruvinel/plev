return {
  id  = "stu-ruffle",
  typ = "study",
  sts = "pendente",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "emulador do flash player em rust (desktop e wasm); leitor canonico do formato swf",
  lnk = { "ref-study", "monster-formato" },
  txt = [=[
por que esta aqui: COMO o swf codifica tweens (matrizes, morphs,
place/remove object) e como o player invalida e renderiza com
performance. e a leitura de referencia do formato que queremos superar.
o que extrair:
- parsing das tags PlaceObject/RemoveObject e DefineMorphShape (swf/src)
- como tween vira interpolacao de matriz 2x3 por frame na display list
- estrategia de invalidacao/dirty state da cena e render sob demanda
- a separacao core/render e os backends (wgpu inclusive) e o alvo wasm
- como depth/clip layers organizam a cena sem arvore pesada
o que NAO copiar: o player em si; compatibilidade actionscript (avm1/avm2).
consome: ws-anim
]=],
}
