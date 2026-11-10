<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

Todos os registros notáveis deste projeto são documentados aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não lançado]

### Adicionado
- "Como usar", no cabeçalho, em português e inglês: os passos (expressão ou perfil de
  alturas, largura, compressão, CPA, verificação e exportação), os atalhos de teclado, o
  que cada ajuda revela e como é contada, as limitações e as referências públicas. Abre
  sem confirmação, não conta ajuda e não depende do exercício aberto; Esc ou "Fechar"
  devolvem o foco ao botão. Os atalhos da página não agem com um diálogo aberto
  (GER-11, MT-40b).
- README da v0.2: o que o app faz, passo a passo, limitações (cada uma com o critério ou
  o ticket que a trataria), metodologia e referências públicas (Parhami, caps. 8, 11 e 12;
  Wallace, 1964; Dadda, 1965; manual da DE2). ADR 0004: uma spec só, viva, atualizada no
  mesmo PR que muda o app, e só referências de acesso público; a spec 00 passa a listar o
  fora de escopo da versão atual. Tickets MT-41 e MT-42 para as limitações da entrada por
  alturas (MT-40a).
- Entrada por alturas: o campo da expressão aceita `{2,2,3; 1,1,1,1}`, as alturas das
  colunas de entrada e, depois do `;`, as de saída, da mais significativa para a menos
  (a notação dos contadores paralelos generalizados do cap. 8 do Parhami, com a saída
  também por alturas). Cada ponto vira uma variável de 1 bit, sem larguras a informar, e
  há um exemplo pronto. Um painel mostra o perfil pedido, as colunas ainda acima dele e,
  quando atingido, a área e o atraso do circuito. Ao lado de Dadda e Wallace, a **solução
  ótima** (menor área, depois menor atraso), em duas etapas e aplicável, e o nome do
  bloco conhecido (FA, HA, somador, *carry-save*, contador ou compressor) são ajudas
  pedidas e contadas (`help.optimal`, `help.optimalPath`, `help.optimalApplied`,
  `help.block`, opcionais; esquema 1 mantido). Perfis que nenhum circuito atinge são
  recusados com a coluna. O ótimo reproduz os 11 perfis de referência do projeto,
  conferidos à mão e por uma busca exaustiva independente (EXP-16, EXP-17, CMP-19,
  AJU-12, MT-31).
- Comparação de soluções: o seu circuito ao lado das soluções de Dadda e de Wallace já
  pedidas, com área, atraso, FA, HA, AND, níveis, largura do CPA, células no caminho
  crítico, área × atraso, área × atraso² e quem é não dominada; a tabela de alturas de
  cada solução entra quando os níveis dela foram vistos. A comparação não revela nem conta
  ajuda (AJU-11, MT-38, versão mínima).
- Tabela de alturas: a altura de cada coluna em cada nível, com os FA e HA formados sobre
  ele, o nível atual e, no fim, o resultado; sempre visível e exportável em Markdown
  (`dotree-alturas.md`), na forma tabular da fig. 8.11 do Parhami (CMP-18, MT-30).
- Solução de Wallace ao lado da de Dadda, com o mesmo fluxo: métricas, depois os níveis
  com a regra do método numa frase, e "Usar a solução de Wallace". A regra é a da
  fig. 8.10 do Parhami: todos os FA que cabem em cada coluna, HA só
  quando ela ainda passaria do alvo $2\lfloor h/3 \rfloor + (h \bmod 3)$. Cada método
  tem a própria contagem no registro de ajudas e no arquivo do exercício
  (`help.wallace`, `help.wallacePath`, `help.wallaceApplied`, opcionais; esquema 1
  mantido) (AJU-10, MT-29).
- Constante somada: `A + 36` e `3(A + 1)` passam a valer. As constantes se juntam num termo
  só, com cor própria, e cada bit 1 vira um ponto fixo em 1 (`one`, mostrado como 1), que o
  aluno comprime como os outros; a constante entra em $W$, na verificação e no VHDL
  (`one <= '1'`). Uma expressão só com constantes é recusada (EXP-13, EXP-15, MT-28).
