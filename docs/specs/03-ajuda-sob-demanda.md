<!-- Caminho relativo: docs/specs/03-ajuda-sob-demanda.md -->

# 03 — Ajuda sob demanda

## Resultado

O app nunca entrega a resposta sem que o usuário peça. Quando ele pede, o app **pergunta
antes de mostrar**, para dar uma última chance de pensar, e entrega a resposta em duas
etapas: primeiro o resultado, depois o caminho (GER-03). É a regra que o autor fixou: o
objetivo é treinar a intuição, não dispensá-la.

## Os degraus da ajuda

Do mais leve ao mais completo. Cada degrau é pedido separadamente.

| Degrau | O que mostra | Confirmação | Duas etapas |
|---|---|---|---|
| **Regras** | Lembrete da notação, sem nada sobre o estado atual | não | não |
| **Dica** | Um fato sobre o estado atual, do catálogo abaixo, sem citar pontos | sim | não |
| **Próxima jogada** | Destaca os pontos de uma célula que a solução Dadda formaria agora | sim | não |
| **Limite inferior** | O menor número possível de FA | sim | sim |
| **Solução automática** | A solução de Dadda, a de Dadda (Parhami) (AJU-13) ou a de Wallace (AJU-10), cada uma pedida à parte, a partir da matriz inicial, num painel à parte | sim | sim |
| **Otimizador** | Os circuitos não dominados em área e atraso, numa expressão ou num perfil (AJU-14) | sim | sim |

### Catálogo de dicas (v0.1)

Cada dica é um código com parâmetros. Nenhuma dica cita pontos específicos nem diz qual
célula formar.

| Código | Texto (pt-BR) | Parâmetros |
|---|---|---|
| `DICA_NIVEIS_RESTANTES` | "Com altura máxima $h$, faltam no mínimo $n$ níveis até o CPA." Só quando o CPA ainda não está disponível. | $h$, $n$ = quantos números da sequência de Dadda (2, 3, 4, 6, 9, 13…) são menores que $h$ |
| `DICA_COLUNA_MAIS_ALTA` | "A coluna $2^j$ é a mais alta ($h$ pontos); o CPA aceita no máximo 3 na primeira coluna e 2 nas demais." | $j$, $h$ |
| `DICA_PONTO_PARADO` | "Há pontos prontos em $t = 0$ que ainda não entraram em nenhuma célula, numa coluna com 3 ou mais pontos." | — |
| `DICA_CPA_DISPONIVEL` | "Este nível já pode ser fechado com o CPA." | — |

