<!-- Caminho relativo: docs/specs/00-transversal.md -->

# 00 — Requisitos transversais

## Problema

Quem estuda árvores de redução (Wallace, Dadda, AMM) resolve tudo à mão: monta a matriz de
pontos, escolhe os contadores nível a nível, conta área e atraso, desenha o diagrama e
escreve o VHDL. Cada tentativa custa uma folha inteira, então se tenta pouco, e a intuição
de **por que** uma escolha é melhor que outra demora a se formar.

## Resultado esperado

Uma aplicação web em que o estudante:

1. digita uma expressão aritmética sem sinal e vê a matriz de pontos correspondente, em
   pontos coloridos por termo ou em variáveis indexadas ($a_i b_j$);
2. comprime a matriz **ele mesmo**, escolhendo cada FA, HA e o CPA final, e vê na hora o
   efeito de cada escolha sobre a altura das colunas, a área e o atraso;
3. obtém o diagrama de blocos, a tabela de ligações e um VHDL estrutural de modelo do
   circuito que **ele** montou;
4. recebe dica ou resposta **apenas quando pede**, e a resposta chega em duas etapas:
   primeiro o resultado, depois o caminho.

**Fracasso** seria: um app que resolve o exercício pelo aluno ao primeiro clique, ou um
desenho que não corresponde ao circuito (contagem, atraso ou VHDL errados).

## Público

Estudantes de graduação e pós em Circuitos Aritméticos e quem os ensina. O app tem de
funcionar sem instalação, num navegador comum.

## Princípio didático

> O app mostra **as consequências** das escolhas do usuário o tempo todo e **as respostas**
> só quando pedidas, em duas etapas.

| Sempre visível (é retorno sobre o próprio trabalho) | Só sob pedido explícito e confirmação (é resposta) |
|---|---|
| Regras da notação (o que um FA e um HA fazem) | Largura da saída $W$ e $v_{max}$ (D-1) |
| Altura de cada coluna no nível atual | Dica sobre o estado atual |
| Contagem de FA, HA e AND do circuito montado | Próxima jogada sugerida |
| Área e atraso do circuito montado (D-3) | Solução automática (Dadda) |
| Recusa de seleção inválida, com a regra que ela viola | Limite inferior de FA |

- **GER-01** — O sistema DEVE exibir $W$, $v_{max}$, dica, próxima jogada, solução
  automática ou limite inferior somente depois de um pedido explícito do usuário seguido de
  confirmação. Responder ao desafio da largura (EXP-09) também revela $W$.
  *Prova:* teste de ponta a ponta `ajuda-nao-aparece-sem-pedido` (detalhado em `AJU-02`).
- **GER-02** — SE o usuário fizer uma seleção inválida, ENTÃO o sistema DEVE recusá-la
  citando a regra violada, sem indicar qual seria a seleção correta.
  *Prova:* teste do núcleo (cada recusa devolve o código da regra e nenhum identificador de
  ponto além dos selecionados); teste de componente da mensagem.

### Revelação em duas etapas

Quando uma resposta tem um resultado e um caminho até ele, o app revela **primeiro o
resultado** e, num segundo pedido, **o caminho**. O resultado dá ao aluno um alvo para
conferir o próprio raciocínio e tentar de novo; o caminho fica para quando ele ainda não
chegou lá.

| Resposta | Etapa 1: resultado | Etapa 2: caminho |
|---|---|---|
| Largura da saída | $W$ | $v_{max}$ com as entradas no máximo e $W = \lceil \log_2(v_{max}+1) \rceil$, com a substituição |
| Limite inferior | O número | A fórmula (pontos − $W$) e o porquê (cada FA elimina 1 ponto, HA nenhum) |
| Solução automática | Métricas finais: níveis, FA, HA, área, atraso | A solução nível a nível |

- **GER-03** — ONDE uma resposta constar da tabela acima, o sistema DEVE revelá-la em duas
  etapas, cada uma com pedido e confirmação próprios, na ordem resultado → caminho.
  *Prova:* teste de ponta a ponta para cada linha da tabela: após a etapa 1, o caminho não
  está renderizado; após a etapa 2, está.

