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
| **Solução automática** | A solução de Dadda ou a de Wallace (AJU-10), cada uma pedida à parte, a partir da matriz inicial, num painel à parte | sim | sim |

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
  **É o método clássico de Dadda** (HA quando falta reduzir só 1). A fig. 8.14 da
  referência do curso (Parhami), com sete operandos de 6 bits, desenha uma variante: usa FA
  também onde falta reduzir 1 e, no terceiro nível, até em colunas que já estão no alvo,
  escolha que nenhuma regra coluna a coluna reproduz. O total de FA é o mesmo (33), mas a
  figura usa 3 HA e o método clássico, 6 (34,5 contra 36 $A_{FA}$; 9,5 contra
  10 $T_{FA}$). A variante "FA sempre que a coluna passa do alvo" iguala os totais da
  figura, mas não os níveis 2 e 3; foi avaliada no MT-29b e não adotada: o app fica com o
  algoritmo publicado, que é o que a próxima jogada ensina.
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
  **Regra (a da referência do curso, Parhami, *Computer Arithmetic*, fig. 8.10):** a cada
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
  (CMP-16), com a etapa 2 (os níveis). Junto do Dadda, uma nota diz que é o clássico
  (AJU-05). Versão mínima do MT-38; o gráfico área × atraso, as versões salvas e os números
  do Quartus ficam para depois.
  *Prova:* teste com o núcleo real dos números de R1, calculados à mão das métricas: aluno
  (R2) 15 $A_{FA}$, 6 $T_{FA}$, 13 FA, 4 HA, 3 níveis, CPA de 6 bits, 90 e 540; Dadda 17,
  7, 8 HA, 4 níveis, CPA de 6, 119 e 833; Wallace 14,5, 6, 3 HA, 4 níveis, CPA de 3, 87 e
  522; só o Wallace é não dominado. Teste da fronteira (empate, compromisso, incompleto);
  teste de componente (nada antes da etapa 1, tabela de alturas só depois da etapa 2,
  registro de ajudas igual); teste de ponta a ponta com R2.

## Fora de escopo

Busca da solução de menor atraso ou menor área; correção
automática do circuito do usuário (AJU-09 substitui o circuito inteiro, não o corrige);
ajuda por IA; "não perguntar de novo" (D-5).
