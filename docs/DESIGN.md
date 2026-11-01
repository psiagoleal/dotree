<!-- Caminho relativo: docs/DESIGN.md -->

# Contrato visual do dotree

> O que a interface precisa respeitar para ficar coerente com as figuras de aula e
> acessível. Os valores moram em `app/web/src/app.css` (*tokens* CSS); este documento diz o
> que cada um significa. A paleta é conferida por teste (`src/lib/palette.spec.ts`, COR-03).

## Princípio

A tela imita a folha de exercício das aulas: colunas de pontos com o peso $2^j$ no topo, o
maior peso à esquerda, células desenhadas como retângulos em volta dos pontos que consomem,
um painel por nível e uma seta de um painel para o seguinte. Tudo o que não é ponto, célula
ou métrica fica em segundo plano.

## Cores dos pontos (COR-01 a COR-03)

| Papel | Token | Claro | Escuro |
|---|---|---|---|
| Termo 1 | `--dot-term-1` | `#2e7d32` verde | `#66bb6a` |
| Termo 2 | `--dot-term-2` | `#c62828` vermelho | `#ef5350` |
| Termo 3 | `--dot-term-3` | `#c25e00` laranja | `#ffa726` |
| Termo 4 | `--dot-term-4` | `#1565c0` azul | `#64b5f6` |
| Termo 5 | `--dot-term-5` | `#ad1457` magenta | `#f06292` |
| Termo 6 | `--dot-term-6` | `#00838f` ciano | `#4dd0e1` |
| Termo 7 | `--dot-term-7` | `#8d6e63` marrom | `#bcaaa4` |
| Termo 8 | `--dot-term-8` | `#827717` oliva | `#d4e157` |
| Saída de célula (soma, vai-um) | `--dot-generated` | `#1d1d1f` preto | `#ececf1` |
| Bit do resultado | `--dot-result` | `#6b2fb3` roxo | `#b197fc` |

- Como nas figuras de aula: cada termo da expressão combinada tem uma cor; o que as células
  produzem é preto; o resultado final é roxo. O roxo fica **reservado** ao resultado.
- Do 9.º termo em diante, as cores se repetem e a legenda numera os termos (COR-03).
- **Regras conferidas por teste**, nos dois temas: contraste de pelo menos 3:1 com o fundo;
  distância perceptual ΔE (CIE76) de pelo menos 20 entre cada cor de termo e as cores de
  ponto gerado e de resultado; pelo menos 25 entre duas cores de termo.
- **Cor nunca é a única pista** (GER-07): a visão de variáveis indexadas (VIS-01) escreve o
  sinal de cada ponto, e o rótulo aparece ao passar o cursor ou focar o ponto.

## Desenho

| Elemento | Forma |
|---|---|
| Ponto | quadrado de 14 px (como nas figuras de aula), cor do papel acima |
| Ponto selecionado | contorno de 2 px na cor de destaque, deslocado 2 px |
| Ponto em foco pelo teclado | o anel de foco padrão (`:focus-visible`) |
| Visão indexada | o sinal em fonte monoespaçada com índices subscritos ($a_1 b_0$), na mesma posição do ponto |
| FA | retângulo de borda contínua (1,5 px) em volta dos 3 pontos consumidos |
| HA | retângulo de borda tracejada em volta dos 2 pontos |
| Caminho crítico | borda de 3 px na cor de destaque nas células críticas (CMP-15, DIA-05) |
| Painel de nível | cabeçalho com $2^j$ por coluna, colunas alinhadas entre painéis, seta para o painel seguinte |
| Diagrama de blocos | blocos FA/HA como retângulos com o nome (`FA12`); ligações rotuladas com o sinal |

## Tipografia e espaço

- Texto: `--font-body` (fonte do sistema). Sinais, identificadores e VHDL: `--font-mono`.
- Escala de espaço: `--space-1` a `--space-5` (0,25 a 2,5 rem). Raio: `--radius`.
- Pesos de coluna e métricas usam números tabulares (`font-variant-numeric: tabular-nums`),
  para alinharem entre painéis.

## Interação

- **Ajuda sob demanda** (AJU-02): todo pedido de resposta abre um diálogo com o foco inicial
  em "vou tentar mais um pouco"; confirmar é a segunda opção, nunca a padrão. Esc também
  conta como "vou tentar". Um só componente (`ConfirmDialog`, `<dialog>` nativo) serve a
  todos os pedidos.
- **Visão** (VIS-01, VIS-03): botões "Pontos" e "Variáveis indexadas" ao lado do título da
  matriz, com a tecla V como atalho; a escolha fica no `localStorage` do navegador.
- **Revelação em duas etapas** (GER-03): resultado primeiro; o caminho fica atrás de um
  segundo botão, que só aparece depois da etapa 1.
- **Recusas** (GER-02): mensagem curta junto aos pontos selecionados, dizendo a regra
  violada, sem sugerir a correção.
- **Tela pequena** (GER-10): abaixo de 768 px, matriz, diagrama e VHDL em modo leitura, com
  um aviso no topo.

## Temas

Claro por padrão; escuro pela preferência do sistema (`prefers-color-scheme`). Todo token
de cor tem os dois valores.