- Placa DE2: com até 18 bits de entrada, o VHDL traz um topo `<nome>_de2` que lê as
  variáveis das chaves `SW` e mostra o resultado nos LEDs `LEDR` e, em hexadecimal, nos
  displays `HEX0` a `HEX7`; um testbench próprio; e o projeto do Quartus II (`.qpf` e
  `.qsf`) com a EP2C35F672C6 e os 92 pinos da placa. O painel diz se esses arquivos vieram
  ou por que não. Compilado no Quartus II 13.0 e simulado no GHDL e no ModelSim (spec 10,
  MT-22).
- "Usar esta solução": depois de ver a solução de Dadda nível a nível, o aluno pode trocar
  o próprio circuito por ela, com confirmação. A troca é um só passo de desfazer, conta no
  registro de ajudas do resumo e vai para o arquivo do exercício (`help.solutionApplied`,
  opcional, esquema 1 mantido). Não aparece no modo leitura (AJU-09, MT-20).
- VHDL comportamental: `<nome>_comportamental.vhd` tem as mesmas portas do topo e calcula
  a expressão com `numeric_std`, sem células, para comparar no Quartus o circuito montado
  com o que o sintetizador escolhe. O testbench confere as duas versões e informa o
  resultado de cada uma; o CI e o ModelSim ASE 10.1d simulam as duas (VHD-10, MT-21).

### Alterado
- "Baixar tabela (Markdown)" virou "Baixar tabela de ligações (Markdown)", para não se
  confundir com a tabela de alturas.
- O botão "Solução automática" virou "Solução de Dadda", e os textos da solução (etapa 2,
  usar, registro de ajudas, aviso de aplicação) passaram a nomear o método.
- O código de erro `constantTerm` deu lugar a `noVariables` (expressão sem variável).
- Os nomes `dut`, `seg7`, `SW`, `LEDR` e `HEX0` a `HEX7` não servem mais como nome da
  entidade, porque o topo da placa os usa (PLA-07).
- O testbench gerado instancia também a versão comportamental: as mensagens finais passam
  a ser "ESTRUTURAL: …" e "COMPORTAMENTAL: …", e a simulação falha se qualquer uma errar.

### Segurança
- `minimatch` forçado para 11.0.1 (override do pnpm) por causa de GHSA-g3hv-75wg-m249
  (CVE-2026-118764, moderada, negação de serviço por padrão de glob). Só o lint usa o
  pacote, com os padrões da nossa configuração, e nada dele vai para o site. O override
  sai quando o ESLint e o typescript-eslint aceitarem a 11.

## [0.1.0] - 2026-11-05

Primeira versão de uso: da expressão à matriz de pontos, compressão feita pelo aluno com
FA, HA e CPA, ajuda sob demanda em duas etapas, diagrama de blocos, tabela de ligações,
verificação exaustiva no navegador e VHDL-93 com testbench, em português e inglês.

### Adicionado
- O exercício fica salvo neste navegador a cada ação e revelação, e a visita seguinte
  oferece retomá-lo ou descartá-lo. Com o armazenamento bloqueado ou cheio, o app funciona
  normalmente e avisa uma vez que o progresso não será lembrado. Um estado salvo
  corrompido ou de esquema antigo não é aberto: a página explica o motivo e oferece baixar
  o conteúdo antes de descartá-lo (MT-15b).
- Exportar e abrir o exercício como arquivo JSON com versão de esquema (expressão,
  larguras, células por nível, células pendentes, resposta do desafio da largura e
  registro de ajudas). Ao abrir, o núcleo refaz cada célula pelas regras; arquivo
  malformado, de outro tipo, de esquema antigo ou futuro, com expressão inválida ou célula
  inconsistente é recusado com o motivo, sem mexer no exercício aberto. O que já tinha sido
  revelado (W, a conta de W, limite inferior, solução) volta à tela sem pedir confirmação
  nem contar de novo. Novo módulo `state` no núcleo, que passou a depender de `serde` e
  `serde_json` (MT-15a).
