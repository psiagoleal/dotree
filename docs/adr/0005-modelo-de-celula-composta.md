<!-- Caminho relativo: docs/adr/0005-modelo-de-celula-composta.md -->

# ADR 0005: Modelo de célula composta: contadores de uma coluna primeiro

- **Status:** Accepted
- **Data:** 2026-11-17
- **Decisores:** Iago Leal
- **Tags:** núcleo, células, custos, VHDL

## Contexto

Até a v0.2, uma célula era um FA ou um HA. Cada uma consome pontos de uma coluna $j$ e
devolve dois: a soma em $j$ e o vai-um em $j+1$. A redução, a netlist, a verificação, o
diagrama, o VHDL e o arquivo do exercício contam com essa forma.

O MT-32 pede células maiores, que o livro de Parhami trata no capítulo de adição de vários
operandos:

- os contadores paralelos $(m;n)$ de uma coluna (7:3 e 15:4);
- o compressor 4:2 e o AMM 2×2, que recebem e devolvem pontos em várias colunas;
- os blocos hierárquicos do MT-27 (um subproduto que vira multiplicador).

A v0.3 traz só os contadores de uma coluna (MT-32a no núcleo, MT-32b na interface). O
modelo, porém, tem de servir aos outros tipos sem refazer a redução.

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Contador como atalho de interface**, que forma os FA da rede interna de uma vez | O aluno não vê o contador como bloco. Área, atraso, diagrama e VHDL mostrariam 4 ou 11 FA, e o custo de um contador dedicado não poderia ser estudado. |
| **Célula genérica já agora**, com entradas e saídas por coluna, como o 4:2 vai exigir | Mexe em toda a redução, no arquivo e na interface para um tipo que não entra na v0.3. A ordem canônica e o vai-um encadeado do CPA teriam de ser redesenhados sem um caso real para testar. |
| **Um atraso por saída** (o bit 0 do 7:3 fica pronto antes dos outros) | É mais fiel ao circuito, mas quebra a regra "todas as saídas de uma célula ficam prontas juntas". O caminho crítico, a busca do ótimo e o diagrama contam com essa regra. |
| **Custo do contador fixo** (4 e 3 para o 7:3) | Deixa de bater com a rede de FA quando o aluno muda o custo do FA (GER-12). |

## Decisão

- **Tipos.** `CellKind` ganha dois tipos de uma coluna:
  - `C73`: 7 entradas, 3 saídas;
  - `C154`: 15 entradas, 4 saídas.

  Selecionar 7 ou 15 pontos de uma coluna forma o contador, assim como 3 formam um FA e
  2 um HA. Outras contagens continuam recusadas.
- **Saídas.** O bit $i$ da contagem vai para a coluna $j+i$:
  - `s<n>` e `k<n>` (bits 0 e 1), como no FA;
  - `k<n>_2` e `k<n>_3` (bits 2 e 3).

  Uma saída que cair numa coluna $\ge W$ é descartada como em CMP-09, porque vale sempre
  0. Na ordem canônica, as saídas de uma célula seguem a ordem dos bits.
- **Tempo.** Todas as saídas ficam prontas em $\max(t_\text{entradas})$ mais o atraso do
  tipo, como no FA e no HA. A regra é pessimista para os bits baixos, e isso fica
  declarado.
- **Custo.** O padrão vem da rede interna de FA que o VHDL monta, com os custos do FA em
  vigor:

  | Contador | Rede interna | Área | Atraso |
  |---|---|---|---|
  | 7:3 | 2 FA em paralelo, 1 FA sobre os bits 0, 1 FA sobre os vai-uns | $4\,A_{FA}$ | $3\,T_{FA}$ |
  | 15:4 | 2 contadores 7:3 e uma soma de 3 bits em *ripple* | $11\,A_{FA}$ | $5\,T_{FA}$ |

  Cada FA reduz a contagem de bits em 1, então $m:n$ precisa de $m-n$ FA. O aluno pode
  dar ao contador área e atraso próprios (GER-12), para estudar um contador dedicado.
  Sem esse valor próprio, o custo segue o do FA.
- **Arquivo.** Os códigos de tipo são `c73` e `c154`. Os custos próprios entram no campo
  `costs` só quando definidos, e o esquema continua 1.
- **VHDL.** Há uma entidade por tipo usado, montada com `full_adder`, num arquivo próprio:
  - `counter_7_3`, com portas `x0`…`x6` → `y0`…`y2`;
  - `counter_15_4`, que instancia dois `counter_7_3`.

  As instâncias se chamam `C73_<n>` e `C154_<n>`.
- **A célula composta de várias colunas fica para depois.** Quando o 4:2 entrar, a
  célula passa a ter entradas por coluna e saídas por coluna. A regra de tempo, a regra
  de descarte e a ordem das saídas desta ADR continuam valendo.

## Consequências

- **Impacto positivo:** o contador aparece como bloco em todas as visões, com custo próprio
  e verificação exaustiva. A solução do aluno com contadores pode ser comparada com a de
  Dadda.
- **Impacto negativo:** o tempo de um contador é o mesmo para todas as saídas, mais lento
  que a rede interna nos bits baixos. As soluções automáticas (Dadda, Wallace, ótima)
  continuam só com FA e HA.
- **Trade-offs aceitos:** a interface só passa a formar contadores no MT-32b. Até lá, o
  módulo WebAssembly recusa seleções de 7 ou 15 pontos com a mesma mensagem de hoje.

## Diretriz de Conformidade de Código

- **Obrigatório:** um tipo novo de célula declara entradas, saídas, custo padrão tirado de
  uma rede interna montada no VHDL, código no arquivo e entidade VHDL conferida pelo GHDL.
  A verificação simula o tipo pela contagem aritmética, não pela rede interna.
- **Proibido:** saídas da mesma célula com prontidões diferentes; custo padrão que não
  venha da rede interna.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
