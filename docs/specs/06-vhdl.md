<!-- Caminho relativo: docs/specs/06-vhdl.md -->

# 06 — VHDL de modelo: estrutural e comportamental

## Resultado

Um ponto de partida em VHDL para o circuito montado: simples, legível, com um testbench que
prova o circuito. Não é ferramenta de síntese; é um modelo que o estudante adapta. Ao lado
do topo estrutural vem uma versão comportamental com as mesmas portas, para comparar no
sintetizador (Quartus) o circuito montado à mão com o que a ferramenta escolhe.

## Arquivos gerados

| Arquivo | Conteúdo |
|---|---|
| `full_adder.vhd` | Entidade `full_adder` (x, y, z → s, k) |
| `half_adder.vhd` | Entidade `half_adder` (x, y → s, k), só quando o circuito tem HA |
| `counter_7_3.vhd` | Entidade `counter_7_3` (x0…x6 → y0, y1, y2) com 4 `full_adder`, quando o circuito tem 7:3 ou 15:4 |
| `counter_15_4.vhd` | Entidade `counter_15_4` (x0…x14 → y0…y3) com 2 `counter_7_3` e 3 `full_adder`, quando o circuito tem 15:4 |
| `<nome>.vhd` | Topo estrutural: portas AND, uma instância por célula, com os identificadores de `05-diagrama.md` |
| `<nome>_comportamental.vhd` | Entidade `<nome>_comportamental`: as mesmas portas do topo e a expressão em `numeric_std`, sem células |
| `tb_<nome>.vhd` | Testbench que percorre os vetores de `04-verificacao.md` e compara as duas versões com a expressão |

`<nome>` é escolhido pelo usuário (padrão `dotree_circuito`) e precisa ser identificador VHDL
válido.

## Critérios

- **VHD-01** — QUANDO o circuito estiver completo, o sistema DEVE gerar os arquivos acima em
  VHDL-93, usando apenas `ieee.std_logic_1164`, `ieee.numeric_std` e `use work.all` (que
  torna `full_adder` e `half_adder` diretamente visíveis, como a ligação padrão de
  componentes do VHDL-93 exige). Os arquivos DEVEM ser ASCII puro: o português dos
  comentários vai sem acento.
  *Prova:* em CI (job `vhdl`), o GHDL 6 em `--std=93` analisa, elabora e simula R2, R3, a
  solução Dadda de R4, um caso de 21 bits, as variáveis `I`, `J`, `N`, `V` e `W`, as variáveis `S` e `K` e os
  contadores de VHD-11; teste do núcleo exige ASCII e o `use work.all`.
- **VHD-02** — O topo DEVE usar os mesmos identificadores de sinal e de instância do
  diagrama e da tabela de ligações; um bit de resultado constante é atribuído com
  `R(j) <= '0'`, e o ponto fixo da constante (EXP-15) com `one <= '1'`. A saída se chama `R`; uma variável de entrada chamada $R$ vira a porta
  `R_IN`.
  *Prova:* teste do núcleo (mesmo conjunto de identificadores de DIA-03) e `2A` com $A$ de
  2 bits.
- **VHD-03** — O topo DEVE trazer no cabeçalho, em comentário, a expressão, as larguras, a
  matriz de pontos em texto e as métricas (FA, HA, AND, área, atraso). Com custos
  diferentes do padrão (GER-12), uma linha a mais diz quais são.
  *Prova:* teste de unidade do gerador (cabeçalho de R2 fixado).
- **VHD-04** — O testbench DEVE calcular o valor esperado com aritmética `unsigned` de
  `numeric_std`, redimensionada para $W$ bits, e percorrer os vetores de `04-verificacao.md`
  (todas as combinações até 20 bits; acima disso, os 4096 vetores da amostra, embutidos como
  constante). DEVE conferir o topo estrutural e o comportamental (VHD-10) contra o mesmo
  valor esperado e, ao terminar, emitir para cada um um `report` com o total de casos
  corretos ou, havendo erro, os 10 primeiros vetores errados e o total de errados; com
  qualquer erro, a simulação DEVE terminar em `severity failure`.
  *Prova:* em CI, o GHDL simula o testbench de R2 e a saída contém as duas linhas
  "ESTRUTURAL: TODOS OS 16384 CASOS CORRETOS" e "COMPORTAMENTAL: …"; com o circuito
  corrompido de VER-03 (FA16 recebendo `k14`) e com o comportamental corrompido (a primeira
  soma trocada por subtração), a simulação acusa "CASOS ERRADOS" na versão errada e falha.
- **VHD-05** — O sistema DEVE permitir copiar cada arquivo e baixar todos num `.zip`.
  *Prova:* teste de ponta a ponta.
- **VHD-06** — Os comentários e as mensagens do testbench DEVEM seguir o idioma escolhido
  na interface; o código e os identificadores são iguais nos dois idiomas.
  *Prova:* teste do núcleo nos dois idiomas: tirando comentários e literais de texto, os
  arquivos são idênticos.
- **VHD-07** — ENQUANTO o circuito estiver incompleto, o sistema DEVE manter a geração de
  VHDL desabilitada, explicando o motivo.
  *Prova:* teste do núcleo (`NotComplete`) e teste de componente.
