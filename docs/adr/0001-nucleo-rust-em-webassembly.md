<!-- Caminho relativo: docs/adr/0001-nucleo-rust-em-webassembly.md -->

# ADR 0001: Núcleo em Rust compilado para WebAssembly, interface SvelteKit estática

- **Status:** Accepted
- **Data:** 2026-10-30
- **Decisores:** Iago Leal
- **Tags:** arquitetura, stack, implantação

## Contexto

O dotree é uma ferramenta interativa: cada clique (formar um FA, desfazer, concluir um
nível) precisa validar a seleção e recalcular matriz, área e atraso na hora (GER-05). A
verificação exaustiva simula até $2^{24}$ combinações (VER-01, VER-02). Não há dado a
guardar no servidor: o exercício é do usuário e fica no navegador dele (GER-04, SAL-01).

O autor fixou a stack em Svelte/SvelteKit + Rust + Docker. Faltava decidir **onde o Rust
roda**.

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **API em Rust (axum) + SvelteKit chamando o servidor** | Ida e volta na rede a cada clique; servidor para manter no ar e pagar; nenhum estado justifica um servidor. |
| **Tudo em TypeScript, sem Rust** | Mais simples de montar, mas abre mão de um núcleo testável com `cargo test` e proptest, reaproveitável numa CLI ou num servidor futuro, e do desempenho da verificação exaustiva. Contraria a stack escolhida. |
| **Rust em WASM agora e servidor depois** | É o caminho desta decisão: o núcleo isolado deixa o servidor (ex.: rodar GHDL, links compartilháveis) como acréscimo futuro, sem reescrita. |

## Decisão

- O domínio (expressão, matriz, células, níveis, métricas, soluções automáticas,
  verificação, leiaute do diagrama, geração de VHDL) mora numa **crate Rust pura**,
  `dotree-core`, sem dependência de navegador nem de WASM.
- Uma segunda crate, `dotree-wasm`, é só a ponte `wasm-bindgen` entre o núcleo e a interface.
- A interface é **SvelteKit (Svelte 5)** com `adapter-static`: o produto é um site estático,
  sem servidor de aplicação.
- O **Docker** garante build reprodutível (etapa Rust → WASM, etapa Node → site) e uma imagem
  final que só serve arquivos estáticos.

## Consequências

- **Impacto positivo:** resposta imediata na interface; hospedagem estática gratuita
  (ex.: GitHub Pages); funciona sem rede depois de carregado; o núcleo é testado sem
  navegador.
- **Impacto negativo:** duas cadeias de ferramentas (cargo + wasm-pack, Node + pnpm); a
  fronteira WASM ↔ TypeScript precisa de tipos mantidos nos dois lados.
- **Trade-offs aceitos:** o pacote WASM aumenta o primeiro carregamento. Nota, não critério:
  mirar < 500 kB comprimido e medir quando houver código.

## Diretriz de Conformidade de Código

- **Proibido:** lógica de domínio (regras de célula, métricas, verificação, geração de VHDL)
  em TypeScript ou em componente Svelte; a interface só exibe e encaminha ações.
- **Proibido:** dependência de `wasm-bindgen`, `web-sys` ou `js-sys` em `dotree-core`.
- **Proibido:** texto em linguagem natural para o usuário dentro do núcleo; o núcleo devolve
  códigos (IDI-05). **Exceção:** os modelos de comentário do VHDL gerado (VHD-06) ficam no
  núcleo, nos dois idiomas, com teste de paridade, porque o VHDL é artefato gerado pelo
  núcleo e não texto de interface.
- **Obrigatório:** toda regra de domínio (montagem da matriz, células, CPA, métricas, ajuda
  calculada, verificação, leiaute do diagrama, geração de VHDL) tem teste em
  `cargo test -p dotree-core`. Critérios de interface (foco, confirmação, persistência,
  troca de visão) provam-se em teste de componente ou de ponta a ponta.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
