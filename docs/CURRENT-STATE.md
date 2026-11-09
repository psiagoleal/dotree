<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Branch:** `main`
- **Data:** 2026-11-09
- **Commit:** `2e099f7` (MT-29, #43). O MT-30 ainda não foi commitado.

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
- [x] MT-8: `help` (Dadda, próxima jogada, dicas, limite); R1 e R4 batem com as contas à
      mão; 6 mutações pegas. **O núcleo da v0.1 está completo (MT-3 a MT-10).**
- [x] Alertas do Dependabot (`minimatch`, `deepmerge`) dispensados pelo autor como risco
      tolerável (só ferramentas de desenvolvimento).
- [x] MT-0d: `docs/DESIGN.md` e paleta em `app.css`, conferida por `palette.spec.ts`.
- [x] MT-11a: ponte `buildMatrix`, formulário, exemplos, matriz colorida e legenda, erros
      traduzidos. A largura de saída fica oculta: a matriz só mostra colunas até a mais alta
      com ponto.
- [x] MT-11b: visão indexada (tecla V, lembrada), desafio de $W$ em duas etapas, diálogo
      de confirmação reutilizável; núcleo expõe `input_max` e `term_max`. 132 testes Rust,
      50 de unidade/componente, 13 e2e; mutação "revelar sem perguntar" pega.
- [x] MT-11b mesclado (#25).
- [x] MT-12a: `Session` na ponte (6 testes de contrato), `can_undo`/`can_redo` no núcleo,
      `LevelPanel` (substitui `DotMatrix`) e `Compression`; e2e monta a árvore do R2 (níveis
      0 e 1 pelo mouse, nível 2 só pelo teclado) com 9 FA, 2 HA, 10 $A_{FA}$, 3 $T_{FA}$.
      139 testes Rust, 64 de unidade/componente, 15 e2e; mutação "recusa engolida" pega.
- [x] MT-12a mesclado (#26).
- [x] MT-12b: CPA com o motivo do bloqueio, painel do resultado ($R_j$ em roxo), caminho
      crítico a pedido, voltar de nível com confirmação, vai-um descartado explicado atrás
      de confirmação, EXP-04. Correção no núcleo: voltar ao nível do CPA só remove o CPA.
      e2e completa o R2 só com Tab, Espaço, F e Enter (13 FA, 4 HA, 15 $A_{FA}$,
      6 $T_{FA}$, críticas 1, 2, 4, 6, 8, 9, 12–17). 145 testes Rust, 78 de
      unidade/componente, 19 e2e; 3 mutações pegas (volta sem confirmação, explicação sem
      confirmação, caminho crítico sem pedido).
- [x] MT-12b mesclado (#27).
- [x] MT-13: ajuda sob demanda (`Rules`, `Coach`, `SolutionHelp`, `help.ts`); `HelpLog` no
      núcleo; `hint`/`nextMove`/`lowerBound`/`solution`/`helpLog` na `Session` (cada chamada
      é uma revelação e é contada; a dica cicla pelo próprio contador). e2e
      `ajuda-nao-aparece-sem-pedido` e uma prova por linha de GER-03. A solução de Dadda de
      R1 tem atraso 7 $T_{FA}$, contra 6 da árvore de R2: Dadda minimiza células, não
      atraso. 152 testes Rust, 94 de unidade/componente, 22 e2e; 5 mutações pegas.
- [x] MT-13 mesclado (#28).
- [x] MT-14 dividido em 14a e 14b. MT-14a: `diagram`/`wiringTable`/`wiringTableMarkdown`
      na `Session`; `diagram.ts` (geometria e SVG únicos para a tela e o arquivo, cores por
      variável CSS com reserva fixa); `CircuitView` com diagrama, tabela e exportações. O e2e
      compara o Markdown baixado com a tabela da spec 05, caractere a caractere, e valida o
      SVG com `DOMParser`. 156 testes Rust, 106 de unidade/componente, 24 e2e; 5 mutações
      pegas. Fixture do R2 compartilhada: `src/lib/testing/r2.ts` (componentes),
      `e2e/r2.ts` (e2e), `app/wasm/tests/common` (ponte).
- [x] MT-14a mesclado (#29).
- [x] MT-14b: `Session.netlist` (JSON, `app/wasm/src/netcodec.rs`, com checagem de cada
      fio) e `Session.vhdl`; classe `Verification` em blocos. Na web: `verify.worker.ts`
      (núcleo próprio, blocos de $2^{16}$ vetores), `verification.ts` (inicia, segue e
      cancela; cancelar = `terminate`), `zip.ts` (zip sem compressão, CRC conferido contra
      `node:zlib`), `VerifyView` (recriado a cada snapshot, então o resultado some quando o
      circuito muda) e `VhdlView`. A prova de VER-04 passou a ser $A + B$ 12 + 12 ($2^{24}$
      casos, ~1,4 s no navegador); $A \times B$ 16 × 16 cai na amostra e termina antes de
      0,5 s. 165 testes Rust, 122 de unidade/componente, 28 e2e; 7 mutações pegas.
- [x] MT-14b mesclado (#30).
- [x] MT-15a (o MT-15 virou 15a e 15b): módulo `dotree_core::state` (o núcleo agora depende
      de `serde` e `serde_json`), com entradas das células por posição na ordem canônica do
      nível, porque `go_to_level` não reaproveita ids. Ao abrir, o núcleo refaz cada célula
      pelas regras. Na ponte: `exercise`, `restore`, `revealWidth`, `revealed` e
      `shownLowerBound`/`shownSolution`, que mostram o já revelado sem contar. Na web:
      exportar e abrir na página, `WidthChallenge` com `initial`/`onpath` e `SolutionHelp`
      lendo o já revelado. 181 testes Rust, 128 de unidade/componente, 31 e2e; 15 mutações
      pegas (uma só depois de reforçar o teste de trabalho após o CPA).
- [x] MT-15a mesclado (#31).
- [x] MT-15b: `persist.ts` (`autosaving` é um `Proxy` sobre a sessão que grava depois de
      cada chamada que muda estado ou revela), oferta de retomada, aviso único de
      armazenamento e estado corrompido com download. Corrigido o layout, que quebrava com
      o armazenamento bloqueado. Dois e2e antigos que recarregavam a página foram ajustados
      (a recarga agora oferece retomar). 181 testes Rust, 132 de unidade/componente, 37
      e2e; 6 mutações pegas.
- [x] Pedidos do laboratório de multiplicadores avaliados em `docs/plano-pos-v0.1.md`
      (MT-19 a MT-27, aguardam decisão do autor).
- [x] MT-15b mesclado (#32). Decisões do autor sobre os pedidos do laboratório registradas
      em `docs/plano-pos-v0.1.md` (ordem aprovada; MT-19 antecipado; placa pelas portas
      padrão do FPGAEmuWeb).
- [x] MT-19: `end <nome>;` nas entidades (VHD-09). Conferido com o GHDL local (imagem do
      CI) e com a análise de portas do próprio FPGAEmuWeb, extraída do código público e
      rodada fora do repositório.
- [x] MT-16: `e2e/quality.e2e.ts` (axe nos dois temas, sem rede, p95, origem) e
      `e2e/narrow.e2e.ts`. *Worker* de verificação único e aguardado no carregamento
      (`Verifier`); nome acessível dos pontos na visão indexada; regiões roláveis
      focáveis; plurais com variantes do Paraglide (`campo=1` / `campo=*`); o Vitest em
      navegador roda com janela de 1280 px (abaixo de 768 px o exercício é só leitura).
      183 testes Rust, 136 de unidade/componente, 44 e2e; 5 mutações pegas.
- [x] MT-16 + MT-19 mesclados (#33).
- [x] MT-17: v0.1.0 publicada (#36). Versões conferidas por `app/core/tests/release.rs`;
      ModelSim ASE 10.1d conferido pelo agente (`~/altera/13.0/modelsim_ase/bin`); workflow
      Release verde, release com a seção do CHANGELOG, imagem `0.1.0` lida sem login no GHCR.
- [x] Plano da disciplina (caps. 4 a 12) combinado com a sessão CA e aprovado pelo autor:
      P9 a P20 e MT-28 a MT-38 em `docs/plano-pos-v0.1.md`; até o projeto final (03/12):
      MT-20, 21, 22, 28, 29, 30, 38 e, se der tempo, 31.
- [x] P21/MT-39 (2026-11-08): multiplicação por constantes como o gerador do SPIRAL,
      reimplementada a partir do artigo do Hcub (o código deles é GPL; o dotree segue MIT);
      depois do MT-33 e do MT-37, sem mexer na fila até o projeto final.
- [x] MT-20: AJU-09 ("Usar esta solução"). `Reduction::adopt`, `Session.applySolution`,
      `HelpLog.solution_applied` (opcional no arquivo). 197 testes Rust, 139 de
      unidade/componente, 45 e2e; 3 mutações pegas (botão no modo leitura, aplicar sem a
      etapa 2, aplicar sem passo de desfazer).
- [x] MT-21: VHD-10. `<nome>_comportamental.vhd` com as portas do topo e a expressão em
      `numeric_std`; testbench confere as duas versões. GHDL (7 casos, 2 estragados de
      propósito) e ModelSim ASE 10.1d conferidos pelo agente.
- [x] MT-22: spec 10 (placa DE2). Topo `<nome>_de2`, testbench da placa, `.qpf`/`.qsf` com
      os 92 pinos; Quartus II 13.0 compilou o R2 (53 LE estrutural, 56 comportamental);
      GHDL e ModelSim conferidos. Falta conferir no FPGAEmuWeb e numa DE2 real.
- [x] MT-28: EXP-13 refeito e EXP-15 (constante aditiva). Ponto fixo `one`; GHDL e
      ModelSim com `5A + 3B + 6`; reduções aleatórias com constante verificadas. Lab do
      cap. 8 dá $W = 14$ (não 13) com 4 bits por variável: confirmar com a CA.
      No CI, o e2e de VER-04 falhou de novo: com $A + B$, a verificação terminava antes
      de o teste ver a barra. Agora usa $AB$ 12 × 12 com a solução de Dadda e um
      MutationObserver desde o clique; uma mutação que esconde a barra é pega.
- [x] Segurança: `minimatch` forçado para 11.0.1 por override (GHSA-g3hv-75wg-m249, alerta
      Dependabot 3). Remover quando o ESLint e o typescript-eslint aceitarem a 11.
- [x] MT-29: AJU-10, solução de Wallace (regra da fig. 8.10 do Parhami, reproduzida nível
      a nível). Achado: o Dadda do dotree gasta 3 HA a mais que a fig. 8.14 do livro no
      mesmo exemplo (AJU-05 manda HA onde o livro usa FA): decidir com o usuário.
- [x] MT-30: CMP-16, tabela de alturas nível a nível com exportação em Markdown; R2
      conferido à mão contra a spec 02.
- [ ] Próximo: MT-38 (comparação de soluções, versão mínima). Antes, decidir o Dadda (achado do MT-29).

## Observações para a próxima sessão

- A trava de worktree para sessões em segundo plano foi desligada neste repositório
  (`.claude/settings.json`, `worktree.bgIsolation = none`) a pedido do autor, porque o
  `AGENTS.md` proíbe o agente de rodar `git worktree`.
- Não use `pkill -f` com um padrão que apareça na própria linha de comando: mata o shell
  da ferramenta (exit 144). Guarde o PID do servidor e use `kill`.
- Abaixo de 768 px o exercício é só leitura (GER-10): testes de componente precisam de
  janela larga (o `vite.config.ts` fixa 1280 px) e e2e com `setViewportSize` estreito
  não acham os botões de ação.
- O e2e de GER-05 dependia de uma corrida (rede cortada antes de o *worker* carregar);
  resolvido esperando o `ready` do *worker* no carregamento. Se voltar a falhar, é o
  primeiro suspeito.
- Testes e2e que recarregam a página encontram a oferta de retomada (SAL-01): retome, ou
  apague `dotree.exercise` antes de recarregar.
- O gateway OpenAI-compatível do ambiente não respondeu desta máquina em 2026-11-03
  (conexão expira) nem em 2026-11-04 (conexão recusada na porta configurada); as revisões
  independentes do MT-14a e do MT-14b ficaram sem ele. Conferir endereço e porta no
  ambiente antes de tentar de novo.
- O e2e de VER-04 cancela por volta de 0,7 s uma verificação de ~1,4 s; se ficar instável
  numa máquina mais rápida, é o primeiro suspeito. Em 2026-11-09 (PR do MT-21) falhou uma
  vez no CI sem a barra de progresso aparecer, com o runner tão rápido quanto esta máquina
  (GER-06: 11,4 ms nos dois) e sem reproduzir aqui (8 de 8, e a suíte com 2 workers). O
  teste agora diz, na falha, o texto de `verify-result` e os erros da página, e o CI guarda
  `test-results` com captura de tela: se voltar, a causa vem no artefato.
- SvelteKit 3 não tem `$lib`: use `#lib/...` com extensão.
- O Quartus II 13.0 está em `~/altera/13.0/quartus/bin` e compila o projeto gerado:
  `quartus_sh --flow compile <nome>` na pasta dos arquivos. O arquivo de pinos da DE2 da
  disciplina serve para `DE2_PINS_FILE=<caminho> cargo test -p dotree-core --test board`.
- A porta 4173 costuma estar ocupada nesta máquina; o e2e usa 4180 (`E2E_PORT`).
- Os testes de propriedade não gravam `*.proptest-regressions` (`failure_persistence: None`):
  as mutações deixavam ali casos que só falham no código mutado (MT-7, MT-9, MT-8).
- O exercício que originou o projeto não pode vir para cá (material de terceiros e
  solução de lista avaliada).

---

## Histórico (mais recente no topo)

| Data | Commit | Resumo | MT |
|------|--------|--------|----|
| 2026-11-09 | `2e099f7` | solução de Wallace (#43); tabela de alturas | MT-30 |
| 2026-11-09 | `b22bbc0` | override do `minimatch` (#41); solução de Wallace | MT-29 |
| 2026-11-09 | `48cb595` | constante aditiva (#40); override do `minimatch` | MT-28 |
| 2026-11-09 | `15a1b9b` | placa DE2 e Quartus (#39); constante aditiva | MT-22, MT-28 |
| 2026-11-09 | `afa4bbd` | VHDL comportamental (#38); placa DE2 e projeto do Quartus | MT-21, MT-22 |
| 2026-11-09 | `86233eb` | usar a solução e plano da disciplina (#37); VHDL comportamental | MT-20, MT-21 |
| 2026-11-06 | `ce19784` | v0.1.0 publicada (#36); plano da disciplina; usar a solução | MT-17, MT-20 |
| 2026-11-05 | `9f07dd2` | qualidade e FPGAEmuWeb (#33); preparação da v0.1.0 | MT-16, MT-19, MT-17 |
| 2026-11-05 | `76654f4` | salvamento automático (#32); acessibilidade, rede, desempenho, tela pequena; VHDL para o FPGAEmuWeb | MT-15b, MT-16, MT-19 |
| 2026-11-05 | `e0bad41` | exportar e abrir (#31); salvamento automático e retomada; plano pós-v0.1 | MT-15a, MT-15b |
| 2026-11-04 | `98e1f38` | verificação e VHDL (#30); exportar e abrir o exercício | MT-14b, MT-15a |
| 2026-11-04 | `5f9ce8a` | diagrama, tabela e exportações (#29); verificação e VHDL na interface | MT-14a, MT-14b |
| 2026-11-03 | `2a53fcd` | ajuda sob demanda (#28); diagrama, tabela e exportações | MT-13, MT-14a |
| 2026-11-01 | `b46fcff` | CPA e resultado (#27); ajuda sob demanda na interface | MT-12b, MT-13 |
| 2026-11-01 | `109697f` | compressão até o CPA (#26); CPA, resultado e caminho crítico na interface | MT-12a, MT-12b |
| 2026-11-01 | `8b15e7d` | visão indexada e desafio de W (#25); compressão na interface até o CPA | MT-11b, MT-12a |
| 2026-11-01 | `dca3ca6` | matriz colorida na interface; depois, visão indexada e desafio de W | MT-11a, MT-11b |
| 2026-10-30 | — | spec revisada, esqueleto, CI/CD, plano e tickets | MT-1, MT-2 |
| 2026-10-30 | `44c96dd` | inicialização: framework, briefing, spec v0.1 (rascunho), ADR 0001 | — |
