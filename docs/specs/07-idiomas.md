<!-- Caminho relativo: docs/specs/07-idiomas.md -->

# 07 — Idiomas: pt-BR e inglês

## Resultado

Todo texto da interface existe em português do Brasil e em inglês desde a primeira versão.

## Critérios

- **IDI-01** — O sistema DEVE obter todo texto da interface de um catálogo de mensagens por
  idioma (Paraglide); nenhum texto visível fica fixo no código dos componentes.
  *Prova:* regra de lint em CI que falha com texto literal visível em componente Svelte.
- **IDI-02** — Os catálogos pt-BR e en DEVEM ter exatamente o mesmo conjunto de chaves.
  *Prova:* teste que compara as chaves dos dois catálogos.
- **IDI-03** — Na primeira visita, o sistema DEVE escolher pt-BR quando o idioma preferido do
  navegador começar com `pt` (inclusive `pt-PT`, por ser o idioma mais próximo disponível) e
  inglês nos demais casos.
  *Prova:* teste de ponta a ponta com `pt-BR`, `pt-PT` e `en-US`.
- **IDI-04** — QUANDO o usuário trocar de idioma, o sistema DEVE traduzir a interface sem
  perder o exercício em andamento e DEVE lembrar a escolha nas próximas visitas.
  *Prova:* teste de ponta a ponta (troca no meio da compressão; estado preservado).
- **IDI-05** — As recusas, dicas e avisos produzidos pelo núcleo DEVEM sair como **códigos**
  com parâmetros, traduzidos na interface. A única exceção são os comentários do VHDL
  gerado (VHD-06), cujos modelos de texto ficam no núcleo nos dois idiomas
  ([ADR 0001](../adr/0001-nucleo-rust-em-webassembly.md)).
  *Prova:* teste do núcleo (recusas de `02-compressao.md` e dicas de `03` são códigos);
  IDI-02 cobrindo esses códigos; teste de paridade dos modelos de comentário VHDL.

## Convenção de termos

FA, HA, CPA, AND e os identificadores de sinal são iguais nos dois idiomas. "Vai-um" ↔
*carry*, "ponto" ↔ *dot*, "nível" ↔ *level*, "matriz de pontos" ↔ *dot diagram*, "variáveis
indexadas" ↔ *indexed variables*, "limite inferior" ↔ *lower bound*.

## Fora de escopo

Outros idiomas; tradução da documentação do repositório, que fica em português.
