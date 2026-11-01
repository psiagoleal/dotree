<!-- Caminho relativo: docs/specs/04-verificacao.md -->

# 04 — Verificação

## Resultado

O usuário (e o projeto) têm prova de que o circuito montado calcula a expressão para todas
as entradas, ou para uma amostra declarada quando elas são muitas.

## Por que verificar se o app já aplica as regras

Todo circuito completo montado pela interface é correto **por construção**: FA e HA
preservam o valor ponderado ($x + y + z = s + 2k$), o vai-um descartado vale sempre 0
(CMP-09), e o app posiciona soma e vai-um sozinho. A verificação existe por três motivos:

1. provar que o **núcleo** implementa essas regras, como teste de propriedade;
2. dar ao usuário a mesma evidência que ele daria em aula: "16 384 casos corretos";
3. ser a referência do testbench VHDL gerado (`VHD`), que usa a mesma sequência de vetores.

## Vetores de teste

| Soma das larguras das entradas | No app | No testbench VHDL |
|---|---|---|
| até 20 bits | todas as combinações | todas as combinações |
| de 21 a 24 bits | todas as combinações | amostra de 4096 vetores |
| acima de 24 bits | amostra de $2^{20}$ vetores | amostra de 4096 vetores |

A amostra começa sempre por "todas as entradas zeradas" e "todas no máximo" e segue com o
gerador **xorshift64\*** de semente fixa `0x00D0_7EE5`. Os 4096 vetores do testbench são os
4096 primeiros da mesma sequência do app.

**Vetor empacotado.** Um vetor de entrada é um inteiro com as variáveis em ordem alfabética,
a primeira nos bits menos significativos (em R1: $A$ nos bits 2..0, $B$ em 5..3, …, $G$ no
bit 13). Na varredura exaustiva, os vetores são 0, 1, 2, … nessa codificação; a amostra é
mascarada para a largura total. A mesma codificação vale para o testbench VHDL.

## Critérios

- **VER-01** — QUANDO o circuito estiver completo e o usuário pedir verificação, o sistema
  DEVE simular o circuito célula a célula para os vetores da tabela acima, comparar com o
  valor da expressão e informar quantos casos conferiu e se foi exaustivo ou por amostra.
  *Prova:* R2 → 16 384 casos, exaustivo, 0 erros.
- **VER-02** — O sistema DEVE gerar a amostra exatamente como descrito acima.
  *Prova:* teste do núcleo com os 8 primeiros vetores fixados.
- **VER-03** — SE algum caso falhar, ENTÃO o sistema DEVE mostrar o primeiro contraexemplo:
  valores das entradas, resultado esperado, resultado obtido e os bits que diferem.
  *Prova:* teste do núcleo com um circuito propositalmente corrompido (vai-um ligado à
  coluna errada), que precisa ser pego.
- **VER-04** — O sistema DEVE executar a verificação sem travar a interface, mostrar o
  progresso quando ela levar mais de 0,5 s e permitir cancelá-la.
  *Prova:* teste de ponta a ponta com $A \times B$ de 16 × 16 bits (progresso visível;
  cancelar devolve a interface sem resultado parcial).
- **VER-05** — ENQUANTO o circuito estiver incompleto, o sistema DEVE manter a verificação
  desabilitada, explicando que ela exige circuito completo.
  *Prova:* teste de componente.
- **VER-06** — O núcleo DEVE ter um teste de propriedade: para expressões e larguras
  aleatórias, qualquer redução montada pela API (células e CPA escolhidos ao acaso) e a
  solução Dadda passam na verificação.
  *Prova:* `cargo test` com proptest (64 casos), caso que falha registrado pelo proptest. A
  parte das reduções aleatórias entrou no MT-7; a da solução Dadda entra no MT-8.
- **VER-07** — O contraexemplo relatado DEVE ser o primeiro vetor que falha na ordem de
  varredura, e a contagem de casos conferidos DEVE incluí-lo.
  *Prova:* teste do núcleo contra um simulador escalar independente, escrito no próprio
  teste.

## Fora de escopo

Verificação formal (equivalência por SAT ou BDD); simulação com atraso de porta.
