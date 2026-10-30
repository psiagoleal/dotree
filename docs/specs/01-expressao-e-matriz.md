<!-- Caminho relativo: docs/specs/01-expressao-e-matriz.md -->

# 01 — Da expressão à matriz de pontos

## Resultado

O usuário digita uma expressão e a largura de cada variável, e o app mostra a matriz de
pontos no estilo das aulas: colunas da direita ($2^0$) para a esquerda, pontos empilhados,
cada termo com sua cor e forma.

## Linguagem da expressão (v0.1)

| Elemento | Forma aceita | Exemplo |
|---|---|---|
| Variável | uma letra maiúscula, `A`–`Z` | `A`, `G` |
| Largura | inteiro de 1 a 16 bits, informado à parte para cada variável | `A: 3` |
| Soma | `+` | `A + B` |
| Produto | `*`, `×`, `·` ou justaposição | `A*B`, `A×B`, `AB`, `2(E+F)` |
| Coeficiente | inteiro positivo | `2(E+F)`, `3A` |
| Parênteses | agrupam e se distribuem | `2(E+F)` = `2E + 2F` |

Variáveis de uma letra tornam a justaposição inequívoca (`AB` só pode ser $A \times B$).

## Como a matriz é formada

1. A expressão é expandida numa **soma de monômios** com coeficiente
   (ex.: $2(E+F) \to 2E + 2F$).
2. Cada coeficiente $k$ se decompõe em potências de dois: $k = \sum 2^{m}$. Cada parcela
   vira uma cópia do monômio deslocada $m$ colunas (ex.: $3A = A + 2A$).
3. Um monômio $X_1 X_2 \cdots X_n$ gera um ponto para cada combinação de bits
   $(x_{1,i_1}, \dots, x_{n,i_n})$, na coluna $i_1 + \dots + i_n$ (mais o deslocamento), com
   rótulo `x1_i1·x2_i2·…` (ex.: `a1·b0`). Com $n \ge 2$, o ponto é uma porta AND de $n$
   entradas; com $n = 1$, é o próprio bit.

## Critérios

- **EXP-01** — QUANDO o usuário confirmar uma expressão válida com as larguras de todas as
  variáveis, o sistema DEVE exibir a matriz de pontos do nível 0 conforme as regras acima.
  *Prova:* teste do núcleo com R1 (18 pontos; alturas 3, 6, 6, 3), R3 e R4.
- **EXP-02** — O sistema DEVE rotular cada ponto com sua origem (`a1·b0`, `e2`, `g`) e
  exibir o rótulo ao passar o cursor ou focar o ponto pelo teclado.
  *Prova:* teste de unidade dos rótulos; teste de componente da matriz.
- **EXP-03** — O sistema DEVE dar a cada monômio da expressão expandida uma cor e uma forma
  próprias, com legenda.
  *Prova:* teste de componente (legenda lista um item por monômio).
- **EXP-04** — O sistema DEVE contar as portas AND (pontos de monômio com $n \ge 2$) e exibir
  essa contagem.
  *Prova:* R1 → 12 AND; R3 → 0; R4 → 16.
- **EXP-05** — QUANDO a expressão mudar, o sistema DEVE descartar a compressão em andamento
  somente depois de confirmação do usuário.
  *Prova:* teste de ponta a ponta.

### Casos de erro

- **EXP-06** — SE a expressão tiver erro de sintaxe, ENTÃO o sistema DEVE apontar a posição
  do erro e o que era esperado ali, sem montar matriz.
  *Prova:* testes do analisador com `A+`, `A**B`, `2(E+F`, `a*b`.
- **EXP-07** — SE uma variável usada não tiver largura informada, ou tiver largura fora de
  1–16, ENTÃO o sistema DEVE recusar a expressão nomeando a variável.
  *Prova:* teste do núcleo.
- **EXP-08** — SE a matriz resultante passar de 256 pontos ou a soma das larguras passar de
  32 bits, ENTÃO o sistema DEVE recusar a expressão informando o limite.
  *Prova:* teste do núcleo com $A \times B$ de 16 × 16 bits (256 pontos: aceita) e um caso
  de 257 pontos (recusa).

## Desafio da largura de saída

- **D-1 ⚠ A CONFIRMAR** — antes de revelar $W$, o app pergunta ao usuário quantos bits terá
  a saída. Ele pode responder (o app diz se acertou e mostra a conta
  $v_{max} \to W$) ou pedir a resposta. Enquanto isso não acontece, a matriz mostra só as
  colunas que têm pontos e cresce à esquerda conforme surgem vai-uns.
- **EXP-09** — ENQUANTO o desafio da largura de saída não tiver sido respondido nem revelado,
  o sistema DEVE ocultar $W$ e $v_{max}$.
  *Prova:* teste de ponta a ponta.

## Exemplos embutidos

- **EXP-10** — O sistema DEVE oferecer exemplos prontos (no mínimo R1, R3 e R4) que
  preenchem a expressão e as larguras.
  *Prova:* teste de componente.

## Fora de escopo

Variáveis com mais de uma letra, expoentes (`A^2`), divisão, constantes aditivas, sinal.
