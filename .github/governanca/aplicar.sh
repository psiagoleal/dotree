#!/usr/bin/env bash
# Caminho relativo: .github/governanca/aplicar.sh
#
# Aplica e confere a governança do repositório público (ADR 0003).
#   .github/governanca/aplicar.sh            aplica tudo (idempotente) e confere
#   .github/governanca/aplicar.sh --conferir só confere, sem alterar nada (leitura)
# Requer: gh autenticado como dono do repositório, jq.
set -euo pipefail

REPO="${REPO:-psiagoleal/dotree}"
DONO="${REPO%%/*}"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API="repos/$REPO"

ruleset_id() { gh api "$API/rulesets" --jq ".[] | select(.name == \"$1\") | .id"; }

aplicar_ruleset() {
  local arq="$1" nome id
  nome="$(jq -r .name "$arq")"
  id="$(ruleset_id "$nome")"
  if [[ -n "$id" ]]; then
    gh api -X PUT "$API/rulesets/$id" --input "$arq" >/dev/null && echo "ruleset atualizado: $nome"
  else
    gh api -X POST "$API/rulesets" --input "$arq" >/dev/null && echo "ruleset criado:     $nome"
  fi
}

aplicar() {
  echo "== aplicando em $REPO"
  # Merge só por squash; branch apagada após o merge; auto-merge disponível.
  gh api -X PATCH "$API" \
    -F allow_merge_commit=false -F allow_rebase_merge=false -F allow_squash_merge=true \
    -f squash_merge_commit_title=PR_TITLE -f squash_merge_commit_message=COMMIT_MESSAGES \
    -F delete_branch_on_merge=true -F allow_auto_merge=true -F allow_update_branch=true >/dev/null
  echo "merge: só squash, apaga a branch, auto-merge"
  # Workflows de PR vindos de fork só rodam depois da sua aprovação, para qualquer externo.
  gh api -X PUT "$API/actions/permissions/fork-pr-contributor-approval" \
    -f approval_policy=all_external_contributors >/dev/null
  # Token dos workflows só com leitura e sem poder aprovar PR.
  gh api -X PUT "$API/actions/permissions/workflow" \
    -f default_workflow_permissions=read -F can_approve_pull_request_reviews=false >/dev/null
  echo "actions: aprovação para todo externo; token só leitura"
  # Alertas de vulnerabilidade, PRs de correção do Dependabot e relato privado de falhas.
  gh api -X PUT "$API/vulnerability-alerts" >/dev/null
  gh api -X PUT "$API/automated-security-fixes" >/dev/null
  gh api -X PUT "$API/private-vulnerability-reporting" >/dev/null
  echo "segurança: alertas, correções automáticas, relato privado"
  aplicar_ruleset "$DIR/main.json"
  aplicar_ruleset "$DIR/tags.json"
}

FALHAS=0
item() { # item <descrição> <obtido> <esperado>
  if [[ "$2" == "$3" ]]; then printf '  ok     %s\n' "$1"
  else printf '  FALHA  %s (obtido: %s; esperado: %s)\n' "$1" "$2" "$3"; FALHAS=$((FALHAS + 1)); fi
}

conferir() {
  echo "== conferindo $REPO"
  local r
  r="$(gh api "$API")"
  item "merge commit desligado"   "$(jq -r .allow_merge_commit <<<"$r")" false
  item "rebase merge desligado"   "$(jq -r .allow_rebase_merge <<<"$r")" false
  item "squash merge ligado"      "$(jq -r .allow_squash_merge <<<"$r")" true
  item "apaga branch após merge"  "$(jq -r .delete_branch_on_merge <<<"$r")" true
  item "secret scanning"          "$(jq -r .security_and_analysis.secret_scanning.status <<<"$r")" enabled
  item "push protection"          "$(jq -r .security_and_analysis.secret_scanning_push_protection.status <<<"$r")" enabled
  item "colaboradores com escrita" "$(gh api "$API/collaborators" --jq '[.[].login] | join(",")')" "$DONO"
  item "aprovação de workflow de fork" \
    "$(gh api "$API/actions/permissions/fork-pr-contributor-approval" --jq .approval_policy)" all_external_contributors
  item "token de workflow só leitura" \
    "$(gh api "$API/actions/permissions/workflow" --jq .default_workflow_permissions)" read
  item "workflow não aprova PR" \
    "$(gh api "$API/actions/permissions/workflow" --jq .can_approve_pull_request_reviews)" false
  item "alertas de vulnerabilidade" \
    "$(gh api "$API/vulnerability-alerts" >/dev/null 2>&1 && echo on || echo off)" on
  item "correções automáticas (Dependabot)" \
    "$(gh api "$API/automated-security-fixes" --jq .enabled 2>/dev/null || echo false)" true
  item "relato privado de vulnerabilidade" \
    "$(gh api "$API/private-vulnerability-reporting" --jq .enabled 2>/dev/null || echo false)" true
  item "Pages publica só do main" \
    "$(gh api "$API/environments/github-pages/deployment-branch-policies" --jq '[.branch_policies[].name] | join(",")')" main

  local id rs
  id="$(ruleset_id 'main protegido')"
  if [[ -z "$id" ]]; then item "ruleset 'main protegido'" ausente presente
  else
    rs="$(gh api "$API/rulesets/$id")"
    item "main: regra ativa"            "$(jq -r .enforcement <<<"$rs")" active
    item "main: ninguém fura a regra"   "$(jq -r '.bypass_actors | length' <<<"$rs")" 0
    item "main: exige PR"               "$(jq -r '[.rules[].type] | index("pull_request") != null' <<<"$rs")" true
    item "main: exige check 'CI ok'" \
      "$(jq -r '[.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context] | join(",")' <<<"$rs")" "CI ok"
    item "main: sem force push"         "$(jq -r '[.rules[].type] | index("non_fast_forward") != null' <<<"$rs")" true
    item "main: não pode ser apagado"   "$(jq -r '[.rules[].type] | index("deletion") != null' <<<"$rs")" true
  fi
  id="$(ruleset_id 'tags de release')"
  if [[ -z "$id" ]]; then item "ruleset 'tags de release'" ausente presente
  else
    rs="$(gh api "$API/rulesets/$id")"
    item "tags v*: regra ativa"         "$(jq -r .enforcement <<<"$rs")" active
    item "tags v*: só admin cria/apaga" "$(jq -r '[.bypass_actors[].actor_type] | join(",")' <<<"$rs")" RepositoryRole
  fi
  echo "== $FALHAS falha(s)"
  [[ "$FALHAS" -eq 0 ]]
}

case "${1:-}" in
  --conferir) conferir ;;
  "") aplicar; conferir ;;
  *) echo "uso: $0 [--conferir]" >&2; exit 2 ;;
esac
