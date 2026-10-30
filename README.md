<!-- Caminho relativo: README.md -->

# dotree

> Monte a matriz de pontos de uma expressão aritmética, comprima-a você mesmo com FA, HA e
> CPA, e veja o diagrama de blocos e o VHDL do circuito que você montou. Uma ferramenta
> didática para treinar a intuição em árvores de redução (Wallace, Dadda).

[![CI](https://github.com/psiagoleal/dotree/actions/workflows/ci.yml/badge.svg)](https://github.com/psiagoleal/dotree/actions/workflows/ci.yml)
![version](https://img.shields.io/badge/version-0.0.0-blue)
![license](https://img.shields.io/badge/license-MIT-green)

**No ar:** <https://psiagoleal.github.io/dotree/> (versão de esqueleto: carrega o núcleo em
WebAssembly e troca de idioma; as funcionalidades chegam ticket a ticket).

*Interface em português e inglês.*

## Estado

Esqueleto pronto: núcleo Rust compilado para WebAssembly, interface SvelteKit estática,
testes de unidade, de componente e de ponta a ponta, imagem Docker e CI/CD com publicação
no GitHub Pages. O contrato da v0.1 está em [`docs/specs/`](docs/specs/README.md); o plano,
em [`docs/TICKETS.md`](docs/TICKETS.md).

## O que o dotree vai fazer (v0.1)

1. **Expressão → pontos.** Você digita algo como `A*B + C*D + 3(E+F) + G` e as larguras de
   cada variável; o app mostra a matriz de pontos, uma cor por termo, e alterna a qualquer
   momento para as variáveis indexadas ($a_1 b_0$).
2. **Você comprime.** Escolhe três pontos de uma coluna para um FA, dois para um HA, nível a
   nível, e fecha com um CPA. Altura das colunas, área ($A_{FA}$) e atraso ($T_{FA}$)
   aparecem na hora.
3. **Ajuda só quando você pede.** Dica, próxima jogada, limite inferior e solução automática
   (Dadda) existem, mas o app pergunta antes de mostrar e revela em duas etapas: primeiro o
   resultado, depois o caminho.
4. **Prova.** Verificação exaustiva do circuito (ex.: 16 384 casos).
5. **Saídas.** Diagrama de blocos (SVG), tabela de ligações e VHDL-93 estrutural com
   testbench.

Tudo roda no navegador: nenhum dado sai do seu computador.

## Pré-requisitos (para desenvolver)

- Rust estável com o alvo `wasm32-unknown-unknown` (o `rust-toolchain.toml` pede) e
  [`wasm-pack`](https://github.com/wasm-bindgen/wasm-pack)
- Node.js 24+ e pnpm 11
- Docker (opcional: imagem e, no futuro, testes do VHDL com GHDL)

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

## Estrutura de diretórios

```
app/core/     # crate dotree-core: domínio puro em Rust
app/wasm/     # crate dotree-wasm: ponte para o navegador
app/web/      # interface SvelteKit (rotas, componentes, i18n, testes)
docs/specs/   # especificação (contrato da v0.1)
docs/adr/     # decisões de arquitetura
docs/plano.md # plano por tickets; painel em docs/TICKETS.md
.github/      # CI, release e Dependabot
```

## Referência

B. Parhami, *Computer Arithmetic: Algorithms and Hardware Designs*, 2.ª ed., Oxford
University Press — caps. 11 e 12 (árvores de redução e multiplicação).

## Como contribuir

1. Leia [`AGENTS.md`](AGENTS.md) e a spec em [`docs/specs/`](docs/specs/README.md).
2. Faça um *fork* e crie uma branch (`feature/minha-feature`).
3. Cite no PR o critério da spec (ex.: `CMP-11`) e o ticket (ex.: `MT-5`).
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
