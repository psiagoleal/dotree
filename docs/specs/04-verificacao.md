<!-- Caminho relativo: docs/specs/04-verificacao.md -->

# 04 — Verificação

## Resultado

O usuário (e o projeto) têm prova de que o circuito montado calcula a expressão para
**todas** as entradas.

## Por que verificar se o app já aplica as regras

Como o app posiciona soma e vai-um sozinho, todo circuito completo montado pela interface é
correto por construção. A verificação existe por três motivos:

1. provar que o **núcleo** está certo, como teste de propriedade em cada caso de referência;
2. dar ao usuário a mesma evidência que ele daria em aula: "65 536 casos corretos";
3. ser a mesma referência do testbench VHDL gerado (`VHD`), que precisa concordar com ela.

## Critérios

- **VER-01** — QUANDO o circuito estiver completo e o usuário pedir verificação, o sistema
  DEVE simular o circuito célula a célula para todas as combinações de entrada, comparar
  com o valor da expressão e informar o número de casos conferidos.
  *Prova:* R2 → 65 536 casos, 0 erros.
- **VER-02** — ONDE a soma das larguras das entradas passar de 24 bits, o sistema DEVE
  verificar uma amostra de $2^{20}$ combinações (sempre incluindo todas zeradas e todas no
  máximo), declarando na tela que a verificação foi por amostragem.
  *Prova:* teste do núcleo com R4 ampliado para 16 × 16 bits.
- **VER-03** — SE algum caso falhar, ENTÃO o sistema DEVE mostrar o primeiro contraexemplo:
  valores das entradas, resultado esperado, resultado obtido e os bits que diferem.
  *Prova:* teste do núcleo com um circuito propositalmente corrompido (ex.: vai-um ligado à
  coluna errada), que precisa ser pego.
- **VER-04** — O sistema DEVE executar a verificação sem travar a interface e mostrar o
  progresso quando ela levar mais de 0,5 s.
  *Prova:* teste de ponta a ponta com o caso de 16 × 16 bits.
- **VER-05** — O núcleo DEVE ter um teste de propriedade: para expressões e larguras
  aleatórias dentro dos limites, as soluções Dadda e Wallace passam na verificação.
  *Prova:* `cargo test` com gerador aleatório (proptest), semente registrada.

## Fora de escopo

Verificação formal (equivalência por SAT/BDD); simulação com atraso de porta.
