<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Data:** 2026-10-30
- **Branch:** `main`
- **Commit:** `44c96dd` (inicialização). As mudanças desta rodada ainda não foram commitadas.

## Metas cumpridas / Em andamento / Próximo passo

- [x] Spec v0.1 revisada: crítica em contexto limpo tratada; cores por termo, visão indexada,
      revelação em duas etapas; exemplos próprios (R1/R2 novos, conferidos por simulação
      exaustiva: 13 FA, 4 HA, 15 $A_{FA}$, 6 $T_{FA}$, 16 384 casos).
- [x] Stack fechada e registrada nas ilhas do `AGENTS.md`; ADR 0002 (entrega).
- [x] MT-1: esqueleto (Rust core + wasm, SvelteKit estático, i18n, testes) com todos os
      comandos executados localmente.
- [x] MT-2: CI/CD, Pages e Docker. Imagem construída e testada localmente; workflows
      validados com actionlint. **Falta:** habilitar o Pages com origem "GitHub Actions",
      fazer o push e ver o CI verde.
- [x] Primeiro CI no GitHub falhou no lint: arquivos que o Paraglide gera em
      `project.inlang/` não estavam no `.prettierignore` (localmente passava porque um
      `format` anterior os tinha formatado). Corrigido e conferido numa cópia limpa.
      Dockerfile sem corepack; Dependabot agrupa minor/patch e isola major.
- [x] Segundo CI falhou em `pnpm run test`: com `BASE_PATH=/dotree` no job inteiro, o
      Vitest em modo navegador não acha a própria página ("Failed to connect to the browser
      session"). O `vite.config.ts` ignora o prefixo sob Vitest e o CI só aplica
      `BASE_PATH` na build e no e2e. Reproduzido antes e conferido depois da correção.
- [x] MT-2 concluído: CI verde no `main`, site no ar, PRs #11 e #12 do Dependabot mesclados.
- [>] MT-18: governança pronta em `.github/governanca/` (ADR 0003); falta o autor aplicar.
      Depois disso, **todo** trabalho entra por branch + PR (o `main` recusa push direto).
- [ ] MT-0: o autor lê e aprova a spec revisada.
- [ ] Próximo ticket de código: MT-3 (analisar a expressão), depois MT-4.

## Observações para a próxima sessão

- A trava de worktree para sessões em segundo plano foi desligada neste repositório
  (`.claude/settings.json`, `worktree.bgIsolation = none`) a pedido do autor, porque o
  `AGENTS.md` proíbe o agente de rodar `git worktree`.
- SvelteKit 3 não tem `$lib`: use `#lib/...` com extensão.
- A porta 4173 costuma estar ocupada nesta máquina; o e2e usa 4180 (`E2E_PORT`).
- O exercício que originou o projeto não pode vir para cá (material de terceiros e
  solução de lista avaliada).

---

## Histórico (mais recente no topo)

| Data | Commit | Resumo | MT |
|------|--------|--------|----|
| 2026-10-30 | — | spec revisada, esqueleto, CI/CD, plano e tickets | MT-1, MT-2 |
| 2026-10-30 | `44c96dd` | inicialização: framework, briefing, spec v0.1 (rascunho), ADR 0001 | — |
