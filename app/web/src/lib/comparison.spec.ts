// Caminho relativo: app/web/src/lib/comparison.spec.ts
// MT-38: the frontier of the non-dominated solutions (figures() runs against the real core
// in Comparison.svelte.spec.ts, in the browser).
import { describe, expect, it } from 'vitest';
import { nonDominated } from './comparison.ts';

describe('nonDominated (MT-38)', () => {
	const point = (area: number, delay: number, complete = true) => ({ area, delay, complete });

	it('R1: Wallace (14.5; 6) dominates the student (15; 6) and Dadda (17; 7)', () => {
		expect(nonDominated([point(15, 6), point(17, 7), point(14.5, 6)])).toEqual([
			false,
			false,
			true
		]);
	});

	it('a trade-off keeps both; equal points do not dominate each other', () => {
		expect(nonDominated([point(15, 6), point(14.5, 6.5)])).toEqual([true, true]);
		expect(nonDominated([point(15, 6), point(15, 6)])).toEqual([true, true]);
	});

	it('an incomplete circuit is out of the frontier and does not dominate', () => {
		expect(nonDominated([point(1, 1, false), point(15, 6)])).toEqual([null, true]);
	});
});
