#!/usr/bin/env bash
# Caminho relativo: app/core/examples/ghdl_casos.sh
#
# Simulates every case written by `vhdl_referencia` with GHDL (--std=93) and checks the
# expected verdict (spec 06, VHD-01, VHD-04). Runs inside the GHDL image:
#   docker run --rm -v "$DIR:/w" -v "$PWD/app/core/examples:/x:ro" \
#     ghdl/ghdl:6.0.0-mcode-ubuntu-24.04 bash /x/ghdl_casos.sh /w
set -uo pipefail

DIR="${1:?uso: ghdl_casos.sh <dir gerado por vhdl_referencia>}"
falhas=0
while read -r caso tb esperado; do
  [[ -z "$caso" ]] && continue
  (
    cd "$DIR/$caso" || exit 2
    read -r -a arquivos < ordem.txt
    for f in "${arquivos[@]}"; do
      ghdl -a --std=93 "$f" || exit 3
    done
    ghdl -e --std=93 "$tb" || exit 3
    ghdl -r --std=93 "$tb" > sim.log 2>&1
  )
  rc=$?
  log="$DIR/$caso/sim.log"
  final="$(grep -E 'CORRETOS|CORRECT|ERRADOS|WRONG' "$log" 2>/dev/null | tail -1 | sed 's/.*(report \(note\|failure\)): //')"
  case "$esperado" in
    ok)    if [[ $rc -eq 0 ]] && grep -qE 'CASOS CORRETOS|CASES CORRECT' "$log"; then v=ok; else v=FALHOU; fi ;;
    falha) if [[ $rc -ne 0 ]] && grep -qE 'CASOS ERRADOS|CASES WRONG' "$log"; then v=ok; else v=FALHOU; fi ;;
    *)     v=FALHOU ;;
  esac
  printf '%-12s esperado=%-5s %-6s %s\n' "$caso" "$esperado" "$v" "$final"
  if [[ "$v" != ok ]]; then
    falhas=$((falhas + 1))
    sed 's/^/    /' "$log" 2>/dev/null | tail -20
  fi
done < "$DIR/casos.txt"
echo "== $falhas caso(s) com veredito inesperado"
[[ $falhas -eq 0 ]]