## Glossário (modelo de domínio)

Os mesmos nomes valem na interface, no núcleo, nos testes e no VHDL gerado.

| Termo | Definição |
|---|---|
| **Ponto** | Um bit numa coluna. Tem identidade própria, coluna $j$ (peso $2^j$), **sinal** que carrega, origem e instante de prontidão $t$. Dois pontos podem carregar o mesmo sinal (ver EXP-02). |
| **Sinal** | O fio lógico: um bit de entrada (`e0`), um produto parcial (`a1_b0`), uma saída de célula (`s3`, `k3`) ou um bit do resultado (`R(j)`). |
| **Coluna** | Conjunto de pontos de mesmo peso. Coluna $j$ ↔ $2^j$. |
| **Matriz de pontos** | Os pontos de um nível, organizados por coluna. |
| **Nível** | Uma etapa da compressão. O nível 0 é a matriz inicial; o nível $n+1$ contém os pontos não usados no nível $n$ mais as saídas das células do nível $n$. O CPA é o último nível. |
| **Produto parcial** | Sinal de entrada formado pelo AND dos bits de um monômio com 2 ou mais variáveis. Cada produto parcial distinto é **uma** porta AND de $n$ entradas. |
| **FA** | Somador completo: consome 3 pontos da coluna $j$, produz soma em $j$ e vai-um em $j+1$. Elimina 1 ponto no total. |
| **HA** | Meio-somador: consome 2 pontos da coluna $j$, produz soma em $j$ e vai-um em $j+1$. Não elimina pontos; desloca um para a coluna seguinte. |
| **Vai-um descartado** | Vai-um que cairia numa coluna $\ge W$. Vale sempre 0 (CMP-09) e não vira ponto; elimina 1 ponto. |
| **CPA** | Somador de propagação de vai-um (*ripple*) que fecha a compressão. Aceita colunas com no máximo 2 pontos, exceto a primeira coluna do CPA, que pode ter 3 (o terceiro faz o papel de *carry-in*). É formado por células FA e HA encadeadas (CMP-11). |
| **Largura da saída $W$** | $W = \lceil \log_2(v_{max}+1) \rceil$, com $v_{max}$ o valor da expressão com todas as entradas no máximo. |
| **Circuito completo** | Todas as colunas com no máximo 1 ponto. É o que encerra o exercício. |

## Convenção de custo

| Célula | Área | Atraso |
|---|---|---|
| FA | $1\,A_{FA}$ | $1\,T_{FA}$ |
| HA | $0{,}5\,A_{FA}$ | $0{,}5\,T_{FA}$ |
| AND (produto parcial) | contada à parte, fora da área em $A_{FA}$ | $0$ |

A prontidão de cada saída é a maior prontidão entre as entradas mais o atraso da célula.
Todo ponto de entrada nasce em $t = 0$. O atraso do circuito é a maior prontidão entre os
bits do resultado. A convenção é fixa na v0.1 (D-2).

- **GER-04** — O sistema DEVE calcular área e atraso pela convenção acima.
  *Prova:* casos de referência R2, R3 e R4 no núcleo.

## Requisitos não-funcionais

- **GER-05** — O sistema DEVE executar todo o processamento no navegador, sem servidor de
  aplicação e sem enviar a expressão ou o circuito para fora do dispositivo.
  *Prova:* o artefato de produção é um site estático; teste de ponta a ponta que bloqueia a
  rede após o carregamento e completa R2.
- **GER-06** — QUANDO o usuário formar ou desfizer uma célula numa matriz de até 256 pontos,
  o sistema DEVE atualizar matriz e métricas em menos de 100 ms (percentil 95).
  *Prova:* teste de ponta a ponta no Chromium do CI, com 50 operações sobre a matriz de
  16 × 16 bits.
- **GER-07** — O sistema DEVE permitir selecionar pontos e formar células pelo teclado, e a
  visão de variáveis indexadas (VIS-01) DEVE distinguir os termos sem depender de cor.
  *Prova:* teste de ponta a ponta só com teclado completando R2; auditoria automática de
  acessibilidade (axe) sem violação grave ou crítica.
