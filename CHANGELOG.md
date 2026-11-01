<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

Todos os registros notáveis deste projeto são documentados aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não lançado]

### Adicionado
- Interface: ajuda sob demanda. Regras sempre abertas; dica (em ciclo, sem citar pontos) e
  próxima jogada (pontos destacados, a célula só se forma com outro clique), ambas
  apagadas na ação seguinte; limite inferior e solução de Dadda em duas etapas (resultado,
  depois o caminho), a solução num painel à parte sem mexer no circuito do aluno. Cada
  pedido pergunta antes, com o foco em "vou tentar mais um pouco"; o resumo do circuito
  completo lista as ajudas usadas. O núcleo ganhou o contador `HelpLog`, e a `Session`, os
  métodos `hint`, `nextMove`, `lowerBound`, `solution` e `helpLog` (MT-13).
- Interface: compressão até o circuito completo. "Fechar com CPA" fica desabilitado com a
  condição que falta; o painel do CPA e o do resultado, com os bits $R_j$ em roxo e a coluna
  vazia como 0; destaque do caminho crítico a pedido; voltar a um nível anterior, com
  confirmação quando algo seria descartado; marca "vai-um sempre 0" com a explicação atrás
  de confirmação enquanto $W$ estiver oculto; confirmação antes de descartar a compressão ao
  montar outra matriz. A `Session` ganhou `closeWithCpa`, `goToLevel` e, no snapshot, o
  motivo do bloqueio do CPA, o resultado e as células críticas (MT-12b).
- Interface: compressão nível a nível. Selecionar pontos com o mouse ou o teclado, formar
  FA e HA (retângulos em volta dos pontos, como nas aulas), recusas que citam a regra,
  concluir nível com um painel por nível, desfazer e refazer (botões e Ctrl+Z/Ctrl+Y),
  métricas ao vivo (FA, HA, AND, área, atraso) e altura de cada coluna. A ponte WASM ganhou
  a `Session`, que guarda o exercício no núcleo (MT-12a).
- Interface: alternância entre pontos e variáveis indexadas ($a_1 b_0$), pelos botões ou
  pela tecla V, lembrada entre visitas; desafio "quantos bits tem a saída?", que diz se o
  aluno acertou e revela $W$ (etapa 1) e, num segundo pedido, a conta de $v_{max}$ a $W$
  (etapa 2); diálogo de confirmação reutilizável, com o foco em "vou tentar mais um pouco".
  O núcleo passou a expor o máximo de cada entrada e de cada termo (MT-11b).
- Interface: formulário da expressão com uma largura por variável e os exemplos R1, R3 e R4;
  matriz de pontos colorida por termo, com legenda, no estilo das figuras de aula; erros e
  avisos do núcleo traduzidos nos dois idiomas. Ponte WASM `buildMatrix` (JSON com códigos)
  (MT-11a).
- Contrato visual (`docs/DESIGN.md`): cores dos pontos no estilo das aulas (um termo por cor,
  saídas de célula em preto, resultado em roxo), desenho de células e painéis, interação;
  paleta em *tokens* CSS conferida por teste de contraste e distância de cor (MT-0d).
- Núcleo: `help` com o método de Dadda (próxima jogada a partir do estado do aluno e solução
  automática, que é seguir a próxima jogada até o fim), catálogo de dicas e limite inferior
  de FA. O R4 do job `vhdl` passou a usar a solução Dadda (MT-8).
- Núcleo: `vhdl` gera `full_adder`, `half_adder`, o topo estrutural e o testbench (exaustivo
  até 20 bits, amostra de 4096 vetores acima), em VHDL-93 ASCII, com comentários e mensagens
  em pt-BR ou inglês. O CI ganhou o job `vhdl`, que simula os casos de referência no GHDL 6 e
  exige que um circuito estragado de propósito falhe (MT-10).
- Conferido à mão: o R2 gerado compila sem avisos no ModelSim ASE 10.1d e dá "TODOS OS 16384
  CASOS CORRETOS".
- Núcleo: `diagram` com o leiaute do diagrama de blocos em grade (linha por nível, coluna por
  peso), ligações rotuladas, saídas, células críticas recalculadas da netlist, tabela de
  ligações e sua exportação em Markdown com cabeçalhos vindos da interface (MT-9).
- Núcleo: `netlist` (o circuito como fios e células, com os identificadores de todos os
  artefatos) e `verify` (simulação bit-paralela, 64 vetores por vez; exaustiva até 24 bits de
  entrada, amostra xorshift64\* acima; contraexemplo; execução em blocos para progresso e
  cancelamento). R2: 16 384 casos corretos (MT-7).
- Núcleo: fechamento com CPA *ripple* (FA, HA ou fio por coluna, *carry-in* na primeira
  coluna), circuito completo com os bits do resultado, métricas (FA, HA, AND, área, atraso)
  e caminhos críticos. R2 reproduzido inteiro: 13 FA, 4 HA, 15 $A_{FA}$, 6 $T_{FA}$ (MT-6).
- Núcleo: compressão nível a nível (`dotree_core::reduce`). FA e HA com prontidão exata em
  meios $T_{FA}$, recusas por código, numeração das células, ordem canônica das entradas,
  vai-um além de $W$ descartado, voltar de nível, desfazer e refazer (MT-5).
- Núcleo: matriz de pontos (`dotree_core::matrix`). Decompõe coeficientes em cópias
  deslocadas, compartilha sinais entre cópias, conta portas AND, calcula $v_{max}$ e a
  largura de saída, e fixa a ordem canônica dos pontos (MT-4).
- Núcleo: linguagem da expressão (`dotree_core::expr`). Expande produtos de somas, combina
  termos iguais, valida larguras e o limite de 32 bits de entrada, e devolve erros como
  códigos com a coluna do problema (MT-3).
- Esqueleto do código: workspace Rust (`dotree-core`, `dotree-wasm`) e interface SvelteKit
  estática que carrega o núcleo em WebAssembly, com pt-BR e inglês (Paraglide).
- Testes de unidade e componente (Vitest) e de ponta a ponta (Playwright) contra a build com
  o caminho do GitHub Pages.
- `Dockerfile` em etapas e imagem servida por nginx sem privilégio na porta 8080.
- CI (Rust, interface, Docker, varredura de segredos), publicação no GitHub Pages, release
  por *tag* com imagem no GHCR, Dependabot.
- Spec v0.1 revisada: cores por termo, visão de variáveis indexadas, revelação em duas
  etapas, exemplos próprios, entrega (`docs/specs/09-entrega.md`).
- Governança do repositório público (ADR 0003): `main` só por PR com o check `CI ok`, só
  *squash*, tags `v*` só por admin, aprovação de workflows de fork, `SECURITY.md`, modelo de
  PR e `.github/governanca/aplicar.sh` para aplicar e conferir.
- ADR 0002 (entrega por GitHub Actions, Pages e GHCR); plano por tickets em `docs/plano.md`;
  mapa de arquitetura em `docs/architecture.md`.

### Alterado
- Casos de referência trocados por exemplos próprios do projeto.
- Wallace e exportação CSV saem da v0.1.

### Corrigido
- Núcleo: voltar ao nível que o CPA fechou reabria as células do CPA como pendentes, com
  entradas de vai-um que não são pontos do nível; agora só remove o CPA (MT-12b).

## [0.0.0] - 2026-10-30

### Adicionado
- Estrutura inicial do repositório e regramento de agentes (perfil pessoal).
- Briefing, rascunho da spec v0.1 e ADR 0001 (núcleo Rust em WebAssembly).
