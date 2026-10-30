<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

Todos os registros notáveis deste projeto são documentados aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não lançado]

### Adicionado
- Esqueleto do código: workspace Rust (`dotree-core`, `dotree-wasm`) e interface SvelteKit
  estática que carrega o núcleo em WebAssembly, com pt-BR e inglês (Paraglide).
- Testes de unidade e componente (Vitest) e de ponta a ponta (Playwright) contra a build com
  o caminho do GitHub Pages.
- `Dockerfile` em etapas e imagem servida por nginx sem privilégio na porta 8080.
- CI (Rust, interface, Docker, varredura de segredos), publicação no GitHub Pages, release
  por *tag* com imagem no GHCR, Dependabot.
- Spec v0.1 revisada: cores por termo, visão de variáveis indexadas, revelação em duas
  etapas, exemplos próprios, entrega (`docs/specs/09-entrega.md`).
- ADR 0002 (entrega por GitHub Actions, Pages e GHCR); plano por tickets em `docs/plano.md`;
  mapa de arquitetura em `docs/architecture.md`.

### Alterado
- Casos de referência trocados por exemplos próprios do projeto.
- Wallace e exportação CSV saem da v0.1.

## [0.0.0] - 2026-10-30

### Adicionado
- Estrutura inicial do repositório e regramento de agentes (perfil pessoal).
- Briefing, rascunho da spec v0.1 e ADR 0001 (núcleo Rust em WebAssembly).
