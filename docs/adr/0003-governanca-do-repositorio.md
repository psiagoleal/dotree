<!-- Caminho relativo: docs/adr/0003-governanca-do-repositorio.md -->

# ADR 0003: Governança do repositório público — tudo por PR, CI verde, só o dono mescla

- **Status:** Accepted
- **Data:** 2026-10-30
- **Decisores:** Iago Leal
- **Tags:** governança, segurança, CI

## Contexto

O repositório é público e publica automaticamente no GitHub Pages (ADR 0002) e no GHCR (por
*tag*). Qualquer pessoa pode abrir PR a partir de um fork, e o Dependabot abre PRs toda
semana. Até aqui, o `main` não tinha proteção: um push ou merge com o CI vermelho chegaria
ao ar. O autor quer o CI verde como condição de entrada e quer ser o único a aceitar e
mesclar PRs.

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Push direto liberado para o dono (lista de exceção)** | Mais rápido, mas o código do próprio dono poderia entrar vermelho no `main` e ir ao ar. |
| **Exigir 1 aprovação em todo PR** | O GitHub não deixa aprovar o próprio PR; com um mantenedor só, exigiria exceção para o dono, o que reabre o item acima. A permissão de escrita já garante que só o dono mescla. |
| **Proteção de branch clássica** | Substituída por *rulesets*, que são versionáveis em JSON e cobrem *tags*. |

## Decisão

- **Tudo entra no `main` por PR, inclusive o trabalho do dono**, com merge só por *squash* e
  histórico linear. Sem lista de exceção.
- **Check obrigatório: `CI ok`**, um job agregador do `ci.yml` que falha se qualquer job
  falhar, for cancelado ou pulado.
- **Só o dono mescla**: ele é o único colaborador com escrita, e o script de conferência
  falha se aparecer outro.
- `main` não pode ser apagado nem receber *force push*; conversas do PR precisam estar
  resolvidas.
- **Tags `v*`** (que publicam a imagem) só podem ser criadas, movidas ou apagadas por admin.
- **Actions:** workflows de PR vindos de fork só rodam depois de aprovação do dono, para
  qualquer externo; token dos workflows só com leitura e sem poder aprovar PR.
- **Segurança:** varredura de segredos com bloqueio no push, alertas de vulnerabilidade,
  PRs de correção do Dependabot e relato privado de vulnerabilidade
  (`.github/SECURITY.md`).
- A configuração mora em `.github/governanca/` (regras em JSON e `aplicar.sh`), aplicada e
  conferida pelo script.

## Consequências

- **Impacto positivo:** nada chega ao Pages sem CI verde; a política é revisável e
  reaplicável; fork malicioso não roda workflow sem o dono ver.
- **Impacto negativo:** toda mudança, mesmo uma vírgula, exige branch, PR e espera do CI.
- **Trade-offs aceitos:** em emergência (CI do GitHub fora do ar), o dono desativa a regra
  pela API, faz a correção e reaplica com `aplicar.sh`, registrando o motivo no handoff.

## Diretriz de Conformidade de Código

- **Proibido:** push direto no `main`; adicionar colaborador com escrita sem novo ADR;
  adicionar exceção (*bypass*) à regra do `main`.
- **Proibido:** workflow com permissão de escrita por padrão; `pull_request_target` que faça
  checkout do código do PR.
- **Obrigatório:** todo novo job de verificação entra no `needs` do job `CI ok`.
- **Obrigatório:** mudança em `.github/governanca/` é aplicada com `aplicar.sh` e conferida
  com `aplicar.sh --conferir` no mesmo trabalho.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