O núcleo lista as dicas que se aplicam ao estado atual, na ordem do catálogo; a cada pedido
a interface mostra a seguinte, em ciclo. (A versão anterior desta regra, "a primeira que se
aplica", mostraria quase sempre a mesma dica.)

## Interação (MT-13)

Os pedidos ficam numa seção "Ajuda" abaixo das ações da compressão; a solução automática
tem um botão por método, "Solução de Dadda" e "Solução de Wallace" (MT-29). Dica e próxima
jogada falam do estado atual e somem na ação seguinte, para que uma resposta velha não
fique na tela. Cada chamada à ajuda do núcleo é uma revelação e é contada (AJU-07); por
isso a interface só chama depois da confirmação. O ciclo das dicas usa o próprio contador:
o $n$-ésimo pedido mostra a dica $n$ módulo a quantidade que se aplica agora.

## Critérios

- **AJU-01** — O sistema DEVE manter as Regras acessíveis a qualquer momento, sem
  confirmação, porque não dependem do exercício.
  *Prova:* teste de componente.
- **AJU-02** — QUANDO o usuário pedir Dica, Próxima jogada, Limite inferior ou Solução
  automática, o sistema DEVE perguntar se ele quer mesmo ver, com o foco inicial em "vou
  tentar mais um pouco", e só mostrar depois da confirmação. A confirmação vale para um
  pedido só (D-5).
  *Prova:* teste de ponta a ponta `ajuda-nao-aparece-sem-pedido`: carrega R1, percorre a
  interface sem pedir ajuda e confere que nenhum texto de $W$, dica, limite ou solução foi
  renderizado; pede, confere que o foco está em "vou tentar mais um pouco", cancela e
  confere de novo; por fim confirma e confere que aparece.
- **AJU-03** — O sistema DEVE mostrar a Solução automática num painel separado, calculada a
  partir da matriz inicial (nível 0), sem alterar o circuito do usuário enquanto ele não
  pedir para usá-la (AJU-09).
  *Prova:* teste de ponta a ponta (o estado do usuário é o mesmo antes e depois).
- **AJU-04** — QUANDO o usuário aceitar a Próxima jogada, o sistema DEVE destacar os pontos
  da primeira célula que o método de Dadda formaria **a partir do estado atual do usuário**,
  percorrendo as colunas da menor para a maior e tomando os pontos na ordem canônica
  (`02-compressao.md`). A célula só é formada se o usuário confirmar a formação.
  As células já formadas pelo usuário no nível contam como formadas: suas somas e vai-uns
  entram na altura que cada coluna terá no nível seguinte. Quando nenhuma coluna passa do
  alvo, a jogada é "concluir o nível"; com todas as colunas em até 2 pontos, "fechar com
  CPA"; com o circuito completo, nada.
  *Prova:* teste do núcleo com saída fixada para R1 no nível 0 (HA em `a0_b1`, `a1_b0`) e a
  partir do fim da árvore de R2 (HA em `f0`, `g0`; HA em `k8`, `s4`; concluir; CPA);
  teste de componente (destaque sem formação).
- **AJU-05** — O sistema DEVE calcular a solução automática pelo método de Dadda: a cada
  nível, a altura-alvo é o maior número da sequência 2, 3, 4, 6, 9, 13… menor que a altura
  máxima atual; cada coluna, da menor para a maior, usa FA (e HA quando falta reduzir só 1)
  até ficar na altura-alvo, contando os vai-uns que chegam; no fim, fecha com o CPA de
  CMP-11.
  A solução é, por construção, seguir a Próxima jogada (AJU-04) desde o nível 0 até o
  fim: sugestão e solução nunca discordam.
  **É o método clássico de Dadda** (HA quando falta reduzir só 1). A fig. 8.14 do
  Parhami (*Computer Arithmetic*), com sete operandos de 6 bits, desenha uma variante: usa FA
  também onde falta reduzir 1 e, no terceiro nível, até em colunas que já estão no alvo,
  escolha que nenhuma regra coluna a coluna reproduz. O total de FA é o mesmo (33), mas a
  figura usa 3 HA e o método clássico, 6 (34,5 contra 36 $A_{FA}$; 9,5 contra
  10 $T_{FA}$). A variante "FA sempre que a coluna passa do alvo" iguala os totais da
  figura, mas não os níveis 2 e 3; no MT-29b ela não substituiu o método clássico, que é o
  algoritmo publicado e o que a próxima jogada ensina. Desde o MT-44 ela é oferecida **ao
  lado**, como a solução Dadda (Parhami) (AJU-13).
  *Prova:* teste do núcleo: para R1 e R4, a solução está correta (VER-01), cada nível
  respeita a altura-alvo, e o número de FA mais vai-uns descartados é pelo menos o limite
  inferior (AJU-06). Contagens calculadas à mão antes do código: R4 → 8 FA, 4 HA,
  10 $A_{FA}$, 5,5 $T_{FA}$; R1 → 13 FA, 8 HA, 17 $A_{FA}$ (4 níveis e o CPA). Sete
  operandos de 6 bits → 5 FA + 1 HA; 10 FA + 1 HA; 5 FA + 2 HA; 6 FA + 1 HA; CPA com
  7 FA + 1 HA (teste de regressão do MT-29b).
- **AJU-06** — O sistema DEVE calcular o limite inferior como (pontos da matriz inicial) −
  $W$ e explicá-lo na etapa 2: cada FA elimina 1 ponto, cada vai-um descartado elimina 1, e
  HA não elimina nenhum; como o circuito completo tem no máximo $W$ pontos, FA + vai-uns
  descartados $\ge$ pontos − $W$. O app chama isso de **limite inferior**, nunca de ótimo.
  *Prova:* R1 → 13; R4 → 8.
- **AJU-07** — O sistema DEVE registrar quantas ajudas de cada degrau foram usadas e mostrar
  esse registro no resumo exibido quando o circuito fica completo (CMP-12).
  *Prova:* teste do núcleo do contador e teste de componente do resumo.
- **AJU-08** — Uma dica NÃO DEVE conter identificador de ponto nem de célula.
  *Prova:* por construção: o tipo `Hint` do núcleo só carrega números (altura, coluna,
  níveis); testes do núcleo para R1 no nível 0, R2 com o CPA disponível e circuito
  completo.

- **AJU-09** — QUANDO o usuário já tiver visto a solução nível a nível (etapa 2 de AJU-03)
  e pedir "Usar esta solução", o sistema DEVE perguntar antes e, confirmado, substituir o
  circuito do usuário pela solução de Dadda inteira **num só passo de desfazer**: Desfazer
  devolve o circuito anterior, e Refazer aplica de novo. Cada aplicação conta no registro
  de ajudas (AJU-07) e vai para o arquivo do exercício (SAL-02). Antes da etapa 2 não há
  botão, e o núcleo recusa o pedido; se o circuito já for igual à solução, nada muda nem é
  contado; no modo leitura (GER-10) o botão não aparece. Aplicar **substitui**, não corrige:
  nada do circuito do usuário é aproveitado.
  *Prova:* teste do núcleo (um passo de desfazer, refazer, recusa com outra matriz, circuito
  já igual); teste da ponte (recusa antes da etapa 2, contagem); teste do núcleo do arquivo
  (campo novo opcional; aplicação sem etapa 2 é revelação inconsistente); teste de ponta a
  ponta: R1 → etapa 2 → usar → "0 erros" na verificação e VHDL disponível; desfazer devolve
  o circuito do usuário; o resumo registra a aplicação.

- **AJU-10** — O sistema DEVE oferecer, ao lado da de Dadda, a solução automática pelo
  método de Wallace, com o mesmo fluxo de AJU-02, AJU-03 e AJU-09: duas etapas confirmadas,
  painel à parte e "Usar a solução de Wallace" só depois da etapa 2 **dela**. Cada método
  tem a própria contagem no registro de ajudas (AJU-07) e no arquivo do exercício
  (`help.wallace`, `help.wallacePath`, `help.wallaceApplied`, opcionais). A etapa 2 mostra,
  junto dos níveis, uma frase com a regra do método.
  **Regra (a da fig. 8.10 do Parhami, *Computer Arithmetic*):** a cada
  nível, com altura máxima $h$, a altura-alvo é $2\lfloor h/3 \rfloor + (h \bmod 3)$, as
  linhas que sobram ao agrupar de 3 em 3 (3 → 2, 4 → 3, 5 → 4, 6 → 4, 7 → 5, 9 → 6,
  13 → 9). Cada coluna, da menor para a maior, usa **todos os FA** que os pontos livres
  permitem (reduz o quanto antes) e um HA **só se** ainda passaria do alvo, contando os
  vai-uns que chegam. No fim, o CPA de CMP-11. Não é o Wallace com HA em todo par que sobra.
  A próxima jogada (AJU-04) e as dicas continuam sendo as de Dadda.
  *Prova:* testes do núcleo com contagens calculadas à mão antes do código: R4 → 3 FA;
  2 FA + 1 HA; CPA com 3 FA + 3 HA (8 FA, 4 HA, 5,5 $T_{FA}$, iguais às de Dadda); R1 →
  4 FA; 4 FA; 2 FA; 2 FA + 1 HA; CPA com 1 FA + 2 HA (13 FA, 3 HA, 14,5 $A_{FA}$,
  6 $T_{FA}$, contra 17 $A_{FA}$ e 7 $T_{FA}$ de Dadda); sete operandos de 6 bits →
  12 FA; 6 FA; 6 FA; 4 FA + 1 HA e um somador de 7 bits, como na figura; propriedade com
  expressões aleatórias (verifica, respeita os alvos e o limite inferior); testes da ponte
  (cada método só se aplica depois da própria etapa 2); teste de componente; teste de
  ponta a ponta: R1 → Wallace nível a nível → usar → "0 erros" e o registro da aplicação.

- **AJU-11** — O sistema DEVE mostrar a **comparação de soluções**: o circuito do usuário
  ao lado de cada solução automática já revelada, com área, atraso, FA, HA, AND, níveis
  antes do CPA, largura do CPA, células no caminho crítico, área × atraso, área × atraso²
  e se a solução é **não dominada** em área e atraso entre as completas (um circuito
  incompleto fica de fora da fronteira). A comparação NÃO DEVE revelar nem contar nada: a
  coluna de um método entra com a etapa 1 dele (as métricas), e a tabela de alturas dele
  (CMP-18), com a etapa 2 (os níveis). Junto do Dadda, uma nota diz que é o clássico
  (AJU-05). Versão mínima do MT-38; o gráfico área × atraso, as versões salvas e os números
  do Quartus ficam para depois.
  *Prova:* teste com o núcleo real dos números de R1, calculados à mão das métricas: aluno
  (R2) 15 $A_{FA}$, 6 $T_{FA}$, 13 FA, 4 HA, 3 níveis, CPA de 6 bits, 90 e 540; Dadda 17,
  7, 8 HA, 4 níveis, CPA de 6, 119 e 833; Wallace 14,5, 6, 3 HA, 4 níveis, CPA de 3, 87 e
  522; só o Wallace é não dominado. Teste da fronteira (empate, compromisso, incompleto);
  teste de componente (nada antes da etapa 1, tabela de alturas só depois da etapa 2,
  registro de ajudas igual); teste de ponta a ponta com R2.
- **AJU-12** — ONDE a matriz vier de um perfil de alturas (EXP-16), o sistema DEVE
  oferecer, ao lado de Dadda e Wallace, a **solução ótima** que atinge o perfil (CMP-19):
  a menor área e, entre as de menor área, o menor atraso e depois o menor número de níveis,
  com FA e HA sobre os pontos mais cedo prontos de cada coluna, tudo medido com os custos
  em vigor (GER-12): trocar os custos refaz a busca. Ela segue o fluxo das
  outras soluções: etapa 1 (métricas), etapa 2 (níveis, com a regra numa frase) e "Usar a
  solução ótima" num passo de desfazer, cada um confirmado e contado à parte. Como a
  solução para no perfil, o último painel é a saída no perfil, que pode ser redundante. Ela entra
  na comparação (AJU-11), onde, com perfil, conta como completa a solução que o atinge.
  SE a busca passar do limite, ENTÃO o sistema DEVE dizer que o perfil é grande demais em
  vez de oferecê-la, sem travar e sem contar nada. O sistema DEVE também oferecer o
  **nome do bloco conhecido** que o perfil descreve (FA, HA, somador de $n$ bits, somador
  de $n$ bits com vai-um de entrada, $n$ FA lado a lado, contador ou compressor) como
  ajuda confirmada e contada: reconhecer o bloco faz parte do exercício. Para uma
  expressão, nem a solução ótima nem o nome do bloco aparecem.
  *Prova:* teste do núcleo: o ótimo reproduz área e atraso dos 11 perfis de referência do
  projeto, conferidos à mão e por uma busca exaustiva independente (a: 3 e 3; b: 1 e 1;
  c: 4 e 3; d: 6 e 4; e: 1,5 e 1,5; f: 2,5 e 2,5; g: 4 e 3; h: 3 e 1; i: 7 e 4; j: 3 e 2;
  k: 2 e 2, em $A_{FA}$ e $T_{FA}$); f) dá FA, FA e HA nos níveis 0, 1 e 2; proptest: com
  saída de um bit por coluna, o ótimo nunca perde para Dadda nem Wallace em (área,
  atraso); a busca de 26 pontos em quatro colunas para no limite; os blocos de cada item.
  Testes da ponte (contagens, `optimalSearch` sem contar, nome revelado de novo sem
  contar); testes de componente (duas etapas, saída redundante, perfil grande demais,
  nada para uma expressão, comparação); teste de ponta a ponta com `{5; 2,1}` (bloco,
  ótimo aplicado, registro e retomada depois de recarregar). Com custos (GER-12), à mão,
  no perfil e) `{2,2; 1,1,1}`: as saídas são 1 FA + 1 HA ($F+H$) ou 4 HA ($4H$); com HA
  de área 0,3, o ótimo vira 4 HA (1,2 contra 1,3; atraso $3H$ = 0,9), com 0,33 ainda 4 HA
  (1,32 contra 1,33) e com 0,34 volta o FA; com FA 0,99 e HA 0,33 as áreas empatam em
  1,32 e decide o atraso (4 HA com FA de atraso 1; FA + HA com atraso 0,4). Teste de
  ponta a ponta: o ótimo revelado é refeito ao trocar os custos, sem contar de novo.
  *Implementação (MT-42):* a busca roda no mesmo *worker* da verificação, ao abrir o perfil
  e de novo depois de trocar os custos; enquanto ela roda, a tela diz "Buscando a solução
  ótima…" com uma barra de progresso indeterminada (a busca não tem etapas para contar) e
  segue usável. A sessão só aceita a resposta se a expressão e os custos ainda forem os da
  busca e se o circuito, refeito pelas regras como um arquivo do exercício (SAL-03),
  atingir o perfil; uma resposta velha é descartada. Sem o *worker*, a busca roda na
  página, como antes. Uma verificação pedida durante a busca espera ela terminar.
  *Prova:* testes da ponte (espera, resposta aceita, grande demais, resposta de outros
  custos ou de outro perfil descartada, circuito adulterado recusado, ótimo revelado
  escondido enquanto a busca é refeita); testes de componente (buscando, resposta, perfil
  grande demais, reserva sem *worker*, resposta velha descartada); teste de ponta a ponta
  com o maior perfil aceito (26 pontos).
