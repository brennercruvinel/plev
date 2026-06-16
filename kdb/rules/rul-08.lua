return {
  id  = "rul-08",
  typ = "rule",
  sts = "reference",
  dom = "theming",
  dat = "2026-03-13",
  ttl = "theming como struct com dimensoes comportamentais",
  lnk = { "idx-rules" },
  txt = [[
definir struct theme com todas as dimensoes de design como tokens de primeira classe. cores, escala tipografica, escala de espacamento e border radius sao a camada visual. motion physics, mass, tension, friction como parametros globais do sistema cinetico, e intent tokens, intent: destructive, constructive, neutral, informational como dado estrutural que propaga para cor, motion e aria simultaneamente, sao camadas comportamentais obrigatorias. componentes recebem &theme e leem tokens dele. zero valores visuais ou comportamentais hardcoded.

com struct, dark mode e theme::dark(), rebranding e um novo theme, e a sensacao fisica do produto, leveza ou solidez, e controlavel via theme.motion.mass e theme.motion.tension propagando coerentemente para todos os comportamentos cineticos. e a unica arquitetura onde coerencia fisica global e uma propriedade de design em vez de animacoes por-componente sem relacao sistemica. nao existe equivalente publico em nenhum framework rust atualmente.
]],
}
