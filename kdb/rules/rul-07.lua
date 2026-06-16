return {
  id  = "rul-07",
  typ = "rule",
  sts = "reference",
  dom = "side-effects",
  dat = "2026-03-13",
  ttl = "side effects isolados com abstracao de runtime",
  lnk = { "idx-rules", "rul-11" },
  txt = [[
nenhum io, network, filesystem, ipc, timers longos, acontece dentro de render(), dentro de callbacks de componente, ou de forma sincrona dentro de handlers de action. io dispara via spawn assincrono e retorna como nova action no fluxo normal.

o spawn de tasks usa tokio no nativo e wasm-bindgen-futures no browser. essa divergencia nao pode vazar para o codigo de dominio como cfg(target_arch) inline, isso criaria exatamente o acoplamento de plataforma que rul-11 proibe. o padrao correto e definir trait taskspawner { fn spawn(&self, fut: impl future<output = action> + static); } com implementacoes concretas por plataforma injetadas na inicializacao, junto com o trait storage. o codigo de dominio chama spawner.spawn(...) sem saber o runtime subjacente. platform-awareness fica confinada ao ponto de inicializacao do app.
]],
}
