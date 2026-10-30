<!-- Caminho relativo: docs/specs/00-transversal.md -->

# 00 — Requisitos transversais

## Problema

Quem estuda árvores de redução (Wallace, Dadda, AMM) resolve tudo à mão: monta a matriz de
pontos, escolhe os contadores nível a nível, conta área e atraso, desenha o diagrama e
escreve o VHDL. Cada tentativa custa uma folha inteira, então se tenta pouco, e a intuição
de **por que** uma escolha é melhor que outra demora a se formar.

## Resultado esperado

Uma aplicação web em que o estudante:

1. digita uma expressão aritmética sem sinal e vê a matriz de pontos correspondente;
2. comprime a matriz **ele mesmo**, escolhendo cada FA, HA e o CPA final, e vê na hora o
   efeito de cada escolha sobre a altura das colunas, a área e o atraso;
3. obtém o diagrama de blocos, a tabela de ligações e um VHDL estrutural de modelo do
   circuito que **ele** montou;
4. recebe dica ou solução automática **apenas quando pede**.

**Fracasso** seria: um app que resolve o exercício pelo aluno ao primeiro clique, ou um
desenho que não corresponde ao circuito (contagem, atraso ou VHDL errados).

## Público

Estudantes de graduação e pós em Circuitos Aritméticos e quem os ensina. O app tem de
funcionar sem instalação, num navegador comum.

## Princípio didático

> O app mostra **as consequências** das escolhas do usuário o tempo todo e **as respostas**
> só quando pedidas.

| Sempre visível (é retorno sobre o próprio trabalho) | Só sob pedido explícito (é resposta) |
|---|---|
| Regras da notação (o que um FA e um HA fazem) | Dica sobre o próximo passo |
| Altura de cada coluna no nível atual | Próxima jogada sugerida |
| Contagem de FA, HA e AND do circuito montado | Solução automática (Dadda, Wallace) |
| Área e atraso do circuito montado (**⚠ A CONFIRMAR**, ver D-3) | Limite inferior de FA e comparação com o ótimo |
| Recusa de seleção inválida, com a regra que ela viola | Largura da saída $W$ (**⚠ A CONFIRMAR**, ver D-1) |

- **GER-01** — O sistema DEVE exibir dica, jogada sugerida, solução automática ou limite
  teórico somente depois de um pedido explícito do usuário seguido de confirmação.
  *Prova:* teste de ponta a ponta `ajuda-nao-aparece-sem-pedido` (detalhado em `AJU`).
- **GER-02** — SE o usuário fizer uma seleção inválida, ENTÃO o sistema DEVE recusá-la
  citando a regra violada, sem indicar qual seria a seleção correta.
  *Prova:* teste de unidade das mensagens de recusa no núcleo; teste de componente na
  interface.

## Glossário (modelo de domínio)

Os mesmos nomes valem na interface, no núcleo, nos testes e no VHDL gerado.

| Termo | Definição |
|---|---|
| **Ponto** | Um bit. Tem coluna $j$ (peso $2^j$), rótulo (ex.: `a1·b0`, `e2`, `s3`, `k3`), origem (produto parcial de entrada ou saída de célula) e instante de prontidão $t$. |
| **Coluna** | Conjunto de pontos de mesmo peso. Coluna $j$ ↔ $2^j$. |
| **Matriz de pontos** | Os pontos de um nível, organizados por coluna. |
| **Nível** | Uma etapa da compressão. O nível 0 é a matriz inicial; o nível $n+1$ contém os pontos não usados no nível $n$ mais as saídas das células do nível $n$. |
| **Produto parcial** | Ponto de entrada: o AND dos bits de um produto (ex.: `a1·b0` = $a_1 \wedge b_0$). Um termo com uma só variável gera pontos sem AND. |
| **FA** | Somador completo: consome 3 pontos da coluna $j$, produz soma em $j$ e vai-um em $j+1$. Elimina 1 ponto no total. |
| **HA** | Meio-somador: consome 2 pontos da coluna $j$, produz soma em $j$ e vai-um em $j+1$. Não elimina pontos; desloca um para a coluna seguinte. |
| **CPA** | Somador de propagação de vai-um (*ripple*) que fecha a compressão quando as colunas têm no máximo 2 pontos. É formado por células FA/HA encadeadas. |
| **Largura da saída $W$** | Número de bits do resultado: $W = \lceil \log_2(v_{max}+1) \rceil$, com $v_{max}$ o valor da expressão com todas as entradas no máximo. |
| **Circuito** | O conjunto de células (FA, HA, AND) e ligações montado pelo usuário. |