- **AJU-13** — O sistema DEVE oferecer, ao lado de Dadda e Wallace, a solução
  **Dadda (Parhami)**, com o mesmo fluxo de AJU-10: duas etapas confirmadas, painel à
  parte, "Usar a solução de Dadda (Parhami)" só depois da etapa 2 dela, contagem própria no
  registro de ajudas e no arquivo (`help.parhami`, `help.parhamiPath`,
  `help.parhamiApplied`, opcionais) e uma coluna na comparação (AJU-11). A etapa 2 mostra a
  regra numa frase. Vale para expressões e perfis de alturas.
  **Regra:** as alturas-alvo de Dadda (AJU-05); em cada coluna acima do alvo, da menor
  para a maior, contando os vai-uns que chegam, um FA sempre que há 3 pontos livres, mesmo
  quando falta reduzir só 1, e um HA só quando restam 2. No fim, o CPA de CMP-11. É a
  variante da fig. 8.14 do Parhami (*Computer Arithmetic*): iguala os totais da figura
  (33 FA, 3 HA), mas não cada nível dela. A próxima jogada (AJU-04) e as dicas continuam
  sendo as do Dadda clássico.
  *Prova:* contagens calculadas antes do código por um script à parte da regra, que
  reproduz também o Dadda clássico do núcleo: sete operandos de 6 bits → 6 FA; 11 FA;
  5 FA; 5 FA + 2 HA; CPA com 6 FA + 1 HA (33 FA, 3 HA, 34,5 $A_{FA}$, contra 36 do
  clássico); cinco operandos de 6 bits → 6 FA; 5 FA; 5 FA + 2 HA; CPA com 5 FA + 2 HA
  (21 FA, 4 HA, 23 $A_{FA}$, contra 24); R1 → 1 FA; 4 FA; 3 FA; 2 FA + 3 HA; CPA com
  3 FA + 2 HA (13 FA, 5 HA, 15,5 $A_{FA}$, 7 $T_{FA}$, os números medidos no MT-29b);
  propriedade com expressões aleatórias (verifica, respeita os alvos de Dadda e o limite
  inferior); arquivo sem os campos abre, contagens vão e voltam, revelação inconsistente
  recusada; teste da ponte (sessão à parte, aplicada só depois da própria etapa 2); testes
  de componente (duas etapas, regra, aplicação, coluna da comparação com os números de
  R1); teste de ponta a ponta: R1 → Dadda (Parhami) nível a nível → usar → "0 erros" e o
  registro.

