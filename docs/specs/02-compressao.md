<!-- Caminho relativo: docs/specs/02-compressao.md -->

# 02 — Compressão: níveis, FA, HA e CPA

## Resultado

O usuário reduz a matriz até restar no máximo um ponto por coluna, escolhendo cada célula.
O app aplica a regra da célula, posiciona as saídas e mostra o efeito na hora, nível a
nível, como nos painéis das aulas.

## Mecânica

- O usuário trabalha sempre no **nível atual**. Ele seleciona pontos do nível atual e forma
  uma célula:
  - 3 pontos da mesma coluna formam um **FA**;
  - 2 pontos da mesma coluna formam um **HA**.
- A soma da célula vai para a mesma coluna e o vai-um para a coluna seguinte, ambos **no
  nível seguinte**. As saídas recebem rótulos `s<n>` e `k<n>`, onde `<n>` é o número da
  célula.
- **Concluir nível** cria o nível seguinte: os pontos não usados passam direto, e as saídas
  das células entram.
- As células são numeradas na ordem nível → coluna crescente → ordem de criação, como na
  tabela de ligações das aulas (FA1, FA2…).

## Critérios — células e níveis

- **CMP-01** — QUANDO o usuário formar um FA com 3 pontos da coluna $j$, o sistema DEVE
  removê-los do nível atual e criar no nível seguinte uma soma na coluna $j$ e um vai-um na
  coluna $j+1$, ambos com prontidão $\max(t_{entradas}) + 1\,T_{FA}$.
  *Prova:* teste do núcleo.
- **CMP-02** — QUANDO o usuário formar um HA com 2 pontos da coluna $j$, o sistema DEVE fazer
  o mesmo com prontidão $\max(t_{entradas}) + 0{,}5\,T_{FA}$.
  *Prova:* teste do núcleo.
- **CMP-03** — O sistema DEVE permitir desfazer e refazer qualquer formação de célula e
  qualquer conclusão de nível, sem limite dentro da sessão.
  *Prova:* teste do núcleo (sequência formar → desfazer → refazer devolve o mesmo estado).
- **CMP-04** — O sistema DEVE permitir voltar a um nível anterior; as células dos níveis
  posteriores são descartadas depois de confirmação.
  *Prova:* teste de ponta a ponta.
- **CMP-05** — O sistema DEVE mostrar os níveis já concluídos lado a lado (ou empilhados),
  cada um com suas células destacadas, no estilo dos painéis de aula.
  *Prova:* teste de componente.

### Seleções inválidas (cada recusa cita a regra, sem sugerir a correção — GER-02)

- **CMP-06** — SE os pontos selecionados estiverem em colunas diferentes, ENTÃO o sistema
  DEVE recusar: "uma célula soma bits do mesmo peso".
- **CMP-07** — SE forem selecionados 1 ou mais de 3 pontos, ENTÃO o sistema DEVE recusar:
  "FA usa 3 pontos; HA usa 2".
- **CMP-08** — SE um ponto selecionado já pertencer a outra célula do mesmo nível, ENTÃO o
  sistema DEVE recusar: "cada ponto entra em uma célula só por nível".
- **CMP-09** — SE o vai-um de uma célula cair numa coluna $\ge W$, ENTÃO o sistema DEVE aceitar
  a célula, marcar esse vai-um como "sempre 0" e explicar que a soma total não ultrapassa
  $2^W - 1$.

*Prova de CMP-06 a CMP-09:* um teste do núcleo para cada recusa, conferindo o código da
regra.

## Critérios — CPA

- **CMP-10** — ENQUANTO houver no nível atual uma coluna com mais de 3 pontos, ou uma coluna
  com 3 pontos que não seja a menor coluna com 2 ou mais pontos, o sistema DEVE manter
  desabilitada a ação "Fechar com CPA", explicando a condição.
  *Prova:* teste do núcleo.
- **CMP-11** — QUANDO o usuário fechar com CPA, o sistema DEVE montar um somador *ripple* da
  menor coluna com 2 ou mais pontos até a última coluna não vazia. Cada coluna recebe um FA
  se tiver 3 entradas (pontos mais o vai-um da anterior), um HA se tiver 2, e um fio se
  tiver 1. Uma coluna inicial com 3 pontos usa o terceiro como *carry-in*.
  *Prova:* R2 → CPA de 4 FA, de $2^1$ a $2^4$, com o vai-um do FA de $2^0$ como *carry-in*.
- **CMP-12** — QUANDO todas as colunas tiverem no máximo 1 ponto, o sistema DEVE declarar o
  circuito completo e nomear os bits do resultado $R_0 \dots R_{W-1}$.
  *Prova:* R2 e R3.

## Critérios — métricas do circuito do usuário

- **CMP-13** — O sistema DEVE exibir, para o nível atual, a altura de cada coluna.
- **CMP-14** — O sistema DEVE exibir a contagem de FA, HA e AND do circuito montado até o
  momento.
- **D-3 ⚠ A CONFIRMAR** — área e atraso do circuito montado ficam visíveis o tempo todo. Eles
  são retorno sobre o trabalho do próprio usuário, não a resposta. A alternativa seria
  pedir que o usuário estime antes de ver.
- **CMP-15** — ONDE área e atraso estiverem visíveis, o sistema DEVE mostrar área em
  $A_{FA}$, atraso em $T_{FA}$ e, ao pedir, destacar o caminho crítico.

*Prova de CMP-13 a CMP-15:* R2 → 12 FA, 0 HA, 12 AND, 12 $A_{FA}$, 5 $T_{FA}$; caminho
crítico FA de $2^0$ → os quatro FA do CPA.

## Fora de escopo

Escolher manualmente a coluna da soma ou do vai-um; células com mais de 3 entradas;
reordenar níveis; CPA com antecipação de vai-um.
