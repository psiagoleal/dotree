// Caminho relativo: app/web/src/lib/locale.spec.ts
import { describe, expect, it } from 'vitest';
import { detectLocale, hasStoredLocale } from './locale';

describe('detectLocale (IDI-03)', () => {
	it.each([
		[['pt-BR', 'en-US'], 'pt-br'],
		[['pt-PT'], 'pt-br'],
		[['pt'], 'pt-br'],
		[['en-US', 'pt-BR'], 'en'],
		[['de-DE'], 'en'],
		[[], 'en']
	] as const)('%j -> %s', (preferred, expected) => {
		expect(detectLocale(preferred)).toBe(expected);
	});
});

describe('hasStoredLocale', () => {
	it('reads the stored choice and survives a throwing storage', () => {
		expect(hasStoredLocale({ getItem: () => 'en' })).toBe(true);
		expect(hasStoredLocale({ getItem: () => null })).toBe(false);
		expect(
			hasStoredLocale({
				getItem: () => {
					throw new Error('blocked');
				}
			})
		).toBe(false);
		expect(hasStoredLocale(undefined)).toBe(false);
	});
});
