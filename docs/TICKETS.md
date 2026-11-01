<!-- Caminho relativo: docs/TICKETS.md -->

# Tickets

> Visão geral rápida do que foi feito e do que falta. **Uma linha por ticket**: identificador
> com link para o detalhe, e uma frase curta. O detalhe mora no destino do link, não aqui.
> Atualizado **sempre** que um ticket é criado, concluído ou abandonado — skill
> `micro-ticket-planner`.

## Em aberto

- [ ] [MT-12](plano.md#mt-12-interface--compressão) — Interface: compressão.
- [ ] [MT-13](plano.md#mt-13-interface--ajuda-sob-demanda-em-duas-etapas) — Interface: ajuda sob demanda em duas etapas.
- [ ] [MT-14](plano.md#mt-14-interface--diagrama-tabela-verificação-e-vhdl) — Interface: diagrama, tabela, verificação e VHDL.
- [ ] [MT-15](plano.md#mt-15-salvar-abrir-e-retomar) — Salvar, abrir e retomar.
- [ ] [MT-16](plano.md#mt-16-acessibilidade-desempenho-e-tela-pequena) — Acessibilidade, desempenho e tela pequena.
- [ ] [MT-17](plano.md#mt-17-publicar-a-v010) — Publicar a v0.1.0.

## Concluídos

- [x] [MT-11b](plano.md#mt-11b-interface--visão-indexada-e-desafio-da-largura) — Interface: visão indexada (tecla V, lembrada), desafio de $W$ em duas etapas e diálogo de confirmação.
- [x] [MT-11a](plano.md#mt-11-interface--expressão-matriz-cores-e-visão-indexada) — Interface: expressão, larguras, exemplos, matriz colorida com legenda e erros traduzidos (ponte WASM `buildMatrix`).
- [x] [MT-0d](plano.md#mt-0d-escrever-o-contrato-visual-docsdesignmd) — Contrato visual em `docs/DESIGN.md`; paleta dos pontos conferida por teste nos dois temas.
- [x] [MT-8](plano.md#mt-8-solução-dadda-próxima-jogada-dicas-e-limite-inferior) — Núcleo: Dadda, próxima jogada, dicas e limite inferior (13 testes; R1 e R4 conferidos à mão).
- [x] [MT-10](plano.md#mt-10-gerar-vhdl-e-verificá-lo-com-ghdl-no-ci) — Núcleo: VHDL-93 e job GHDL no CI (R2: 16 384 casos corretos no GHDL e no ModelSim ASE 10.1d).
- [x] [MT-9](plano.md#mt-9-leiaute-do-diagrama-e-tabela-de-ligações) — Núcleo: leiaute do diagrama e tabela de ligações (10 testes; tabela de R2 idêntica à da spec 05).
- [x] [MT-7](plano.md#mt-7-verificar-o-circuito) — Núcleo: netlist e verificação bit-paralela (14 testes + proptest; R2: 16 384 casos corretos).
- [x] [MT-6](plano.md#mt-6-fechar-com-cpa-e-calcular-métricas) — Núcleo: CPA, métricas e caminho crítico (15 testes; R2 completo: 13 FA, 4 HA, 15 A_FA, 6 T_FA).
- [x] [MT-5](plano.md#mt-5-formar-células-e-níveis) — Núcleo: células e níveis (18 testes; R2 até o CPA bate com a tabela da spec 05).
- [x] [MT-4](plano.md#mt-4-gerar-a-matriz-de-pontos) — Núcleo: matriz de pontos (15 testes; R1, R3, R4, EXP-01 a EXP-03, EXP-08).
- [x] [MT-3](plano.md#mt-3-analisar-a-expressão) — Núcleo: analisar a expressão (27 testes; EXP-01, 05 a 08, 11, 13, 14).
- [x] [MT-18](plano.md#mt-18-proteger-o-repositório-público) — Repositório protegido: `aplicar.sh --conferir` com 0 falhas; PR #13 mesclado pelo novo fluxo.
- [x] [MT-0](plano.md#mt-0-ler-e-aprovar-a-spec-revisada) — Spec v0.1 aprovada pelo autor.
- [x] [MT-2](plano.md#mt-2-montar-cicd-pages-e-imagem-docker) — CI/CD, Pages e Docker: CI verde no `main` e site no ar.
- [x] [MT-1](plano.md#mt-1-criar-o-esqueleto-do-código) — Esqueleto: Rust (core + wasm), SvelteKit estático, i18n, testes de unidade, componente e ponta a ponta.
- [x] MT-0b — Questionário de stack fechado (pnpm, Vitest, Playwright, Paraglide, CSS com tokens, convenções) e registrado no `AGENTS.md`.
- [x] MT-0c — Crítica da spec em contexto limpo; 24 apontamentos tratados na revisão da spec.

## Recusados

<!-- - [~] [MT-0](plano.md#mt-0) — <frase curta> · motivo: <uma frase> · fonte: <quem recusou> -->

## Abandonados

<!-- - [~] [MT-0](plano.md#mt-0) — <frase curta> · motivo: <uma frase> -->
