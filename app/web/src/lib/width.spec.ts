// Caminho relativo: app/web/src/lib/width.spec.ts
import { describe, expect, it } from 'vitest';
import { widthPath } from './width';
import type { Input, Term } from './core';

// R1 as the core returns it (wasm/tests/bridge.rs).
const terms: Term[] = [
	{ coefficient: '1', vars: 'AB', max: '49' },
	{ coefficient: '1', vars: 'CD', max: '9' },
	{ coefficient: '3', vars: 'E', max: '9' },
	{ coefficient: '3', vars: 'F', max: '3' },
	{ coefficient: '1', vars: 'G', max: '1' }
];
const inputs: Input[] = [
	{ var: 'A', width: 3, max: '7' },
	{ var: 'B', width: 3, max: '7' },
	{ var: 'C', width: 2, max: '3' },
	{ var: 'D', width: 2, max: '3' },
	{ var: 'E', width: 2, max: '3' },
	{ var: 'F', width: 1, max: '1' },
	{ var: 'G', width: 1, max: '1' }
];

describe('widthPath (GER-03, step 2 of the width challenge)', () => {
	it('R1: formula, substitution, term values and the power-of-two bounds', () => {
		const p = widthPath({ terms, inputs, vMax: '71', outputWidth: 7 });
		expect(p.inputs).toEqual([
			{ var: 'A', width: 3, max: '7' },
			{ var: 'B', width: 3, max: '7' },
			{ var: 'C', width: 2, max: '3' },
			{ var: 'D', width: 2, max: '3' },
			{ var: 'E', width: 2, max: '3' },
			{ var: 'F', width: 1, max: '1' },
			{ var: 'G', width: 1, max: '1' }
		]);
		expect(p.symbolic).toBe('AB + CD + 3E + 3F + G');
		expect(p.substituted).toBe('7·7 + 3·3 + 3·3 + 3·1 + 1');
		expect(p.termValues).toBe('49 + 9 + 9 + 3 + 1');
		expect(p.vMax).toBe('71');
		expect(p.vMaxPlusOne).toBe('72');
		expect(p.lower).toEqual({ exponent: 6, value: '64' });
		expect(p.upper).toEqual({ exponent: 7, value: '128' });
	});

	it('a single term skips the line of term values', () => {
		const p = widthPath({
			terms: [{ coefficient: '1', vars: 'A', max: '15' }],
			inputs: [{ var: 'A', width: 4, max: '15' }],
			vMax: '15',
			outputWidth: 4
		});
		expect(p.symbolic).toBe('A');
		expect(p.substituted).toBe('15');
		expect(p.termValues).toBeNull();
		expect(p.lower).toEqual({ exponent: 3, value: '8' });
		expect(p.upper).toEqual({ exponent: 4, value: '16' });
	});

	it('keeps exact values beyond the range of a JS number', () => {
		const p = widthPath({
			terms: [{ coefficient: '4294967295', vars: 'A', max: '1095216660225' }],
			inputs: [{ var: 'A', width: 8, max: '255' }],
			vMax: '1095216660225',
			outputWidth: 40
		});
		expect(p.substituted).toBe('4294967295·255');
		expect(p.upper).toEqual({ exponent: 40, value: '1099511627776' });
	});
});
