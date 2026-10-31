<!-- Caminho relativo: docs/specs/01-expressao-e-matriz.md -->

# 01 — Da expressão à matriz de pontos

## Resultado

O usuário digita uma expressão e a largura de cada variável, e o app mostra a matriz de
pontos no estilo das aulas: colunas da direita ($2^0$) para a esquerda, cada coluna com seu
peso no topo, pontos empilhados e **coloridos por termo**. A qualquer momento ele pode trocar
os pontos pelas variáveis indexadas que eles representam ($a_1 b_0$).

## Linguagem da expressão (v0.1)

| Elemento | Forma aceita | Exemplo |
|---|---|---|
| Variável | uma letra maiúscula, `A`–`Z` | `A`, `G` |
| Largura | inteiro de 1 a 16 bits, informado à parte para cada variável | `A: 3` |
| Soma | `+` | `A + B` |
| Produto | `*`, `×`, `·` ou justaposição | `A*B`, `A×B`, `AB`, `3(E+F)` |
| Coeficiente | inteiro decimal de 1 a 4 294 967 295; por justaposição só **antes** de variável ou parêntese (`A2` é erro; `A*2` vale) | `3(E+F)`, `2A` |
| Parênteses | agrupam e se distribuem, inclusive produto de somas | `(A+B)(C+D)` = `AC + AD + BC + BD` |
| Espaços | ignorados | `3 (E + F)` = `3(E+F)` |

Variáveis de uma letra tornam a justaposição inequívoca (`AB` só pode ser $A \times B$).

## Como a matriz é formada

1. **Expandir.** A expressão vira uma soma de monômios com coeficiente
   (ex.: $3(E+F) \to 3E + 3F$).
2. **Combinar termos iguais.** Monômios com as mesmas variáveis somam os coeficientes
   (ex.: `AB + AB` → $2AB$; `2A + A` → $3A$).
3. **Decompor o coeficiente em binário.** O coeficiente $k$ vira a soma das potências de dois
   dos seus bits 1, e cada uma gera uma **cópia** do monômio deslocada $m$ colunas
   (ex.: $3E = E + 2E$; $4D = 4D$, uma cópia só).
4. **Gerar os pontos.** Um monômio $X_1 \cdots X_n$ gera um ponto para cada combinação de
   bits $(x_{1,i_1}, \dots, x_{n,i_n})$, na coluna $i_1 + \dots + i_n$ mais o deslocamento.
   - Com $n = 1$, o ponto carrega o próprio bit (sinal `e0`).
   - Com $n \ge 2$, o ponto carrega um produto parcial (sinal `a1_b0`), que é **uma** porta
     AND de $n$ entradas.
   - As cópias de um mesmo monômio carregam **os mesmos sinais**: em $3E$, o ponto de $e_0$
     na coluna $2^0$ e o da coluna $2^1$ são o mesmo fio (*fan-out*), e nenhuma porta é
     duplicada.
5. **Ordem de geração.** Os pontos são gerados, e numerados, nesta ordem: termo (na ordem da
   expressão combinada), depois cópia (deslocamento crescente), depois combinação de bits em
   ordem lexicográfica, com a **primeira** variável do monômio variando mais devagar
   ($a_0b_0, a_0b_1, a_0b_2, a_1b_0, \dots$). É a ordem que desempata a ordem canônica de
   `02-compressao.md`; em R1, a coluna $2^1$ fica `a0_b1, a1_b0, c0_d1, c1_d0, e1, e0, f0`.

## Critérios — expressão e matriz

- **EXP-01** — QUANDO o usuário confirmar uma expressão válida com as larguras de todas as
  variáveis usadas, o sistema DEVE exibir a matriz de pontos do nível 0 conforme as regras
  acima.
  *Prova:* teste do núcleo com R1 (20 pontos; alturas 5, 7, 5, 2, 1), R3 e R4, e com
  `AB + AB` (= $2AB$), `2A + A` (= $A + 2A$) e `(A+B)(C+D)`.
- **EXP-02** — O sistema DEVE dar a cada ponto um identificador único e um sinal; pontos que
  são cópias deslocadas do mesmo monômio DEVEM compartilhar o sinal.
  *Prova:* teste do núcleo (R1: os 4 pontos de $E$ e $2E$ usam só os sinais `e0` e `e1`).
- **EXP-03** — O sistema DEVE contar as portas AND como o número de produtos parciais
  distintos.
  *Prova:* R1 → 13; R3 → 0; R4 → 16; `3AB` com 2 bits cada → 4.
- **EXP-04** — QUANDO a expressão ou a largura de uma variável mudar, o sistema DEVE
  descartar a compressão em andamento somente depois de confirmação do usuário.
  *Prova:* teste de ponta a ponta.

## Critérios — cores (estilo das aulas)

Como nas figuras de aula: cada termo da expressão tem uma cor; pontos produzidos por células
(somas e vai-uns) são **pretos**; os bits do resultado, no fim, ganham uma cor própria de
"resultado". As células formadas aparecem como um retângulo envolvendo os pontos que
consumiram.

