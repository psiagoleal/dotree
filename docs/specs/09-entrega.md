<!-- Caminho relativo: docs/specs/09-entrega.md -->

# 09 — Entrega: CI, publicação e imagem Docker

## Resultado

Cada mudança é verificada automaticamente, e o que chega a `main` vai ao ar no GitHub Pages
sem passo manual, num endereço que pode ser enviado a quem quiser ver (inclusive o
professor). A mesma build roda num contêiner para quem quiser hospedar por conta própria.
Decisão e alternativas: [ADR 0002](../adr/0002-entrega-github-pages-e-actions.md).

## Critérios

- **ENT-01** — QUANDO houver *push* ou *pull request* para `main`, o CI DEVE rodar, e falhar
  se qualquer um falhar: formatação e lint do Rust (`cargo fmt --check`, `cargo clippy` sem
  avisos), testes do núcleo (`cargo test`), build do WASM, checagem de tipos e lint da
  interface, testes de unidade e componente, build do site e testes de ponta a ponta no
  Chromium.
  *Prova:* o workflow `ci.yml` existe com esses passos, e um PR com um teste quebrado de
  propósito fica vermelho (registrado no ticket que criar o CI).
- **ENT-02** — QUANDO o CI passar num *push* para `main`, o sistema DEVE publicar o site
  estático em `https://psiagoleal.github.io/dotree/`.
  *Prova:* o job `pages` do `ci.yml` publica; teste de fumaça pós-publicação abre o endereço e
  encontra o título do app.
- **ENT-03** — O site publicado DEVE funcionar sob o caminho `/dotree/`: rotas, arquivos
  estáticos e o módulo `.wasm` carregam a partir desse prefixo.
  *Prova:* o teste de ponta a ponta roda contra a build com o mesmo prefixo.
- **ENT-04** — O repositório DEVE ter um `Dockerfile` em etapas (Rust → WASM, Node → site,
  servidor estático) cuja imagem final serve o site na porta 8080, sem ferramentas de build.
  *Prova:* CI constrói a imagem, sobe o contêiner e faz `curl` na página inicial e no
  `.wasm` (código 200 e tipo `application/wasm`).
- **ENT-05** — QUANDO uma *tag* `v*` for criada, o sistema DEVE publicar a imagem em
  `ghcr.io/psiagoleal/dotree` com a versão e criar a *release* do GitHub com a seção do
  CHANGELOG.
  *Prova:* workflow `release.yml`; conferido na primeira *tag*.
- **ENT-06** — ONDE houver gerador de VHDL (spec 06), o CI DEVE analisar e simular o VHDL
  gerado com GHDL em contêiner (VHD-01, VHD-04).
  *Prova:* job `vhdl` no `ci.yml` (desde o MT-10): `examples/vhdl_referencia.rs` gera os
  casos e `examples/ghdl_casos.sh` confere o veredito de cada um; o `CI ok` depende dele.
- **ENT-07** — O repositório DEVE ter atualização automática de dependências (Dependabot)
  para cargo, npm, GitHub Actions e Docker, semanal.
  *Prova:* `.github/dependabot.yml`.
- **ENT-08** — O CI DEVE varrer segredos em todo *push* e *pull request*.
  *Prova:* job `segredos` (gitleaks) no `ci.yml`.

## Fora de escopo

Pré-visualização por PR; hospedagem fora do GitHub; servidor de aplicação.
