<!-- Caminho relativo: docs/specs/03-ajuda-sob-demanda.md -->

# 03 — Ajuda sob demanda

## Resultado

O app nunca entrega a resposta sem que o usuário peça. Quando ele pede, o app **pergunta
antes de mostrar**, para dar uma última chance de pensar. É a regra que o autor fixou: o
objetivo é treinar a intuição, não dispensá-la.

## Os degraus da ajuda

Do mais leve ao mais completo. Cada degrau é pedido separadamente.

| Degrau | O que mostra | Exemplo (R1, nível 0) |
|---|---|---|
| **Regras** | Lembrete da notação, sem nada sobre o estado atual | "Um FA consome 3 pontos de uma coluna e devolve 1 nela e 1 na seguinte." |
| **Dica** | Um fato sobre o estado atual, sem dizer o que fazer | "A coluna $2^1$ tem 6 pontos; a altura-alvo de Dadda para o próximo nível é 4." |
| **Próxima jogada** | Destaca uma célula que a solução automática formaria agora | Destaca 3 pontos de $2^1$ como FA |
| **Solução automática** | Uma solução completa (Dadda ou Wallace) num painel à parte | Níveis, células, área e atraso |
| **Comparação** | Limite inferior de FA e as métricas das soluções automáticas ao lado das do usuário | "Limite: 12 FA. Você: 14 FA, 6 $T_{FA}$." |

## Critérios

- **AJU-01** — O sistema DEVE manter as Regras acessíveis a qualquer momento, sem
  confirmação, porque não dependem do exercício.
  *Prova:* teste de componente.
- **AJU-02** — QUANDO o usuário pedir Dica, Próxima jogada, Solução automática ou Comparação,
  o sistema DEVE perguntar se ele quer mesmo ver, com a opção "vou tentar mais um pouco"
  como padrão do diálogo, e só mostrar depois da confirmação.
  *Prova:* teste de ponta a ponta `ajuda-nao-aparece-sem-pedido`: carrega R1, percorre a
  interface sem pedir ajuda e confere que nenhum texto de dica, limite ou solução foi
  renderizado; depois pede, cancela e confere de novo; por fim confirma e confere que
  aparece.
- **AJU-03** — O sistema DEVE mostrar a Solução automática num painel separado, sem alterar
  o circuito do usuário.
  *Prova:* teste de ponta a ponta (o estado do usuário é o mesmo antes e depois).
- **AJU-04** — QUANDO o usuário aceitar a Próxima jogada, o sistema DEVE apenas destacar os
  pontos sugeridos. A célula só é formada se o usuário confirmar a formação.
  *Prova:* teste de componente.
- **AJU-05** — O sistema DEVE oferecer duas soluções automáticas: Dadda (alturas-alvo
  2, 3, 4, 6, 9, 13…) e Wallace (comprimir o máximo possível a cada nível).
  *Prova:* teste do núcleo: para R1 e R4, cada solução é correta (VER-01), respeita a regra
  do seu método em cada nível e tem pelo menos tantos FA quanto o limite inferior.
- **AJU-06** — O sistema DEVE calcular o limite inferior de FA como
  (pontos da matriz inicial) − $W$, e explicar de onde ele vem (cada FA elimina 1 ponto;
  HA não elimina nenhum).
  *Prova:* R1 → 12; R4 → 8.
- **AJU-07** — O sistema DEVE registrar quantas ajudas de cada degrau foram usadas e mostrar
  esse registro no resumo do exercício concluído.
  *Prova:* teste do núcleo do contador e teste de componente do resumo.
- **D-5 ⚠ A CONFIRMAR** — a confirmação de AJU-02 vale para cada pedido. Não existe "não
  perguntar de novo" na v0.1.

## Fora de escopo

Busca da solução ótima (menor atraso ou menor área) por força bruta; correção automática do
circuito do usuário; ajuda por IA.
