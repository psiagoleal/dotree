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
| **Solução automática** | A solução Dadda a partir da matriz inicial, num painel à parte | sim | sim |

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
  partir da matriz inicial (nível 0), sem alterar o circuito do usuário.
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
  *Prova:* teste do núcleo: para R1 e R4, a solução está correta (VER-01), cada nível
  respeita a altura-alvo, e o número de FA mais vai-uns descartados é pelo menos o limite
  inferior (AJU-06). Contagens calculadas à mão antes do código: R4 → 8 FA, 4 HA,
  10 $A_{FA}$, 5,5 $T_{FA}$; R1 → 13 FA, 8 HA, 17 $A_{FA}$ (4 níveis e o CPA).
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

## Fora de escopo

Solução de Wallace (v0.2); busca da solução de menor atraso ou menor área; correção
automática do circuito do usuário; ajuda por IA; "não perguntar de novo" (D-5).
