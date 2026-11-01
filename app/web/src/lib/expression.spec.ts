// Caminho relativo: app/web/src/lib/expression.spec.ts
import { describe, expect, it } from 'vitest';
import { EXAMPLES, variablesIn } from './expression';

describe('variablesIn', () => {
	it('lists each upper-case letter once, sorted', () => {
		expect(variablesIn('B*A + 3(A+C)')).toEqual(['A', 'B', 'C']);
		expect(variablesIn('')).toEqual([]);
		expect(variablesIn('a*b')).toEqual([]);
	});

	it('every example gives a width to each of its variables (EXP-12)', () => {
		for (const ex of EXAMPLES) {
			expect(Object.keys(ex.widths).sort()).toEqual(variablesIn(ex.expression));
		}
	});
});
