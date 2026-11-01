// Caminho relativo: app/web/src/lib/signal.spec.ts
import { describe, expect, it } from 'vitest';
import { signalParts } from './signal';

describe('signalParts (VIS-01)', () => {
	it('splits a partial product into letter and index per factor', () => {
		expect(signalParts('a1_b0')).toEqual([
			{ name: 'a', index: '1' },
			{ name: 'b', index: '0' }
		]);
	});

	it('handles single bits and multi-digit indices of generated dots', () => {
		expect(signalParts('e0')).toEqual([{ name: 'e', index: '0' }]);
		expect(signalParts('k14')).toEqual([{ name: 'k', index: '14' }]);
		expect(signalParts('a0_b1_c2')).toHaveLength(3);
	});

	it('returns null for anything else, so the raw text is shown', () => {
		expect(signalParts('')).toBeNull();
		expect(signalParts('a_b0')).toBeNull();
		expect(signalParts('R(2)')).toBeNull();
	});
});
