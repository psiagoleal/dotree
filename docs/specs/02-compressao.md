<!-- Caminho relativo: docs/specs/02-compressao.md -->

# 02 — Compressão: níveis, FA, HA e CPA

## Resultado

O usuário reduz a matriz até restar no máximo um ponto por coluna, escolhendo cada célula.
O app aplica a regra da célula, posiciona as saídas e mostra o efeito na hora, nível a
nível, como nos painéis das aulas.

## Mecânica

- O usuário trabalha sempre no **nível atual**. Seleciona pontos do nível atual e forma uma
  célula. O tipo sai do número de pontos: **3 pontos formam um FA; 2 formam um HA.**
- A soma vai para a mesma coluna e o vai-um para a coluna seguinte, ambos **no nível
  seguinte**. As saídas da célula $n$ carregam os sinais `s<n>` e `k<n>`.
- As células formadas **sobre** o nível $n$ consomem pontos dele e produzem o nível $n+1$.
- **Concluir nível** cria o nível seguinte: os pontos não usados passam direto, e as saídas
  das células entram.
- **Numeração das células:** em ordem de nível; dentro do nível, por coluna crescente; dentro
  da coluna, pela ordem em que foram formadas. A numeração é refeita ao concluir o nível, e
  é a mesma do diagrama, da tabela de ligações e do VHDL.
- **Ordem das entradas** de uma célula (x, y, z), usada no diagrama e no VHDL: ordem
  canônica dos pontos, que é: menor prontidão primeiro; empatando, pontos de entrada antes
  de saídas de célula; entre entradas, a ordem de geração (EXP-01); entre saídas, o número
  da célula, soma antes do vai-um. No CPA, o vai-um da coluna anterior é sempre a última
  entrada.

## Critérios — células e níveis

- **CMP-01** — QUANDO o usuário formar uma célula com 3 pontos da coluna $j$, o sistema DEVE
  criar um FA: remover os pontos do nível atual e criar no nível seguinte a soma na coluna
  $j$ e o vai-um na coluna $j+1$, ambos com prontidão $\max(t_{entradas}) + 1\,T_{FA}$.
  *Prova:* teste do núcleo.
- **CMP-02** — QUANDO o usuário formar uma célula com 2 pontos da coluna $j$, o sistema DEVE
  criar um HA do mesmo modo, com prontidão $\max(t_{entradas}) + 0{,}5\,T_{FA}$.
  *Prova:* teste do núcleo.
- **CMP-03** — O sistema DEVE permitir desfazer e refazer qualquer formação de célula,
  conclusão de nível e volta de nível, até o início do exercício. Uma ação nova depois de
  desfazer apaga o que poderia ser refeito.
  *Prova:* teste do núcleo: 1000 operações aleatórias seguidas de 1000 desfazer devolvem o
  estado inicial, e de 1000 refazer devolvem o estado final.
- **CMP-04** — O sistema DEVE permitir voltar a um nível anterior $k$: as células formadas
  sobre $k$ voltam a ficar pendentes, para edição, e as dos níveis posteriores são
  descartadas depois de confirmação. Voltar também é uma ação que se desfaz.
  Voltar ao nível que o CPA fechou remove o CPA: as células dele não voltam como pendentes,
  porque encadeiam vai-uns que não são pontos do nível. Quando voltar não descarta nada (o
  nível anterior ao atual, sem células pendentes), não há o que confirmar.
  *Prova:* teste do núcleo (estado após voltar) e teste de ponta a ponta (confirmação).
- **CMP-05** — O sistema DEVE mostrar os níveis já concluídos em painéis, um por nível, com
  as colunas alinhadas por peso e o cabeçalho $2^j$, cada célula como um retângulo
  envolvendo os pontos que consumiu, e uma seta de cada painel para o seguinte. Dentro da
  coluna, as células vêm primeiro, cada uma com as entradas na ordem $x, y, z$, e depois os
  pontos soltos, na ordem canônica. O nível atual fica no último painel, com as células
  pendentes também em retângulos.
  *Prova:* teste de componente (R2: 5 painéis, um por nível de 0 a 3 e um do resultado;
  17 retângulos).

### Ações recusadas (cada recusa cita a regra, sem sugerir a correção — GER-02)

- **CMP-06** — SE os pontos selecionados estiverem em colunas diferentes, ENTÃO o sistema
  DEVE recusar: "uma célula soma bits do mesmo peso".
- **CMP-07** — SE a seleção tiver 0, 1 ou mais de 3 pontos, ENTÃO o sistema DEVE recusar:
  "FA usa 3 pontos; HA usa 2".
- **CMP-08** — SE um ponto selecionado já pertencer a outra célula do mesmo nível, ENTÃO o
  sistema DEVE recusar: "cada ponto entra em uma célula só por nível".
- **CMP-16** — SE o usuário pedir para concluir um nível sem nenhuma célula formada nele,
  ENTÃO o sistema DEVE recusar: "um nível precisa de pelo menos uma célula".

