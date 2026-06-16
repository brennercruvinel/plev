return {
  id  = "rul-11",
  typ = "rule",
  sts = "reference",
  dom = "persistence",
  dat = "2026-03-13",
  ttl = "persistencia via trait com migracao versionada",
  lnk = { "idx-rules", "rul-07" },
  txt = [[
definir trait storage { fn load(...) -> result<t>; fn save(...) -> result<()>; }. implementacoes concretas, rusqlite, sled, indexeddb, filesystem, ficam em modulos separados injetados na inicializacao. dominio e ui dependem do trait, nunca da implementacao.

alem do trait de acesso, definir trait migration { fn version() -> u32; fn up(db: &mut dyn storage) -> result<()>; } com vetor de migrations aplicadas em ordem na inicializacao. schema de dados persistidos versiona junto com o codigo, toda mudanca de estrutura e uma nova migration registrada. app em producao com dados reais nao pode assumir que o schema no device do usuario corresponde ao schema atual do codigo. sem migrations, atualizacoes de app ou corrompem dados silenciosamente ou exigem reset forcado, ambos sao falhas de produto.
]],
}
