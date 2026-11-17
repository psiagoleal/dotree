// Caminho relativo: app/web/src/lib/costs.spec.ts
import { describe, expect, it } from 'vitest';
import { DEFAULT_COSTS } from './core';
import { costText, isDefaultCosts, parseCost } from './costs';

describe('parseCost (GER-12)', () => {
	it('reads decimals with a comma or a point, in hundredths', () => {
		expect(parseCost('1')).toBe(100);
		expect(parseCost('0,5')).toBe(50);
		expect(parseCost('0.3')).toBe(30);
		expect(parseCost(' 1,25 ')).toBe(125);
		expect(parseCost(',75')).toBe(75);
		expect(parseCost('0.01')).toBe(1);
		expect(parseCost('100')).toBe(10_000);
		expect(parseCost('2.')).toBe(200);
	});

	it('refuses what is not a cost from 0.01 to 100 with at most two decimals', () => {
		for (const bad of [
			'',
			' ',
			'0',
			'0,00',
			'0.001',
			'1,234',
			'100.01',
			'101',
			'-1',
			'a',
			'1e2',
			'1,2,3',
			'.'
		])
			expect(parseCost(bad), bad).toBeNull();
	});
});

describe('costText', () => {
	it('writes hundredths in the locale, without trailing zeros', () => {
		expect(costText(30, 'pt-br')).toBe('0,3');
		expect(costText(125, 'en')).toBe('1.25');
		expect(costText(10_000, 'pt-br')).toBe('100');
		expect(parseCost(costText(1234, 'pt-br'))).toBe(1234);
	});
});

describe('isDefaultCosts', () => {
	it('is true only for FA 1 / 1 and HA 0.5 / 0.5', () => {
		expect(isDefaultCosts(DEFAULT_COSTS)).toBe(true);
		expect(isDefaultCosts({ ...DEFAULT_COSTS, haDelay: 51 })).toBe(false);
	});
});
