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
