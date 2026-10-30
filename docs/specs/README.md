<!-- Caminho relativo: docs/specs/README.md -->

# Specs do dotree

> **Estado: v0.1 aprovada pelo autor em 2026-10-30.** Decisões D-1 a D-6 confirmadas; spec
> criticada em contexto limpo e corrigida. Mudanças daqui em diante seguem a regra de
> manutenção abaixo (EXP-13 e EXP-14 entraram com o MT-3).

A spec fixa **o resultado** e dura. A sequência de trabalho (o plano) mora em
`docs/TICKETS.md`, e as decisões com alternativa descartada moram em `docs/adr/`. Os testes
são a spec em forma executável: cada critério tem uma prova associada.

## Como ler

- Cada critério tem um identificador (`EXP-03`, `CMP-07`…) para ser citado em ticket, teste
  e commit. Identificadores não são renumerados: um critério removido deixa o número vago.
- Os critérios estão em **EARS**: `O sistema DEVE…`, `QUANDO…`, `ENQUANTO…`, `ONDE…`,
  `SE… ENTÃO…`. Nunca "deveria" ou "pode".
- **Prova** diz como o critério é verificado. Enquanto não houver código, ela nomeia o teste
  que nascerá falhando (skill `teste-primeiro`).

## Índice

| Arquivo | Assunto | Prefixos |
|---|---|---|
| [`00-transversal.md`](00-transversal.md) | Objetivo, princípio didático, revelação em duas etapas, glossário, custo, não-funcionais, casos de referência | `GER` |
| [`01-expressao-e-matriz.md`](01-expressao-e-matriz.md) | Expressão → matriz, cores por termo, visão de variáveis indexadas, desafio de $W$ | `EXP`, `COR`, `VIS` |
| [`02-compressao.md`](02-compressao.md) | Níveis, FA, HA, CPA e métricas do circuito do usuário | `CMP` |
| [`03-ajuda-sob-demanda.md`](03-ajuda-sob-demanda.md) | Dicas, próxima jogada, limite inferior e solução Dadda — só quando pedidas | `AJU` |
| [`04-verificacao.md`](04-verificacao.md) | Simulação, amostragem e contraexemplo | `VER` |
| [`05-diagrama.md`](05-diagrama.md) | Diagrama de blocos, nomes e tabela de ligações | `DIA` |
| [`06-vhdl.md`](06-vhdl.md) | VHDL-93 estrutural e testbench | `VHD` |
| [`07-idiomas.md`](07-idiomas.md) | pt-BR e inglês | `IDI` |
| [`08-salvar.md`](08-salvar.md) | Salvar, abrir e retomar | `SAL` |
| [`09-entrega.md`](09-entrega.md) | CI, GitHub Pages, imagem Docker, *release* | `ENT` |

## Decisões confirmadas pelo autor

| Id | Decisão | Onde |
|---|---|---|
| D-1 | O app pergunta a largura $W$ antes de revelá-la | EXP-09, EXP-10 |
| D-2 | Custo fixo: FA = 1 $A_{FA}$/1 $T_{FA}$; HA = 0,5/0,5; AND fora da área | 00, GER-04 |
| D-3 | Área e atraso do próprio circuito sempre visíveis | CMP-15 |
| D-4 | Alvo ≥ 768 px; abaixo disso, só leitura | GER-10 |
| D-5 | Confirmação a cada pedido de ajuda | AJU-02 |
| D-6 | Compartilhar por link fica fora da v0.1 | 08 |
| — | Wallace e exportação CSV ficam para a v0.2 | 00, 03, 05 |
| — | Cores por termo no estilo das aulas; alternância pontos ↔ variáveis indexadas | COR, VIS |
| — | Revelação em duas etapas: resultado, depois caminho | GER-03 |
| — | Exemplos próprios do projeto (nenhum enunciado de terceiros) | 00 |

## Regra de manutenção

Mudou o comportamento, a spec muda **no mesmo trabalho**. Se uma mudança contraria a spec,
decide-se explicitamente qual dos dois está errado — nunca os dois divergindo em silêncio.
