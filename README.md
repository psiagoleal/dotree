<!-- Caminho relativo: README.md -->

# dotree

> Monte a matriz de pontos de uma expressão aritmética, comprima-a você mesmo com FA, HA e
> CPA, e veja o diagrama de blocos e o VHDL do circuito que você montou. Uma ferramenta
> didática para treinar a intuição em árvores de redução (Wallace, Dadda).

![status](https://img.shields.io/badge/status-spec%20em%20revis%C3%A3o-orange)
![version](https://img.shields.io/badge/version-0.0.0-blue)
![license](https://img.shields.io/badge/license-MIT-green)

*English version of the interface: planned from v0.1 (pt-BR and English).*

## Estado

O projeto está na fase de **especificação**: ainda não há código. O contrato do que a v0.1
fará está em [`docs/specs/`](docs/specs/README.md), e as decisões de arquitetura em
[`docs/adr/`](docs/adr/README.md).

## O que o dotree vai fazer (v0.1)

1. **Expressão → pontos.** Você digita algo como `A*B + C*D + 2(E+F) + G` e as larguras de
   cada variável; o app mostra a matriz de pontos, uma cor e uma forma por termo.
2. **Você comprime.** Escolhe três pontos de uma coluna para um FA, dois para um HA, nível a
   nível, e fecha com um CPA. Altura das colunas, área ($A_{FA}$) e atraso ($T_{FA}$)
   aparecem na hora.
3. **Ajuda só quando você pede.** Dica, próxima jogada, solução automática (Dadda, Wallace)
   e limite inferior de FA existem, mas o app pergunta antes de mostrar.
4. **Prova.** Verificação exaustiva do circuito (ex.: 65 536 casos).
5. **Saídas.** Diagrama de blocos (SVG), tabela de ligações e VHDL-93 estrutural com
   testbench.

Tudo roda no navegador: o núcleo é escrito em Rust e compilado para WebAssembly; a
interface é SvelteKit. Nenhum dado sai do seu computador.

## Pré-requisitos (para desenvolver)

- Rust estável com o alvo `wasm32-unknown-unknown` e `wasm-pack`
- Node.js 24+ e pnpm
- Docker (build reprodutível e testes do VHDL gerado com GHDL)

## Instalação e uso

Ainda não há o que instalar: os comandos entram aqui quando o primeiro código existir e
tiver sido executado.

## Estrutura de diretórios

```
app/core/     # crate dotree-core: domínio puro em Rust
app/wasm/     # crate dotree-wasm: ponte para o navegador
app/web/      # interface SvelteKit
docs/specs/   # especificação (contrato da v0.1)
docs/adr/     # decisões de arquitetura
docs/insumos/ # briefing
```

## Referência

B. Parhami, *Computer Arithmetic: Algorithms and Hardware Designs*, 2.ª ed., Oxford
University Press — caps. 11 e 12 (árvores de redução e multiplicação).

## Como contribuir

1. Leia [`AGENTS.md`](AGENTS.md) e a spec em [`docs/specs/`](docs/specs/README.md).
2. Faça um *fork* e crie uma branch (`feature/minha-feature`).
3. Cite no PR o critério da spec que a mudança atende (ex.: `CMP-11`).
4. Garanta que linter e testes passam.

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
