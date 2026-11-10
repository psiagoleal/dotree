<!-- Caminho relativo: docs/specs/10-placa.md -->

# 10 — Placa DE2 e projeto do Quartus

## Resultado

Com o circuito completo e até 18 bits de entrada, o estudante leva o circuito para a placa
DE2 sem escrever nada: as variáveis vêm das chaves `SW`, o resultado aparece nos
LEDs vermelhos `LEDR` e em hexadecimal nos displays `HEX0` a `HEX7`. Junto vem o projeto do
Quartus II com o dispositivo e as atribuições de pinos. As portas têm os nomes que o
FPGAEmuWeb liga sem mapeamento manual.

## Arquivos gerados

Além dos arquivos de `06-vhdl.md`, quando o circuito cabe na placa (PLA-01):

| Arquivo | Conteúdo |
|---|---|
| `<nome>_de2.vhd` | Entidade `<nome>_de2`, com as portas da placa; instancia o topo estrutural `<nome>` |
| `tb_<nome>_de2.vhd` | Testbench que percorre todas as combinações das chaves usadas e confere `LEDR` e `HEX` |
| `<nome>.qpf` | Projeto do Quartus II |
| `<nome>.qsf` | Família, dispositivo, entidade de topo, arquivos VHDL e atribuições de pinos |

## Critérios

- **PLA-01** — QUANDO o circuito estiver completo, tiver no máximo 18 bits de entrada e
  $W \le 32$, o sistema DEVE gerar os quatro arquivos acima. SE passar desses limites,
  ENTÃO o sistema NÃO DEVE gerá-los e DEVE explicar o motivo, com o número de bits.
  *Prova:* teste do núcleo (R2 gera; 19 bits de entrada e $W = 33$ não geram, cada um com
  seu motivo) e teste de componente (o aviso aparece).
- **PLA-02** — O topo da placa DEVE ter as portas `SW : in std_logic_vector(17 downto 0)`,
  `LEDR : out std_logic_vector(17 downto 0)` e `HEX0` a `HEX7 : out
  std_logic_vector(6 downto 0)`, os nomes do manual da DE2 e do FPGAEmuWeb. As variáveis
  DEVEM ocupar as chaves na mesma ordem dos vetores do testbench: a primeira variável
  começa em `SW(0)`. O cabeçalho DEVE dizer, em comentário, que chaves formam cada variável.
  *Prova:* teste do núcleo (portas e `A <= SW(2 downto 0)` em R2) e teste de que as
  chaves de cada variável seguem os deslocamentos do testbench.
- **PLA-03** — `LEDR(j)` DEVE mostrar `R(j)` para $j < \min(W, 18)$; os demais LEDs
  DEVEM ficar apagados.
  *Prova:* teste do núcleo e o testbench da placa no GHDL.
- **PLA-04** — Os displays DEVEM mostrar $R$ em hexadecimal, `HEX0` com o dígito menos
  significativo, usando $\lceil W/4 \rceil$ displays; os demais DEVEM ficar apagados. Os
  segmentos são ativos em nível baixo, e o bit $i$ de `HEXn` é o segmento $a + i$
  ($a$ a $g$, como no manual).
  *Prova:* teste do núcleo que compara a tabela de segmentos gerada com os segmentos
  acesos de cada dígito, descritos à parte; o testbench da placa no GHDL.
- **PLA-05** — O `.qsf` DEVE declarar a família Cyclone II, o dispositivo EP2C35F672C6, a
  entidade de topo `<nome>_de2`, VHDL-93 e todos os arquivos VHDL do circuito, menos os
  testbenches, e atribuir as 92 portas da placa (`SW`, `LEDR`, `HEX0` a `HEX7`) aos pinos
  da DE2.
  *Prova:* teste do núcleo (92 atribuições distintas, `SW[0]` em `PIN_N25`, `LEDR[0]` em
  `PIN_AE23`); a tabela inteira conferida contra o arquivo de pinos da placa (ver
  "Pinos"); compilação no Quartus II 13.0 registrada abaixo.
- **PLA-06** — O testbench da placa DEVE percorrer todas as combinações dos bits de
  entrada nas chaves e conferir `LEDR` e cada display contra a expressão, emitindo
  "PLACA: TODOS OS N CASOS CORRETOS" ou os erros, com `severity failure`.
  *Prova:* em CI, o GHDL simula o testbench da placa de R2.
- **PLA-07** — Os nomes `<nome>_de2` e `tb_<nome>_de2` e os nomes internos do topo da placa
  (`dut`, `seg7`, as portas da placa) NÃO DEVEM colidir com o nome escolhido: um nome que
  coincida com eles DEVE ser recusado como em VHD-08.
  *Prova:* teste do núcleo.

## Pinos

A tabela de pinos (`app/core/src/board/mod.rs`) segue o *DE2 Development and Education
Board User Manual* (Terasic), tabelas das chaves, dos LEDs e dos displays de sete
segmentos, que é a mesma informação do arquivo `DE2_pin_assignments` distribuído com a
placa. Em 2026-11-09 (MT-22), as 92 atribuições foram conferidas uma a uma contra duas
cópias desse arquivo, sem divergência. Para
repetir a conferência, há um teste que lê um arquivo de pinos indicado em
`DE2_PINS_FILE` e o compara com a tabela; sem a variável, o teste é pulado. O arquivo de
pinos não é copiado para o repositório.

## Compatibilidade

Em 2026-11-09 (MT-22), conferido pelo agente nesta máquina:

- **Quartus II 13.0 SP0 Web Edition:** o projeto gerado do R2 compilou do início ao fim
  (análise, fitter, assembler com o `.sof` e análise de tempo), sem erros. O relatório de
  pinos do fitter traz as 92 portas nos pinos da tabela. Os avisos são os esperados: sem
  licença para compilação paralela, projeto sem relógio, LEDs e displays sem uso presos em
  nível fixo e chaves sem uso. O topo ocupou 53 elementos lógicos; trocando o componente
  pela versão comportamental (como o cabeçalho do topo explica), compilou com 56. Nenhum dos
  dois usou multiplicador embutido. Os números incluem os decodificadores de 7 segmentos.
- **GHDL 6 (CI) e ModelSim ASE 10.1d:** o testbench da placa do R2 deu "PLACA: TODOS OS
  16384 CASOS CORRETOS"; um caso com $W = 21$ (LEDs truncados, último display com 1 bit)
  deu todos os 1024 corretos; o topo estragado de propósito (o dígito 1 desenhado como 7)
  acusou 6540 casos errados. O ModelSim compilou sem avisos.

Ainda não conferido: abrir o topo da placa no FPGAEmuWeb e gravar o `.sof` numa DE2 real.

## Fora de escopo

Mais de 18 bits de entrada (painel de operandos em palavras, MT-23); outras placas; `KEY` e
relógio; programação da placa.
