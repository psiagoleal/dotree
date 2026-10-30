<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Data:** 2026-10-30
- **Branch:** `main`
- **Commit:** nenhum ainda (repositório recém-criado, nada versionado)

## Metas cumpridas / Em andamento / Próximo passo

- [x] Nome escolhido (`dotree`; livre no crates.io e no npm).
- [x] Repositório criado em `~/dev/dotree` (`git init -b main`) e framework aplicado com
      `setup-profile.sh pessoal .`.
- [x] Briefing registrado em `docs/insumos/briefing.md`.
- [x] Rascunho da spec v0.1 em `docs/specs/` (00 a 08, critérios em EARS com prova).
- [x] ADR 0001 (Rust → WASM, SvelteKit estático, Docker para build).
- [x] Ilhas do `AGENTS.md` preenchidas (confidencialidade, stack, comandos comentados,
      layout, fluxo, princípio didático).
- [ ] **Próximo:** o autor revisa a spec e decide D-1 a D-6 (marcadas "⚠ A CONFIRMAR").
- [ ] Questionário de stack: gerenciador Node, testes da interface, i18n, estilização,
      hospedagem, convenção de commit, idioma dos identificadores.
- [ ] Crítica da spec em contexto limpo; depois, micro-tickets de código.
- [ ] Primeiro commit e criação do repositório público (o autor executa — `AGENTS.md`
      seção 8: o agente não roda git que altera estado).

## Contexto para a próxima sessão

- O projeto nasceu de um exercício de Circuitos Aritméticos (repositório `mestrado`,
  `disciplinas/457_CA/entregas/cap11-12/`). Os casos de referência R1 e R2 da spec vêm de lá
  e foram conferidos por simulação exaustiva em Python e em VHDL (ModelSim).
- O material do professor **não** pode vir para cá.

---

## Histórico (mais recente no topo)

| Data | Commit | Resumo | MT |
|------|--------|--------|----|
| 2026-10-30 | — | inicialização: framework, briefing, spec v0.1 (rascunho), ADR 0001 | — |
