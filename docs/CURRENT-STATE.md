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
- [x] MT-18: governança aplicada (`aplicar.sh --conferir` com 0 falhas). **Todo** trabalho
      entra por branch + PR: o `main` recusa push direto.
- [x] MT-0: spec v0.1 aprovada pelo autor.
- [x] MT-3: `dotree_core::expr` com 27 testes (verificados com 3 mutações). A spec ganhou
      EXP-13 (constantes recusadas) e EXP-14 (coeficiente grande demais).
- [x] MT-4: `dotree_core::matrix` com 15 testes (R1: 20 pontos, alturas 5,7,5,2,1, W = 7,
      13 AND). Ordem canônica de geração escrita na spec 01 (passo 5).
- [x] MT-5: `dotree_core::reduce` com 18 testes; reproduz R2 até o CPA igual à tabela da
      spec 05; 8 mutações pegas. Spec 02 ganhou CMP-17 e deixou CMP-03/CMP-04 explícitos.
- [x] MT-6: CPA, métricas e caminho crítico; R2 completo reproduzido (13 FA, 4 HA,
      15 $A_{FA}$, 6 $T_{FA}$, críticas 1, 2, 4, 6, 8, 9, 12–17); 6 mutações pegas.
- [x] MT-7: `netlist` + `verify`; R2 com 16 384 casos corretos; proptest de 64 reduções
      aleatórias; 6 mutações pegas (uma só depois de reforçar o teste com um simulador
      escalar independente). Pior caso exaustivo (2^24 vetores, 245 células): 1,6 s nativo.
- [x] MT-9: `diagram`; a tabela de R2 sai idêntica à da spec 05 (o teste lê a spec); o
      caminho crítico calculado pela netlist concorda com o da redução em R2 e em 64 casos
      aleatórios; 6 mutações pegas. A netlist ganhou nível, CPA, `W` e versão parcial.
- [x] MT-10: `vhdl` + job `vhdl` no CI. O GHDL achou dois defeitos que os testes de
      estrutura não viam (caractere fora do Latin-1 e células sem ligação por falta de
      `use work.all`). R2 deu 16 384 casos corretos no GHDL e no ModelSim ASE 10.1d.
- [ ] Próximo: MT-8 (Dadda, próxima jogada, dicas, limite), último ticket só de núcleo;
      depois a interface (MT-0d e MT-11 em diante).

## Observações para a próxima sessão

- A trava de worktree para sessões em segundo plano foi desligada neste repositório
  (`.claude/settings.json`, `worktree.bgIsolation = none`) a pedido do autor, porque o
  `AGENTS.md` proíbe o agente de rodar `git worktree`.
- SvelteKit 3 não tem `$lib`: use `#lib/...` com extensão.
- A porta 4173 costuma estar ocupada nesta máquina; o e2e usa 4180 (`E2E_PORT`).
- Testes de mutação deixam `app/core/tests/*.proptest-regressions` com casos que só falham no
  código mutado: apagar antes de commitar (aconteceu no MT-7 e no MT-9).
- O exercício que originou o projeto não pode vir para cá (material de terceiros e
  solução de lista avaliada).

---

## Histórico (mais recente no topo)

| Data | Commit | Resumo | MT |
|------|--------|--------|----|
| 2026-10-30 | — | spec revisada, esqueleto, CI/CD, plano e tickets | MT-1, MT-2 |
| 2026-10-30 | `44c96dd` | inicialização: framework, briefing, spec v0.1 (rascunho), ADR 0001 | — |
