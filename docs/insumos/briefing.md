<!-- Caminho relativo: docs/insumos/briefing.md -->

# Briefing — dotree

> Registro do pedido original, nas palavras do autor, para que a spec possa ser conferida
> contra ele. Não é spec: a spec está em [`docs/specs/`](../specs/README.md).

## Origem

O projeto nasceu de um exercício de Circuitos Aritméticos (mestrado, UFSC, 2026): projetar
o circuito de uma expressão como $A \times B + C \times D + 2(E+F) + G$, com operandos de
poucos bits, usando a **notação de pontos** — cada bit é um ponto, cada coluna é um peso
$2^j$ — e comprimindo a matriz com somadores completos (FA), meio-somadores (HA) e um
somador de propagação de vai-um (CPA) final.

Resolver à mão envolveu sempre os mesmos passos:

1. montar a matriz de pontos a partir da expressão;
2. escolher, nível a nível, quais trios viram FA e quais pares viram HA;
3. contar área e atraso;
4. desenhar o diagrama de blocos e a tabela de ligações;
5. escrever o VHDL estrutural e um testbench exaustivo.

## O pedido (2026-10-30)

> Uma aplicação web simples para fazer exatamente esse tipo de exercício: montar a
> representação com pontos, comprimir até ficar plana escolhendo quais grupos de pontos
> viram FA, HA, CPA etc., e gerar o diagrama de blocos. Talvez já apresentar um código
> VHDL para cada bloco do diagrama — algo simples, só como modelo.
>
> A ideia é que seja **didático**: facilitar o desenvolvimento dos exercícios, mostrar
> conceitos e **treinar a intuição** do usuário.

## Decisões já tomadas pelo autor

| Tema | Decisão |
|---|---|
| Nome | `dotree` (alternativa considerada: `bitpile`) |
| Stack | Svelte 5 / SvelteKit + Rust + Docker |
| Papel do Rust | Núcleo isolado em uma crate, compilado para WebAssembly e executado no navegador |
| Idiomas | pt-BR e inglês desde o início |
| Repositório | Público, na conta pessoal do GitHub |
| Postura didática | A interface **não** entrega resposta nem sugestão de imediato: **pergunta** se o usuário quer a resposta automática ou a dica, para incentivá-lo a raciocinar antes |

## Material de terceiros

Enunciados, figuras e soluções do professor **não** entram neste repositório (que é
público). Os exemplos embutidos no app são expressões genéricas; a referência ao livro-texto
é bibliográfica: B. Parhami, *Computer Arithmetic: Algorithms and Hardware Designs*,
caps. 11–12.
