return {
  id  = "edt-flash-novo",
  typ = "vision",
  sts = "aprovado",
  dom = "animation-editor",
  dat = "2026-06-11",
  ttl = "a gui minima de autoria: o novo flash, construido no proprio plev",
  lnk = { "monster-formato", "ths-compilador", "ref-study" },
  txt = [[
vamos ter sim uma gui minima para construir/autorar animacoes, tipo o
flash. dogfooding: o editor e um app plev (prova a engine enquanto cria
conteudo pra ela). nasce DEPOIS do codec e do scripting verdes (backend
antes de ui, sempre).

forma minima v0 (crate 3-char, ex: mot):
- stage central: renderiza a cena do formato monster em edicao
- timeline embaixo: keyframes visiveis, scrub, play/pause, fps
- painel de propriedades: posicao/escala/rotacao/cor/easing do no
  selecionado
- export/import do formato v0; salvar e reabrir e o teste de aceitacao
- norte de ux: theatre.js (editor realtime de keyframes para a web; o
  feeling de scrub instantaneo e o que queremos)

scripting (o espirito actionscript): rhai embarcado como linguagem de
comportamento: on(frame/event), set de propriedades, controle de timeline.
seguro, leve, rust-native. o actionscript foi o que fez do flash uma
ferramenta de criacao e nao so de playback; rhai e essa alma em 2026.

arqueologia (estudar para nao repetir, nunca copiar):
- wick-editor: o sucessor espiritual do flash no browser; estudar o
  modelo de cena/clip
- synfig e opentoonz: cortaram muito mato; ui horrivel em c++, mas o
  modelo de tweening/bones/camadas e battle-tested; reconhecer o esforco
- pencil2d: minimalismo de timeline
- glaxnimate e friction: os atuais; modelos de keyframe/propriedade limpos
- openfl/haxeflixel: a api do flash reimplementada; mapa de que api o
  mundo sentiu falta
- thorvg: vetor leve com lottie player embutido; concorrente direto do
  nosso player

o pecado a nao repetir (ver plev): o flash morreu de opacidade
semantica. o nosso formato carrega a track de descricao textual e o
editor a expoe como campo de autoria (o autor descreve a cena; o nlp
leve do build so preenche o que faltar).
]],
}
