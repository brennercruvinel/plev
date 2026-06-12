return {
  id  = "shc-absorcao",
  typ = "plan",
  sts = "ws-showcase",
  dom = "showcase-library",
  dat = "2026-06-11",
  ttl = "absorver as demos phi no showcase + biblioteca linda e responsiva",
  lnk = { "ref-study", "prs-transpiler", "org-docs" },
  txt = [[
diretiva: nao copiar o visual nem o formato das demos; aprender, refatorar,
absorver o que ha de bom (dock, input, animacao, layout) padronizado na
linguagem hoff. backend testado antes de ui. a home ja tem input,
draggable, drag-and-drop: as abas novas estendem, nao duplicam.

mapa de absorcao (da analise individual das 10 demos):
- makepad_charts (o melhor achado): nucleo de desenho de charts (linha com
  eixos/grid/dots, barras, area empilhada, donut+legenda, reveal) vira
  widget puro testavel + aba charts didatica de alto nivel. minerar
  plotters/charming/egui_graphs (ref/vis) para eixos/escalas/legendas;
  lista leve sem clone: rerun, poloto, ratatui, nannou, bevy_prototype_
  lyon, petgraph (grafos)
- message_dock: o componente-produto ("nenhum concorrente tem isso"):
  dock de chat flutuante, hover levanta avatar, click faz morph de
  largura, glass BackdropBlur. o morph por hover e SO o primeiro dos
  efeitos. reconstruir sobre Tween/Spring dt-based + TextMeasurer
- builder_demo: aba builder ROBUSTA com tudo que aprendemos: hit regions
  reais (render_interactive; os botoes da demo eram fake), layout
  content-driven, texto medido. meta: melhor que toda demo de concorrente
  analisada
- todo_app: aba app (o app completo pequeno): dominio TodoItem/Filter +
  animacoes Tween por item + TextInput + filtros (achou 2 heuristicas
  chars*fator classicas; mata-las e parte da licao)
- text_input_demo: campos de linha unica entram no forms (o widget
  TextInput tem hoje ZERO consumidores): foco por click e tab, escape
  desfoca, click posiciona cursor
- counter: padrao Component/Lifecycle como exemplo minimo com estado
- layers/text/mobile_input: nada estrutural; amostras unicode/cjk viram
  smoke test de shaping

biblioteca (entregavel 2: "nao um monte de coisa jogada"):
- focus states FALTAM em 100 por cento dos widgets de src/ui/widgets:
  adicionar focused + focus ring consistente + navegacao por tab
- doc-header por widget no padrao do card.rs; re-exportar TextInput junto
- hover ja e consistente; manter
- auto-animate nativo (monster-formato item d) chega para todos de graca

gates por aba: testes puros de geometria/estado; viewport 600/1500px;
captura window-id com pixel vs tokens hoff; hover E focus visiveis;
render-on-demand respeitado (zero busy loop).
]],
}
