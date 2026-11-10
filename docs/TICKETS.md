<!-- Caminho relativo: docs/TICKETS.md -->

# Tickets

> Visão geral rápida do que foi feito e do que falta. **Uma linha por ticket**: identificador
> com link para o detalhe, e uma frase curta. O detalhe mora no destino do link, não aqui.
> Atualizado **sempre** que um ticket é criado, concluído ou abandonado — skill
> `micro-ticket-planner`.

## Em aberto

Nenhum: a v0.1.0 foi publicada.

## v0.2 (aprovado em 2026-11-06)

- [ ] [MT-43](plano-pos-v0.1.md#mt-43-publicar-a-v020) — Publicar a v0.2.0: versões, CHANGELOG, ModelSim à mão, *tag*.

## Depois da v0.2 (ordem por dependências)

- [ ] [MT-41](plano-pos-v0.1.md#mt-41-dica-e-próxima-jogada-que-respeitam-o-perfil-de-saída) — Dica e próxima jogada que param no perfil de saída.
- [ ] [MT-42](plano-pos-v0.1.md#mt-42-busca-do-ótimo-fora-da-linha-principal) — Busca do ótimo num *worker*, sem travar a página.
- [ ] [MT-35](plano-pos-v0.1.md#mt-35-matriz-módulo-2n-1-e-2n1-p16) — Matriz módulo $2^n-1$ e $2^n+1$.
- [ ] [MT-32](plano-pos-v0.1.md#mt-32-célula-composta-p13) — Célula composta (ADR própria): contadores, 4:2, AMM 2×2.
- [ ] [MT-33](plano-pos-v0.1.md#mt-33-ponto-invertido-e-números-com-sinal-p14) — Ponto invertido: Baugh-Wooley e dígito −1.
- [ ] [MT-34](plano-pos-v0.1.md#mt-34-booth-radix-2-e-radix-4-p15) — Booth radix-2 e radix-4.
- [ ] [MT-36](plano-pos-v0.1.md#mt-36-escolha-do-cpa-final-p17) — Escolha do CPA final.
- [ ] [MT-23](plano-pos-v0.1.md#mt-23-painel-de-operandos-em-palavras-p6) — Painel de operandos em palavras na DE2.
- [ ] [MT-24](plano-pos-v0.1.md#mt-24-aritmética-larga-no-núcleo-p1-parte-1) — Aritmética larga no núcleo.
- [ ] [MT-25](plano-pos-v0.1.md#mt-25-limites-de-64-bits-e-matriz-navegável-p1-parte-2) — 64 bits por variável e matriz navegável.
- [ ] [MT-26](plano-pos-v0.1.md#mt-26-fatias-de-variáveis-p2) — Fatias de variáveis (`A[63:32]`).
- [ ] [MT-27](plano-pos-v0.1.md#mt-27-blocos-hierárquicos-p3) — Blocos hierárquicos (subprodutos como multiplicadores).
- [ ] [MT-37](plano-pos-v0.1.md#mt-37-variável-derivada-e-constante-grande-p18) — Variável derivada e constante grande.
- [ ] [MT-39](plano-pos-v0.1.md#mt-39-multiplicação-por-constantes-mcm-p21) — Multiplicação por constantes (Hcub, reimplementado a partir do artigo).

## Concluídos

- [x] [MT-40b](plano-pos-v0.1.md#mt-40b-ajuda-dentro-do-app) — "Como usar" no cabeçalho, nos dois idiomas, sem contar ajuda (GER-11).
- [x] [MT-40a](plano-pos-v0.1.md#mt-40a-documentação-pública-e-referências) — README da v0.2, limitações, metodologia e referências públicas; spec viva (ADR 0004); varredura das referências não públicas.
- [x] [MT-31b](plano-pos-v0.1.md#mt-31-entrada-por-alturas-com-perfil-alvo-de-saída-p12) — Interface da entrada por alturas: campo, painel do perfil, solução ótima e nome do bloco como ajuda contada.
- [x] [MT-31a](plano-pos-v0.1.md#mt-31-entrada-por-alturas-com-perfil-alvo-de-saída-p12) — Núcleo da entrada por alturas: perfil, circuito ótimo (11 perfis de referência) e bloco conhecido.
- [x] [MT-38](plano-pos-v0.1.md#mt-38-comparação-de-soluções-p20) — Comparação de soluções, versão mínima (AJU-11): tabela lado a lado e fronteira não dominada.
- [x] [MT-30](plano-pos-v0.1.md#mt-30-tabela-de-alturas-nível-a-nível-p11) — Tabela de alturas nível a nível, exportável em Markdown (CMP-18).
- [x] [MT-29](plano-pos-v0.1.md#mt-29-solução-de-wallace-p10) — Solução de Wallace ao lado da de Dadda, com o mesmo fluxo de revelação e aplicação.
- [x] [MT-28](plano-pos-v0.1.md#mt-28-constante-aditiva-p9) — Constante aditiva (`+36`) como pontos fixos em 1, comprimidos como os outros.
- [x] [MT-22](plano-pos-v0.1.md#mt-22-topo-para-a-de2-até-18-bits-e-projeto-do-quartus-p6-p8) — Topo DE2 até 18 bits e projeto do Quartus, compilado no Quartus II 13.0.
- [x] [MT-21](plano-pos-v0.1.md#mt-21-vhdl-comportamental-p4) — VHDL comportamental com `numeric_std`, conferido pelo mesmo testbench.
- [x] [MT-20](plano-pos-v0.1.md#mt-20-aplicar-a-solução-de-dadda-ao-exercício-p5) — "Usar esta solução": troca o circuito pela de Dadda num passo de desfazer, contada como ajuda.
- [x] [MT-17](plano.md#mt-17-publicar-a-v010) — v0.1.0 publicada: release, imagem pública no GHCR e site no ar.
- [x] [MT-19](plano-pos-v0.1.md#mt-19-vhdl-compatível-com-o-fpgaemuweb-p7) — VHDL com `end <nome>;` e um arquivo por componente, lido pelo FPGAEmuWeb (antecipado para a v0.1).
- [x] [MT-16](plano.md#mt-16-acessibilidade-desempenho-e-tela-pequena) — Acessibilidade (axe sem violação grave), sem rede após carregar, 100 ms no p95, só a própria origem, modo leitura abaixo de 768 px.
- [x] [MT-15b](plano.md#mt-15b-salvamento-automático-e-retomada) — Salvamento automático, retomada, armazenamento indisponível e estado corrompido.
- [x] [MT-15a](plano.md#mt-15a-exportar-e-abrir-o-exercício) — Exportar e abrir o exercício como JSON (núcleo `state`, recusas explicadas, revelações de volta).
- [x] [MT-14b](plano.md#mt-14b-interface--verificação-e-vhdl) — Interface: verificação num *worker* com progresso e cancelamento, VHDL com cópia e `.zip`.
- [x] [MT-14a](plano.md#mt-14-interface--diagrama-tabela-verificação-e-vhdl) — Interface: diagrama de blocos em SVG, tabela de ligações e exportações SVG e Markdown.
- [x] [MT-13](plano.md#mt-13-interface--ajuda-sob-demanda-em-duas-etapas) — Interface: ajuda sob demanda em duas etapas (regras, dica, próxima jogada, limite inferior, solução Dadda, registro no resumo).
- [x] [MT-12b](plano.md#mt-12b-interface--cpa-circuito-completo-e-caminho-crítico) — Interface: CPA, circuito completo, caminho crítico, voltar de nível (R2 completo só pelo teclado).
- [x] [MT-12a](plano.md#mt-12-interface--compressão) — Interface: sessão WASM, seleção por mouse e teclado, células, recusas, painéis por nível, desfazer/refazer e métricas (árvore do R2 pela interface).
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
