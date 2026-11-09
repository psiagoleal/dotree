// Caminho relativo: app/web/src/lib/term.spec.ts
import { describe, expect, it } from 'vitest';
import { termText } from './term';

describe('termText (COR-01, EXP-15)', () => {
	it('writes the coefficient only when it is not 1, except for the constant', () => {
		expect(termText({ coefficient: '3', vars: 'E' })).toBe('3E');
		expect(termText({ coefficient: '1', vars: 'AB' })).toBe('AB');
		expect(termText({ coefficient: '36', vars: '' })).toBe('36');
		expect(termText({ coefficient: '1', vars: '' })).toBe('1');
	});
});
