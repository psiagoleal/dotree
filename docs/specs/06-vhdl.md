<!-- Caminho relativo: docs/specs/06-vhdl.md -->

# 06 — VHDL estrutural de modelo

## Resultado

Um ponto de partida em VHDL para o circuito montado: simples, legível, com um testbench que
prova o circuito. Não é ferramenta de síntese; é um modelo que o estudante adapta.

## Arquivos gerados

| Arquivo | Conteúdo |
|---|---|
| `full_adder.vhd` | Entidade `full_adder` (x, y, z → s, k) |
| `half_adder.vhd` | Entidade `half_adder` (x, y → s, k), só quando o circuito tem HA |
| `<nome>.vhd` | Topo estrutural: portas AND, uma instância por célula, com os identificadores de `05-diagrama.md` |
| `tb_<nome>.vhd` | Testbench que percorre os vetores de `04-verificacao.md` e compara com a expressão |

`<nome>` é escolhido pelo usuário (padrão `dotree_circuito`) e precisa ser identificador VHDL
válido.

## Critérios

- **VHD-01** — QUANDO o circuito estiver completo, o sistema DEVE gerar os arquivos acima em
  VHDL-93, usando apenas `ieee.std_logic_1164`, `ieee.numeric_std` e `use work.all` (que
  torna `full_adder` e `half_adder` diretamente visíveis, como a ligação padrão de
  componentes do VHDL-93 exige). Os arquivos DEVEM ser ASCII puro: o português dos
  comentários vai sem acento.
  *Prova:* em CI (job `vhdl`), o GHDL 6 em `--std=93` analisa, elabora e simula R2, R3, a
  solução Dadda de R4 e um caso de 21 bits; teste do núcleo exige ASCII e o `use work.all`.
- **VHD-02** — O topo DEVE usar os mesmos identificadores de sinal e de instância do
  diagrama e da tabela de ligações; um bit de resultado constante é atribuído com
  `R(j) <= '0'`. A saída se chama `R`; uma variável de entrada chamada $R$ vira a porta
  `R_IN`.
  *Prova:* teste do núcleo (mesmo conjunto de identificadores de DIA-03) e `2A` com $A$ de
  2 bits.
- **VHD-03** — O topo DEVE trazer no cabeçalho, em comentário, a expressão, as larguras, a
  matriz de pontos em texto e as métricas (FA, HA, AND, área, atraso).
  *Prova:* teste de unidade do gerador (cabeçalho de R2 fixado).
- **VHD-04** — O testbench DEVE calcular o valor esperado com aritmética `unsigned` de
  `numeric_std`, redimensionada para $W$ bits, e percorrer os vetores de `04-verificacao.md`
  (todas as combinações até 20 bits; acima disso, os 4096 vetores da amostra, embutidos como
  constante). Ao terminar, DEVE emitir `report` com o total de casos corretos ou, havendo
  erro, os 10 primeiros vetores errados e o total de errados, com `severity failure`.
  *Prova:* em CI, o GHDL simula o testbench de R2 e a saída contém "16384 CASOS CORRETOS";
  com o circuito corrompido de VER-03 (FA16 recebendo `k14`), a simulação acusa
  "CASOS ERRADOS" e falha.
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

## Compatibilidade com ferramentas

O alvo declarado é VHDL-93 sem construções de VHDL-2008, para que o código também rode no
ModelSim ASE 10.1d e no Quartus usados em aula. A garantia **automática** é o GHDL em
`--std=93` no CI. A compatibilidade com o ModelSim é conferida à mão antes de cada versão e
registrada no CHANGELOG; em 2026-11-01 (MT-10), o R2 gerado compilou sem avisos no ModelSim
ASE 10.1d e deu "TODOS OS 16384 CASOS CORRETOS".

Duas lições do primeiro contato com simuladores reais, que viraram critério:
- o GHDL recusa caracteres fora do Latin-1 até em comentário (um travessão bastou), por isso
  os arquivos são ASCII;
- o GHDL segue à risca a regra de ligação padrão do VHDL-93 e deixava todas as células sem
  ligação; o ModelSim é permissivo e escondia o problema. O `use work.all` resolve nos dois.

## Fora de escopo

Verilog; arquivo de projeto do Quartus; atribuição de pinos de placa; VHDL comportamental
ou otimizado.
