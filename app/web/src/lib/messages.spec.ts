// Caminho relativo: app/web/src/lib/messages.spec.ts
// IDI-02: the pt-br and en catalogs must have exactly the same keys.
import { describe, expect, it } from 'vitest';
import ptBr from '../../messages/pt-br.json';
import en from '../../messages/en.json';

const keys = (catalog: Record<string, unknown>) =>
	Object.keys(catalog)
		.filter((k) => k !== '$schema')
		.sort();

describe('message catalogs (IDI-02)', () => {
	it('pt-br and en have the same keys', () => {
		expect(keys(ptBr)).toEqual(keys(en));
	});
});

describe('counts agree with their number', () => {
	it.each(['pt-br', 'en'] as const)('singular for 1, plural otherwise, in %s', async (locale) => {
		const { m } = await import('#lib/paraglide/messages.js');
		const o = { locale };
		const one = [
			m.matrix_summary({ dots: 1, ands: 1 }, o),
			m.matrix_column_label({ column: 0, height: 1 }, o),
			m.hint_min_levels_left({ height: 3, levels: 1 }, o),
			m.solution_summary({ levels: 1, fa: 1, ha: 0, area: '1', delay: '1' }, o)
		].join(' ');
		const many = [
			m.matrix_summary({ dots: 20, ands: 0 }, o),
			m.matrix_column_label({ column: 0, height: 0 }, o),
			m.hint_min_levels_left({ height: 6, levels: 2 }, o),
			m.solution_summary({ levels: 4, fa: 13, ha: 8, area: '17', delay: '7' }, o)
		].join(' ');
		const plural = locale === 'en' ? /\b(dots|gates|levels)\b/ : /\b(pontos|portas|níveis)\b/;
		expect(one).not.toMatch(plural);
		expect(many.match(new RegExp(plural, 'g'))).toHaveLength(5);
	});
});
