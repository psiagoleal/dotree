<!-- Caminho relativo: docs/specs/05-diagrama.md -->

# 05 — Diagrama de blocos e tabela de ligações

## Resultado

A partir do circuito completo, o app desenha o diagrama de blocos e monta a tabela de
ligações: os dois artefatos que o estudante copia para a folha de resposta.

## Critérios

- **DIA-01** — QUANDO o circuito estiver completo, o sistema DEVE desenhar o diagrama de
  blocos com cada célula (FA, HA) como um bloco, organizado por nível (de cima para baixo) e
  por coluna (peso maior à esquerda), com as ligações entre blocos.
  *Prova:* teste do núcleo do leiaute (posição de cada bloco em R2) e teste visual de
  referência (captura comparada) na interface.
- **DIA-02** — O sistema DEVE representar os produtos parciais como um bloco de portas AND
  no topo, com a opção de expandi-lo em portas individuais.
  *Prova:* teste de componente.
- **DIA-03** — O sistema DEVE rotular cada ligação com o nome do sinal (`a1·b0`, `s3`, `k3`,
  `R0`), os mesmos nomes da matriz e do VHDL.
  *Prova:* teste do núcleo: o conjunto de rótulos do diagrama é igual ao conjunto de sinais
  do VHDL gerado.
- **DIA-04** — O sistema DEVE gerar a tabela de ligações com uma linha por célula: nome,
  nível, coluna, entradas x, y, z, saída soma, saída vai-um.
  *Prova:* R2 → 12 linhas, iguais à tabela conferida no caso de referência.
- **DIA-05** — QUANDO o usuário pedir, o sistema DEVE destacar o caminho crítico no diagrama.
  *Prova:* R2 → FA de $2^0$ e os quatro FA do CPA destacados.
- **DIA-06** — O sistema DEVE exportar o diagrama em SVG e a tabela de ligações em Markdown
  e CSV.
  *Prova:* teste de ponta a ponta (arquivos baixados, SVG válido).
- **DIA-07** — ENQUANTO o circuito estiver incompleto, o sistema DEVE desenhar o diagrama
  parcial até o último nível concluído, sinalizando que está incompleto.
  *Prova:* teste de componente.

## Fora de escopo

Edição do diagrama à mão (arrastar blocos); exportação em PDF ou PNG; esquemático no nível
de porta lógica dentro de FA e HA.