- **AJU-14** — O sistema DEVE oferecer o **otimizador**, numa expressão ou num perfil de
  alturas: os circuitos que nenhum outro domina em área e atraso, medidos com os custos em
  vigor (GER-12), da menor área para a maior (e do maior atraso para o menor). Ele segue
  o fluxo das soluções, com contagem própria no registro de ajudas e no arquivo
  (`help.optimizer`, `help.optimizerPath`, `help.optimizerApplied`, opcionais): a etapa 1
  mostra as métricas de cada opção e as põe na comparação (AJU-11); a etapa 2, os níveis
  de cada uma, com a regra numa frase, e "Usar a opção $n$", confirmado e num passo de
  desfazer (AJU-09). A tela diz como as opções foram achadas: por busca exata; por busca
  exata até uma área, acima da qual vêm da heurística; ou só pela heurística.
  **Modelo:** o do AJU-12. Os pontos se distinguem só pela coluna e pela hora em que ficam
  prontos, e cada nível escolhe quantas células de cada tipo forma em cada coluna, as
  maiores sobre os pontos mais cedo prontos. A meta é o perfil ou, numa expressão, o
  circuito completo, que pode fechar com o CPA (CMP-10, CMP-11). Os contadores 7:3 e 15:4
  só entram quando custam menos, em área ou em atraso, que a rede de FA que os substitui
  (4 FA com 3 de profundidade; 11 FA com 5; ADR 0005). Com os custos padrão, ficam de
  fora.
  **Busca exata:** a área sobe pelas somas que os custos das células permitem, e em cada
  área procura o menor atraso abaixo do da opção anterior; cada resposta é um ponto novo
  da fronteira. Uma coluna desce, por célula, o número de entradas menos 1, e cada célula
  manda uma saída para a coluna de cima. Assim, da coluna mais baixa para a mais alta,
  quanto cada coluna ainda precisa descer dá um limite inferior da área, que corta a
  busca. Ela para na maior área entre as opções da heurística ou desiste depois de
  1 000 000 de níveis tentados, o limite do AJU-12. Nesse caso, as opções valem como
  exatas até a última área terminada. Acima de 64 pontos, só a heurística roda.
  **Heurística:** regras nível a nível, com os alvos de Dadda ou de Wallace e uma de três
  políticas: HA só quando falta reduzir 1 (Dadda), FA sempre que há 3 pontos livres
  (Dadda (Parhami)) ou todos os FA que os pontos permitem (Wallace). Cada regra roda com e
  sem os contadores que valem a pena. A elas se somam as soluções de Dadda, Dadda
  (Parhami) e Wallace. As opções são a fronteira de tudo o que foi achado, então nenhuma
  é dominada por esses três métodos.
  **Onde roda:** como o AJU-12 (MT-42), no *worker*, ao abrir o exercício e depois de
  trocar os custos. Enquanto isso, a tela diz "Buscando as opções do otimizador…". A
  sessão só aceita a resposta da expressão, das larguras e dos custos em vigor, e cada
  opção é refeita pelas regras e precisa estar completa (ou no perfil). Sem o *worker*,
  roda na página. A busca acabar não recria a seção de ajuda, então um diálogo aberto
  naquele momento continua aberto.
  *Prova:* testes do núcleo:
  - nos 11 perfis de referência, a opção de menor área tem a área e o atraso do ótimo do
    AJU-12, com os custos padrão e com HA de 0,3;
  - *proptest*: em perfis pequenos aleatórios, a opção de menor área é o ótimo do AJU-12;
  - R1, achado pela busca e verificado exaustivamente: exata até 15,5 $A_{FA}$, com duas
    opções, (14,5; 6), a área e o atraso do Wallace, e (15; 5,5);
  - *proptest* com expressões aleatórias: nenhuma opção é dominada por Dadda, Dadda
    (Parhami) ou Wallace, e todas verificam;
  - o 32 × 32 e o perfil de 26 pontos vão pela heurística;
  - com um 7:3 de área 2, `{7; 1,1,1}` dá um contador (2 e 3 $A_{FA}$ e $T_{FA}$);
  - um perfil que a entrada já atinge dá uma opção sem células;
  - o CPA da busca segue CMP-10;
  - as regras com os alvos e a política de cada método reproduzem a área e o atraso dele
    em R1.

  Testes da ponte: espera, etapas e contagem, aplicação num passo de desfazer, contagens no
  arquivo; resposta de outros custos, de outra expressão ou de outras larguras
  descartada; opção adulterada ou incompleta recusada; reserva. Testes de componente:
  buscando, as duas etapas, comparação, aplicação, custos novos, reserva, perfil,
  exercício reaberto e o diálogo que continua aberto. Teste de ponta a ponta: R1 com
  interação durante a busca (GER-06), a opção 2 aplicada e verificada; o 32 × 32 pela
  heurística.

## Fora de escopo

Comparação de várias heurísticas e o gráfico área × atraso (corte do MT-46); otimalidade
fora do modelo do AJU-12 e do AJU-14; correção
automática do circuito do usuário (AJU-09 substitui o circuito inteiro, não o corrige);
ajuda por IA; "não perguntar de novo" (D-5).