- **CMP-17** — SE a seleção tiver um ponto que não pertence ao nível atual, ou o mesmo ponto
  duas vezes, ENTÃO o sistema DEVE recusar sem alterar o estado.

*Prova de CMP-06, CMP-07, CMP-08, CMP-16 e CMP-17:* um teste do núcleo para cada recusa,
conferindo o código da regra; outro confere que recusas não mudam o estado nem o histórico.

### Vai-um além da saída

- **CMP-09** — QUANDO o vai-um de uma célula cair numa coluna $\ge W$, o sistema DEVE aceitar
  a célula, não criar o ponto do vai-um e marcá-lo como "sempre 0". ENQUANTO $W$ estiver
  oculto (EXP-09), a explicação do porquê fica atrás de pedido e confirmação, porque ela
  revela $W$.
  *Prova:* teste do núcleo (célula na coluna $W-1$ com vai-um descartado) e teste de ponta a
  ponta (a explicação não aparece sem pedido).

## Critérios — CPA

- **CMP-10** — O sistema DEVE habilitar "Fechar com CPA" somente quando, no nível atual:
  não houver células pendentes; nenhuma coluna tiver mais de 3 pontos; houver no máximo uma coluna com 3 pontos, e ela for
  a menor coluna com 2 ou mais pontos; e houver pelo menos uma coluna com 2 ou mais pontos.
  Fora disso, o botão fica desabilitado com a condição que falta.
  *Prova:* teste do núcleo (tabela de alturas habilitadas e desabilitadas).
- **CMP-11** — QUANDO o usuário fechar com CPA, o sistema DEVE montar um somador *ripple* que
  começa na menor coluna com 2 ou mais pontos e segue para a esquerda enquanto houver ponto
  ou vai-um chegando. Cada coluna recebe:
  - um **FA**, se tiver 3 entradas (pontos mais o vai-um da coluna anterior);
  - um **HA**, se tiver 2;
  - um **fio**, se tiver 1 (o próprio ponto, ou o vai-um, vira o bit do resultado).

  Uma primeira coluna com 3 pontos usa o terceiro como *carry-in*. O vai-um da última célula
  que cair numa coluna $\ge W$ segue CMP-09.
  *Prova:* R2 → CPA de $2^0$ a $2^5$ com FA, FA, HA, FA, FA, HA; o vai-um do HA de $2^5$ é
  $R_6$.
- **CMP-12** — QUANDO todas as colunas tiverem no máximo 1 ponto, o sistema DEVE declarar o
  circuito completo e nomear os bits do resultado $R_0 \dots R_{W-1}$. Uma coluna sem ponto
  é o bit constante 0.
  *Prova:* R2, R3 e `2A` com $A$ de 2 bits ($R_0 = 0$).

## Interação (MT-12a)

Seleção pelo mouse (clique alterna) ou pelo teclado (Tab até o ponto, Espaço alterna). Atalhos
fora de campos de texto: **F** forma a célula, **Esc** limpa a seleção, **Ctrl+Z** desfaz,
**Ctrl+Y** ou **Ctrl+Shift+Z** refaz. Uma recusa aparece junto aos botões, mantém a seleção
para o aluno ver o que escolheu e some na próxima mudança de seleção.

## Interação (MT-12b)

"Fechar com CPA" fica ao lado das outras ações; desabilitado, mostra embaixo a condição que
falta (CMP-10), com o mesmo texto que uma recusa daria. Com o circuito completo, as ações de
formar e concluir saem de cena, e o último painel vira o resultado, com $R_0 \dots R_{W-1}$
(CMP-12). O caminho crítico é um botão de alternância, sem confirmação, porque mostra só o
circuito do próprio usuário (CMP-15). A célula com vai-um descartado leva a marca "k=0", e
uma nota abaixo dos painéis oferece "Por quê?" (CMP-09).

## Critérios — métricas do circuito do usuário

- **CMP-13** — O sistema DEVE exibir, para o nível atual, a altura de cada coluna.
- **CMP-14** — O sistema DEVE exibir a contagem de FA, HA e AND do circuito montado até o
  momento.
- **CMP-15** — O sistema DEVE exibir a área em $A_{FA}$ e o atraso em $T_{FA}$ do circuito
  montado até o momento (D-3) e, quando o usuário pedir, destacar **todos** os caminhos
  críticos (todas as células que estão em algum caminho de atraso máximo).
  As métricas contam também as células pendentes do nível atual, para que formar uma célula
  já mostre o efeito.
  *Prova de CMP-13 a CMP-15:* R2 → 13 FA, 4 HA, 13 AND, 15 $A_{FA}$, 6 $T_{FA}$; células
  críticas 1, 2, 4, 6, 8, 9 e 12 a 17, obtidas à mão das prontidões do caso de referência.

## Fora de escopo

Escolher manualmente a coluna da soma ou do vai-um ("modo de ligação manual", candidato à
v0.2); células com mais de 3 entradas; reordenar níveis; CPA com antecipação de vai-um.