## Convenção de custo

| Célula | Área | Atraso |
|---|---|---|
| FA | $1\,A_{FA}$ | $1\,T_{FA}$ |
| HA | $0{,}5\,A_{FA}$ | $0{,}5\,T_{FA}$ |
| AND (produto parcial) | contado à parte, fora da área em $A_{FA}$ | $0$ |

O instante de prontidão de cada saída é o maior instante entre as entradas mais o atraso da
célula. Todo ponto de entrada nasce em $t = 0$. O atraso do circuito é o maior instante
entre os bits do resultado.

- **GER-03** — O sistema DEVE calcular área e atraso pela convenção acima.
  *Prova:* casos de referência R1 e R2 (abaixo) no núcleo.
- **D-2 ⚠ A CONFIRMAR** — a convenção é fixa na v0.1 (sem tela para mudar os pesos).

## Requisitos não-funcionais

- **GER-04** — O sistema DEVE executar todo o processamento no navegador, sem servidor de
  aplicação e sem enviar a expressão ou o circuito para fora do dispositivo.
  *Prova:* o artefato de produção é um site estático; teste de ponta a ponta sem rede após o
  carregamento.
- **GER-05** — QUANDO o usuário formar ou desfizer uma célula numa matriz de até 256 pontos,
  o sistema DEVE atualizar matriz e métricas em menos de 100 ms.
  *Prova:* teste de desempenho do núcleo (`cargo bench` ou medição no teste de ponta a ponta).
- **GER-06** — O sistema DEVE permitir selecionar pontos e formar células pelo teclado, e
  DEVE distinguir os termos da expressão por rótulo e forma, não só por cor.
  *Prova:* teste de ponta a ponta só com teclado; auditoria automática de acessibilidade
  (axe) sem violação grave.
- **GER-07** — O sistema NÃO DEVE coletar telemetria nem exigir conta.
  *Prova:* revisão do artefato de produção (nenhuma requisição além dos próprios arquivos).
- **D-4 ⚠ A CONFIRMAR** — tela-alvo: computador e tablet (largura ≥ 768 px); no celular o app
  abre e mostra os resultados, mas a compressão por toque fica fora da v0.1.

## Casos de referência

Valores conferidos à mão e por simulação exaustiva antes do projeto. Viram testes do núcleo
e não podem regredir.

**R1 — soma de produtos com fator 2.**
$R = A \times B + C \times D + 2(E+F) + G$, com $A, C, E$ de 3 bits, $B, D, F$ de 2 bits e
$G$ de 1 bit.

| Grandeza | Valor |
|---|---|
| Pontos na matriz inicial | 18 (12 produtos parciais + 3 de $E$ + 2 de $F$ + 1 de $G$) |
| Altura das colunas $2^0..2^3$ | 3, 6, 6, 3 |
| Portas AND | 12 |
| $W$ | 6 ($v_{max} = 63$) |
| Limite inferior de FA | $18 - 6 = 12$ |
| Combinações de entrada | $2^{16} = 65\,536$ |

**R2 — solução só com FA para R1** (reproduzida passo a passo pelo teste):

| Nível | Células | Colunas |
|---|---|---|
| 1 | 5 FA: dois em $2^1$, dois em $2^2$, um em $2^3$ | $2^0$ espera |
| 2 | 3 FA: um em $2^0$, um em $2^2$, um em $2^3$ | vai-um de um FA de $2^1$ passa direto |
| CPA | 4 FA, de $2^1$ a $2^4$; o vai-um do FA de $2^0$ entra como *carry-in* | vai-um final é $R_5$ |

Resultado: **12 FA, 0 HA, 12 AND, área 12 $A_{FA}$, atraso 5 $T_{FA}$, 65 536 casos corretos.**

**R3 — trivial.** $R = A$ com $A$ de 4 bits: 4 pontos, uma por coluna, nenhuma célula,
$W = 4$, atraso 0.

**R4 — multiplicador 4×4.** $R = A \times B$, 4 bits cada: 16 pontos, alturas
1, 2, 3, 4, 3, 2, 1, $W = 8$, limite inferior de 8 FA.

## Fora de escopo da v0.1 (vale para todas as specs)

- Números com sinal (complemento de dois, Baugh-Wooley) e subtração.
- Constantes aditivas na expressão (ex.: `+ 5`).
- Blocos AMM como célula composta; compressores 4:2; CPA que não seja *ripple*.
- Contas de usuário, servidor, compartilhamento por link.
- Síntese para FPGA dentro do app.
