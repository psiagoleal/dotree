<!-- Caminho relativo: docs/specs/06-vhdl.md -->

# 06 — VHDL estrutural de modelo

## Resultado

Um ponto de partida em VHDL para o circuito montado: simples, legível, com um testbench que
prova o circuito. Não é ferramenta de síntese; é um modelo que o estudante adapta.

## Arquivos gerados

| Arquivo | Conteúdo |
|---|---|
| `full_adder.vhd` | Entidade `full_adder` (x, y, z → s, k) |
| `half_adder.vhd` | Entidade `half_adder` (x, y → s, k) — só quando o circuito tem HA |
| `<nome>.vhd` | Topo estrutural: portas AND, uma instância por célula, com os nomes do diagrama |
| `tb_<nome>.vhd` | Testbench que percorre as entradas e compara com a expressão |

## Critérios

- **VHD-01** — QUANDO o circuito estiver completo, o sistema DEVE gerar os arquivos acima em
  VHDL-93, usando apenas `ieee.std_logic_1164` e `ieee.numeric_std`.
  *Prova:* em CI, o GHDL (`--std=93`) analisa e elabora os arquivos gerados para R1/R2, R3
  e R4.
- **VHD-02** — O topo DEVE usar os mesmos nomes de sinal e de instância do diagrama e da
  tabela de ligações (FA1…, `s1`, `k1`, `R(0)`…).
  *Prova:* teste do núcleo (mesmo conjunto de nomes de DIA-03).
- **VHD-03** — O topo DEVE trazer no cabeçalho, em comentário, a expressão, as larguras, a
  matriz de pontos em texto e as métricas (FA, HA, AND, área, atraso).
  *Prova:* teste de unidade do gerador.
- **VHD-04** — O testbench DEVE ser exaustivo quando a soma das larguras for de até 20 bits e,
  acima disso, usar uma amostra fixa e declarada no próprio arquivo. Ao terminar, DEVE
  emitir `report` com o total de casos corretos ou o primeiro erro.
  *Prova:* em CI, o GHDL simula o testbench de R2 e a saída contém
  "65536 CASOS CORRETOS"; com o circuito corrompido de VER-03, a simulação acusa o erro.
- **VHD-05** — O sistema DEVE permitir copiar cada arquivo e baixar todos num `.zip`.
  *Prova:* teste de ponta a ponta.
- **VHD-06** — O código gerado DEVE ser compatível com o ModelSim ASE 10.1d (VHDL-93, sem
  construções de VHDL-2008).
  *Prova:* revisão manual registrada no ticket que fechar esta spec; em CI, a garantia
  automática é o GHDL em `--std=93`.
- **VHD-07** — Os comentários do VHDL gerado DEVEM seguir o idioma escolhido na interface;
  identificadores ficam iguais nos dois idiomas.
  *Prova:* teste de unidade do gerador nos dois idiomas.

## Fora de escopo

Verilog; arquivo de projeto do Quartus; atribuição de pinos de placa; VHDL comportamental
ou otimizado.