- **GER-08** — O sistema NÃO DEVE coletar telemetria nem exigir conta.
  *Prova:* teste de ponta a ponta que registra as requisições e confere que todas vão para a
  própria origem do site.
- **GER-09** — SE o módulo WebAssembly não carregar, ENTÃO o sistema DEVE mostrar uma tela de
  erro com o motivo e um link para abrir *issue*, em vez de uma interface quebrada.
  *Prova:* teste de ponta a ponta com o arquivo `.wasm` bloqueado.
- **GER-10** — ENQUANTO a largura da janela for menor que 768 px, o sistema DEVE manter
  matriz, diagrama e VHDL visíveis em modo leitura e desabilitar a formação de células, com
  um aviso (D-4).
  *Prova:* teste de ponta a ponta com janela de 390 px.

## Casos de referência

Exemplos **próprios do projeto**, conferidos por simulação exaustiva antes do código. Viram
testes do núcleo e não podem regredir.

**R1 — soma de produtos com coeficiente 3.**
$R = A \times B + C \times D + 3(E+F) + G$, com $A$ e $B$ de 3 bits, $C$, $D$ e $E$ de
2 bits, $F$ e $G$ de 1 bit.

| Grandeza | Valor |
|---|---|
| Monômios após expandir e decompor | $AB$, $CD$, $E$, $2E$, $F$, $2F$, $G$ |
| Pontos na matriz inicial | 20 (9 de $AB$ + 4 de $CD$ + 4 de $E$ + 2 de $F$ + 1 de $G$) |
| Altura das colunas $2^0..2^6$ | 5, 7, 5, 2, 1, 0, 0 |
| Portas AND | 13 |
| $v_{max}$ e $W$ | 71 e 7 |
| Limite inferior de FA | $20 - 7 = 13$ |
| Combinações de entrada | $2^{14} = 16\,384$ |

**R2 — uma solução de R1** (o teste a reproduz célula a célula; a tabela de ligações
completa está em `05-diagrama.md`):

| Células formadas sobre o nível | Células |
|---|---|
| 0 | FA1 e FA2 em $2^1$; HA3 em $2^2$ |
| 1 | FA4 em $2^1$; FA5 e FA6 em $2^2$; FA7 em $2^3$ |
| 2 | FA8 em $2^0$; FA9 em $2^2$; FA10 em $2^3$; HA11 em $2^4$ |
| 3 (CPA) | de $2^0$ a $2^5$: FA12, FA13, HA14, FA15, FA16, HA17 (a coluna $2^0$ entra com 3 pontos) |

Resultado: **13 FA, 4 HA, 13 AND, área 15 $A_{FA}$, atraso 6 $T_{FA}$, 16 384 casos
corretos.** Existe outra solução com 13 FA e 3 HA (14,5 $A_{FA}$) e atraso 6,5 $T_{FA}$: o
exemplo tem um compromisso real entre área e atraso.

**R3 — trivial.** $R = A$ com $A$ de 4 bits: 4 pontos, um por coluna, nenhuma célula,
$W = 4$, atraso 0.

**R4 — multiplicador 4×4.** $R = A \times B$, 4 bits cada: 16 pontos, alturas
1, 2, 3, 4, 3, 2, 1, 16 AND, $W = 8$, limite inferior de 8 FA.

## Fora de escopo da v0.1 (vale para todas as specs)

- Números com sinal (complemento de dois, Baugh-Wooley) e subtração.
- Constantes aditivas na expressão (ex.: `+ 5`).
- Blocos AMM como célula composta; compressores 4:2; CPA que não seja *ripple*.
- Solução automática de Wallace (v0.2); exportação em CSV (v0.2).
- Contas de usuário, servidor, compartilhamento por link (D-6).
- Síntese para FPGA dentro do app.
- Medir se o app de fato melhora o aprendizado (estudo com alunos): desejável, fora do
  software.
