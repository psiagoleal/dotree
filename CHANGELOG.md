<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

Todos os registros notáveis deste projeto são documentados aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não lançado]

### Adicionado
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

## [0.0.0] - 2026-10-30

### Adicionado
- Estrutura inicial do repositório e regramento de agentes (perfil pessoal).
- Briefing, rascunho da spec v0.1 e ADR 0001 (núcleo Rust em WebAssembly).
