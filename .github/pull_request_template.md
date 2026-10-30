<!-- Caminho relativo: .github/pull_request_template.md -->

## O que muda

<!-- Uma ou duas frases. -->

## Critérios e ticket

- Spec: <!-- ex.: CMP-11, VER-03 -->
- Ticket: <!-- ex.: MT-5 -->

## Como foi verificado

- [ ] `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
- [ ] `pnpm --dir app/web run check && pnpm --dir app/web run lint && pnpm --dir app/web run test`
- [ ] `BASE_PATH=/dotree pnpm --dir app/web run build && BASE_PATH=/dotree pnpm --dir app/web run test:e2e`
- [ ] Spec, ADR e `docs/TICKETS.md` atualizados no mesmo PR, se o comportamento mudou
