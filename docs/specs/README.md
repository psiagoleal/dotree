<!-- Caminho relativo: docs/specs/README.md -->

# Specs do dotree

> **Estado: RASCUNHO v0.1 — aguardando aprovação do autor.** Nenhum código antes da
> aprovação. Decisões marcadas com **⚠ A CONFIRMAR** são propostas do agente, não do autor.

A spec fixa **o resultado** e dura. A sequência de trabalho (o plano) mora em
`docs/TICKETS.md`, e as decisões com alternativa descartada moram em `docs/adr/`. Os testes
são a spec em forma executável: cada critério tem uma prova associada.

## Como ler

- Cada critério tem um identificador (`EXP-03`, `CMP-07`…) para ser citado em ticket, teste
  e commit.
- Os critérios estão em **EARS**: `O sistema DEVE…`, `QUANDO…`, `ENQUANTO…`, `ONDE…`,
  `SE… ENTÃO…`. Nunca "deveria" ou "pode".
- **Prova** diz como o critério será verificado. Enquanto não houver código, ela nomeia o
  teste que nascerá falhando (skill `teste-primeiro`).

## Índice

| Arquivo | Assunto | Prefixo |
|---|---|---|
| [`00-transversal.md`](00-transversal.md) | Objetivo, princípio didático, glossário, convenção de custo, não-funcionais, casos de referência | `GER` |
| [`01-expressao-e-matriz.md`](01-expressao-e-matriz.md) | Da expressão digitada à matriz de pontos | `EXP` |
| [`02-compressao.md`](02-compressao.md) | Níveis, FA, HA, CPA e métricas do circuito do usuário | `CMP` |
| [`03-ajuda-sob-demanda.md`](03-ajuda-sob-demanda.md) | Dicas, próxima jogada e solução automática — só quando pedidas | `AJU` |
| [`04-verificacao.md`](04-verificacao.md) | Simulação exaustiva e contraexemplo | `VER` |
| [`05-diagrama.md`](05-diagrama.md) | Diagrama de blocos e tabela de ligações | `DIA` |
| [`06-vhdl.md`](06-vhdl.md) | VHDL estrutural de modelo e testbench | `VHD` |
| [`07-idiomas.md`](07-idiomas.md) | pt-BR e inglês | `IDI` |
| [`08-salvar.md`](08-salvar.md) | Salvar, abrir e retomar um exercício | `SAL` |

## Regra de manutenção

Mudou o comportamento, a spec muda **no mesmo trabalho**. Se uma mudança contraria a spec,
decide-se explicitamente qual dos dois está errado — nunca os dois divergindo em silêncio.
