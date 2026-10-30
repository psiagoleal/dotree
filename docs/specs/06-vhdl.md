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
  VHDL-93, usando apenas `ieee.std_logic_1164` e `ieee.numeric_std`.
  *Prova:* em CI, o GHDL (`--std=93`) analisa e elabora os arquivos gerados para R2, R3 e
  para a solução Dadda de R4.
- **VHD-02** — O topo DEVE usar os mesmos identificadores de sinal e de instância do
  diagrama e da tabela de ligações; um bit de resultado constante é atribuído com
  `R(j) <= '0'`.
  *Prova:* teste do núcleo (mesmo conjunto de identificadores de DIA-03) e `2A` com $A$ de
  2 bits.
- **VHD-03** — O topo DEVE trazer no cabeçalho, em comentário, a expressão, as larguras, a
  matriz de pontos em texto e as métricas (FA, HA, AND, área, atraso).
  *Prova:* teste de unidade do gerador (cabeçalho de R2 fixado).
- **VHD-04** — O testbench DEVE calcular o valor esperado com aritmética `unsigned` de
  `numeric_std`, redimensionada para $W$ bits, e percorrer os vetores de `04-verificacao.md`
  (todas as combinações até 20 bits; acima disso, os 4096 vetores da amostra, embutidos como
  constante). Ao terminar, DEVE emitir `report` com o total de casos corretos ou o primeiro
  erro.
  *Prova:* em CI, o GHDL simula o testbench de R2 e a saída contém "16384 CASOS CORRETOS";
  com o circuito corrompido de VER-03, a simulação acusa o erro.
- **VHD-05** — O sistema DEVE permitir copiar cada arquivo e baixar todos num `.zip`.
  *Prova:* teste de ponta a ponta.
- **VHD-06** — Os comentários do VHDL gerado DEVEM seguir o idioma escolhido na interface;
  os identificadores são iguais nos dois idiomas.
  *Prova:* teste de unidade do gerador nos dois idiomas (só comentários diferem).
- **VHD-07** — ENQUANTO o circuito estiver incompleto, o sistema DEVE manter a geração de
  VHDL desabilitada, explicando o motivo.
  *Prova:* teste de componente.

## Compatibilidade com ferramentas

O alvo declarado é VHDL-93 sem construções de VHDL-2008, para que o código também rode no
ModelSim ASE 10.1d e no Quartus usados em aula. A garantia **automática** é só o GHDL em
`--std=93`; a compatibilidade com o ModelSim é conferida à mão antes de cada versão e
registrada no CHANGELOG.

## Fora de escopo

Verilog; arquivo de projeto do Quartus; atribuição de pinos de placa; VHDL comportamental
ou otimizado.
