<!-- Caminho relativo: README.md -->

# dotree

> Monte a matriz de pontos de uma expressão aritmética, comprima-a você mesmo com FA, HA e
> CPA, e veja o diagrama de blocos e o VHDL do circuito que você montou. Uma ferramenta
> didática para treinar a intuição em árvores de redução (Wallace, Dadda).

[![CI](https://github.com/psiagoleal/dotree/actions/workflows/ci.yml/badge.svg)](https://github.com/psiagoleal/dotree/actions/workflows/ci.yml)
![version](https://img.shields.io/badge/version-0.1.0-blue)
![license](https://img.shields.io/badge/license-MIT-green)

**No ar:** <https://psiagoleal.github.io/dotree/> — sempre o que está no `main`. Última
versão marcada: 0.1.0; a 0.2.0 está em preparação.

*Interface em português e inglês. Tudo roda no navegador: nenhum dado sai do seu
computador.*

## Estado

O `main` tem a v0.2 quase completa: o exercício da v0.1 (da expressão ao VHDL) mais
constante aditiva, soluções de Dadda e de Wallace aplicáveis, comparação de soluções,
tabela de alturas, topo para a placa DE2 e entrada por perfil de alturas com solução ótima.
O contrato é uma spec só, viva ([`docs/specs/`](docs/specs/README.md), ADR
[0004](docs/adr/0004-spec-viva-e-referencias-publicas.md)); o que vem depois está em
[`docs/plano-pos-v0.1.md`](docs/plano-pos-v0.1.md) e no painel
[`docs/TICKETS.md`](docs/TICKETS.md); as mudanças de cada versão, no
[`CHANGELOG.md`](CHANGELOG.md).

## O que o dotree faz

1. **Expressão → pontos.** Você digita uma soma de produtos sem sinal, como
   `A*B + C*D + 3(E+F) + G` ou `3(A + 1) + B`, e a largura de cada variável (até 16 bits).
   O app mostra a matriz de pontos, uma cor por termo, e alterna a qualquer momento para as
   variáveis indexadas ($a_1 b_0$, tecla V). Constantes viram pontos fixos em 1.
2. **Ou um perfil de alturas.** `{2,2,3; 1,1,1,1}` dá as alturas das colunas de entrada e,
   depois do `;`, as de saída, do bit mais significativo ao menos: a notação dos contadores
   paralelos generalizados (Parhami, cap. 8). O exercício termina quando o seu circuito
   atinge o perfil, e o app mostra a área e o atraso do bloco.
3. **Você comprime.** Escolhe três pontos de uma coluna para um FA, dois para um HA, nível a
   nível, pelo mouse ou pelo teclado, e fecha com um CPA. Altura das colunas, área
   ($A_{FA}$), atraso ($T_{FA}$), caminho crítico e a tabela de alturas nível a nível
   aparecem na hora; dá para desfazer e voltar a um nível.
4. **Ajuda só quando você pede.** Largura do resultado, dica, próxima jogada, limite
   inferior, as soluções de Dadda e de Wallace e, nos perfis, a solução ótima e o nome do
   bloco. O app pergunta antes de mostrar, revela em duas etapas (primeiro o resultado,
   depois o caminho) e conta cada revelação no resumo e no arquivo do exercício. Uma
   solução revelada pode substituir o seu circuito, num passo que se desfaz.
5. **Comparação.** O seu circuito ao lado das soluções, com área, atraso, área × atraso e a
   marca das não dominadas. Comparar não revela nem conta nada.
6. **Prova.** Verificação do circuito num *worker*, com progresso e cancelamento:
   exaustiva até 24 bits de entrada (ex.: 16 384 casos), por amostra acima, com
   contraexemplo se houver erro.
7. **Saídas.** Diagrama de blocos (SVG), tabelas de ligações e de alturas (Markdown) e
   VHDL-93: estrutural, comportamental (`numeric_std`) e um testbench que confere os dois.
   Com até 18 bits de entrada, também o topo para a placa DE2 e o projeto do Quartus II.
   Tudo em arquivos soltos ou num `.zip`.
8. **Seu trabalho fica.** O exercício é salvo no navegador a cada ação e pode ser exportado
   e reaberto como arquivo JSON.

## Passo a passo

1. Abra o site e escolha um exemplo ("Soma de produtos", "Multiplicador 4×4", "Perfil de
   alturas"…) ou digite a sua expressão e as larguras.
2. Responda, quando quiser, quantos bits tem o resultado. Até lá a matriz mostra só as
   colunas com pontos; o app diz se você acertou, e a conta só aparece se você pedir.
3. Selecione pontos de uma coluna (clique, ou Tab e Espaço), forme a célula (F) e conclua o
   nível para passar ao seguinte. Quando restarem no máximo duas linhas, feche com o CPA;
   se ainda não der, o botão diz o que falta. Ctrl+Z desfaz e Ctrl+Y refaz.
4. Acompanhe área, atraso e alturas. Se travar, peça uma ajuda: ela fica registrada.
5. Com o circuito completo, verifique-o, compare com as soluções e exporte o diagrama, as
   tabelas e o VHDL. Simule o testbench no GHDL ou no ModelSim.

## Limitações

Cada uma aponta para o critério da spec ou o ticket que a trataria.

- **Sem sinal:** nada de complemento de dois, subtração ou Booth (MT-33, MT-34).
- **Só FA e HA** como células; contadores maiores, compressor 4:2 e blocos AMM não entram
  no exercício (MT-32). Na entrada por alturas, o nome do bloco é só uma ajuda (AJU-12).
- **CPA só *ripple*** (MT-36).
- **Tamanho:** até 16 bits por variável, 32 bits de entrada e 256 pontos (MT-24, MT-25);
  perfil até 26 pontos e 16 colunas (EXP-16).
- **Custo é modelo, não síntese:** FA = 1 $A_{FA}$ e 1 $T_{FA}$, HA = 0,5 e 0,5, AND fora
  da área (GER-04, decisão D-2). Os números do Quartus podem ordenar as soluções de outro
  jeito.
- **Verificação por amostra** acima de 24 bits de entrada (spec 04).
- **Placa DE2:** o topo compila no Quartus II 13.0 e o testbench da placa passa no GHDL e
  no ModelSim, mas não foi conferido numa placa real nem no FPGAEmuWeb (spec 10). Acima de
  18 bits de entrada não há topo para a placa (MT-23).
- **Perfil de alturas:** a dica e a próxima jogada seguem o Dadda até a soma completa e
  podem sugerir reduzir além do perfil (MT-41); a busca do ótimo roda na página e pode
  travá-la por um instante nos perfis grandes (MT-42).
- **Tela pequena:** abaixo de 768 px de largura o exercício fica em modo leitura (GER-10,
  decisão D-4).

## Metodologia

- **Spec como contrato.** Cada comportamento é um critério numerado em
  [`docs/specs/`](docs/specs/README.md) (ex.: `CMP-11`), escrito antes do código e citado
  no teste que o prova, no ticket e no commit. A spec é uma só e muda no mesmo PR que muda
  o app.
- **Teste primeiro.** Núcleo em Rust com testes de unidade e de propriedade; interface com
  testes de componente (Vitest no navegador) e de ponta a ponta (Playwright), incluindo
  acessibilidade (axe, nos dois temas), ausência de rede e tempo de resposta.
- **Testes que pegam defeitos.** A cada ticket, defeitos são introduzidos de propósito
  (mutações) para conferir que algum teste falha.
- **Conferência independente.** Os casos de referência são do próprio projeto, calculados à
  mão e conferidos por simulação exaustiva; os 11 perfis de referência da solução ótima
  também por uma busca exaustiva escrita à parte. O VHDL gerado é simulado no GHDL a cada
  PR e conferido à mão no ModelSim ASE 10.1d antes de cada versão; o projeto da placa
  compilou no Quartus II 13.0.
- **Entrega.** Todo trabalho entra no `main` por PR com o CI verde, e o site é publicado a
  cada merge (ADR [0002](docs/adr/0002-entrega-github-pages-e-actions.md),
  [0003](docs/adr/0003-governanca-do-repositorio.md)).
- **Com agente de IA.** O código e os documentos foram escritos com o apoio de um agente de
  programação (Claude Code), em micro-tickets; o autor aprova spec, ADR e cada merge, e cada
  commit registra o agente e o modelo usados.

## Pré-requisitos (para desenvolver)

- Rust estável com o alvo `wasm32-unknown-unknown` (o `rust-toolchain.toml` pede) e
  [`wasm-pack`](https://github.com/wasm-bindgen/wasm-pack)
- Node.js 24+ e pnpm 11
- Docker (opcional: a imagem do site e os testes do VHDL com GHDL)

## Instalação e uso

```bash
git clone https://github.com/psiagoleal/dotree.git
cd dotree
pnpm --dir app/web install
pnpm --dir app/web exec playwright install chromium   # para os testes de interface
pnpm --dir app/web run dev                            # http://localhost:5173
```

Testes e checagens (os mesmos do CI):

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
pnpm --dir app/web run check && pnpm --dir app/web run lint && pnpm --dir app/web run test
BASE_PATH=/dotree pnpm --dir app/web run build && BASE_PATH=/dotree pnpm --dir app/web run test:e2e
```

Com Docker:

```bash
docker build -t dotree .
docker run --rm -p 8080:8080 dotree    # http://localhost:8080
```

Ou a imagem publicada de cada versão:

```bash
docker run --rm -p 8080:8080 ghcr.io/psiagoleal/dotree:0.1.0
```

## Estrutura de diretórios

```
app/core/     # crate dotree-core: domínio puro em Rust
app/wasm/     # crate dotree-wasm: ponte para o navegador
app/web/      # interface SvelteKit (rotas, componentes, i18n, testes)
docs/specs/   # especificação única e viva (ADR 0004)
docs/adr/     # decisões de arquitetura
docs/plano.md # plano da v0.1; o que vem depois em docs/plano-pos-v0.1.md
.github/      # CI, release e Dependabot
```

## Referências

- B. Parhami, *Computer Arithmetic: Algorithms and Hardware Designs*, 2.ª ed., Oxford
  University Press, 2010 — cap. 8 (adição de múltiplos operandos: notação de pontos,
  árvores de redução, contadores paralelos generalizados), cap. 11 (multiplicadores em
  árvore) e cap. 12 (variações de multiplicadores).
- C. S. Wallace, "A suggestion for a fast multiplier", *IEEE Transactions on Electronic
  Computers*, vol. EC-13, n.º 1, pp. 14–17, 1964.
- L. Dadda, "Some schemes for parallel multipliers", *Alta Frequenza*, vol. 34,
  pp. 349–356, 1965.
- Terasic, *DE2 Development and Education Board User Manual* — pinos das chaves, dos LEDs e
  dos displays usados no topo para a placa.

## Como contribuir

1. Leia [`AGENTS.md`](AGENTS.md) e a spec em [`docs/specs/`](docs/specs/README.md).
2. Faça um *fork* e crie uma branch (`feature/minha-feature`).
3. Abra um PR (o `main` só aceita mudanças por PR com o CI verde) citando o critério da
   spec (ex.: `CMP-11`) e o ticket (ex.: `MT-5`); o modelo do PR já pede isso.
4. Garanta que os comandos de checagem acima passam.

## Licença

Distribuído sob a licença MIT. Veja [`LICENSE`](./LICENSE).

---

## Apoie

**Feito com ❤️ por Iago Leal** | [☕ Apoie o criador]

Se este projeto ajudou você, considere apoiar:

- Buy Me a Coffee: https://buymeacoffee.com/psiagoleal

<a href="https://buymeacoffee.com/psiagoleal" target="_blank" rel="noopener">
  <img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me a Coffee" height="41" width="174" />
</a>
