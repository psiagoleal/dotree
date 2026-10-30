<!-- Caminho relativo: docs/specs/07-idiomas.md -->

# 07 — Idiomas: pt-BR e inglês

## Resultado

Todo texto da interface existe em português do Brasil e em inglês desde a primeira versão.

## Critérios

- **IDI-01** — O sistema DEVE obter todo texto da interface de um catálogo de mensagens por
  idioma; nenhum texto visível pode ficar fixo no código dos componentes.
  *Prova:* checagem em CI que falha se um componente tiver texto literal visível fora do
  catálogo (lint de i18n).
- **IDI-02** — Os catálogos pt-BR e en DEVEM ter exatamente o mesmo conjunto de chaves.
  *Prova:* teste que compara as chaves dos dois catálogos.
- **IDI-03** — Na primeira visita, o sistema DEVE escolher pt-BR quando o idioma do navegador
  começar com `pt` e inglês nos demais casos.
  *Prova:* teste de ponta a ponta com os dois idiomas de navegador.
- **IDI-04** — QUANDO o usuário trocar de idioma, o sistema DEVE traduzir a interface sem
  perder o exercício em andamento, e DEVE lembrar a escolha nas próximas visitas.
  *Prova:* teste de ponta a ponta (troca no meio da compressão; estado preservado).
- **IDI-05** — As mensagens de recusa e de ajuda geradas pelo núcleo DEVEM sair como
  **códigos** com parâmetros, traduzidos na interface. O núcleo não contém texto em
  linguagem natural para o usuário.
  *Prova:* teste do núcleo (as recusas de CMP-06 a CMP-09 são códigos) e IDI-02 cobrindo
  esses códigos.

## Convenção de termos

FA, HA, CPA, AND e os rótulos de sinal são iguais nos dois idiomas. "Vai-um" ↔ *carry*,
"ponto" ↔ *dot*, "nível" ↔ *level*, "matriz de pontos" ↔ *dot matrix* (ou *dot diagram*).

## Fora de escopo

Outros idiomas; tradução da documentação do repositório (que fica em português).
