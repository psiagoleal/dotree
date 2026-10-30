<!-- Caminho relativo: docs/adr/0002-entrega-github-pages-e-actions.md -->

# ADR 0002: Entrega por GitHub Actions, GitHub Pages e imagem no GHCR

- **Status:** Accepted
- **Data:** 2026-10-30
- **Decisores:** Iago Leal
- **Tags:** entrega, CI, hospedagem

## Contexto

O dotree é um site estático (ADR 0001). O autor quer um endereço público para mostrar o
trabalho ao professor desde as primeiras versões, e quer o CI/CD pronto antes das
funcionalidades, para que cada uma entre já verificada. O repositório está no GitHub, é
público e o projeto não tem orçamento.

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Netlify / Vercel / Cloudflare Pages** | Bons para site estático, mas acrescentam conta, painel e integração fora do GitHub sem ganho para um site sem servidor. |
| **Hospedagem própria (VPS) com a imagem Docker** | Custo e manutenção sem necessidade enquanto o app for só WASM. Continua possível com a imagem publicada. |
| **CI em outro serviço** | O GitHub Actions é gratuito para repositório público e já integra Pages, GHCR e Dependabot. |

## Decisão

- **CI:** GitHub Actions, num workflow `ci.yml` que roda em *push* e *pull request* para
  `main` (critérios ENT-01, ENT-06, ENT-08).
- **Publicação:** job do mesmo workflow que, depois de todos os outros passarem num *push*
  para `main`, publica a build em **GitHub Pages** sob `/dotree/` (ENT-02, ENT-03).
- **Imagem:** `Dockerfile` em etapas; CI constrói e testa a imagem; *tags* `v*` publicam em
  `ghcr.io/psiagoleal/dotree` e criam a *release* (ENT-04, ENT-05).
- **Dependências:** Dependabot semanal (ENT-07).

## Consequências

- **Impacto positivo:** custo zero; tudo no mesmo lugar do código; endereço estável para
  enviar ao professor; cada mudança chega ao ar verificada.
- **Impacto negativo:** o caminho base `/dotree/` precisa ser respeitado pela build (rotas,
  arquivos e `.wasm`); o Pages não permite cabeçalhos HTTP próprios.
- **Trade-offs aceitos:** sem pré-visualização por PR na v0.1.

## Diretriz de Conformidade de Código

- **Proibido:** caminho absoluto fixo (`/arquivo`) no código da interface; usar o caminho base
  do SvelteKit.
- **Proibido:** publicar no Pages a partir de máquina local; só o CI publica.
- **Obrigatório:** todo passo de verificação do CI roda também localmente pelo mesmo comando
  declarado na ilha `comandos-exatos` do `AGENTS.md`.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
