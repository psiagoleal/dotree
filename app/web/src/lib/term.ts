// Caminho relativo: app/web/src/lib/term.ts
// How a combined term is written in the legend and in the width challenge: `3E`, `AB` and,
// for the additive constant (EXP-15), the number itself, even when it is 1.
import type { Term } from './core';

export function termText(t: Pick<Term, 'coefficient' | 'vars'>): string {
	return (t.coefficient === '1' && t.vars !== '' ? '' : t.coefficient) + t.vars;
}
