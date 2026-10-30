<!-- Caminho relativo: docs/specs/05-diagrama.md -->

# 05 — Diagrama de blocos e tabela de ligações

## Resultado

A partir do circuito, o app desenha o diagrama de blocos e monta a tabela de ligações: os
dois artefatos que o estudante copia para a folha de resposta.

## Nomes

| O quê | Na tela (visão indexada) | Identificador (diagrama, tabela, VHDL) |
|---|---|---|
| Bit de entrada | $e_0$ | `e0` |
| Produto parcial | $a_1 b_0$ | `a1_b0` |
| Saídas da célula $n$ | $s_n$, $k_n$ | `s<n>`, `k<n>` |
| Bit $j$ do resultado | $R_j$ | `R(j)` |
| Célula $n$ | FA$n$ / HA$n$ | `FA<n>` / `HA<n>` |

O identificador é o mesmo em todos os artefatos gerados e é um identificador VHDL válido.

## Leiaute

- Uma **linha** por nível, de cima para baixo: no topo, o bloco de portas AND; depois as
  células formadas sobre o nível 0, 1, 2…; por último, o CPA.
- Uma **coluna** por peso, com o peso maior à esquerda.
- Células da mesma linha e da mesma coluna ficam lado a lado, em ordem de número.

## Critérios

- **DIA-01** — QUANDO houver ao menos um nível concluído, o sistema DEVE desenhar o diagrama
  de blocos com cada célula (FA, HA) como um bloco posicionado pelo leiaute acima, e com as
  ligações entre blocos.
  *Prova:* teste do núcleo do leiaute: posição (linha, coluna, ordem) de cada uma das 17
  células de R2 fixada no teste.
- **DIA-02** — O sistema DEVE representar os produtos parciais como um bloco de portas AND
  no topo, com a opção de expandi-lo em portas individuais.
  *Prova:* teste de componente.
- **DIA-03** — O sistema DEVE rotular cada ligação com o identificador do sinal.
  *Prova:* teste do núcleo: o conjunto de identificadores do diagrama é igual ao conjunto de
  sinais declarados ou usados no VHDL gerado.
- **DIA-04** — O sistema DEVE gerar a tabela de ligações com uma linha por célula: nome,
  nível, coluna, entradas x, y, z (na ordem canônica de `02-compressao.md`), soma e vai-um
  (`—` quando descartado ou inexistente).
  *Prova:* R2 → igual à tabela de referência abaixo.
- **DIA-05** — QUANDO o usuário pedir, o sistema DEVE destacar no diagrama todas as células
  em algum caminho crítico (CMP-15).
  *Prova:* teste do núcleo: R2 → o conjunto de células críticas calculado é o mesmo destacado
  (teste de componente).
- **DIA-06** — O sistema DEVE exportar o diagrama em SVG e a tabela de ligações em Markdown.
  *Prova:* teste de ponta a ponta (arquivos baixados; SVG válido; tabela igual a DIA-04).
- **DIA-07** — ENQUANTO o circuito estiver incompleto, o sistema DEVE desenhar o diagrama até
  o último nível concluído e sinalizar que está incompleto; as exportações ficam
  desabilitadas.
  *Prova:* teste de componente.

## Tabela de ligações de referência (R2)

| Célula | Nível | Coluna | x | y | z | Soma | Vai-um |
|---|---|---|---|---|---|---|---|
| FA1 | 0 | $2^1$ | `a0_b1` | `a1_b0` | `c0_d1` | `s1` | `k1` |
| FA2 | 0 | $2^1$ | `c1_d0` | `e1` | `e0` | `s2` | `k2` |
| HA3 | 0 | $2^2$ | `a0_b2` | `a1_b1` | — | `s3` | `k3` |
| FA4 | 1 | $2^1$ | `f0` | `s1` | `s2` | `s4` | `k4` |
| FA5 | 1 | $2^2$ | `a2_b0` | `c1_d1` | `e1` | `s5` | `k5` |
| FA6 | 1 | $2^2$ | `s3` | `k1` | `k2` | `s6` | `k6` |
| FA7 | 1 | $2^3$ | `a1_b2` | `a2_b1` | `k3` | `s7` | `k7` |
| FA8 | 2 | $2^0$ | `a0_b0` | `c0_d0` | `e0` | `s8` | `k8` |
| FA9 | 2 | $2^2$ | `s5` | `k4` | `s6` | `s9` | `k9` |
| FA10 | 2 | $2^3$ | `k5` | `s7` | `k6` | `s10` | `k10` |
| HA11 | 2 | $2^4$ | `a2_b2` | `k7` | — | `s11` | `k11` |
| FA12 | CPA | $2^0$ | `f0` | `g0` | `s8` | `s12` → `R(0)` | `k12` |
| FA13 | CPA | $2^1$ | `k8` | `s4` | `k12` | `s13` → `R(1)` | `k13` |
| HA14 | CPA | $2^2$ | `s9` | `k13` | — | `s14` → `R(2)` | `k14` |
| FA15 | CPA | $2^3$ | `k9` | `s10` | `k14` | `s15` → `R(3)` | `k15` |
| FA16 | CPA | $2^4$ | `s11` | `k10` | `k15` | `s16` → `R(4)` | `k16` |
| HA17 | CPA | $2^5$ | `k11` | `k16` | — | `s17` → `R(5)` | `k17` → `R(6)` |

Note o *fan-out* de $E$ e $F$: `e0`, `e1` e `f0` aparecem em duas colunas, porque $3E = E +
2E$ e $3F = F + 2F$ usam os mesmos fios.

## Fora de escopo

Edição do diagrama à mão (arrastar blocos); exportação em PDF, PNG ou CSV; esquemático no
nível de porta lógica dentro de FA e HA.