- **COR-01** — O sistema DEVE colorir os pontos de entrada pelo termo da expressão
  **expandida e combinada** a que pertencem (passo 2 acima, antes da decomposição binária),
  com legenda que mostra cada termo na sua cor. As cópias de um termo (passo 3) têm a cor do
  termo.
  *Prova:* teste de componente (R1: legenda com 5 termos, $AB$, $CD$, $3E$, $3F$ e $G$, cada
  um numa cor; os pontos de $E$ e de $2E$ têm a mesma cor).
- **COR-02** — O sistema DEVE desenhar em preto os pontos produzidos por células, e numa cor
  de resultado única os bits do resultado do circuito completo.
  *Prova:* teste de componente.
- **COR-03** — O sistema DEVE usar uma paleta de 8 cores distinguíveis entre si e do preto,
  nos temas claro e escuro. A partir do 9.º termo as cores se repetem, e a legenda numera
  os termos.
  *Prova:* teste de unidade da paleta (contraste mínimo de 3:1 contra o fundo em ambos os
  temas; nenhuma cor da paleta com distância perceptual baixa do preto).

## Critérios — visão de variáveis indexadas

- **VIS-01** — O sistema DEVE oferecer, a qualquer momento e em toda tela que mostre pontos
  (matriz, níveis, solução automática, diagrama parcial), a troca entre a visão **pontos** e
  a visão **variáveis indexadas**, em que cada ponto é substituído pelo seu sinal escrito em
  notação matemática ($a_1 b_0$, $e_0$, $s_3$, $k_3$, $R_2$).
  *Prova:* teste de ponta a ponta: em cada tela citada, alterna e confere que cada ponto
  virou o texto do seu sinal, na mesma posição.
- **VIS-02** — QUANDO o usuário alternar a visão, o sistema DEVE manter a posição de cada
  ponto, a seleção em andamento, as cores e o estado do exercício.
  *Prova:* teste de componente (seleciona 2 pontos, alterna, a seleção continua).
- **VIS-03** — O sistema DEVE oferecer atalho de teclado para alternar a visão e lembrar a
  última escolha nas próximas visitas.
  *Prova:* teste de ponta a ponta.
- **VIS-04** — O sistema DEVE mostrar o sinal de um ponto, na visão de pontos, ao passar o
  cursor ou focá-lo pelo teclado.
  *Prova:* teste de componente.

## Desafio da largura de saída (D-1)

- **EXP-09** — ENQUANTO o desafio da largura de saída não tiver sido respondido nem revelado,
  o sistema DEVE ocultar $W$ e $v_{max}$ e mostrar só as colunas que têm pontos; a matriz
  cresce à esquerda conforme surgem vai-uns.
  *Prova:* teste de ponta a ponta.
- **EXP-10** — QUANDO o usuário responder o desafio, o sistema DEVE dizer se acertou e, em
  qualquer caso, revelar $W$ (etapa 1). A conta $v_{max} \to W$ é a etapa 2 (GER-03).
  *Prova:* teste de ponta a ponta com resposta certa e errada.

## Casos de erro

- **EXP-05** — SE a expressão tiver erro de sintaxe, ENTÃO o sistema DEVE apontar a posição
  do erro (coluna do caractere, contada a partir de 1) e o que era esperado ali, sem montar
  matriz.
  *Prova:* testes do analisador com `A+`, `A**B`, `3(E+F`, `a*b`, `A2`, `0A` e expressão
  vazia.
- **EXP-06** — SE uma variável usada não tiver largura informada, ou tiver largura fora de
  1–16, ENTÃO o sistema DEVE recusar a expressão nomeando a variável.
  *Prova:* teste do núcleo.
- **EXP-07** — SE uma variável aparecer mais de uma vez no mesmo monômio (`A*A`, `AA`,
  `A(A+B)`), ENTÃO o sistema DEVE recusar a expressão: potências estão fora da v0.1.
  *Prova:* teste do núcleo.
- **EXP-08** — SE a matriz resultante passar de 256 pontos ou a soma das larguras passar de
  32 bits, ENTÃO o sistema DEVE recusar a expressão informando o limite.
  *Prova:* teste do núcleo com $A \times B$ de 16 × 16 bits (256 pontos: aceita) e um caso
  de 257 pontos (recusa).
- **EXP-11** — ONDE houver largura informada para uma variável que a expressão não usa, o
  sistema DEVE ignorá-la e avisar.
  *Prova:* teste do núcleo (aviso `UnusedWidth`) e teste de componente (o aviso aparece).
- **EXP-13** — SE a expressão expandida tiver um termo constante (`A + 5`, `3(A + 1)`, `2`),
  ENTÃO o sistema DEVE recusá-la: constantes aditivas estão fora da v0.1.
  *Prova:* teste do núcleo.
- **EXP-14** — SE um coeficiente literal passar de 4 294 967 295, ou um coeficiente expandido
  não couber em 64 bits, ENTÃO o sistema DEVE recusar a expressão com o código do erro, sem
  travar.
  *Prova:* teste do núcleo (`99999999999A` e o produto de três coeficientes máximos).

## Exemplos embutidos

- **EXP-12** — O sistema DEVE oferecer exemplos prontos (no mínimo R1, R3 e R4) que
  preenchem a expressão e as larguras.
  *Prova:* teste de componente.

## Fora de escopo

Variáveis com mais de uma letra, expoentes (`A^2`), divisão, constantes aditivas, sinal.