- Interface: verificação do circuito completo num *worker* (o R2 dá "16.384 casos,
  exaustivo, 0 erros"), com barra de progresso depois de 0,5 s, cancelamento sem resultado
  parcial e, se algum caso falhar, o primeiro contraexemplo (entradas, esperado, obtido em
  decimal e binário, bits que diferem). Painel VHDL com nome da entidade validado pelo
  núcleo, cópia de cada arquivo e `.zip` com todos; verificação e VHDL desabilitados, com o
  motivo, enquanto o circuito está incompleto. A `Session` ganhou `netlist` e `vhdl`; a
  classe `Verification` roda no *worker* (MT-14b).
- Interface: diagrama de blocos em SVG (uma linha por nível, maior peso à esquerda, bloco
  de portas AND que se expande em portas, ligações rotuladas com o sinal, vai-um do CPA na
  horizontal, bits $R_j$ ao fim, células críticas destacadas pelo mesmo botão dos painéis)
  e tabela de ligações, ambos até o último nível concluído; exportações em SVG e Markdown
  só com o circuito completo (o Markdown sai igual à tabela de referência da spec 05). A
  `Session` ganhou `diagram`, `wiringTable` e `wiringTableMarkdown` (MT-14a).
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
  FA e HA (retângulos em volta dos pontos), recusas que citam a regra,
  concluir nível com um painel por nível, desfazer e refazer (botões e Ctrl+Z/Ctrl+Y),
  métricas ao vivo (FA, HA, AND, área, atraso) e altura de cada coluna. A ponte WASM ganhou
  a `Session`, que guarda o exercício no núcleo (MT-12a).
- Interface: alternância entre pontos e variáveis indexadas ($a_1 b_0$), pelos botões ou
  pela tecla V, lembrada entre visitas; desafio "quantos bits tem a saída?", que diz se o
  aluno acertou e revela $W$ (etapa 1) e, num segundo pedido, a conta de $v_{max}$ a $W$
  (etapa 2); diálogo de confirmação reutilizável, com o foco em "vou tentar mais um pouco".
  O núcleo passou a expor o máximo de cada entrada e de cada termo (MT-11b).
- Interface: formulário da expressão com uma largura por variável e os exemplos R1, R3 e R4;
  matriz de pontos colorida por termo, com legenda; erros e
  avisos do núcleo traduzidos nos dois idiomas. Ponte WASM `buildMatrix` (JSON com códigos)
  (MT-11a).
- Contrato visual (`docs/DESIGN.md`): cores dos pontos (um termo por cor,
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
- O VHDL fecha cada entidade com `end <nome>;` (antes `end entity <nome>;`), forma que o
  FPGAEmuWeb exige para ler as portas do topo; cada componente continua num arquivo
  próprio (VHD-09, MT-19).
- Conferido à mão para esta versão: o R2 gerado compila sem avisos no ModelSim ASE 10.1d e
  dá "TODOS OS 16384 CASOS CORRETOS"; o R2 estragado de propósito falha (MT-17).
- A verificação usa um único *worker*, carregado junto com a página; cancelar envia uma
  mensagem em vez de encerrar o *worker* (MT-16).
- Abaixo de 768 px o exercício fica em modo leitura, com aviso: matriz, ajuda, diagrama,
  verificação e VHDL continuam disponíveis, mas não se formam células (GER-10, MT-16).
- Casos de referência trocados por exemplos próprios do projeto.
- Wallace e exportação CSV saem da v0.1.

### Corrigido
- Sem rede depois de carregar a página, a verificação falhava, porque o *worker* e o WASM
  só eram buscados ao rodar; a mensagem de erro também saía vazia (GER-05, MT-16).
- Acessibilidade: na visão indexada, o nome do ponto agora é o texto visível (`a1b0`); o
  diagrama e a tabela de ligações, roláveis, recebem foco pelo teclado (GER-07, MT-16).
- Plurais: "1 ponto", "1 porta AND", "1 nível" (antes "1 pontos" etc.); a recusa do estado
  salvo diz "o conteúdo salvo" em vez de "o arquivo" (MT-16).
- Com os dados do site bloqueados no navegador, a página não montava: o layout lia
  `window.localStorage` direto ao escolher o idioma (MT-15b).
- Núcleo: voltar ao nível que o CPA fechou reabria as células do CPA como pendentes, com
  entradas de vai-um que não são pontos do nível; agora só remove o CPA (MT-12b).

## [0.0.0] - 2026-10-30

### Adicionado
- Estrutura inicial do repositório e regramento de agentes (perfil pessoal).
- Briefing, rascunho da spec v0.1 e ADR 0001 (núcleo Rust em WebAssembly).
