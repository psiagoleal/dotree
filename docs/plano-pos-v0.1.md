<!-- Caminho relativo: docs/plano-pos-v0.1.md -->

# Plano depois da v0.1 — escala, placa e capítulos 4 a 12 do Parhami

> **Plano, não spec.** Registra pedidos levantados ao usar o dotree com multiplicadores
> grandes (64 × 64 numa placa DE2), com a avaliação de cada um e uma ordem
> proposta. Nada aqui muda o escopo da v0.1 sem decisão do autor; cada ticket só começa
> depois que a spec correspondente for atualizada e aprovada (`spec-como-contrato`).
> Avaliação feita em 2026-11-05 sobre o código do commit `e0bad41`. Os pedidos P9 a P21,
> sobre os capítulos 4 a 12 do Parhami, foram avaliados em 2026-11-06 sobre o commit
> `9f07dd2` (ver [Capítulos 4 a 12](#capítulos-4-a-12-do-parhami)).

## Decisões do autor (2026-11-05)

- Ordem proposta aprovada. Todos os pedidos entram no plano, inclusive os grandes.
- **MT-19** entra junto com o MT-16, antes da v0.1.0.
- **MT-20** (aplicar a solução): implementar. A adequação didática é reavaliada depois do
  uso; a spec 03 passa a permitir a substituição do circuito pela solução, contada como
  ajuda.
- **MT-22** (placa): seguir as portas que o FPGAEmuWeb já mapeia por padrão, em vez de uma
  tabela de pinos própria. Conferido no código público do emulador (lido para entender o
  comportamento; o repositório não declara licença, então nada dele é copiado):
  - a entidade de topo é lida até `end entity;` ou `end <nome>;`;
  - cada `component X is` do topo puxa o arquivo `X.vhd`;
  - portas com estes nomes ligam-se sozinhas à placa emulada: `CLOCK_50`, `CLK_500Hz`,
    `CLK_1Hz`, `CLK_10Hz`, `KEY` (4 bits), `SW` (18), `LEDR` (18), `HEX0` a `HEX7` (7 cada,
    ativos em 0);
  - os pinos reais da DE2 para o `.qsf` vêm do manual da placa, conferidos contra o
    arquivo de pinos distribuído com ela (que fica fora do repositório).
- **MT-27** (blocos hierárquicos): escrever a ADR antes do código.

## Decisões do autor (2026-11-06)

- **v0.2:** MT-20, MT-21, MT-22, MT-28, MT-29, MT-30 e MT-38 (versão mínima); MT-31 se
  der tempo. Entregável: *tag* `v0.2.0` e demonstração no FPGAEmuWeb. O relatório da
  síntese no Quartus (estrutural × comportamental) fica fora deste repositório.
- **Constante aditiva:** entra como ponto fixo em 1, comprimido como qualquer outro; o app
  não simplifica a célula que o recebe (fica como melhoria futura).
- **Depois da v0.2:** ordem por dependências, com o módulo $2^n-1$ (P16) adiantado, porque
  os sistemas de resíduos (cap. 4) vêm antes da multiplicação. Marcos: $2^n-1$ primeiro;
  depois a célula composta (P13); por fim números com sinal e Booth (P14, P15), os
  capítulos 9 e 10.
- **Modelo de custo** (unitário, em $A_{FA}$ e $T_{FA}$): FA = 1 $A_{FA}$ e 1 $T_{FA}$; HA =
  0,5 e 0,5; MUX $2^a{:}1$ = $a/2$ $A_{FA}$ e $a/2$ $T_{FA}$; portas AND contadas à parte. O
  dotree já usa o de FA e HA.
- **Multiplicação por constantes (2026-11-08):** as funções do gerador do SPIRAL (P21) entram
  reimplementadas em Rust **a partir dos artigos**, sem portar nem ler o código deles, que é
  GPL; o dotree continua MIT. O gerador online serve só como referência para os testes.
- **Comparação de soluções (P20):** aprovada.

## Os pedidos, avaliados

| # | Pedido | Situação hoje | Esforço | Specs afetadas |
|---|---|---|---|---|
| P1 | Larguras maiores: até 64 bits por variável, 128 bits de entrada, 4096 pontos; matriz navegável (zoom, rolagem); verificação por amostra | Limites de 16 bits, 32 bits e 256 pontos (`expr::MAX_WIDTH`, `MAX_INPUT_BITS`, `matrix::MAX_DOTS`). A amostra já existe acima de 24 bits. As entradas são `u64` e o valor esperado `u128`, então 64 bits por variável estoura em somas de produtos (A·B + C·D tem 129 bits) | **Grande** | 00, 01, 04, 06, 09 (desempenho) |
| P2 | Fatiar variáveis: `A[63:32]` e `A[31:0]` em vez de inventar variáveis, mantendo as portas `A` e `B` inteiras | A gramática só aceita uma letra por variável; sinais e portas saem de uma variável inteira | **Médio-grande** | 01, 05, 06 |
| P3 | Blocos hierárquicos: um subproduto ($A_H B_H$) como multiplicador instanciado, para o dividir-em-4 aparecer como 4 multiplicadores e a soma final | Fora da v0.1 ("blocos AMM como célula composta", spec 00). Netlist, diagrama e VHDL só conhecem FA, HA e AND | **Grande** | 00, 02, 05, 06 |
| P4 | VHDL comportamental (`*` e `+` de `numeric_std`) além do estrutural, para comparar recursos e caminho crítico | Fora da v0.1 (spec 06). O gerador já tem nome, larguras e testbench | **Pequeno-médio** | 06 |
| P5 | "Usar esta solução": formar as células de Dadda no exercício | O núcleo já calcula a redução de Dadda inteira (`help::dadda_solution`); a interface só mostra métricas e níveis. A spec 03 exclui a "correção automática do circuito do usuário" | **Pequeno** no código; **decisão didática** | 03 |
| P6 | Topo para a DE2 (SW, KEY, LEDR, HEX); acima de 18 bits de entrada, um painel que carrega os operandos em palavras pelas KEY e mostra o produto em fatias nos displays | Fora da v0.1 ("atribuição de pinos de placa", spec 06). Até 18 bits é só ligação; acima disso é uma máquina de estados com testbench próprio | **Médio** (≤ 18 bits) a **grande** (painel) | 06, nova spec de placa |
| P7 | Compatibilidade com o FPGAEmuWeb: `end <nome>;` em vez de `end entity <nome>;`, um arquivo por componente | Um arquivo por componente **já existe** (`full_adder.vhd`, `half_adder.vhd`, topo, testbench). Falta só a forma curta do `end`, que todo VHDL aceita e o GHDL do CI confere | **Pequeno** | 06 |
| P8 | Projeto do Quartus: `.qpf`/`.qsf` com a DE2 (EP2C35F672C6) e as atribuições de pinos | Fora da v0.1 (spec 06). Sem pinos é texto fixo; com pinos depende de P6 e de uma tabela de pinos conferida no manual da placa | **Pequeno** (sem pinos) a **médio** | 06, nova spec de placa |

### Dependências entre os pedidos

- **P1 precisa de P5 para ser usável.** Comprimir 4096 pontos à mão não é exercício; com
  matriz grande, o caminho é aplicar a solução e estudar o circuito.
- **P3 precisa de P2.** Os subprodutos são produtos de fatias ($A_H = A[63:32]$).
- **P8 precisa de P6** para as atribuições de pinos.
- P4, P5 e P7 não dependem de nada além da v0.1.

## Ordem proposta

```mermaid
flowchart LR
  v01[v0.1.0] --> MT19[MT-19 VHDL compatível com o FPGAEmuWeb]
  v01 --> MT20[MT-20 aplicar a solução de Dadda]
  v01 --> MT21[MT-21 VHDL comportamental]
  v01 --> MT22[MT-22 topo DE2 até 18 bits + projeto Quartus]
  MT22 --> MT23[MT-23 painel de operandos em palavras]
  MT20 --> MT24[MT-24 aritmética larga no núcleo]
  MT24 --> MT25[MT-25 matriz navegável e limites de 64 bits]
  v01 --> MT26[MT-26 fatias de variáveis]
  MT26 --> MT27[MT-27 blocos hierárquicos]
  MT25 --> MT27
```

- **Fácil e de valor imediato (v0.2):** MT-19, MT-20, MT-21. Se o autor quiser, o MT-19
  cabe antes da v0.1.0: é só a forma do `end`, conferida pelo GHDL.
- **Placa (v0.2 ou v0.3):** MT-22 e MT-23.
- **Escala (v0.3):** MT-24 a MT-27. Exigem revisão das specs 00, 01, 05 e
  06 e provavelmente uma ADR (aritmética larga no núcleo e modelo de bloco hierárquico).

## Tickets propostos

### MT-19: VHDL compatível com o FPGAEmuWeb (P7)
- **Objetivo:** fechar entidade, arquitetura e componente na forma curta (`end <nome>;`,
  `end <arquitetura>;`, `end component;`), mantendo um arquivo por componente.
- **Arquivos no escopo:** `app/core/src/vhdl/**`, `docs/specs/06-vhdl.md`
- **Critério de aceite:** testes do núcleo sem `end entity` nem `end architecture`; job
  GHDL do CI verde; o autor confere um `.zip` no FPGAEmuWeb.
- **Decidido:** o emulador só lê o fechamento da entidade (`end entity;` ou `end <nome>;`);
  basta `end <nome>;` nas entidades. **Feito (2026-11-05)**, junto com o MT-16: GHDL com
  os mesmos vereditos do CI, e a análise de portas do emulador leu as 8 portas do R2.

### MT-20: Aplicar a solução de Dadda ao exercício (P5)
- **Objetivo:** botão "usar esta solução" depois da etapa 2 da solução, com confirmação;
  substitui o circuito do aluno pela solução, desfazível num passo, e fica no registro de
  ajudas.
- **Arquivos no escopo:** `app/wasm/src/**`, `app/web/src/**`, `docs/specs/03-ajuda-sob-demanda.md`
- **Critério de aceite:** e2e (R1 → solução aplicada → verificação "0 erros" e VHDL
  disponível; desfazer devolve o circuito do aluno).
- **Decidido:** implementar; aplicar não corrige, substitui, e conta como revelação do
  caminho. A spec 03 é ajustada no ticket.
- **Feito (2026-11-06):** AJU-09 na spec 03 e nota na spec 08. `Reduction::adopt` guarda o
  estado final da solução como um passo do histórico (desfazer devolve o circuito do aluno;
  recusa outra matriz e o circuito já igual); `Session.applySolution` exige a etapa 2 e
  conta `solutionApplied`; botão "Usar esta solução" com confirmação, fora do modo leitura.

### MT-21: VHDL comportamental (P4)
- **Objetivo:** gerar `<nome>_comportamental.vhd` com a mesma interface, usando `unsigned`,
  `*` e `+` de `numeric_std` e `resize` para W bits; o mesmo testbench serve aos dois.
- **Arquivos no escopo:** `app/core/src/vhdl/**`, `app/web/src/**`, `docs/specs/06-vhdl.md`
- **Critério de aceite:** testes do núcleo; job GHDL roda o testbench contra as duas
  arquiteturas (R2: 16 384 casos corretos em ambas).
- **Feito (2026-11-09):** VHD-10 na spec 06. Entidade própria `<nome>_comportamental` (as
  mesmas portas, um arquivo por entidade, como pede o VHD-09), para o Quartus escolher o
  topo; o testbench instancia as duas e relata cada uma. Coeficiente acima de $2^{31}-1$
  vai como cadeia de bits. GHDL e ModelSim: R2 correto nas duas; o comportamental
  estragado de propósito falha só nele.

### MT-22: Topo para a DE2 até 18 bits e projeto do Quartus (P6, P8)
- **Objetivo:** com até 18 bits de entrada, gerar um topo que liga as entradas às chaves
  `SW`, o resultado a `LEDR` e `HEX` (hexadecimal); exportar `.qpf`/`.qsf` para a
  EP2C35F672C6 com as atribuições de pinos.
- **Arquivos no escopo:** `app/core/src/vhdl/**` (ou módulo novo `board`), `app/web/src/**`,
  nova spec de placa
- **Critério de aceite:** testes do núcleo; tabela de pinos conferida contra o manual da
  DE2, com referência à fonte e sem copiar o documento; GHDL compila o topo.
- **Decidido:** portas com os nomes padrão do FPGAEmuWeb (`SW`, `KEY`, `LEDR`, `HEX0`–`HEX7`),
  que ele liga sem mapeamento manual. Spec nova, `10-placa.md`.
- **Feito (2026-11-09):** spec 10 (PLA-01 a PLA-07). Módulo `board` (limites, pinos,
  segmentos); topo `<nome>_de2`, `tb_<nome>_de2`, `.qpf` e `.qsf` quando há até 18 bits de
  entrada e $W \le 32$. Pinos conferidos contra duas cópias do arquivo de pinos da DE2;
  Quartus II 13.0 compilou o projeto do R2 com os pinos certos; GHDL e ModelSim conferiram o
  testbench da placa e pegaram o topo estragado de propósito.

### MT-23: Painel de operandos em palavras (P6)
- **Objetivo:** acima de 18 bits de entrada, uma máquina de estados que carrega cada
  operando em palavras pelas `KEY` e mostra o resultado em fatias nos displays.
- **Critério de aceite:** testbench próprio do painel no job GHDL.
- **Depende de:** MT-22

### MT-24: Aritmética larga no núcleo (P1, parte 1)
- **Objetivo:** entradas e valores acima de 64 e 128 bits (inteiro largo próprio ou crate),
  sem perder a simulação bit-paralela.
- **Decisão pendente:** ADR sobre o tipo de inteiro largo.
- **Depende de:** MT-20 (sem aplicar a solução, a escala não é praticável)

### MT-25: Limites de 64 bits e matriz navegável (P1, parte 2)
- **Objetivo:** 64 bits por variável, 128 de entrada, 4096 pontos; matriz com zoom e
  rolagem; desempenho medido com 4096 pontos.
- **Depende de:** MT-24

### MT-26: Fatias de variáveis (P2)
- **Objetivo:** `A[63:32]` na expressão, com a largura da variável inteira declarada uma
  vez; sinais e portas mantêm `A` inteira.
- **Decisão pendente:** sintaxe (`A[63:32]` ou `A_H` declarado como fatia) e o que a
  matriz mostra.

### MT-27: Blocos hierárquicos (P3)
- **Objetivo:** subprodutos como blocos multiplicadores instanciados (netlist, diagrama,
  VHDL), cada um com o próprio exercício ou a solução automática.
- **Decisão pendente:** modelo do bloco e se o aluno comprime cada bloco ou só a soma
  final. Precisa de ADR.
- **Depende de:** MT-25, MT-26

## Capítulos 4 a 12 do Parhami

Referência: B. Parhami, *Computer Arithmetic*, 2.ª ed., caps. 4 a 12. Os exemplos e casos
de referência do plano são do próprio projeto.

### O que a v0.1 já cobre

- **Contadores paralelos generalizados (cap. 8), quase todos:** um contador é uma matriz
  de bits independentes ({2,2,3; …} = `A+B+C+2D+2E+4F+4G`, larguras 1). Falta parar num
  perfil de saída (P12).
- **Somas de produtos sem constante aditiva**, **multiplicação por constante em parte**
  (constante expandida em pontos, até 32 bits de entrada) e o **AMM** (cap. 12) como soma
  de produtos.

### Os pedidos

| # | Pedido (capítulo) | Situação hoje | Esforço | Specs |
|---|---|---|---|---|
| P9 | Constante aditiva: `+36`, `+8`, `+1164` (caps. 5 e 8) | O parser já a representa como monômio sem variáveis e a recusa de propósito (`ConstantTerm`, EXP-13); um sinal sem fatores já vale 1 | **Pequeno-médio** | 00, 01, 05, 06 |
| P10 | Solução de Wallace ao lado da de Dadda (cap. 8) | Só Dadda (`help::dadda_solution`) | **Pequeno-médio** | 03 |
| P11 | Tabela de alturas nível a nível (cap. 8) | As alturas aparecem por painel, não numa tabela | **Pequeno** | 02 |
| P12 | Entrada por alturas com perfil-alvo de saída: `{2,2,3; 1,1,1,1}`, `{3,3,3; 1,2,2,1}` (cap. 8) | A redução sempre para em 2 linhas + CPA. Na notação, depois do ";" vêm as alturas de saída por coluna | **Pequeno** | 01, 02 |
| P13 | Célula composta: contador 7:3, (5,5;4), compressor 4:2, AMM 2×2 (caps. 8 e 12) e multiplicador hierárquico (P3) | A célula é só `CellKind::{Fa, Ha}`, com duas saídas; um 7:3 tem 3 saídas em 3 colunas | **Grande** | 00, 02, 05, 06, 08 |
| P14 | Ponto invertido: Baugh-Wooley e recodificação com dígito −1 (cap. 9) | Sem sinal e sem NOT | **Médio-grande** | 00, 01, 04, 06 |
| P15 | Booth radix-2 e radix-4, com mux e múltiplos (caps. 9 e 10) | Sem sinal selecionado por mux | **Grande** | 00, 01, 02, 06 |
| P16 | Matriz módulo $2^n-1$ e depois $2^n+1$ (caps. 4 e 8) | Sem volta de colunas. A árvore com volta não fecha laço; o CPA modular segue o padrão de dois somadores e um MUX 2:1, sem laço. $2^n+1$ pede P14 | **Médio** ($2^n-1$) a **grande** | 01, 02, 04, 06 |
| P17 | Escolha do CPA final: CLA, prefixo, *carry-select* (caps. 6 e 7) | Só *ripple* | **Médio-grande** | 00, 02, 05, 06 |
| P18 | Variável derivada: múltiplos pré-computados ($7A$) e constante grande (cap. 9) | Literal até $2^{32}$ (EXP-14), 256 pontos | **Grande** | 01, 02 |
| P19 | Animação de *shift-add*, à esquerda e à direita (cap. 9) | Fora do modelo combinacional | **Médio** | nova |
| P20 | Comparação visual de soluções do mesmo problema | Métricas de uma solução por vez | **Pequeno-médio** (mínima) | 02, 03 |
| P21 | Bloco multiplicador por constantes (MCM), como o gerador do SPIRAL: Hcub com limite de profundidade (cap. 9) | Constante só expandida em pontos; sem subtração nem múltiplo pré-computado | **Médio** | 01, 02, 06, nova |

### Dependências

- P12 fecha os contadores generalizados **sem** célula composta; P13 só é preciso para
  **usar** esses blocos como células.
- P13 serve ao P3 (MT-27), ao P15 e ao P17.
- P14 precisa de P9 (a correção de sinal é constante); P15 precisa de P14 e de P13.
- P16 com $2^n+1$ precisa de P14 (volta invertida).
- P18 precisa de P1 (MT-24) e de P3 (MT-27).
- P20 fica completo com P10 (Wallace).
- P21 precisa de P14 (o Hcub subtrai) e de P18 (os múltiplos viram variáveis derivadas).

```mermaid
flowchart LR
  v01[v0.1.0] --> MT28[MT-28 constante]
  v01 --> MT30[MT-30 tabela de alturas]
  v01 --> MT31[MT-31 perfil-alvo]
  MT20[MT-20 aplicar solução] --> MT29[MT-29 Wallace]
  MT29 --> MT38[MT-38 comparação]
  MT28 --> MT33[MT-33 ponto invertido]
  v01 --> MT35[MT-35 módulo 2^n-1 e 2^n+1]
  MT33 --> MT35
  v01 --> MT32[MT-32 célula composta]
  MT32 --> MT27[MT-27 blocos hierárquicos]
  MT32 --> MT34[MT-34 Booth]
  MT33 --> MT34
  MT32 --> MT36[MT-36 escolha do CPA]
  MT27 --> MT37[MT-37 variável derivada]
  MT33 --> MT39[MT-39 multiplicação por constantes]
  MT37 --> MT39
```

## Tickets propostos (P9 a P21)

### MT-28: Constante aditiva (P9)
- **Objetivo:** aceitar termos constantes (`+36`, `3(A+1)`), combinados num só; seus bits em 1
  viram pontos fixos com cor própria, que o aluno comprime como os outros; VHDL com `'1'`.
- **Arquivos no escopo:** `app/core/src/{expr,matrix,netlist,vhdl,diagram}/**`,
  `app/web/src/**`, specs 00, 01, 05 e 06
- **Critério de aceite:** testes do núcleo; e2e com uma soma de produtos com constante;
  GHDL verde com a constante.
- **Feito (2026-11-09):** EXP-13 refeito (só recusa expressão sem variável) e EXP-15 novo.
  O sinal sem fatores, que a matriz já gerava como AND vazio, ganhou o nome `one`, `'1'` no
  VHDL e o rótulo 1 na tela. O e2e usa uma expressão própria, `3(A + 1) + B`; um caso
  maior (46 pontos, 2 deles fixos, $W = 14$) foi conferido localmente.

### MT-29: Solução de Wallace (P10)
- **Objetivo:** `help::wallace_solution` (em cada nível, o máximo de FA e depois HA), com o
  mesmo fluxo de revelação e aplicação do MT-20.
- **Critério de aceite:** testes do núcleo com R1 e R2 conferidos à mão; e2e de aplicar.
- **Depende de:** MT-20
- **Feito (2026-11-09):** AJU-10. A regra é a da fig. 8.10 do Parhami: alvo $2\lfloor h/3 \rfloor + (h \bmod 3)$, todos os FA que cabem e HA só se a
  coluna ainda passaria do alvo; reproduz a figura nível a nível (12, 6, 6, 4 + 1 HA).
  R1: 13 FA, 3 HA, 14,5 $A_{FA}$, 6 $T_{FA}$ (Dadda: 17 e 7). **Achado:** a fig. 8.14
  (Dadda, mesmos sete operandos) dá 6, 11, 7 e 4 FA + 1 HA, mas o Dadda do dotree dá
  5 FA + 1 HA, 10 + 1, 5 + 2, 6 + 1 (33 FA e 6 HA no total, contra 33 e 3): o livro usa FA
  também onde falta reduzir só 1, e AJU-05 manda usar HA. Mudar AJU-05 altera R1, R4 e a
  próxima jogada; fica para decisão (proposta: ticket próprio depois da v0.2).
  **Decidido (MT-29b, 2026-11-09):** manter o Dadda clássico. A figura não segue regra
  coluna a coluna (no 3.º nível põe FA em colunas já no alvo); a variante "FA sempre que
  passa do alvo" iguala os totais da figura (33 FA, 3 HA), mas não os níveis 2 e 3, e em R1
  dá 15,5 $A_{FA}$ e 7 $T_{FA}$, ainda atrás do Wallace (14,5 e 6). Spec 03 documenta a
  diferença; teste de regressão do Dadda clássico nos sete operandos.

### MT-30: Tabela de alturas nível a nível (P11)
- **Objetivo:** tabela com a altura de cada coluna em cada nível, exportável em Markdown.
- **Critério de aceite:** teste com as alturas do R2 (spec 05).
- **Feito (2026-11-09):** CMP-18 na spec 02 (não na 05: a tabela é da compressão), com a
  tabela de R2 calculada à mão e conferida pelo teste do núcleo e pelo arquivo baixado no
  teste de ponta a ponta. Forma tabular da fig. 8.11 do Parhami: linhas por nível, coluna
  mais alta à esquerda, FA e HA por nível. Só o circuito do aluno; as soluções de Dadda e
  Wallace ganham a tabela na comparação (MT-38).

### MT-31: Entrada por alturas com perfil-alvo de saída (P12)
- **Objetivo:** aceitar `{2,2,3; 1,1,1,1}` (alturas de entrada e de saída, do bit mais
  significativo ao menos), parar quando o perfil de saída for atingido e mostrar área e
  atraso do bloco; o nome do bloco conhecido aparece só como revelação opcional.
- **Critério de aceite:** a solução ótima reproduz as áreas e os atrasos de 11 perfis de
  referência do projeto, conferidos à mão e por uma busca exaustiva independente.
- **MT-31a, núcleo (feito em 2026-11-10):** EXP-16, EXP-17, CMP-19 e AJU-12. Módulo
  `dotree_core::profile`: leitura do perfil, matriz equivalente (uma variável de 1 bit por
  ponto), perfil atingido, circuito ótimo por aprofundamento iterativo na área (FA e HA
  sobre os pontos mais cedo prontos; poda por 1 meia-unidade de área por ponto excedente;
  limite de 1 milhão de níveis tentados) e classificação do bloco. Os 11 perfis batem; a
  leitura recusa os perfis que nenhum circuito atinge (coluna que nunca esvazia).
- **MT-31b, interface (feito em 2026-11-10):** o campo aceita a notação, sem larguras, com
  exemplo; painel do perfil (pedido, colunas acima, atingido com área e atraso); a solução
  ótima como terceiro método, no mesmo fluxo de Dadda e Wallace e na comparação; o nome
  do bloco como ajuda confirmada e contada; quatro contagens opcionais no arquivo
  (esquema 1).

### MT-32: Célula composta (P13)
- **Objetivo:** modelo de célula com entradas e saídas em várias colunas, custo e atraso
  vindos da rede interna; contadores, compressor 4:2 e AMM 2×2 como os primeiros tipos.
- **Decisão pendente:** ADR própria (modelo de célula composta), que vale também para o MT-27.

### MT-33: Ponto invertido e números com sinal (P14)
- **Objetivo:** pontos negados e a constante de correção; Baugh-Wooley e recodificação com
  dígito −1 para constantes.
- **Depende de:** MT-28

### MT-34: Booth radix-2 e radix-4 (P15)
- **Depende de:** MT-32, MT-33

### MT-35: Matriz módulo $2^n-1$ e $2^n+1$ (P16)
- **Objetivo:** pesos com volta direta ($2^n-1$) e invertida ($2^n+1$); CPA modular com dois
  somadores e MUX 2:1. Começa por $2^n-1$.
- **Depende de:** MT-33 (só a parte $2^n+1$)

### MT-36: Escolha do CPA final (P17)
- **Objetivo:** *ripple*, *carry-select* e prefixo, com o modelo de custo do MUX.
- **Depende de:** MT-32

### MT-37: Variável derivada e constante grande (P18)
- **Depende de:** MT-24, MT-27

### MT-38: Comparação de soluções (P20)
- **Objetivo:** guardar soluções do mesmo problema (aluno, Dadda, Wallace, versões salvas)
  e compará-las lado a lado: área, atraso, FA, HA, AND, níveis, largura do CPA, caminhos
  críticos, área × atraso e área × atraso²; gráfico área × atraso com a fronteira das
  soluções não dominadas. Depois: perfis de alturas sobrepostos e números da síntese no
  Quartus.
- **Critério de aceite:** e2e com R1 (aluno × Dadda × Wallace) e os números conferidos
  contra as métricas de cada solução.
- **Depende de:** MT-29
- **Feito, versão mínima (2026-11-09):** AJU-11. Tabela lado a lado (aluno, Dadda clássico,
  Wallace) com todas as medidas do objetivo, menos o gráfico, e a linha "não dominada";
  tabela de alturas de cada solução cujos níveis foram revelados. Nada é revelado nem
  contado pela comparação. Em R1, o Wallace (14,5 $A_{FA}$, 6 $T_{FA}$) domina R2 e o
  Dadda. Falta: gráfico com a fronteira, versões salvas, perfis sobrepostos, Quartus.

### MT-39: Multiplicação por constantes, MCM (P21)
- **Objetivo:** dado $Y = c_1 A$ (ou várias constantes sobre a mesma variável), calcular o
  grafo de somadores do Hcub, com limite de profundidade opcional, e usar os múltiplos
  como variáveis derivadas; o aluno compara, no MT-38, a constante expandida em pontos (CSD)
  com os múltiplos pré-computados mais a árvore.
- **Fonte:** Y. Voronenko e M. Püschel, "Multiplierless multiple constant multiplication",
  *ACM Transactions on Algorithms*, 2007. Implementação a partir do artigo, sem consultar o
  código GPL do SPIRAL; RAG-n fica de fora (precisa da tabela pré-calculada deles) e o BHM
  fica para depois.
- **Critério de aceite:** número de somadores e profundidade iguais aos do gerador online
  num conjunto de casos anotados (só os números, com data e opções usadas); grafo e VHDL
  conferidos pelo GHDL.
- **Depende de:** MT-33, MT-37

A animação de *shift-add* (P19) fica sem ticket até haver espaço depois da v0.4.

## Antes da v0.2.0 (decisão do autor, 2026-11-10)

### MT-40a: Documentação pública e referências
- **Objetivo:** README da v0.2 (o que o app faz, passo a passo, limitações declaradas,
  metodologia e referências); ADR e spec onde faltarem.
- **Decisões do autor:** a spec é **uma só e viva**, atualizada a cada versão (não uma
  spec por versão); o histórico de cada versão fica no git e no CHANGELOG. O projeto é
  público: só se citam livros e artigos de acesso público (Parhami; Wallace, 1964; Dadda,
  1965), **nunca** material didático não publicado, nem por referência (ADR 0004). Inclui
  a varredura das menções que já existem no repositório.
- **Critério de aceite:** nenhuma menção a material didático não publicado nos arquivos
  versionados do projeto; cada limitação declarada aponta para a spec ou o ticket que a
  trataria; README conferido contra o app publicado.

### MT-40b: Ajuda dentro do app
- **Objetivo:** seção "Como usar", em português e inglês, aberta pelo cabeçalho: os
  passos, os atalhos, o que cada ajuda revela e como é contada, as limitações e as
  referências.
- **Critério de aceite:** testes de componente, de acessibilidade (GER-07) e de ponta a
  ponta; os textos não revelam solução de nenhum exercício.
- **Depende de:** MT-40a
- **Feito (2026-11-10):** GER-11 na spec 00. `Guide.svelte` no cabeçalho, com o texto
  montado em `guide.ts` a partir do catálogo (cinco partes, testadas nos dois idiomas);
  diálogo modal que devolve o foco ao botão mesmo quando o clique não o focou; `keys.ts`
  ignora atalhos dentro de diálogo aberto. E2e no meio de um exercício (o registro de
  ajudas salvo não muda, V não troca a visão), em 390 px e em inglês; axe com o guia
  aberto nos dois temas. 5 mutações pegas (uma só depois do teste do clique sem foco).

### MT-43: Publicar a v0.2.0
- **Objetivo:** versões em 0.2.0 (crates, pacote web, badge do README), seção `[0.2.0]` do
  CHANGELOG, aviso "Em construção" da página revisto, conferência manual do VHDL no
  ModelSim ASE 10.1d (estrutural, comportamental e topo da placa), *tag* `v0.2.0` criada
  pelo autor.
- **Critério de aceite:** `app/core/tests/release.rs` e o e2e de ENT-03 com 0.2.0; release
  do GitHub e imagem `0.2.0` no GHCR (ENT-05); site no ar com a versão.
- **Depende de:** MT-40b
- **Preparado (2026-11-10):** versões em 0.2.0 (`Cargo.toml`, `package.json`, badge e
  imagem do README), seção `[0.2.0]` do CHANGELOG (as notas da release saem dela, conferido
  com o `awk` do `release.yml`); o aviso "Em construção" virou uma nota que não envelhece,
  com o link da spec, conferida por e2e nos dois idiomas. ModelSim ASE 10.1d: os 11 casos
  de `vhdl_referencia` compilaram sem avisos e deram o veredito esperado (8 corretos, 3
  estragados acusados).
- **Feito (2026-11-12):** *tag* `v0.2.0` pelo autor (#51); workflow Release verde, release
  com a seção do CHANGELOG, imagem `ghcr.io/psiagoleal/dotree:0.2.0` lida sem login
  (ENT-05), site no ar com "versão 0.2.0" e a nota nova.

## Limitações da entrada por alturas (achadas no MT-31b, 2026-11-10)

### MT-41: Dica e próxima jogada que respeitam o perfil de saída
- **Objetivo:** num exercício de perfil, a dica e a próxima jogada (AJU-03, AJU-04) param
  quando o perfil é atingido e sugerem células que levam a ele, em vez de seguir o Dadda
  até a soma completa.
- **Situação hoje:** as duas seguem o Dadda; num perfil com saída redundante, sugerem
  reduzir além do pedido. A solução ótima (AJU-12) já respeita o perfil.
- **Critério de aceite:** testes do núcleo com perfis de saída redundante; a sugestão nunca
  passa do perfil.

### MT-42: Busca do ótimo fora da linha principal
- **Objetivo:** rodar a busca do circuito ótimo (AJU-12) num *worker*, como a verificação
  (VER), com indicação de progresso, para a página não travar nos perfis grandes.
- **Situação hoje:** a busca roda na linha principal na primeira vez que a ajuda abre; no
  pior caso medido (26 pontos), leva cerca de 0,5 s fora do navegador.
- **Critério de aceite:** e2e que interage com a página durante a busca; GER-06 (100 ms no
  p95) medido com o maior perfil aceito.