- **VHD-08** — SE o nome escolhido não for um identificador básico de VHDL-93 (letra
  inicial; letras, dígitos e `_` sem `__` nem `_` final), for palavra reservada ou coincidir,
  sem distinguir maiúsculas, com qualquer nome do projeto (portas, sinais, instâncias,
  componentes), ENTÃO o sistema DEVE recusá-lo.
  *Prova:* teste do núcleo com nomes válidos e inválidos.

- **VHD-09** — Cada entidade DEVE fechar com `end <nome>;` (sem `end entity`), e cada
  componente declarado no topo DEVE vir num arquivo `<componente>.vhd`, para que o
  FPGAEmuWeb, que lê as portas do topo até `end <nome>;`, abra o projeto sem edição.
  *Prova:* teste do núcleo (fechamento e um arquivo por componente); job GHDL do CI.

- **VHD-10** — O sistema DEVE gerar `<nome>_comportamental.vhd`, com a entidade
  `<nome>_comportamental`, as mesmas portas do topo e uma arquitetura que calcula a
  expressão com `unsigned`, `*`, `+` e `resize` de `numeric_std`, cada produto e a soma
  reduzidos a $W$ bits, sem instanciar células. Um coeficiente que não cabe em `integer` do
  VHDL-93 ($> 2^{31}-1$) DEVE ir como cadeia de bits.
  *Prova:* teste do núcleo (portas idênticas às do topo, a expressão de R2, coeficiente de
  $3 \cdot 10^9$ e a variável `R_IN`); em CI, o GHDL simula R2, R3, R4, a amostra de 21
  bits e o coeficiente grande nas duas versões.

- **VHD-11** — QUANDO o circuito tiver contadores (CMP-20), o sistema DEVE gerar a
  entidade de cada tipo usado, montada com `full_adder` pela rede da ADR 0005, e instanciar
  cada contador como `C73_<n>` ou `C154_<n>`, com as saídas descartadas em `open`.
  *Prova:* testes do núcleo (arquivos e ordem, instância, `open`, nomes reservados, mesmos
  identificadores do diagrama, cabeçalho com custos); em CI, o GHDL simula o 7:3 isolado
  (128 casos), o 15:4 isolado (32 768) e R1 com um 7:3 (16 384), e acusa o 7:3 estragado
  de propósito (as saídas do último FA trocadas).

## Compatibilidade com ferramentas

O alvo declarado é VHDL-93 sem construções de VHDL-2008, para que o código também rode no
ModelSim ASE 10.1d e no Quartus II 13.0, que ainda suportam a DE2. A garantia **automática** é o GHDL em
`--std=93` no CI. A compatibilidade com o ModelSim é conferida à mão antes de cada versão e
registrada no CHANGELOG; em 2026-11-01 (MT-10), o R2 gerado compilou sem avisos no ModelSim
ASE 10.1d e deu "TODOS OS 16384 CASOS CORRETOS". Em 2026-11-06, para a v0.1.0 (já com
`end <nome>;`), o mesmo resultado, e o R2 estragado de propósito acusou erro já no vetor 30.
Em 2026-11-09 (MT-21), com a versão comportamental, o ModelSim compilou R2, o coeficiente
grande e o comportamental estragado sem avisos; os dois primeiros deram todos os casos
corretos nas duas versões, e o terceiro acusou 9216 casos errados só na comportamental.
Em 2026-11-10, para a v0.2.0, os 11 casos de `examples/vhdl_referencia.rs` (os do GHDL no
CI, com constante e placa DE2) compilaram sem avisos no ModelSim; os 8 corretos deram todos
os casos certos em cada versão, e os 3 estragados de propósito acusaram erro só na versão
estragada.

No Quartus, os `*` da versão comportamental podem ser mapeados nos multiplicadores
embutidos da FPGA. Para comparar só elementos lógicos com a versão estrutural, é preciso
pedir multiplicadores em lógica na síntese.

Duas lições do primeiro contato com simuladores reais, que viraram critério:
- o GHDL recusa caracteres fora do Latin-1 até em comentário (um travessão bastou), por isso
  os arquivos são ASCII;
- o FPGAEmuWeb lê a entidade de topo procurando `end <nome>;` ou
  `end entity;` e recusa `end entity <nome>;`; em 2026-11-05 (MT-19), a análise de portas
  do emulador leu as 8 portas do R2 gerado e reuniu os 3 arquivos do projeto;
- o GHDL segue à risca a regra de ligação padrão do VHDL-93 e deixava todas as células sem
  ligação; o ModelSim é permissivo e escondia o problema. O `use work.all` resolve nos dois;
- o VHDL não distingue maiúsculas: o laço `for n` e a variável `v` do testbench escondiam
  as portas de variáveis `N` e `V`, e a constante `W` da versão comportamental escondia a
  porta `W` (achado com o 15:4 do MT-32a, que usa as variáveis `A` a `O`). Os nomes locais
  agora têm mais de uma letra, e a constante se chama `R_WIDTH`;
- o bit 1 de uma variável `S` se chamava `s1`, o mesmo nome da soma do FA1, e o topo
  declarava o sinal duas vezes (o mesmo com `K` e `k<n>`). O bit sozinho de `S` e `K` agora
  é `s1_in` e `k1_in` (spec 05, MT-49), e o GHDL simula o caso `nomes_s_k`.

## Fora de escopo

Verilog; VHDL otimizado. O topo para a placa DE2 e o projeto do Quartus estão em
[`10-placa.md`](10-placa.md), que também reserva alguns nomes (PLA-07).
